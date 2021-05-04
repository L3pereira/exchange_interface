mod orderbook;
mod orderbook_service;
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
use tokio_stream::wrappers::ReceiverStream;
use common::*;
use gateway_in::{
    reader_task,
    writer_task,
    get_snapshot,
    exchanges_services,
    settings::*
};
use futures_util::StreamExt;
use rust_decimal::Decimal;
use aggregated_order_book::{Level as PriceLevel, AggregatedBook};
// use orderbook_service::{OrderbookService};
// use orderbook_service::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
use tonic::{transport::Server, Request, Response, Status};
use orderbook::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
use orderbook::{Summary, Level ,Empty};

const CONFIG_PATH: &str = "config.json"; 
const LOG_CONFIG_PATH: &str = "log_config.yaml";
use std::sync::Once;
static INIT: Once = Once::new();
pub fn setup_log() -> () { 
    INIT.call_once(|| {
        log4rs::init_file(LOG_CONFIG_PATH, Default::default()).unwrap();
    });
}

async fn binance_stream_init(output_stream_tx_ch: broadcast::Sender<SnapshotData>) {
    use exchanges_services::binance::*;
    let mut the_file = File::open(CONFIG_PATH).expect("file should open read only:\n ");
    let mut buff = String::new(); 
    the_file.read_to_string(&mut buff).unwrap();
    let binance_config =  BinanceConfig::new(buff).expect("binance_config:\n ");
    let symbol = binance_config.symbols.get(0).expect("symbol:\n ");
    let web_socket_url = binance_config.websocket_urls.get(symbol).expect("websocket_urls:\n ").clone();
    let snapshot_url = binance_config.snapshot_urls.get(symbol).expect("snapshot_urls:\n ").clone();
    
    let (ws_stream, _) = match connect_async(web_socket_url).await{
        Ok(result) => result,
        Err(err) => {log::error!("\nError in Binance Stream init:\n {:?}", err); return}
    };

    let (writer, reader) = ws_stream.split();

    let (writer_tx_ch, writer_rx_ch): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(20);

    let writer_settings = WriterSettings::new(symbol.clone(), writer, writer_rx_ch);
    tokio::spawn(writer_task(writer_settings));

    let (reader_tx_ch, reader_rx_ch) = broadcast::channel(10);
    // let mut reader_rx_ch_2 = reader_tx_ch.subscribe();

    let reader_settings = ReaderSettings::new(symbol.clone(), reader, reader_tx_ch);
    tokio::spawn(reader_task(reader_settings));

    // tokio::spawn(async move {log::debug!("reader_rx_ch_2 {:?}", reader_rx_ch_2.recv().await);});
    
    let (output_tx_ch, mut output_rx_ch) =  broadcast::channel(10);

    let deserialize_settings = DeserializeSettings::new(symbol.clone(), deserialize_stream, reader_rx_ch, output_tx_ch, writer_tx_ch);
    tokio::spawn(stream_management_task(deserialize_settings));
    // log::error!("Binance  url{:?}", snapshot_url.as_str());
    // log::error!("Binance Snapshot{:?}", deserialize_snapshot(symbol.clone(), get_snapshot(snapshot_url).await.unwrap()));
    // while let Ok(message) = output_rx_ch.recv().await{
    //     log::debug!("{:?}", message);
    //     // println!("{:?}", message)
    // }
    let mut snapshot_message = deserialize_snapshot(symbol.clone(), get_snapshot(snapshot_url.clone()).await.unwrap()).unwrap();
    let mut is_first_event = true;
    let mut previuos_event_last_timestamp:u64 = 0;
    // log::info!("snapshot_message\n\n{:?}\n\n", snapshot_message);       

    while let Ok(message) = output_rx_ch.recv().await{
        if is_first_event && message.last_update_id_timestamp <= snapshot_message.timestamp {
            continue;   
            // log::debug!("1\n\n{:?}-{:?}-{:?}\n\n", message.first_update_id_timestamp, 
            // message.last_update_id_timestamp ,snapshot_message.timestamp);      
        }
        else if is_first_event && (message.first_update_id_timestamp <= (snapshot_message.timestamp + 1)) 
            && (message.last_update_id_timestamp >= (snapshot_message.timestamp + 1)){             
                // log::debug!("2\n\n{:?}-{:?}-{:?}\n\n", message.first_update_id_timestamp, 
                // message.last_update_id_timestamp ,snapshot_message.timestamp);  
            is_first_event = false;
            previuos_event_last_timestamp = message.last_update_id_timestamp;
        }
        else if is_first_event || message.first_update_id_timestamp != previuos_event_last_timestamp{
            // log::debug!("3\n\n{:?}-{:?}-{:?}\n\n", message.first_update_id_timestamp, 
            // message.last_update_id_timestamp ,snapshot_message.timestamp); 
            snapshot_message = deserialize_snapshot(symbol.clone(), get_snapshot(snapshot_url.clone()).await.unwrap()).unwrap();
            is_first_event = true;
            continue;
        }
        // log::debug!("4\n\n{:?}-{:?}-{:?}\n\n", message.first_update_id_timestamp, 
        // message.last_update_id_timestamp ,snapshot_message.timestamp);

        for (price, volume) in message.ask_to_update.into_iter(){
            if volume == Decimal::new(0,0) {
                snapshot_message.ask_to_update.remove_entry(&price);           
            }
            else{
                snapshot_message.ask_to_update.entry(price).or_insert(volume);
            }
        }
        for (price, volume) in message.bid_to_update.into_iter(){
            if volume == Decimal::new(0,0) {
                snapshot_message.bid_to_update.remove_entry(&price);           
            }
            else{
                snapshot_message.bid_to_update.entry(price).or_insert(volume);
            }             
        }

        output_stream_tx_ch.send(snapshot_message.clone());
    }
}
async fn bitstamp_stream_init(output_stream_tx_ch: broadcast::Sender<SnapshotData> ) {

    use exchanges_services::bitstamp::*;
    let mut the_file = File::open(CONFIG_PATH).expect("file should open read only:\n ");
    let mut buff = String::new(); 
    the_file.read_to_string(&mut buff).unwrap();
    let bitstamp_config =  BitstampConfig::new(buff).expect("bitstamp_config:\n ");
 
    let symbol = bitstamp_config.symbols.get(0).expect("symbol:\n ");
    let websocket_base_url = bitstamp_config.websocket_url;
    let websocket_payload_init = bitstamp_config.websocket_payloads.get(symbol).expect("websocket_urls:\n ").clone();
    let snapshot_url = bitstamp_config.snapshot_urls.get(symbol).expect("snapshot_urls:\n ").clone();

    let (ws_stream, _) = match connect_async(websocket_base_url).await{
        Ok(result) => result,
        Err(err) => {log::error!("\nError in Bitstamp Stream init:\n {:?}", err); return}
    };
    let (writer, reader) = ws_stream.split();

    let (writer_tx_ch, writer_rx_ch): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(20);

    let writer_settings = WriterSettings::new(symbol.clone(), writer, writer_rx_ch);
    tokio::spawn(writer_task(writer_settings));

    let (reader_tx_ch, reader_rx_ch) = broadcast::channel(10);
    //let mut reader_rx_ch_2 = reader_tx_ch.subscribe();

    let reader_settings = ReaderSettings::new(symbol.clone(), reader, reader_tx_ch);
    tokio::spawn(reader_task(reader_settings));

    writer_tx_ch.send(websocket_payload_init).await;
    // tokio::spawn(async move {log::debug!("reader_rx_ch_2 {:?}", reader_rx_ch_2.recv().await);});
    
    let (output_tx_ch, mut output_rx_ch) =  broadcast::channel(10);

    let deserialize_settings = DeserializeSettings::new(symbol.clone(), deserialize_stream, reader_rx_ch, output_tx_ch, writer_tx_ch);
    tokio::spawn(stream_management_task(deserialize_settings));
    // log::error!("Bitstamp Snapshot {:?}", deserialize_snapshot(symbol.clone(), get_snapshot(snapshot_url).await.unwrap()));
    //let mut snapshot_message = deserialize_snapshot(symbol.clone(), get_snapshot(snapshot_url.clone()).await.unwrap()).unwrap();


    //log::debug!("{:?}", snapshot_message);
    while let Ok(message) = output_rx_ch.recv().await{
 
        let snapshot_data = SnapshotData{
            exchange: message.exchange,
            symbol: message.symbol,
            timestamp: message.last_update_id_timestamp,
            bid_to_update: message.bid_to_update,
            ask_to_update: message.ask_to_update
        };
        output_stream_tx_ch.send(snapshot_data);
        //output_stream_tx_ch.send()

        // println!("{:?}", message)
    }

}
fn set_response_stream(agrregate_book_result: &mut AggregatedBook, exchange: Exchange) -> Summary{
    let asks = agrregate_book_result.get_top_asks(10);
    let bids = agrregate_book_result.get_top_bids(10);
    let spread = asks.iter().next().unwrap().price - bids.iter().next().unwrap().price;
    let mut level_asks = Vec::new();
    let mut level_bids = Vec::new();
    for ask in asks.iter().rev() {
        level_asks.push(Level{
            exchange: ask.exchange.to_string(),
            price: ask.price.to_string().parse::<f64>().unwrap(),
            amount: ask.volume.to_string().parse::<f64>().unwrap(),
        })
    }
    for bid in bids.iter().rev() {
        level_bids.push(Level{
            exchange: bid.exchange.to_string(),
            price: bid.price.to_string().parse::<f64>().unwrap(),
            amount: bid.volume.to_string().parse::<f64>().unwrap(),
        })
    }
    Summary {
        spread: spread.to_string().parse::<f64>().unwrap(),
        asks: level_asks,
        bids: level_bids,
    }
}

#[derive(Default)]
pub struct OrderbookService {}
#[tonic::async_trait]
impl OrderbookAggregator for OrderbookService {
// Specify the output of rpc call
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;
// implementation for rpc call
    async fn book_summary(&self, request: Request<Empty>) -> Result<Response<Self::BookSummaryStream>, Status> {
// creating a queue or channel
        let (mut tx, rx) = mpsc::channel(4);
        let (binance_output_tx_ch, mut binance_output_rx_ch) =  broadcast::channel(10);
        let (bitstamp_output_tx_ch, mut bitstamp_output_rx_ch) =  broadcast::channel(10);

        tokio::spawn(binance_stream_init(binance_output_tx_ch));
        tokio::spawn(bitstamp_stream_init(bitstamp_output_tx_ch));

        tokio::spawn(async move {
            let mut agrregate_book_result = AggregatedBook::new();
            loop{                
                tokio::select! {
                    val = binance_output_rx_ch.recv() => {                 
                        match val {
                            Ok(snap_shot)=> agrregate_book_result.update_book(snap_shot),
                            Err(err)=> {log::error!("\nError in Binance  :\n {:?}", err); return}        
                        };
                        let response = set_response_stream(&mut agrregate_book_result, Exchange::Binance);
                        tx.send(Ok(response)).await;        
                    }
                    val = bitstamp_output_rx_ch.recv() => {
                        match val {
                            Ok(snap_shot)=> agrregate_book_result.update_book(snap_shot),
                            Err(err)=> {log::error!("\nError in Bitstamp  :\n {:?}", err); return}        
                        };
                        let response= set_response_stream(&mut agrregate_book_result, Exchange::Bitstamp);
                        tx.send(Ok(response)).await;
                    }
                }
            }
        });
// returning our reciever so that tonic can listen on reciever and send the response to client
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() 
-> Result<(), Box<dyn std::error::Error>> 
{
// defining address for our service
    let addr = "[::1]:50051".parse().unwrap();
// creating a service
    let orderbook_service = OrderbookService::default();
    println!("Server listening on {}", addr);
// adding our service to our server.
    Server::builder()
        .add_service(OrderbookAggregatorServer::new(orderbook_service))
        .serve(addr)
        .await?;
    Ok(())
}

