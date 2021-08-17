
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
pub struct BitstampConfig{
    pub websocket_url: Url,
    pub websocket_payloads: HashMap<String, Message>,
    pub snapshot_urls: HashMap<String, Url>,
    pub symbols: Vec<String>
}
impl BitstampConfig {
    pub fn new(json_str: String) -> Result<Self>{

        let config: BitstampConfig = serde_json::from_str(&json_str)
            .context("JSON was not well-formatted config bitstamp")?;

        Ok(config)

    } 
}

impl<'de> Deserialize<'de> for BitstampConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let outter_config: OutterConfig = Deserialize::deserialize(deserializer)?;

        let bitstamp_config = outter_config.exchanges.bitstamp;

        let mut snapshot_hashmap: HashMap<Symbol, Url> = HashMap::new();
        let mut websocket_payloads: HashMap<Symbol, Message> = HashMap::new();

        
        let path = bitstamp_config.snapshot_base_url.path();
        for symbol in bitstamp_config.symbols.iter(){
            let mut new_snapshot_url = bitstamp_config.snapshot_base_url.clone();
            new_snapshot_url.set_path(symbol);
            new_snapshot_url.set_path(&format!("{}/{}", path, symbol));
            snapshot_hashmap.insert(symbol.clone(), new_snapshot_url);

            let payload_message =  format!("{{\"event\": \"bts:subscribe\", \"data\": {{ \"channel\": \"order_book_{}\" }} }}", symbol);
            websocket_payloads.insert(symbol.clone(), Message::Text(payload_message)); 
        }
 

        let config = BitstampConfig{
            websocket_url: bitstamp_config.websocket_base_url,
            websocket_payloads: websocket_payloads,
            snapshot_urls: snapshot_hashmap,
            symbols: bitstamp_config.symbols

        };
        Ok(config)

    }
}

pub struct BitstampService{
    pub config: BitstampConfig
}
impl BitstampService{
    pub fn new(config: BitstampConfig) -> Self{
        BitstampService{
            config: config
        }
    }
}
#[async_trait]
impl ExchangeInit for BitstampService{
    async fn stream_init_task(&mut self, output_stream_tx_ch: Sender<SnapshotData>) -> Result<()> {
        let task_name = "--Binance Stream Management Task--";

        let symbol = self.config.symbols.get(0)
            .context(format!("Error in {:?}:\ninput_rx_ch:\n", task_name))?;
        
        let web_socket_url = self.config.websocket_url.clone();

        let snapshot_url = self.config.snapshot_urls.get(symbol)
            .context(format!("Error in {:?}:\ninput_rx_ch:\n", task_name))?;

        let websocket_payload_init = self.config.websocket_payloads.get(symbol)
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

        writer_tx_ch.send(websocket_payload_init.clone()).await
            .context(format!("Error in {:?}:\ninput_rx_ch:\n", task_name))?;

        let (output_tx_ch, output_rx_ch) =  broadcast::channel(10);    
        let deserialize_settings = DeserializeSettings::new(symbol.clone(), reader_rx_ch, output_tx_ch, writer_tx_ch);
        tokio::spawn(<BitstampService as ExchangeService>::stream_management_task(deserialize_settings));
        
        <BitstampService as ExchangeService>::
            snapshot_task(symbol.clone(), snapshot_url.clone(), output_rx_ch, output_stream_tx_ch).await?;
        Ok(())
    }
}
#[async_trait]
impl ExchangeService for BitstampService{

    async fn stream_management_task(mut deserialize_settings: DeserializeSettings) {

        let task_name = "--Bitstamp Stream Management Task--";
        log::info!("{:?} Init", task_name);
        loop{
            match <BitstampService as ExchangeService>::websocket_msg_process(&mut deserialize_settings).await {
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
        let task_name = "--Bitstamp websocket_msg_process--";
     
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
                // log::info!("text_data \n\n{:?}:\n", text_data);
                if let Ok(outter) = serde_json::from_str::<OuterBitstampNoData>(&text_data){
                    if outter.event == "bts:subscription_succeeded" {
                        log::info!("Info in {:?}:\n Subscription Succeeded", task_name);
                        return Ok(());
                    } 
   
                }
                // let outter = serde_json::from_str::<OuterBitstamp2>(&text_data)
                //     .context(format!("Error in {:?}:\nDesirializing:\n", task_name))?;
                // if outter.event == "bts:subscription_succeeded" {
                //     log::info!("Info in {:?}:\n Subscription Succeeded", task_name);
                //     return Ok(());
                // } 


                let data = <BitstampService as ExchangeService>::deserialize_stream(text_data)
                    .context(format!("Error in {:?}:\nDesirializing:\n", task_name))?;

                deserialize_settings.output_tx_ch.send(data)
                    .context(format!("Error in {:?}:\nSending data:\n", task_name))?;

            },
            Message::Binary(_) => log::warn!("Warning in {:?}: binary data sent:\n", task_name)
        }
        Ok(())

    }

    async fn snapshot_task(
        _: Symbol, 
        _: Url, 
        mut output_rx_ch: Receiver<DepthData>, 
        output_stream_tx_ch: Sender<SnapshotData>) -> Result<()> {

        while let Ok(message) = output_rx_ch.recv().await{
 
            let snapshot_data = SnapshotData{
                exchange: message.exchange,
                symbol: message.symbol,
                timestamp: message.last_update_id_timestamp,
                bid_to_update: message.bid_to_update,
                ask_to_update: message.ask_to_update
            };
            output_stream_tx_ch.send(snapshot_data)
                .context("JSON was not well-formatted deserialize_snapshot binance")?;
        }
        Ok(())
    }

    fn deserialize_stream(json_str: String) -> Result<DepthData> {
        let task_name = "--Bitstamp deserialize_stream Task--";
        log::info!("bitstamp deserialize stream Init");
    
        let outer_bitstamp: OuterBitstamp = serde_json::from_str(&json_str)
            .context(format!("Error in {:?}:\n", task_name))?;
        
        let mut bid_to_update : BTreeMap<Price, Volume>= BTreeMap::new();
        let mut ask_to_update : BTreeMap<Price, Volume>= BTreeMap::new();
    
        for pair in outer_bitstamp.data.bid_to_update {
            let price: Price = pair[0];
            let volume: Volume = pair[1];
            bid_to_update.insert(price, volume);
        }
        for pair in outer_bitstamp.data.ask_to_update {
            let price: Price = pair[0];
            let volume: Volume = pair[1];
            ask_to_update.insert(price, volume);
        }
        let result = DepthData {    
            exchange: Exchange::Bitstamp,
            symbol: outer_bitstamp.symbol.replace( "order_book_", "").to_uppercase(),
            first_update_id_timestamp: serde_json::from_str(&outer_bitstamp.data.first_update_id_timestamp)
                .context(format!("Error in {:?}:\n", task_name))?,
            last_update_id_timestamp: serde_json::from_str(&outer_bitstamp.data.last_update_id_timestamp)            
                .context(format!("Error in {:?}:\n", task_name))?,
            bid_to_update: bid_to_update,
            ask_to_update: ask_to_update
        };
    
        Ok(result)
    }
    
    fn deserialize_snapshot(symbol: Symbol, json_str: String) -> Result<SnapshotData> {
    
        let outer_bitstamp_snapshot: OuterBitstampSnapshot = serde_json::from_str(&json_str)
            .context("JSON was not well-formatted deserialize_snapshot bitstamp")?;
    
        let mut bid_to_update : BTreeMap<Price, Volume>= BTreeMap::new();
        let mut ask_to_update : BTreeMap<Price, Volume>= BTreeMap::new();
    
        for pair in outer_bitstamp_snapshot.bid_to_update {
            let price: Price = pair[0];
            let volume: Volume = pair[1];
            bid_to_update.insert(price, volume);
        }
        for pair in outer_bitstamp_snapshot.ask_to_update {
            let price: Price = pair[0];
            let volume: Volume = pair[1];
            ask_to_update.insert(price, volume);
        }
    
        let result = SnapshotData {
            exchange: Exchange::Bitstamp,
            symbol: symbol,
            timestamp: serde_json::from_str(&outer_bitstamp_snapshot.micro_timestamp)
                .context("timestamp JSON was not well-formatted deserialize_snapshot bitstamp")?,
            bid_to_update: bid_to_update,
            ask_to_update: ask_to_update
        };
    
        Ok(result)
    }

}
