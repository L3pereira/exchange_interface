use url::Url;
use std::collections::HashMap;
use tokio_tungstenite::tungstenite::protocol::Message;
use crate::{
    ExchangesConfig,
    binance_config_utils::*,
    bitstamp_config_utils::*
}; 

// static INIT: Once = Once::new();
// fn setup() -> () { 
//     INIT.call_once(|| {
//         const LOG_CONFIG_PATH: &str = "src/tests/log_config.yaml";
//         log4rs::init_file(LOG_CONFIG_PATH, Default::default()).unwrap();
//     });
// }


#[test]
fn test_binance_config(){

    let data = r#"{
        "binance": {
            "websocket_base_url": "wss://stream.binance.com:9443/stream",
            "websocket_rate_ms": 100,
            "symbols":["ethbtc","ltcbtc","bnbbtc"],
            "snapshot_depth": 10,
            "snapshot_base_url":"https://api.binance.com/api/v3/depth"
       },
       "bitstamp": {
           "websocket_base_url": "wss://ws.bitstamp.net",
           "symbols":["ethbtc","ltcbtc","bnbbtc"],
           "snapshot_base_url":"https://www.bitstamp.net/api/v2/order_book/"
       },
       "grpc_server": "127.0.0.1:50051",
       "web_server": "127.0.0.1:8080",
       "client_websocket":"ws://127.0.0.1:8080/rates"
    }"#;

    let snapshot_url = Url::parse("https://api.binance.com/api/v3/depth").unwrap();
    let mut ethbtc_snapshot = snapshot_url.clone();
    ethbtc_snapshot.set_query(Some("symbol=ETHBTC&limit=10")); 
    let mut ltcbtc_snapshot = snapshot_url.clone();
    ltcbtc_snapshot.set_query(Some("symbol=LTCBTC&limit=10"));
    let mut bnbbtc_snapshot = snapshot_url.clone();
    bnbbtc_snapshot.set_query(Some("symbol=BNBBTC&limit=10"));

    let websocket_url = Url::parse("wss://stream.binance.com:9443/").unwrap();
    let mut ethbtc_websocket = websocket_url.clone();
    ethbtc_websocket.set_path("/ws/ethbtc@depth@100ms");
    let mut ltcbtc_websocket = websocket_url.clone();
    ltcbtc_websocket.set_path("/ws/ltcbtc@depth@100ms");
    let mut bnbbtc_websocket = websocket_url.clone();
    bnbbtc_websocket.set_path("/ws/bnbbtc@depth@100ms");

    let mut websocket_hashmap: HashMap<String, Url> = HashMap::new();
    websocket_hashmap.insert("ETHBTC".to_string(), ethbtc_websocket);
    websocket_hashmap.insert("LTCBTC".to_string(), ltcbtc_websocket);
    websocket_hashmap.insert("BNBBTC".to_string(), bnbbtc_websocket);

    let mut snapshot_hashmap: HashMap<String, Url> = HashMap::new();
    snapshot_hashmap.insert("ETHBTC".to_string(), ethbtc_snapshot);
    snapshot_hashmap.insert("LTCBTC".to_string(), ltcbtc_snapshot);
    snapshot_hashmap.insert("BNBBTC".to_string(), bnbbtc_snapshot);
    

    let expected =  BinanceConfig{
        websocket_urls: websocket_hashmap,
        websocket_rate_ms: 100,
        snapshot_urls: snapshot_hashmap,
        snapshot_depth: 10,
        symbols: vec!["ETHBTC".to_string(), "LTCBTC".to_string(), "BNBBTC".to_string()]
    };

    let result = serde_json::from_str::<ExchangesConfig>(&data).unwrap();

    assert_eq!(expected, result.binance);
}


#[test]
fn test_bitstamp_config(){

    let data = r#"{
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
       }, 
       "grpc_server": "127.0.0.1:50051",
       "web_server": "127.0.0.1:8080",
       "client_websocket":"ws://127.0.0.1:8080/rates"
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
    let result = serde_json::from_str::<ExchangesConfig>(&data).unwrap();
 
    assert_eq!(expected, result.bitstamp);
}