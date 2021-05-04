// use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};
//use orderbook::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
// use orderbook::{Summary};

// #[derive(Default)]
// pub struct OrderbookService {}
// #[tonic::async_trait]
// impl OrderbookAggregator for OrderbookService {
// // Specify the output of rpc call
//     type SendStreamStream=mpsc::Receiver<Result<Summary, Status>>;
// // implementation for rpc call
//     async fn book_summary(
//         &self, request: Request<Empty>,
//     ) -> Result<Response<Self::SendStreamStream>, Status> {
// // creating a queue or channel
//         let (mut tx, rx) = mpsc::channel(4);
// // creating a new task
//         tokio::spawn(async move {
// // looping and sending our response using stream
//             for _ in 0..4{
// // sending response to our channel
//                 tx.send(Ok(Summary {
//                     message: format!("hello"),
//                 }))
//                 .await;
//             }
//         });
// // returning our reciever so that tonic can listen on reciever and send the response to client
//         Ok(Response::new(rx))
//     }
// }



