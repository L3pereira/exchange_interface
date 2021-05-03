use std::fs::File;
use std::io::Read;
use tokio_tungstenite::{
    connect_async, 
    tungstenite::{
        protocol::Message,
        error:: {
            CapacityError,
            Error as WsError
        }
    },
};
use tokio::{
    sync::{oneshot, watch, broadcast, mpsc},
    time::{sleep, Duration}
};
use common::*;
use gateway_in::{
    reader_task,
    writer_task,
    get_snapshot,
    exchanges_services,
    settings::*
};
use futures_util::StreamExt;
const CONFIG_PATH: &str = "config.json"; 
const LOG_CONFIG_PATH: &str = "log_config.yaml";

use std::sync::Once;
static INIT: Once = Once::new();
pub fn setup_log() -> () { 
    INIT.call_once(|| {
        log4rs::init_file(LOG_CONFIG_PATH, Default::default()).unwrap();
    });
}


async fn binance_stream_init() {
    use exchanges_services::binance::*;
    let mut the_file = File::open(CONFIG_PATH).expect("file should open read only:\n ");
    let mut buff = String::new(); 
    the_file.read_to_string(&mut buff).unwrap();
    let binance_config =  BinanceConfig::new(buff).expect("binance_config:\n ");
    let symbol = binance_config.symbols.get(0).expect("symbol:\n ");
    let web_socket_url = binance_config.websocket_urls.get(symbol).expect("websocket_urls:\n ").clone();
    let snapshot_url = binance_config.snapshot_urls.get(symbol).expect("snapshot_urls:\n ").clone();
    
    log::debug!("{}", web_socket_url);
    let (ws_stream, _) = connect_async(web_socket_url).await.expect("Failed to connect");
    let (writer, reader) = ws_stream.split();

    let (writer_tx_ch, writer_rx_ch): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(20);

    let writer_settings = WriterSettings::new(symbol.clone(), writer, writer_rx_ch);
    tokio::spawn(writer_task(writer_settings));

    let (reader_tx_ch, reader_rx_ch) = broadcast::channel(10);
    let mut reader_rx_ch_2 = reader_tx_ch.subscribe();

    let reader_settings = ReaderSettings::new(symbol.clone(), reader, reader_tx_ch);
    tokio::spawn(reader_task(reader_settings));

    tokio::spawn(async move {log::debug!("reader_rx_ch_2 {:?}", reader_rx_ch_2.recv().await);});
    
    let (output_tx_ch, mut output_rx_ch) =  broadcast::channel(10);

    let deserialize_settings = DeserializeSettings::new(symbol.clone(), deserialize_stream, reader_rx_ch, output_tx_ch, writer_tx_ch);
    tokio::spawn(stream_management_task(deserialize_settings));
    log::error!("Binance  url{:?}", snapshot_url.as_str());
    log::error!("Binance Snapshot{:?}", deserialize_snapshot(symbol.clone(), get_snapshot(snapshot_url).await.unwrap()));
    // while let Ok(message) = output_rx_ch.recv().await{
    //     log::debug!("{:?}", message);
    //     // println!("{:?}", message)
    // }

}
async fn bitstamp_stream_init() {
    use exchanges_services::bitstamp::*;
    let mut the_file = File::open(CONFIG_PATH).expect("file should open read only:\n ");
    let mut buff = String::new(); 
    the_file.read_to_string(&mut buff).unwrap();
    let bitstamp_config =  BitstampConfig::new(buff).expect("bitstamp_config:\n ");

    let symbol = bitstamp_config.symbols.get(0).expect("symbol:\n ");
    let websocket_base_url = bitstamp_config.websocket_url;
    let websocket_payload_init = bitstamp_config.websocket_payloads.get(symbol).expect("websocket_urls:\n ").clone();
    let snapshot_url = bitstamp_config.snapshot_urls.get(symbol).expect("snapshot_urls:\n ").clone();

    let (ws_stream, _) = connect_async(websocket_base_url).await.expect("Failed to connect");
    let (writer, reader) = ws_stream.split();

    let (writer_tx_ch, writer_rx_ch): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(20);

    let writer_settings = WriterSettings::new(symbol.clone(), writer, writer_rx_ch);
    tokio::spawn(writer_task(writer_settings));

    let (reader_tx_ch, reader_rx_ch) = broadcast::channel(10);
    let mut reader_rx_ch_2 = reader_tx_ch.subscribe();

    let reader_settings = ReaderSettings::new(symbol.clone(), reader, reader_tx_ch);
    tokio::spawn(reader_task(reader_settings));

    writer_tx_ch.send(websocket_payload_init).await;
    // tokio::spawn(async move {log::debug!("reader_rx_ch_2 {:?}", reader_rx_ch_2.recv().await);});
    
    let (output_tx_ch, mut output_rx_ch) =  broadcast::channel(10);

    let deserialize_settings = DeserializeSettings::new(symbol.clone(), deserialize_stream, reader_rx_ch, output_tx_ch, writer_tx_ch.clone());
    tokio::spawn(stream_management_task(deserialize_settings));
    log::error!("Bitstamp  url{:?}", snapshot_url.as_str());
    log::error!("Bitstamp Snapshot {:?}", deserialize_snapshot(symbol.clone(), get_snapshot(snapshot_url).await.unwrap()));
    // while let Ok(message) = output_rx_ch.recv().await{
    //     log::debug!("{:?}", message);
    //     // println!("{:?}", message)
    // }

}
#[tokio::main]
async fn main() {
    setup_log();
    println!("Hello, world!");
    //binance_stream_init().await;
    bitstamp_stream_init().await;

}
