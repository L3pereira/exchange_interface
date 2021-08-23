
use std::{
    collections::{BTreeMap, HashMap}
};
use anyhow::{Context, Result};
use futures_util::StreamExt;
use url::Url;
use tokio_tungstenite::{
    connect_async, 
    tungstenite::protocol::Message,
};
use tokio::sync::{broadcast, mpsc};
use serde::{Deserialize, Deserializer};
use async_trait::async_trait;
use common::*;
use crate::*;
use crate::settings::DeserializeSettings;
use crate::exchanges_services::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinanceConfig{
    pub websocket_rate_ms: u32,
    pub snapshot_urls: HashMap<Symbol, Url>,
    pub websocket_urls: HashMap<Symbol, Url>,
    pub snapshot_depth: u32,
    pub symbols: Vec<String>
}
impl BinanceConfig {
    pub fn new(json_str: String) -> Result<Self>{
        
        let config: BinanceConfig = serde_json::from_str(&json_str)
            .context("JSON was not well-formatted config binance")?;

        Ok(config)
    } 
}
impl<'de> Deserialize<'de> for BinanceConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let outter_config: OutterConfig = Deserialize::deserialize(deserializer)?;
        // do better hex decoding than this
        let binance_config = outter_config.exchanges.binance;

        let mut snapshot_hashmap: HashMap<Symbol, Url> = HashMap::new();
        let mut symbol_websocket_url_hashmap: HashMap<Symbol, Url> = HashMap::new();

        for symbol in binance_config.symbols.iter(){
            let symbol_lower_case = symbol.to_lowercase();
            let mut symbol_snapshot_url = binance_config.snapshot_base_url.clone();
            symbol_snapshot_url.set_query(Some(&format!("symbol={}&limit={}", symbol, binance_config.snapshot_depth)));
            snapshot_hashmap.insert(symbol.clone(), symbol_snapshot_url);

            let mut symbol_websocket_url = binance_config.websocket_base_url.clone();
            symbol_websocket_url.set_path(&format!("/ws/{}@depth@{}ms", symbol_lower_case.clone(), &binance_config.websocket_rate_ms));
            symbol_websocket_url_hashmap.insert(symbol.clone(), symbol_websocket_url); 
        }

        let config = BinanceConfig{
            websocket_urls: symbol_websocket_url_hashmap,
            websocket_rate_ms: binance_config.websocket_rate_ms,
            snapshot_urls: snapshot_hashmap,
            snapshot_depth: binance_config.snapshot_depth,
            symbols: binance_config.symbols

        };

        Ok(config)

    }
}
pub struct BinanceService{
    pub config: BinanceConfig
}
impl BinanceService{
    pub fn new(config: BinanceConfig) -> Self{
        BinanceService{
            config: config
        }
    }
}
#[async_trait]
impl ExchangeInit for BinanceService{
    async fn stream_init_task(&mut self, output_stream_tx_ch: Sender<SnapshotData>) -> Result<()> {
        let task_name = "--Binance Stream Management Task--";

        let symbol = self.config.symbols.get(0)
            .context(format!("Error in {:?}:\ninput_rx_ch:\n", task_name))?;

        let web_socket_url = self.config.websocket_urls.get(symbol)
            .context(format!("Error in {:?}:\ninput_rx_ch:\n", task_name))?;

        let snapshot_url = self.config.snapshot_urls.get(symbol)
            .context(format!("Error in {:?}:\ninput_rx_ch:\n", task_name))?;

        let (ws_stream, _) = connect_async(web_socket_url).await
            .context(format!("Error in {:?}:\ninput_rx_ch:\n", task_name))?;

        let (writer, reader) = ws_stream.split();

        let (writer_tx_ch, writer_rx_ch): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(20);
        
        let writer_settings = WriterSettings::new(symbol.clone(), writer, writer_rx_ch);
        tokio::spawn(writer_task(writer_settings));

        let (reader_tx_ch, reader_rx_ch) = broadcast::channel(10);
        let reader_settings = ReaderSettings::new(symbol.clone(), reader, reader_tx_ch);
        tokio::spawn(reader_task(reader_settings));

        let (output_tx_ch, output_rx_ch) =  broadcast::channel(10);    
        let deserialize_settings = DeserializeSettings::new(symbol.clone(), reader_rx_ch, output_tx_ch, writer_tx_ch);
        tokio::spawn(<BinanceService as ExchangeService>::stream_management_task(deserialize_settings));
        
        <BinanceService as ExchangeService>::
            snapshot_task(symbol.clone(), snapshot_url.clone(), output_rx_ch, output_stream_tx_ch).await?;
        Ok(())
    }
}

#[async_trait]
impl ExchangeService for BinanceService{

    async fn stream_management_task(mut deserialize_settings: DeserializeSettings){
      let task_name = "--Binance Stream Management Task--";
      log::info!("{:?} Init", task_name);
      loop{
          match <BinanceService as ExchangeService>::websocket_msg_process(&mut deserialize_settings).await {
              Ok(_)=> continue,
              Err(err) => {
                  log::error!("{:?}", err);

                  match &err.downcast_ref::<broadcast::error::RecvError>() {
                      Some(err) => {
                          match err {
                              broadcast::error::RecvError::Lagged(x) => {
                                  log::trace!("Trace in {:?}:\ninput_rx_ch lagged:\n{:?}\n", task_name, x); 
                                  continue;
                              },
                              broadcast::error::RecvError::Closed => {
                                  log::warn!("Warning in {:?}:\ninput_rx_ch closed:\n", task_name); 
                                  break;
                              }
                  
                          }
                      },
                      None =>  log::warn!("Warning in {:?}:\ninput_rx_ch closed:\n", task_name)
                  };          
              }
          };
      } 
      log::info!("{:?} End", task_name);
    }

    async fn websocket_msg_process(deserialize_settings: &mut DeserializeSettings) -> Result<()> {
        let task_name = "--Binance Stream Management Task--";

        let input_msg = deserialize_settings.input_rx_ch.recv().await
            .context(format!("Error in {:?}:\ninput_rx_ch:\n", task_name))?;
        log::trace!("{:?}:\nReceived message from reader\n{:?}", task_name, input_msg);

        match input_msg {
                
            Message::Close(close_data) => log::warn!("Warning in {:?}:\nClose message received:\n {:?}", task_name, close_data),
            Message::Ping(ping_data) => {
                let pong_msg = Message::Pong(ping_data);

                deserialize_settings.writer_tx_ch.send(pong_msg).await
                    .context(format!("Error in {:?}:\nSending pong:\n", task_name))?;
                log::trace!("Trace in {:?}:\nSent pong", task_name)
            },
            Message::Pong(pong_data) => log::warn!("Warning in {:?}:\nPong message received:\n {:?}", task_name, pong_data),
            Message::Text(text_data) => {

                let data = <BinanceService as ExchangeService>::deserialize_stream(text_data)
                    .context(format!("Error in {:?}:\nDesirializing:\n", task_name))?;

                deserialize_settings.output_tx_ch.send(data)
                    .context(format!("Error in {:?}:\nSending data:\n", task_name))?;

            },
            Message::Binary(_) => log::warn!("Warning in {:?}: binary data sent:\n", task_name)
        }
        Ok(())

    }

    /// How to manage a local order book correctly,
    /// 
    /// u -> last_update_id_timestamp,
    /// 
    /// U -> first_update_id_timestamp,
    /// 
    /// lastUpdateId -> snapshot_message.timestamp,
    /// 
    /// 1 Open a stream to wss://stream.binance.com:9443/ws/bnbbtc@depth.
    /// 
    /// 2 Buffer the events you receive from the stream.
    /// 
    /// 3 Get a depth snapshot from https://api.binance.com/api/v3/depth?symbol=BNBBTC&limit=1000 .
    /// 
    /// 4 Drop any event where u is <= lastUpdateId in the snapshot.
    /// 
    /// 5 The first processed event should have U <= lastUpdateId+1 AND u >= lastUpdateId+1.
    /// 
    /// 6 While listening to the stream, each new event's U should be equal to the previous event's u+1.
    /// 
    /// 7 The data in each event is the absolute quantity for a price level.
    /// 
    /// 8 If the quantity is 0, remove the price level.
    /// 
    /// 9 Receiving an event that removes a price level that is not in your local order book can happen and is normal.
    /// 

    async fn snapshot_task(
        symbol: Symbol, 
        snapshot_url: Url, 
        mut output_rx_ch: Receiver<DepthData>, 
        output_stream_tx_ch: Sender<SnapshotData>) -> Result<()> {

        let task_name = "--Binance Snapshot Task Task--";
        let snapshot = get_snapshot(snapshot_url.clone()).await
            .context(format!("Error in {:?}:\n({:?})get_snapshot:\n", task_name, 1))?;

        let mut snapshot_message = <BinanceService as ExchangeService>::
            deserialize_snapshot(symbol.clone(), snapshot)
            .context(format!("Error in {:?}:\n({:?}) deserialize_snapshot:\n", task_name,1))?;

        let mut is_first_event = true;
        let mut previuos_event_last_timestamp:u64 = 0;
            
        let update_book_func = |message: DepthData, snapshot_message: &mut SnapshotData| { 

            for (price, volume) in message.ask_to_update.into_iter(){
                if volume == Decimal::new(0,0) {
                    snapshot_message.ask_to_update.remove_entry(&price);           
                }
                else{
                    snapshot_message.ask_to_update.entry(price).or_insert(volume);
                }
            }

            for (price, volume) in message.bid_to_update.into_iter(){
                if volume == Decimal::new(0,0) {
                    snapshot_message.bid_to_update.remove_entry(&price);           
                }
                else{
                    snapshot_message.bid_to_update.entry(price).or_insert(volume);
                }             
            }

            output_stream_tx_ch.send(snapshot_message.clone())
        };

        while let Ok(message) = output_rx_ch.recv().await{
            if is_first_event && message.last_update_id_timestamp <= snapshot_message.timestamp {
                continue;      
            }
            else if is_first_event && (message.first_update_id_timestamp <= (snapshot_message.timestamp + 1)) 
                && (message.last_update_id_timestamp >= (snapshot_message.timestamp + 1)){

                is_first_event = false;
                previuos_event_last_timestamp = message.last_update_id_timestamp;

                update_book_func(message, &mut snapshot_message)?;
            }
            else if !is_first_event && message.first_update_id_timestamp == (previuos_event_last_timestamp + 1){
                update_book_func(message, &mut snapshot_message)?;
            }
            else{
                let snapshot = get_snapshot(snapshot_url.clone()).await
                    .context(format!("Error in {:?}:\n{:?})get_snapshot:\n", task_name, 2))?;

                snapshot_message =  <BinanceService as ExchangeService>::
                    deserialize_snapshot(symbol.clone(), snapshot)
                        .context(format!("Error in {:?}:\n({:?}) deserialize_snapshot:\n", task_name, 2))?;

                is_first_event = true;
                continue;
            }
            // log::debug!("4\n\n{:?}-{:?}-{:?}\n\n", message.first_update_id_timestamp, 
            // message.last_update_id_timestamp ,snapshot_message.timestamp);
    


        }
        Ok(())
    }

    fn deserialize_stream(json_str: String) -> Result<DepthData>{
        log::info!("binance deserialize stream Init");
    
        let outer_binance: OuterBinance = serde_json::from_str(&json_str)
            .context("JSON was not well-formatted deserialize_stream binance")?;
    
        let mut bid_to_update : BTreeMap<Price, Volume>= BTreeMap::new();
        let mut ask_to_update : BTreeMap<Price, Volume>= BTreeMap::new();
    
        for pair in outer_binance.bid_to_update {
            let price: Price = pair[0];
            let volume: Volume = pair[1];
            bid_to_update.insert(price, volume);
        }
        for pair in outer_binance.ask_to_update {
            let price: Price = pair[0];
            let volume: Volume = pair[1];
            ask_to_update.insert(price, volume);
        }
        let result = DepthData {    
            exchange: Exchange::Binance,
            symbol: outer_binance.symbol,
            first_update_id_timestamp: outer_binance.first_update_id_timestamp,
            last_update_id_timestamp: outer_binance.last_update_id_timestamp,
            bid_to_update: bid_to_update,
            ask_to_update: ask_to_update
        };
        Ok(result)
    }

    fn deserialize_snapshot(symbol: Symbol, json_str: String) -> Result<SnapshotData>{
        
        let outer_binance_snapshot: OuterBinanceSnapshot = serde_json::from_str(&json_str)
            .context("JSON was not well-formatted deserialize_snapshot binance")?;
        let mut bid_to_update : BTreeMap<Price, Volume>= BTreeMap::new();
        let mut ask_to_update : BTreeMap<Price, Volume>= BTreeMap::new();
    
        for pair in outer_binance_snapshot.bid_to_update {
            let price: Price = pair[0];
            let volume: Volume = pair[1];
            bid_to_update.insert(price, volume);
        }
        for pair in outer_binance_snapshot.ask_to_update {
            let price: Price = pair[0];
            let volume: Volume = pair[1];
            ask_to_update.insert(price, volume);
        }
    
        let result = SnapshotData {
            exchange: Exchange::Binance,
            symbol: symbol,
            timestamp: outer_binance_snapshot.timestamp,
            bid_to_update: bid_to_update,
            ask_to_update: ask_to_update
        };
    
        Ok(result)
    }
}



