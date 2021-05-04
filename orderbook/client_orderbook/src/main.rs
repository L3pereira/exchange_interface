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

    let mut client = OrderbookAggregatorClient::connect("http://[::1]:50051").await?;

    // let response = client.book_summary(Request::new(Empty)).await?;
    let mut stream = client.book_summary(Request::new(Empty{})).await?.into_inner();
   
    while let Some(data) = stream.message().await? {
        println!("\n\nPrice|Amount|Exchange");
        for ask in data.asks.iter(){
            println!("{:?}|{:?}|{:?}", ask.price, ask.amount, ask.exchange);
        }
        println!("\nSpread---------{:?}-------------\n", data.spread);
        for bid in data.bids.iter().rev(){
            println!("{:?}|{:?}|{:?}", bid.price, bid.amount, bid.exchange);
        }
        println!("\n----------------------------------------------------\n");
    }
    Ok(())
}
