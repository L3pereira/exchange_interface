
use std::{
    str::FromStr,
    collections::{BTreeMap, VecDeque, HashMap}
};
use url::Url;
use rust_decimal::Decimal;
use tokio_tungstenite::{
    tungstenite::protocol::Message,
};
use tokio::{
    sync::{oneshot, watch, broadcast, mpsc},
    time::{sleep, Duration}
};
use common::{
    DepthData,
    SnapshotData,
    ErrorMessage,
    Price,
    Volume,
    Symbol,
    ErrCode,
    ErrMsg
};

use crate::settings::DeserializeSettings;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinanceConfig{
    pub websocket_rate_ms: u64,
    pub snapshot_urls: HashMap<Symbol, Url>,
    pub websocket_urls: HashMap<Symbol, Url>,
    pub snapshot_depth: u64,
    pub symbols: Vec<String>
}
impl BinanceConfig {
    pub fn new(mut json_str: String) -> Result<Self, ErrorMessage>{
        use serde_json::{Value, Map};
        let json : Value = serde_json::from_str(&mut json_str)
                .map_err(|e| ErrorMessage::new(200, format!("JSON was not well-formatted {:?}", e)))?;

        match &json["exchanges"]["binance"]{
            Value::Object(value) => value,
            _ => return Err(ErrorMessage::new(201, "exchanges was not well-formatted".to_string()))
        };
        let websocket_base_url: &str = match json["exchanges"]["binance"]["websocket_base_url"].as_str(){
            Some(value) =>  value,
            None => return Err(ErrorMessage::new(202, "binance websocket_base_url was not well-formatted".to_string()))
        };
        let websocket_rate_ms: u64 = match json["exchanges"]["binance"]["websocket_rate_ms"].as_u64(){
            Some(value) =>  value,
            None => return Err(ErrorMessage::new(203, "binance websocket_rate_ms was not well-formatted".to_string()))
        };
        let snapshot_depth: u64 = match json["exchanges"]["binance"]["snapshot_depth"].as_u64(){
            Some(value) =>  value,
            None => return Err(ErrorMessage::new(204, "binance snapshot_depth was not well-formatted".to_string()))
        };
        let snapshot_base_url: &str = match json["exchanges"]["binance"]["snapshot_base_url"].as_str(){
            Some(value) =>  value,
            None => return Err(ErrorMessage::new(205, "binance snapshot_base_url was not well-formatted".to_string()))
        };
   
        let symbols: Vec<Value> = match json["exchanges"]["binance"]["symbols"].as_array(){
            Some(value) =>  value.to_vec(),
            None => return Err(ErrorMessage::new(206, "binance symbols was not well-formatted".to_string()))
        };
        let mut symbols_str :Vec<String> = Vec::new();
        let mut symbol_snapshot_hashmap: HashMap<String, Url> = HashMap::new();
        let mut symbol_websocket_url_hashmap: HashMap<String, Url> = HashMap::new();

        let snapshot_base_url: Url = match Url::parse(snapshot_base_url) {
            Ok(value) =>  value,
            Err(err) => return Err(ErrorMessage::new(207, format!("binance snapshot_base_url url parse error: {}", err)))
        };
        let mut websocket_base_url: Url = match Url::parse(websocket_base_url) {
            Ok(value) =>  value,
            Err(err) => return Err(ErrorMessage::new(209, format!("binance websocket_base_url url parse error: {}", err)))
        };

        for symbol in symbols.into_iter(){
            let symbol =  match symbol.as_str(){
                Some(value) => {
                    let value_lower_case = value.to_lowercase();
                    let mut symbol_snapshot_url = snapshot_base_url.clone();
                    symbol_snapshot_url.set_query(Some(&format!("symbol={}&limit={}", value.to_uppercase(), snapshot_depth)));
                    symbol_snapshot_hashmap.insert(value_lower_case.clone(), symbol_snapshot_url);

                    let mut symbol_websocket_url = websocket_base_url.clone();
                    symbol_websocket_url.set_path(&format!("/ws/{}@depth@{}ms", value_lower_case.clone(), websocket_rate_ms));
                    symbol_websocket_url_hashmap.insert(value_lower_case.clone(), symbol_websocket_url); 
                    value_lower_case                 
                },
                None => return Err(ErrorMessage::new(208, "binance symbol was not well-formatted".to_string()))
            };
            symbols_str.push(symbol.to_string());
        } 

        let config = BinanceConfig{
            websocket_urls: symbol_websocket_url_hashmap,
            websocket_rate_ms: websocket_rate_ms,
            snapshot_urls: symbol_snapshot_hashmap,
            snapshot_depth: snapshot_depth,
            symbols: symbols_str

        };

        Ok(config)
    } 
}


pub fn deserialize_stream(symbol: Symbol, mut json_str: String) -> Result<DepthData, ErrorMessage>{
    log::info!("binance deserialize stream Init");
    
    let json : serde_json::Value = serde_json::from_str(&mut json_str)
    .map_err(|e| ErrorMessage::new(100, format!("JSON was not well-formatted {}", e)))?;

    let mut bid_to_update_map : BTreeMap<Price, Volume>= BTreeMap::new();
    let mut ask_to_update_map : BTreeMap<Price, Volume>= BTreeMap::new();

    // let symbol = match json["s"].as_str(){
    //     Some(value) =>  value,
    //     None => return Err(ErrorMessage::new(101, "u was not well-formatted".to_string()))
    // };
    let first_update_id_timestamp = match json["U"].as_u64(){
        Some(value) =>  value,
        None => return Err(ErrorMessage::new(102, "U was not well-formatted".to_string()))
    };

    let last_update_id_timestamp = match json["u"].as_u64(){
        Some(value) =>  value,
        None => return Err(ErrorMessage::new(103, "u was not well-formatted".to_string()))
    };


    ///////////////////////////////////////////////////////////////////////////////////////////////////////////

    let ask_to_update_array = match json["a"].as_array(){
        Some(value) =>  value,
        None => return Err(ErrorMessage::new(104, "a was not well-formatted".to_string()))
    };

    for ask in ask_to_update_array{

        let ask = match ask.as_array(){
            Some(value) =>  value,
            None => return Err(ErrorMessage::new(105, "a was not well-formatted".to_string()))
        };

        if ask.len() != 2 {
            return Err(ErrorMessage::new(106, "ask was not well-formatted (lack of price or volume)".to_string()))
        }
        else {

            let price = match Decimal::from_str(ask[0].as_str().unwrap_or("x")){
                Ok(value) =>  value,
                Err(err) =>  return Err(ErrorMessage::new(107, format!("ask price JSON was not well-formatted {}", err)))
            };
            let volume = match Decimal::from_str(ask[1].as_str().unwrap_or("x")){
                Ok(value) =>  value,
                Err(err) =>  return Err(ErrorMessage::new(108, format!("ask volume JSON was not well-formatted {}", err)))
            };

            ask_to_update_map
            .entry(price)
            .or_insert(volume);

        }
    }
    /////////////////////////////////////////////////////////////////////////////////////////////////////////// 
    
    let bid_to_update_array = match json["b"].as_array(){
        Some(value) =>  value,
        None => return Err(ErrorMessage::new(109, "b was not well-formatted".to_string()))
    };

    for bid in bid_to_update_array{
        let bid = match bid.as_array(){
            Some(value) =>  value,
            None => return Err(ErrorMessage::new(110, "b was not well-formatted".to_string()))
        };

        if bid.len() != 2 {
            return Err(ErrorMessage::new(111, "bid was not well-formatted (lack of price or volume)".to_string()))
        }
        else {
 
            let price = match Decimal::from_str(bid[0].as_str().unwrap_or("x")){
                Ok(value) =>  value,
                Err(err) =>  return Err(ErrorMessage::new(112, format!("price JSON was not well-formatted {}", err)))
            };
            let volume = match Decimal::from_str(bid[1].as_str().unwrap_or("x")){
                Ok(value) =>  value,
                Err(err) =>  return Err(ErrorMessage::new(113, format!("volume JSON was not well-formatted {}", err)))
            };

            bid_to_update_map
            .entry(price)
            .or_insert(volume);

        }
    }
    let depth_data = DepthData {
        symbol: symbol.to_string(),
        first_update_id_timestamp:first_update_id_timestamp,
        last_update_id_timestamp: last_update_id_timestamp,
        bid_to_update: bid_to_update_map,
        ask_to_update: ask_to_update_map
    };

    Ok(depth_data)
}


pub fn deserialize_snapshot(symbol: Symbol, mut json_str: String) -> Result<SnapshotData, ErrorMessage>{
    
    let json : serde_json::Value = serde_json::from_str(&mut json_str)
    .map_err(|e| ErrorMessage::new(300, format!("JSON was not well-formatted {}", e)))?;

    let mut bid_to_update_map : BTreeMap<Price, Volume>= BTreeMap::new();
    let mut ask_to_update_map : BTreeMap<Price, Volume>= BTreeMap::new();


    let last_update_id = match json["lastUpdateId"].as_u64(){
        Some(value) =>  value,
        None => return Err(ErrorMessage::new(302, "lastUpdateId was not well-formatted".to_string()))
    };

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////

    let ask_to_update_array = match json["asks"].as_array(){
        Some(value) =>  value,
        None => return Err(ErrorMessage::new(304, "asks were not well-formatted".to_string()))
    };

    for ask in ask_to_update_array{

        let ask = match ask.as_array(){
            Some(value) =>  value,
            None => return Err(ErrorMessage::new(305, "a was not well-formatted".to_string()))
        };

        if ask.len() != 2 {
            return Err(ErrorMessage::new(306, "ask was not well-formatted (lack of price or volume)".to_string()))
        }
        else {

            let price = match Decimal::from_str(ask[0].as_str().unwrap_or("x")){
                Ok(value) =>  value,
                Err(err) =>  return Err(ErrorMessage::new(307, format!("ask price JSON was not well-formatted {}", err)))
            };
            let volume = match Decimal::from_str(ask[1].as_str().unwrap_or("x")){
                Ok(value) =>  value,
                Err(err) =>  return Err(ErrorMessage::new(308, format!("ask volume JSON was not well-formatted {}", err)))
            };

            ask_to_update_map
            .entry(price)
            .or_insert(volume);

        }
    }
    /////////////////////////////////////////////////////////////////////////////////////////////////////////// 
    
    let bid_to_update_array = match json["bids"].as_array(){
        Some(value) =>  value,
        None => return Err(ErrorMessage::new(309, "bids was not well-formatted".to_string()))
    };

    for bid in bid_to_update_array{
        let bid = match bid.as_array(){
            Some(value) =>  value,
            None => return Err(ErrorMessage::new(310, "bids was not well-formatted".to_string()))
        };

        if bid.len() != 2 {
            return Err(ErrorMessage::new(311, "bid was not well-formatted (lack of price or volume)".to_string()))
        }
        else {
 
            let price = match Decimal::from_str(bid[0].as_str().unwrap_or("x")){
                Ok(value) =>  value,
                Err(err) =>  return Err(ErrorMessage::new(312, format!("price JSON was not well-formatted {}", err)))
            };
            let volume = match Decimal::from_str(bid[1].as_str().unwrap_or("x")){
                Ok(value) =>  value,
                Err(err) =>  return Err(ErrorMessage::new(313, format!("volume JSON was not well-formatted {}", err)))
            };

            bid_to_update_map
            .entry(price)
            .or_insert(volume);

        }
    }
    let snapshot_data = SnapshotData {
        symbol: symbol,
        timestamp: last_update_id,
        bid_to_update: bid_to_update_map,
        ask_to_update: ask_to_update_map
    };

    Ok(snapshot_data)
}

pub async fn stream_management_task(mut settings: DeserializeSettings)
  {

    let task_name = "--Binance Stream Management Task--";
    log::info!("{:?} Init", task_name);
    loop{
        match settings.input_rx_ch.recv().await {
            Ok(input_msg) => {
                log::trace!("{:?}:\nReceived message from reader\n{:?}", task_name, input_msg);
                match input_msg {
                    
                    Message::Close(close_data) => log::warn!("Warning in {:?}:\nClose message received:\n {:?}", task_name, close_data),
                    Message::Ping(ping_data) => {
                        let pong_msg = Message::Pong(ping_data);
                        match settings.writer_tx_ch.send(pong_msg).await {
                            Ok(_) => log::trace!("Trace in {:?}:\nSent pong", task_name),
                            Err(err) => log::error!("Error in {:?}:\nSending pong:\n{:?}", task_name, err)
                        }
                    },
                    Message::Pong(pong_data) => log::warn!("Warning in {:?}:\nPong message received:\n {:?}", task_name, pong_data),
                    Message::Text(text_data) => {
        
                        match (settings.deserialize_fn)(settings.symbol.clone(), text_data) {
                            Ok(depth_data) => {                 
                                if let Err(err) = settings.output_tx_ch.send(depth_data) {
                                    log::error!("Error in {:?}:\nSending data:\n {:?}", task_name, err);
                                }
                            },
                            Err(err) => log::error!("Error in {:?}:\nDesirializing:\n{:?}", task_name, err)
                        }
                    },
                    Message::Binary(_) => log::warn!("Warning in {:?}: binary data sent:\n", task_name)
                }
            },
            Err(err) => {
                log::debug!("--Stream Management Task Error-- {}", err);
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
            }
    
        }
    }
    log::info!("{:?} End", task_name);   
}