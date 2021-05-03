pub mod settings;
pub mod exchanges_services;
mod tests;

use std::{
    str::FromStr,
    collections::{BTreeMap, VecDeque}
};
use url::Url;
use settings::{ReaderSettings, WriterSettings, DeserializeSettings};
use common::{
    DepthData,
    SnapshotData,
    ErrorMessage,
    Price,
    Volume
};
use futures_util::{
    SinkExt, 
    StreamExt,
    stream::{Stream},
    sink::{Sink}
};

use tokio_tungstenite::{
    tungstenite::protocol::Message,
    tungstenite::error::Error as WsError
};
use tokio::{
    sync::{oneshot, watch, broadcast, mpsc},
    time::{sleep, Duration}
};
use log::{debug, error, info, warn};
use rust_decimal::Decimal;
const CONFIG_PATH: &str = "config.json"; 
const LOG_CONFIG_PATH: &str = "log_config.yaml";



pub async fn reader_task<S>(mut settings: ReaderSettings<S>) 
    where S: Stream<Item=Result<Message, WsError>> + Unpin
{
    // sleep(Duration::from_secs(5)).await;
    let task_name = "--Reader Task--";
    log::info!("{:?} Init", task_name);   
    while let Some(message) = settings.websocket_reader.next().await {      
        match message {
            Ok(message) => {
                log::trace!("{:?}:\n{:?}", task_name, message);
                if let Err(err) = settings.output_tx_ch.send(message) {
                    log::error!("Error in {:?}\noutput_tx_ch:\n{:?}", task_name, err);
                    break;
                }
            },
            Err(err) => log::error!("Error in {:?}:\nReading message from stream:\n{:?}", task_name, err)
        }       
    }
    log::info!("{:?} End", task_name);
}


pub async fn writer_task<S>(mut settings: WriterSettings<S>) 
    where S: Sink<Message, Error= WsError>  + Unpin
{
    let task_name = "--Writer Task--";
    log::info!("{:?} Init", task_name);
    while let Some(message) = settings.input_rx_ch.recv().await { 
        if let  Err(err) = settings.websocket_writer.send(message).await {
            log::error!("Error in {:?}:\nSending message to stream:\n{:?}", task_name, err)
        }
    }
    log::info!("{:?} End", task_name);

}


pub async fn get_snapshot(url: Url) -> Result<String, ErrorMessage>{
    let client = reqwest::Client::new();

    match client.get(url).send().await {
        Ok(request)=> { 
            match request.text().await {
                Ok(body)=> Ok(body),
                Err(err) => {
                           
                    log::error!("Request (body) snapshot error {:?}", err);
                    Err(ErrorMessage::new(11, format!("Request (body) snapshot error {:?}", err)))
                }
            }
        },
        Err(err) => {
            log::error!("Request snapshot error {:?}", err);
            Err(ErrorMessage::new(12, format!("Request snapshot error {:?}", err)))
        }
    }
}



// let file_path = "unit_tests/limit_order_tests/price_levels_test.json";   
// let mut file = File::open(file_path).expect("file should open read only");
// let mut buffer = String::new();
// file.read_to_string(&mut buffer).unwrap();
// let json : serde_json::Value = serde_json::from_str(&mut buffer).expect("JSON was not well-formatted");



