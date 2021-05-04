use std::{
    str::FromStr,
    collections::{BTreeMap, VecDeque, HashMap},
    sync::Once
    //time::Duration
};
use url::Url;

use pretty_assertions::{assert_eq, assert_ne};
use log::{debug, error, info, warn};
use rust_decimal::Decimal;
use tokio::{
    sync::{oneshot, watch, broadcast, mpsc},
    time::{sleep, Duration}
};
use tokio_tungstenite::{
    tungstenite::protocol::Message,
};
use common::*;

use crate::exchanges_services::binance::{
    BinanceConfig, deserialize_stream, deserialize_snapshot
};
use crate::settings::DeserializeSettings;

#[test]
fn binance_config_test(){
    super::setup();
    let data = r#"{
        "exchanges": {
            "binance": {
                 "websocket_base_url": "wss://stream.binance.com:9443/stream",
                 "websocket_rate_ms": 100,
                 "symbols":["ETHBTC","ltcbtc","bnbbtc"],
                 "snapshot_depth": 10,
                 "snapshot_base_url":"https://api.binance.com/api/v3/depth"
            },
            "bitstamp": {
                "websocket_base_url": "wss://ws.bitstamp.net",
                "symbols":["ethbtc","ltcbtc","bnbbtc"],
                "snapshot_base_url":"https://www.bitstamp.net/api/v2/order_book/"
            }
        }   
    }"#;

    let mut snapshot_url = Url::parse("https://api.binance.com/api/v3/depth").unwrap();
    let mut ethbtc_snapshot = snapshot_url.clone();
    ethbtc_snapshot.set_query(Some("symbol=ETHBTC&limit=10")); 
    let mut ltcbtc_snapshot = snapshot_url.clone();
    ltcbtc_snapshot.set_query(Some("symbol=LTCBTC&limit=10"));
    let mut bnbbtc_snapshot = snapshot_url.clone();
    bnbbtc_snapshot.set_query(Some("symbol=BNBBTC&limit=10"));

    let mut websocket_url = Url::parse("wss://stream.binance.com:9443/").unwrap();
    let mut ethbtc_websocket = websocket_url.clone();
    ethbtc_websocket.set_path("/ws/ethbtc@depth@100ms");
    let mut ltcbtc_websocket = websocket_url.clone();
    ltcbtc_websocket.set_path("/ws/ltcbtc@depth@100ms");
    let mut bnbbtc_websocket = websocket_url.clone();
    bnbbtc_websocket.set_path("/ws/bnbbtc@depth@100ms");

    let mut websocket_hashmap: HashMap<String, Url> = HashMap::new();
    websocket_hashmap.insert("ethbtc".to_string(), ethbtc_websocket);
    websocket_hashmap.insert("ltcbtc".to_string(), ltcbtc_websocket);
    websocket_hashmap.insert("bnbbtc".to_string(), bnbbtc_websocket);

    let mut snapshot_hashmap: HashMap<String, Url> = HashMap::new();
    snapshot_hashmap.insert("ethbtc".to_string(), ethbtc_snapshot);
    snapshot_hashmap.insert("ltcbtc".to_string(), ltcbtc_snapshot);
    snapshot_hashmap.insert("bnbbtc".to_string(), bnbbtc_snapshot);
    

    let expected =  BinanceConfig{
        websocket_urls: websocket_hashmap,
        websocket_rate_ms: 100,
        snapshot_urls: snapshot_hashmap,
        snapshot_depth: 10,
        symbols: vec!["ethbtc".to_string(), "ltcbtc".to_string(), "bnbbtc".to_string()]
    };

    let result= BinanceConfig::new(data.to_string());
    assert_eq!(Ok(expected), result);
}

#[test]
fn deserialize_stream_binance_test(){
    super::setup();
    let data =  r#"
    {        
        "e": "depthUpdate",
        "E": 123456789,
        "s": "BNBBTC",
        "U": 157,
        "u": 160,
        "b": [["0.0024", "10"]],
        "a": [["0.0026","100"]]
    }"#; 

    let mut bid_to_update: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut ask_to_update: BTreeMap<Price, Volume> =  BTreeMap::new();
    bid_to_update.insert(
        Decimal::from_str("0.0024").unwrap(), 
        Decimal::from_str("10").unwrap());
    ask_to_update.insert(
        Decimal::from_str("0.0026").unwrap(), 
        Decimal::from_str("100").unwrap());
 
    let expected = DepthData {
        exchange: Exchange::Binance,
        symbol: "bnbbtc".to_string(),
        first_update_id_timestamp: 157,
        last_update_id_timestamp: 160,
        bid_to_update: bid_to_update,
        ask_to_update: ask_to_update  
     };

    let result = deserialize_stream("bnbbtc".to_string(), data.to_string());
    assert_eq!(Ok(expected), result);

}

#[test]
fn deserialize_snapshot_binance_test(){
    super::setup();
    let data =  r#"{
        "lastUpdateId":1833980193,
        "bids":[
            ["0.01074200","0.60000000"],
            ["0.01074100","4.12000000"]
        ],
        "asks":[
            ["0.01074300","5.74000000"],
            ["0.01074400","39.45000000"]
        ]
    }"#; 
    let mut bid_to_update: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut ask_to_update: BTreeMap<Price, Volume> =  BTreeMap::new();
    bid_to_update.insert(
        Decimal::from_str("0.01074200").unwrap(), 
        Decimal::from_str("0.60000000").unwrap());
    bid_to_update.insert(
        Decimal::from_str("0.01074100").unwrap(), 
        Decimal::from_str("4.12000000").unwrap());

    ask_to_update.insert(
        Decimal::from_str("0.01074300").unwrap(), 
        Decimal::from_str("5.74000000").unwrap());
    ask_to_update.insert(
        Decimal::from_str("0.01074400").unwrap(), 
        Decimal::from_str("39.45000000").unwrap());
 
    let expected = SnapshotData {
        exchange: Exchange::Binance,
        symbol: "bnbbtc".to_string(),
        timestamp: 1833980193,
        bid_to_update: bid_to_update,
        ask_to_update: ask_to_update
     };
 
    let result = deserialize_snapshot("bnbbtc".to_string(), data.to_string());
    assert_eq!(Ok(expected), result);

}

#[tokio::test]
async fn stream_management_task_binance_test() {
    super::setup();
    let deserialize_fn = |symbol: Symbol ,json: String| -> Result<DepthData, ErrorMessage> {

        let mut bid_to_update: BTreeMap<Price, Volume> =  BTreeMap::new();
        let mut ask_to_update: BTreeMap<Price, Volume> =  BTreeMap::new();
        bid_to_update.insert(
            Decimal::from_str("0.01074200").unwrap(), 
            Decimal::from_str("0.60000000").unwrap());
        ask_to_update.insert(
            Decimal::from_str("0.01074300").unwrap(), 
            Decimal::from_str("5.74000000").unwrap());
        let data = DepthData {
            exchange: Exchange::Binance,
            symbol: "bnbbtc".to_string(),
            first_update_id_timestamp: 1833980193,
            last_update_id_timestamp: 183398019344444,
            bid_to_update: bid_to_update,
            ask_to_update: ask_to_update         
        };
        Ok(data)

    };

    let data = r#"{        
            "e": "depthUpdate",
            "E": 123456789,
            "s": "BNBBTC",
            "U": 1833980193,
            "u": 183398019344444,
            "b": [["0.01074200", "0.60000000"]],
            "a": [["0.01074300","5.74000000"]]
        }"#; 
    let (input_tx_ch, mut input_rx_ch) =  broadcast::channel(3);
    let (output_tx_ch, mut output_rx_ch) =  broadcast::channel(3);
    let (writer_tx_ch, mut writer_rx_ch): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(20);
    let settings = DeserializeSettings::new("bnbbtc".to_string(), deserialize_fn, input_rx_ch, output_tx_ch, writer_tx_ch);
    
    tokio::spawn(crate::exchanges_services::binance::stream_management_task(settings));
    input_tx_ch.send(Message::Text(data.to_string()));
    input_tx_ch.send(Message::Ping(vec![1_u8, 2, 3]));
    let result_data_serialized: DepthData = (deserialize_fn("bnbbtc".to_string(), "".to_string())).unwrap();
    // Deserialize Task Test        |      Deserialize Task        |     Deserialize Task Test
    //-->input_tx_ch-->Broadcast ch-->input_rx_ch --> output_tx_ch-->Broadcast ch-->output_rx_ch
    //  Deserialize Task |             |   Deserialize Task Test
    //-->writer_tx_ch ----> MPSC ch  --> writer_rx_ch
    assert_eq!(output_rx_ch.recv().await, Ok(result_data_serialized));
    assert_eq!(writer_rx_ch.recv().await, Some(Message::Pong(vec![1_u8, 2, 3])));
}