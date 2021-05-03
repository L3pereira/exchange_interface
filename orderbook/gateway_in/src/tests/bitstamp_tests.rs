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
    Symbol,
    Volume
};

use crate::exchanges_services::bitstamp::{
    BitstampConfig, deserialize_stream, deserialize_snapshot
};

use crate::settings::DeserializeSettings;

#[test]
fn bitstamp_config_test(){
    super::setup();
    let data = r#"{
        "exchanges": {
            "binance": {
                 "websocket_base_url": "wss://stream.binance.com:9443/stream",
                 "websocket_rate_ms": 100,
                 "symbols":["ETHBTC","LTCBTC","BNBBTC"],
                 "snapshot_depth": 10,
                 "snapshot_base_url":"https://api.binance.com/api/v3/depth"
            },
            "bitstamp": {
                "websocket_base_url": "wss://ws.bitstamp.net",
                "symbols":["ETHBTC","LTCBTC","BNBBTC"],
                "snapshot_base_url":"https://www.bitstamp.net/api/v2/order_book"
            }
        }   
    }"#;

    let snapshot_url = Url::parse("https://www.bitstamp.net/api/v2/order_book").unwrap();
    let path = snapshot_url.path();
    let mut ethbtc_snapshot = snapshot_url.clone();
    ethbtc_snapshot.set_path(&format!("{}/{}", path, "ETHBTC"));
    let mut ltcbtc_snapshot = snapshot_url.clone();
    ltcbtc_snapshot.set_path(&format!("{}/{}", path, "LTCBTC"));
    let mut bnbbtc_snapshot = snapshot_url.clone();
    bnbbtc_snapshot.set_path(&format!("{}/{}", path, "BNBBTC"));

    let websocket_url = Url::parse("wss://ws.bitstamp.net").unwrap();

    let mut websocket_payloads: HashMap<String, Message> = HashMap::new();
    let payload_message = "{\"event\": \"bts:subscribe\", \"data\": { \"channel\": \"order_book_XXXX\" } }";

    websocket_payloads.insert("ETHBTC".to_string(), Message::Text(payload_message.replace("\"order_book_XXXX\"", "\"order_book_ETHBTC\"")));
    websocket_payloads.insert("LTCBTC".to_string(), Message::Text(payload_message.replace("\"order_book_XXXX\"", "\"order_book_LTCBTC\"")));
    websocket_payloads.insert("BNBBTC".to_string(), Message::Text(payload_message.replace("\"order_book_XXXX\"", "\"order_book_BNBBTC\"")));

    let mut snapshot_hashmap: HashMap<String, Url> = HashMap::new();
    snapshot_hashmap.insert("ETHBTC".to_string(), ethbtc_snapshot);
    snapshot_hashmap.insert("LTCBTC".to_string(), ltcbtc_snapshot);
    snapshot_hashmap.insert("BNBBTC".to_string(), bnbbtc_snapshot);
    

    let expected =  BitstampConfig{
        websocket_url: websocket_url,
        websocket_payloads: websocket_payloads,
        snapshot_urls: snapshot_hashmap,
        symbols: vec!["ETHBTC".to_string(), "LTCBTC".to_string(), "BNBBTC".to_string()]
    };

    let result= BitstampConfig::new(data.to_string());
    assert_eq!(Ok(expected), result);
}

#[test]
fn deserialize_stream_bitstamp_test(){
    super::setup();
    let data =  r#"
    {   
        "event": "data",
        "channel": "order_book_ethbtc",
        "data":{
            "timestamp": "1833980193",
            "microtimestamp": "1833980193555559",
            "bids":[
                ["0.01074200","0.60000000"],
                ["0.01074100","4.12000000"]
            ],
            "asks":[
                ["0.01074300","5.74000000"],
                ["0.01074400","39.45000000"]
            ]
        }
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
 
    let expected = DepthData {
        symbol: "bnbbtc".to_string(),
        first_update_id_timestamp: 1833980193,
        last_update_id_timestamp: 1833980193555559,
        bid_to_update: bid_to_update,
        ask_to_update: ask_to_update  
     };
    //  println!("{:?}", )
    let result = deserialize_stream("bnbbtc".to_string(), data.to_string());
    assert_eq!(Ok(expected), result);

}

#[test]
fn deserialize_snapshot_bitstamp_test(){
    super::setup();
    let data =  r#"{
        "microtimestamp":"1833980193054545",
        "timestamp":"1833980193",
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
        symbol: "bnbbtc".to_string(),
        timestamp: 1833980193054545,
        bid_to_update: bid_to_update,
        ask_to_update: ask_to_update
     };
 
    let result = deserialize_snapshot("bnbbtc".to_string(), data.to_string());
    assert_eq!(Ok(expected), result);

}

#[tokio::test]
async fn stream_management_task_bitstamp_test() {
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
                symbol: "ethbtc".to_string(),
                first_update_id_timestamp: 1833980193,
                last_update_id_timestamp: 1833980193555559,
                bid_to_update: bid_to_update,
                ask_to_update: ask_to_update         
            };
            Ok(data)
        

    };

    let data =  r#"
    {   
        "event": "data",
        "channel": "order_book_ethbtc",
        "data":{
            "timestamp": "1833980193",
            "microtimestamp": "1833980193555559",
            "bids":[
                ["0.01074200","0.60000000"]
            ],
            "asks":[
                ["0.01074300","5.74000000"]
            ]
        }
    }"#;
    let (input_tx_ch, mut input_rx_ch) =  broadcast::channel(3);
    let (output_tx_ch, mut output_rx_ch) =  broadcast::channel(3);
    let (writer_tx_ch, mut writer_rx_ch): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(20);
    let settings = DeserializeSettings::new("ethbtc".to_string(), deserialize_fn, input_rx_ch, output_tx_ch, writer_tx_ch);
    
    tokio::spawn(crate::exchanges_services::bitstamp::stream_management_task(settings));
    input_tx_ch.send(Message::Text("{\"event\":\"bts:subscription_succeeded\",\"channel\":\"order_book_ethbtc\",\"data\":{}}".to_string()));
    input_tx_ch.send(Message::Text(data.to_string()));
    input_tx_ch.send(Message::Ping(vec![1_u8, 2, 3]));
    let result_data_serialized: DepthData = (deserialize_fn("ethbtc".to_string(), "".to_string())).unwrap();
    // Deserialize Task Test        |      Deserialize Task        |     Deserialize Task Test
    //-->input_tx_ch-->Broadcast ch-->input_rx_ch --> output_tx_ch-->Broadcast ch-->output_rx_ch
    //  Deserialize Task |             |   Deserialize Task Test
    //-->writer_tx_ch ----> MPSC ch  --> writer_rx_ch
    assert_eq!(output_rx_ch.recv().await, Ok(result_data_serialized));
    assert_eq!(writer_rx_ch.recv().await, Some(Message::Pong(vec![1_u8, 2, 3])));
}
