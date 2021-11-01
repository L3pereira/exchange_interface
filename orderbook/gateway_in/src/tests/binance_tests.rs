use std::{
    str::FromStr,
    collections::BTreeMap
};

use pretty_assertions::assert_eq;

use rust_decimal::Decimal;
use tokio::{
    sync::{broadcast, mpsc},

};
use tokio_tungstenite::tungstenite::protocol::Message;
use common::*;

use crate::exchanges_services::{
    binance::*,
    ExchangeService
};
use crate::settings::DeserializeSettings;



#[test]
fn deserialize_stream_binance_test(){
    // super::setup();
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
    let symbol = "BNBBTC".to_string();
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
        symbol: symbol.clone(),
        first_update_id_timestamp: 157,
        last_update_id_timestamp: 160,
        bid_to_update: bid_to_update,
        ask_to_update: ask_to_update  
     };

    let result =  <BinanceService as ExchangeService>::deserialize_stream(data.to_string()).unwrap();

    assert_eq!(expected, result);

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
    let symbol = "BNBBTC".to_string();
    let expected = SnapshotData {
        exchange: Exchange::Binance,
        symbol: symbol.clone(),
        timestamp: 1833980193,
        bid_to_update: bid_to_update,
        ask_to_update: ask_to_update
     };
   
    let result =  <BinanceService as ExchangeService>::deserialize_snapshot(symbol.clone(), data.to_string()).unwrap();
    assert_eq!(expected, result);

}

#[tokio::test(flavor = "multi_thread")]
async fn stream_management_task_binance_test() {
    super::setup();

    let data = r#"{        
            "e": "depthUpdate",
            "E": 123456789,
            "s": "BNBBTC",
            "U": 1833980193,
            "u": 183398019344444,
            "b": [["0.01074200", "0.60000000"]],
            "a": [["0.01074300","5.74000000"]]
        }"#;

    let symbol = "ETHBTC".to_string();
    let (input_tx_ch, input_rx_ch) =  broadcast::channel(10);
    let (writer_tx_ch, mut writer_rx_ch): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(20);
    let (output_tx_ch, mut output_rx_ch) =  broadcast::channel(10);
    let deserialize_settings = DeserializeSettings::new(symbol.clone(), input_rx_ch, output_tx_ch, writer_tx_ch);
   
    tokio::task::spawn(async move {
        <BinanceService as ExchangeService>::stream_management_task(deserialize_settings).await
    });
    input_tx_ch.send(Message::Text(data.to_string())).ok();
    input_tx_ch.send(Message::Ping(vec![1_u8, 2, 3])).ok();
   
    let mut bid_to_update: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut ask_to_update: BTreeMap<Price, Volume> =  BTreeMap::new();
    let symbol = "BNBBTC".to_string();
    bid_to_update.insert(
        Decimal::from_str("0.01074200").unwrap(), 
        Decimal::from_str("0.60000000").unwrap());
    ask_to_update.insert(
        Decimal::from_str("0.01074300").unwrap(), 
        Decimal::from_str("5.74000000").unwrap());
    let expected = DepthData {
        exchange: Exchange::Binance,
        symbol: symbol,
        first_update_id_timestamp: 1833980193,
        last_update_id_timestamp: 183398019344444,
        bid_to_update: bid_to_update,
        ask_to_update: ask_to_update         
    };
    // Deserialize Task Test        |      Deserialize Task        |     Deserialize Task Test
    //-->input_tx_ch-->Broadcast ch-->input_rx_ch --> output_tx_ch-->Broadcast ch-->output_rx_ch
    //  Deserialize Task |             |   Deserialize Task Test
    //-->writer_tx_ch ----> MPSC ch  --> writer_rx_ch
    assert_eq!(output_rx_ch.recv().await.unwrap(), expected);
    assert_eq!(writer_rx_ch.recv().await, Some(Message::Pong(vec![1_u8, 2, 3])));
}