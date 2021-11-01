mod orderbook;
use tokio::{
    sync::{oneshot, watch, broadcast, mpsc},
    time::{sleep, Duration}
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};
use orderbook::{Summary, Level ,Empty};
use orderbook::orderbook_aggregator_client::{OrderbookAggregatorClient};



#[tokio::main] 
async fn main() ->Result<(), Box<dyn std::error::Error>>{

    let mut client = OrderbookAggregatorClient::connect("http://127.0.0.1:50051").await?;

    // let response = client.book_summary(Request::new(Empty)).await?;
    let mut stream = client.book_summary(Request::new(Empty{})).await?.into_inner();
   
    while let Some(data) = stream.message().await? {
        println!("\n\nPrice|Amount|Exchange");
        for ask in data.asks.iter(){
            println!("{:?}|{:?}|{:?}", ask.price, ask.amount, ask.exchange);
        }
        println!("\nSpread--------({:?})-------------\n", data.spread);
        let bin_ask = data.asks.iter().filter_map(|x| if x.exchange == "Binance" {Some(x.price)}else{None}).rev().collect::<Vec<f64>>();
        let bin_bid = data.bids.iter().filter_map(|x| if x.exchange == "Binance" {Some(x.price)}else{None}).rev().collect::<Vec<f64>>();
        let bit_ask = data.asks.iter().filter_map(|x| if x.exchange == "Bitstamp" {Some(x.price)}else{None}).rev().collect::<Vec<f64>>();
        let bit_bid = data.bids.iter().filter_map(|x| if x.exchange == "Bitstamp" {Some(x.price)}else{None}).rev().collect::<Vec<f64>>();
        
        if bin_ask.get(0).is_some() && bin_bid.get(0).is_some(){
            let spread = bin_ask.get(0).unwrap() - bin_bid.get(0).unwrap();
            println!("\nBin Spread--------({:?})----({:?})--({:?})-------\n",spread, bin_ask.get(0).unwrap(), bin_bid.get(0).unwrap());
        }
        if bit_ask.get(0).is_some() && bit_bid.get(0).is_some(){
            let spread = bit_ask.get(0).unwrap() - bit_bid.get(0).unwrap();
            println!("\nBit Spread--------({:?})----({:?})--({:?})-------\n",spread, bit_ask.get(0).unwrap(), bit_bid.get(0).unwrap());
        }
        for bid in data.bids.iter().rev(){
            println!("{:?}|{:?}|{:?}", bid.price, bid.amount, bid.exchange);
        }
        println!("\n----------------------------------------------------\n");
    }
    Ok(())
}
