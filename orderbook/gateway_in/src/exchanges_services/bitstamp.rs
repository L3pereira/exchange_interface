
use std::{
    str::FromStr,
    collections::{BTreeMap, VecDeque, HashMap}
};
use url::Url;
use rust_decimal::Decimal;
use tokio_tungstenite::{
    tungstenite::protocol::Message,
};
use common::{
    DepthData,
    SnapshotData,
    ErrorMessage,
    Price,
    Symbol,
    Volume,
    ErrCode,
    ErrMsg,

};
use tokio::{
    sync::{oneshot, watch, broadcast, mpsc},
    time::{sleep, Duration}
};
use crate::settings::DeserializeSettings;


pub enum BitstampResponse{
    SubscriptionSuceeded

}


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitstampConfig{
    pub websocket_url: Url,
    pub websocket_payloads: HashMap<String, Message>,
    pub snapshot_urls: HashMap<String, Url>,
    pub symbols: Vec<String>
}
impl BitstampConfig {
    pub fn new(mut json_str: String) -> Result<Self, ErrorMessage>{
        use serde_json::{Value, Map};
        let json : Value = serde_json::from_str(&mut json_str)
                .map_err(|e| ErrorMessage::new(200, format!("JSON was not well-formatted {:?}", e)))?;

        match json["exchanges"]["bitstamp"].as_object(){
            Some(value) => value,
            _ => return Err(ErrorMessage::new(201, "exchanges was not well-formatted".to_string()))
        };

        //println!("{:?}",bitstamp["websocket_base_"]);
        let websocket_base_url: &str = match json["exchanges"]["bitstamp"]["websocket_base_url"].as_str(){
            Some(value) =>  value,
            None => return Err(ErrorMessage::new(202, "bitstamp websocket_base_url was not well-formatted".to_string()))
        };

        let snapshot_base_url: &str = match json["exchanges"]["bitstamp"]["snapshot_base_url"].as_str(){
            Some(value) =>  value,
            None => return Err(ErrorMessage::new(205, "bitstamp snapshot_base_url was not well-formatted".to_string()))
        };
   
        let symbols: Vec<Value> = match json["exchanges"]["bitstamp"]["symbols"].as_array(){
            Some(value) =>  value.to_vec(),
            None => return Err(ErrorMessage::new(206, "bitstamp symbols was not well-formatted".to_string()))
        };
        let mut symbols_str :Vec<Symbol> = Vec::new();
        let mut snapshot_hashmap: HashMap<Symbol, Url> = HashMap::new();
        let mut websocket_payloads: HashMap<Symbol, Message> = HashMap::new();
        let snapshot_base_url: Url = match Url::parse(snapshot_base_url) {
            Ok(value) =>  value,
            Err(err) => return Err(ErrorMessage::new(207, format!("bitstamp url parse error: {}", err)))
        };
        let path = snapshot_base_url.path();
        for symbol in symbols.into_iter(){
            let symbol =  match symbol.as_str(){
                Some(value) => {
                    let mut new_snapshot_url = snapshot_base_url.clone();
                    new_snapshot_url.set_path(value);
                    new_snapshot_url.set_path(&format!("{}/{}", path, value));
                    snapshot_hashmap.insert(value.to_string(), new_snapshot_url);

                    let payload_message =  format!("{{\"event\": \"bts:subscribe\", \"data\": {{ \"channel\": \"order_book_{}\" }} }}", value);
                    websocket_payloads.insert(value.to_string(), Message::Text(payload_message)); 
                    value
                },
                None => return Err(ErrorMessage::new(208, "bitstamp symbol was not well-formatted".to_string()))
            };
            symbols_str.push(symbol.to_string());
        }
 

        let websocket_base_url: Url = match Url::parse(websocket_base_url) {
            Ok(value) =>  value,
            Err(err) => return Err(ErrorMessage::new(209, format!("binance url parse error: {}", err)))
        };

        let config = BitstampConfig{
            websocket_url: websocket_base_url,
            websocket_payloads: websocket_payloads,
            snapshot_urls: snapshot_hashmap,
            symbols: symbols_str

        };
        Ok(config)
    } 
}

pub fn deserialize_stream(symbol: Symbol, mut json_str: String) -> Result<DepthData, ErrorMessage>{
    let json : serde_json::Value = serde_json::from_str(&mut json_str)
    .map_err(|e| ErrorMessage::new(100, format!("JSON was not well-formatted {}", e)))?;

    let mut bid_to_update_map : BTreeMap<Price, Volume>= BTreeMap::new();
    let mut ask_to_update_map : BTreeMap<Price, Volume>= BTreeMap::new();


    let micro_timestamp: u64 = match json["data"]["microtimestamp"].as_str(){
        Some(value) =>  value.parse::<u64>().map_err(|_| ErrorMessage::new(1022, "microtimestamp was not well-formatted".to_string()))?,
        None => return Err(ErrorMessage::new(102, "microtimestamp was not well-formatted".to_string()))
    };

    let timestamp = match json["data"]["timestamp"].as_str(){
        Some(value) =>  value.parse::<u64>().map_err(|_| ErrorMessage::new(1033, "timestamp was not well-formatted".to_string()))?,
        None => return Err(ErrorMessage::new(103, "timestamp was not well-formatted".to_string()))
    };


    ///////////////////////////////////////////////////////////////////////////////////////////////////////////

    let ask_to_update_array = match json["data"]["asks"].as_array(){
        Some(value) =>  value,
        None => return Err(ErrorMessage::new(104, "asks was not well-formatted".to_string()))
    };

    for ask in ask_to_update_array{

        let ask = match ask.as_array(){
            Some(value) =>  value,
            None => return Err(ErrorMessage::new(105, "asks was not well-formatted".to_string()))
        };

        if ask.len() != 2 {
            return Err(ErrorMessage::new(106, "asks was not well-formatted (lack of price or volume)".to_string()))
        }
        else {

            let price = match Decimal::from_str(ask[0].as_str().unwrap_or("x")){
                Ok(value) =>  value,
                Err(err) =>  return Err(ErrorMessage::new(107, format!("asks price JSON was not well-formatted {}", err)))
            };
            let volume = match Decimal::from_str(ask[1].as_str().unwrap_or("x")){
                Ok(value) =>  value,
                Err(err) =>  return Err(ErrorMessage::new(108, format!("asks volume JSON was not well-formatted {}", err)))
            };

            ask_to_update_map
            .entry(price)
            .or_insert(volume);

        }
    }
    /////////////////////////////////////////////////////////////////////////////////////////////////////////// 
    
    let bid_to_update_array = match json["data"]["bids"].as_array(){
        Some(value) =>  value,
        None => return Err(ErrorMessage::new(109, "bids was not well-formatted".to_string()))
    };

    for bid in bid_to_update_array{
        let bid = match bid.as_array(){
            Some(value) =>  value,
            None => return Err(ErrorMessage::new(110, "bids was not well-formatted".to_string()))
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
        symbol: symbol,
        first_update_id_timestamp:timestamp,
        last_update_id_timestamp: micro_timestamp,
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


    let last_update_id: u64 = match json["microtimestamp"].as_str(){
        Some(value) =>  value.parse::<u64>().map_err(|_| ErrorMessage::new(3022, "microtimestamp was not well-formatted".to_string()))?,
        None => return Err(ErrorMessage::new(302, "microtimestamp was not well-formatted".to_string()))
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
    let task_name = "--Bitstamp Stream Management Task--";
    log::info!("{:?} Init", task_name);
    loop{
        match settings.input_rx_ch.recv().await {
            Ok(input_msg) => {
                match input_msg {
                    Message::Close(close_data) => log::warn!("Warning in {:?}:\n close message received:\n {:?}", task_name, close_data),
                    Message::Ping(ping_data) => {
                        let pong_msg = Message::Pong(ping_data);
                        match settings.writer_tx_ch.send(pong_msg).await {
                            Ok(_) => log::trace!("Trace in {:?}:\n Sent pong", task_name),
                            Err(err) => log::error!("Error in {:?}:\n sending pong:\n {:?}", task_name, err)
                        }
                    },
                    Message::Pong(pong_data) => log::warn!("Warning in {:?}:\n pong message received:\n {:?}", task_name, pong_data),
                    Message::Text(mut text_data) => {
                        let json: serde_json::Value = match serde_json::from_str(&mut text_data){
                            Ok(json) => {
                                
                                let json: serde_json::Value = json;
                                if let Some(value) = json["event"].as_str(){
                                    if value == "bts:subscription_succeeded" {
                                        log::info!("Info in {:?}:\n Subscription Succeeded", task_name);
                                        continue;
                                    }
                                }
                                json                       
                            },
                            Err(err) => {log::error!("Error in {:?}:\n Server response JSON was not well-formatted:\n {:?}", task_name, err); continue}
                        };

                        match (settings.deserialize_fn)(settings.symbol.clone(), text_data) {
                            Ok(depth_data) => {                 
                                if let Err(err) = settings.output_tx_ch.send(depth_data) {
                                    log::error!("Error in {:?}:\n sending data:\n {:?}", task_name, err);
                                }
                            },
                            Err(err) => log::error!("Error in {:?}:\n desirializing:\n {:?}", task_name, err)
                        }
                    },
                    Message::Binary(_) => log::warn!("Warning in {:?}:\n binary data sent:\n", task_name)
                }
            },
            Err(err) => {
                match err {
                    broadcast::error::RecvError::Lagged(x) => {
                        log::trace!("Trace in {:?}:\n input_rx_ch lagged:\n {:?}\n", task_name, x); 
                        continue;
                    },
                    broadcast::error::RecvError::Closed => {
                        log::warn!("Warning in {:?}:\n input_rx_ch closed:\n", task_name); 
                        break;
                    }
    
                }
            }
    
        }
    }
    log::info!("{:?} End", task_name);   
}