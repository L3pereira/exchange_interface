#![deny(
    //  missing_docs, // not compatible with big_array
      trivial_casts,
      trivial_numeric_casts,
      unsafe_code,
      unused_import_braces,
      unused_qualifications,
      warnings
  )]

mod orderbook;
mod orderbook_service;
mod aggregated_order_book;

#[cfg(test)]
mod tests;


use anyhow::Result;
use std::io::{ Error, ErrorKind};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use lazy_static::lazy_static;
use common::*;
use gateway_in::exchanges_services::{binance::*, bitstamp::*, ExchangeInit};
use crate::aggregated_order_book::AggregatedBook;
use tonic::{transport::Server, Request, Response, Status};
use orderbook::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
use orderbook::{Summary, Level ,Empty};

const CONFIG_PATH: &str = "../config.json"; 
const LOG_CONFIG_PATH: &str = "log_config.yaml";

use std::sync::Once;
static INIT: Once = Once::new();
pub fn setup_log() -> () { 
    INIT.call_once(|| {
        log4rs::init_file(LOG_CONFIG_PATH, Default::default()).unwrap();
    });
}

lazy_static! {
    static ref CONFIG: ExchangesConfig = match setup_config(CONFIG_PATH){
        Ok(config) => config,
        Err(err) => {
            log::error!("\n{:?}", err);
            panic!("\n{:?}", err);
        }
    };   
}

fn init( mut service: (impl ExchangeInit + 'static + Send), output_stream_tx_ch: broadcast::Sender<SnapshotData>)  {
    tokio::spawn(async move {service.stream_init_task(output_stream_tx_ch).await});
}

fn set_response_stream(agrregate_book_result: &mut AggregatedBook) -> Result<Summary> {
    
    let asks = agrregate_book_result.get_top_asks(20);
    let bids = agrregate_book_result.get_top_bids(20);
    let spot_ask = asks.get(0).ok_or(anyhow::Error::new(Error::from(ErrorKind::NotFound)))?; 
    let spot_bid = bids.get(0).ok_or(anyhow::Error::new(Error::from(ErrorKind::NotFound)))?; 
    let spread = spot_ask.price - spot_bid.price;
    log::trace!("\n{:?} = {:?} - {:?}", spread, spot_ask.price, spot_bid.price);
    let mut level_asks = Vec::new();
    let mut level_bids = Vec::new();
    for ask in asks.iter().rev() {
        level_asks.push(Level{
            exchange: ask.exchange.to_string(),
            price: ask.price.to_string().parse::<f64>()?,
            amount: ask.volume.to_string().parse::<f64>()?,
        })
    }
    for bid in bids.iter().rev() {
        level_bids.push(Level{
            exchange: bid.exchange.to_string(),
            price: bid.price.to_string().parse::<f64>()?,
            amount: bid.volume.to_string().parse::<f64>()?,
        })
    }
    let summary = Summary {
        spread: spread.to_string().parse::<f64>()?,
        asks: level_asks,
        bids: level_bids,
    };
    Ok(summary)
}

#[derive(Default)]
pub struct OrderbookService {}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookService {
// Specify the output of rpc call
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;
// implementation for rpc call
    async fn book_summary(&self, _: Request<Empty>) -> Result<Response<Self::BookSummaryStream>, Status> {
        // let task_name = "--book_summary Task--";

        let binance_service = BinanceService::new(CONFIG.binance.clone());
        let bitstamp_service = BitstampService::new(CONFIG.bitstamp.clone());

        let (tx, rx) = mpsc::channel(4);
        let (binance_output_tx_ch, mut binance_output_rx_ch) =  broadcast::channel(10);
        let (bitstamp_output_tx_ch, mut bitstamp_output_rx_ch) =  broadcast::channel(10);

        init(binance_service, binance_output_tx_ch);
        init(bitstamp_service, bitstamp_output_tx_ch);


        tokio::spawn(async move {
            let mut agrregate_book_result = AggregatedBook::new();
            loop{   
                
                tokio::select! {

                    val = bitstamp_output_rx_ch.recv() => {
                        match val {
                            Ok(snap_shot)=> {
                                agrregate_book_result.update_book(snap_shot);
                                match set_response_stream(&mut agrregate_book_result){
                                    Ok(response) => {let _ = tx.send(Ok(response)).await;},
                                    Err(err) => log::error!("\nError in Bitstamp  :\n {:?}", err)
                                };
                            },
                            Err(err)=> {log::error!("\nError in Bitstamp  :\n {:?}", err); return}        
                        };
                    }
                    val = binance_output_rx_ch.recv() => {                 
                        match val {
                            Ok(snap_shot)=> {
                                agrregate_book_result.update_book(snap_shot);
                                match set_response_stream(&mut agrregate_book_result){
                                    Ok(response) =>{let _ = tx.send(Ok(response)).await;},
                                    Err(err) => log::error!("\nError in Binance  :\n {:?}", err)
                                };
                            },
                            Err(err)=> {log::error!("\nError in Binance  :\n {:?}", err); return}        
                        };
       
                    }

                }
            }
        });
// returning our reciever so that tonic can listen on reciever and send the response to client
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}


#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> 
{
    // setup_log();
    let addr = CONFIG.grpc_server.parse()?;

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

