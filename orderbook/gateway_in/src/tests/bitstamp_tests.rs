use std::{
    str::FromStr,
    collections::BTreeMap
};

use pretty_assertions::assert_eq;
use rust_decimal::Decimal;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio::sync::{broadcast, mpsc};
use common::*;
use crate::exchanges_services::{
    bitstamp::*,
    ExchangeService
};
use crate::settings::DeserializeSettings;



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
    let symbol = "ETHBTC".to_string();
    let expected = DepthData {
        exchange: Exchange::Bitstamp,
        symbol: symbol.clone(),
        first_update_id_timestamp: 1833980193,
        last_update_id_timestamp: 1833980193555559,
        bid_to_update: bid_to_update,
        ask_to_update: ask_to_update  
     };

    let result = <BitstampService as ExchangeService>::deserialize_stream(data.to_string()).unwrap();
    assert_eq!(expected, result);

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
    let symbol = "BNBBTC".to_string();
    let expected = SnapshotData {
        exchange: Exchange::Bitstamp,
        symbol: "BNBBTC".to_string(),
        timestamp: 1833980193054545,
        bid_to_update: bid_to_update,
        ask_to_update: ask_to_update
     };

     
    let result = <BitstampService as ExchangeService>::deserialize_snapshot(symbol.clone(), data.to_string()).unwrap();
    assert_eq!(expected, result);

}

#[tokio::test(flavor = "multi_thread")]
async fn stream_management_task_bitstamp_test() {
    super::setup();
   
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
    let symbol = "ETHBTC".to_string();
    let (input_tx_ch, input_rx_ch) =  broadcast::channel(10);
    let (writer_tx_ch, mut writer_rx_ch): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(20);
    let (output_tx_ch, mut output_rx_ch) =  broadcast::channel(10);
    let deserialize_settings = DeserializeSettings::new(symbol.clone(), input_rx_ch, output_tx_ch, writer_tx_ch);
      
    tokio::task::spawn(async move {
        <BitstampService as ExchangeService>::stream_management_task(deserialize_settings).await
    });

    input_tx_ch.send(Message::Text("{\"event\":\"bts:subscription_succeeded\",\"channel\":\"order_book_ethbtc\",\"data\":{}}".to_string())).ok();
    input_tx_ch.send(Message::Text(data.to_string())).ok();
    input_tx_ch.send(Message::Ping(vec![1_u8, 2, 3])).ok();


    let mut bid_to_update: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut ask_to_update: BTreeMap<Price, Volume> =  BTreeMap::new();

    bid_to_update.insert(
        Decimal::from_str("0.01074200").unwrap(), 
        Decimal::from_str("0.60000000").unwrap());
    ask_to_update.insert(
        Decimal::from_str("0.01074300").unwrap(), 
        Decimal::from_str("5.74000000").unwrap());
    let expected = DepthData {
        exchange: Exchange::Bitstamp,
        symbol: symbol,
        first_update_id_timestamp: 1833980193,
        last_update_id_timestamp: 1833980193555559,
        bid_to_update: bid_to_update,
        ask_to_update: ask_to_update         
    };
    // Deserialize Task Test        |      Deserialize Task        |     Deserialize Task Test
    //-->input_tx_ch-->Broadcast ch-->input_rx_ch --> output_tx_ch-->Broadcast ch-->output_rx_ch
    //  Deserialize Task |             |   Deserialize Task Test
    //-->writer_tx_ch ----> MPSC ch  --> writer_rx_ch
    assert_eq!(output_rx_ch.recv().await, Ok(expected));
    assert_eq!(writer_rx_ch.recv().await, Some(Message::Pong(vec![1_u8, 2, 3])));
}
