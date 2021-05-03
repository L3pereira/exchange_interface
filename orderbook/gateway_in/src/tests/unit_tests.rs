use std::{
    str::FromStr,
    collections::{BTreeMap, VecDeque},
    net::SocketAddr,
    cmp::min,
    pin::Pin
    //time::Duration
};
use url::Url;
use futures_util::{
    io::{AsyncRead, AsyncWrite},    
    SinkExt, 
    StreamExt,
    stream::{SplitSink, SplitStream,  Stream},
    sink::{Sink},
    task::{Context, Poll}
};
use tokio::{
    sync::{oneshot, watch, broadcast, mpsc},
    time::{sleep, Duration}
};
use tokio_tungstenite::{
    accept_async, client_async, connect_async, 
    tungstenite::{
        protocol::Message,
        error:: {
            CapacityError,
            Error as WsError
        }
    },
    WebSocketStream
    
};
use pretty_assertions::{assert_eq, assert_ne};
use log::{debug, error, info, warn};
use rust_decimal::Decimal;
use common::{
    DepthData,
    ErrorMessage,
    Price,
    Volume
};
use crate::settings::{ReaderSettings, WriterSettings, DeserializeSettings};
use super::mocks::{MockWebSocketStream};

#[tokio::test]
async fn reader_task_test() {
    super::setup();
    let (r_sender, r_receiver) = broadcast::channel(3);
    let (w_sender, _) = broadcast::channel(3);
    r_sender.send(Message::Text("Msg 1".to_string()));
    r_sender.send(Message::Text("Msg 2".to_string()));
    r_sender.send(Message::Text("Msg 3".to_string()));
    let stream = MockWebSocketStream::new(r_receiver, w_sender);
    let (_, reader) = stream.split();

    let (output_tx_ch, mut input_rx_ch) = broadcast::channel(10);
    let settings = ReaderSettings::new("BNBBTC".to_string(), reader, output_tx_ch);
 
    tokio::spawn(crate::reader_task(settings));

    assert_eq!(input_rx_ch.recv().await, Ok(Message::Text("Msg 1".to_string())));
    assert_eq!(input_rx_ch.recv().await, Ok(Message::Text("Msg 2".to_string())));
    assert_eq!(input_rx_ch.recv().await, Ok(Message::Text("Msg 3".to_string())));
}
#[tokio::test]
async fn writer_task_test() {
    super::setup();
    
    let (_, r_receiver) =  broadcast::channel(3);
    let (w_sender, mut w_receiver)=  broadcast::channel(3); 
    let stream = MockWebSocketStream::new(r_receiver, w_sender);
    let (writer, _) = stream.split();
    let (output_tx_ch, input_rx_ch): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(20);
    
    let settings = WriterSettings::new("BNBBTC".to_string(), writer, input_rx_ch);
    
    tokio::spawn(crate::writer_task(settings));
    output_tx_ch.send(Message::Text("Msg 1".to_string())).await;
    output_tx_ch.send(Message::Text("Msg 2".to_string())).await;
    output_tx_ch.send(Message::Text("Msg 3".to_string())).await;
    
    assert_eq!(w_receiver.recv().await, Ok(Message::Text("Msg 1".to_string())));
    assert_eq!(w_receiver.recv().await, Ok(Message::Text("Msg 2".to_string())));
    assert_eq!(w_receiver.recv().await, Ok(Message::Text("Msg 3".to_string())));
}



// #[tokio::test]
// async fn get_snapshot_test(){
//     let url = Url::parse("https://api.binance.com/api/v3/depth?symbol=BNBBTC&limit=5").unwrap();
//     let body = crate::get_snapshot(url).await.unwrap();
//     println!("Body:\n{}", body);


//     assert_eq!(1,1);
    
// }
// #[tokio::test]
// async fn it_works() {
//     //"wss://echo.websocket.org"
//     //"wss://stream.binance.com:9443/ws/btcusdt@aggTrade"
//     //let url = Url::parse("wss://echo.websocket.org").expect("Failed to parse url");
//     println!("Hello, World! sent to channel 1 ");
    
//     let url = Url::parse("wss://stream.binance.com:9443/ws/ethbtc@depth@100ms").expect("Failed to parse url");


//     let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
//     //let tcp = TcpStream::connect("174.129.224.73:80").await.expect("Failed to connect");
//     // let tcp = TcpStream::connect("52.68.7.206:9443").await.expect("Failed to connect");
//     //let (ws_stream, _) = client_async(url, tcp).await.expect("Client failed to connect");
//     let (write, mut read) = ws_stream.split();

//     println!("Hello, World! sent to channel 2 ");


//     // let (mpsc_tx, mpsc_rx) = mpsc::channel(20);
//     // let (b_tx, b_rx) = broadcast::channel(20);
//     // let w = run_connection_writer(write, mpsc_rx);
//     // let r = run_connection_reader(read, b_tx);
//     // let router = run_router(mpsc_tx.clone(), b_rx);
//     // tokio::spawn(router);
//     // tokio::spawn(w);
//     // tokio::spawn(r);

//     // println!("Hello, World! sent to channel");
//     let text = Message::text("Hello, World!");
//     let close = Message::Close(None);
//     // mpsc_tx.clone().send(text).await;
    
//     // println!("waiting 10");
//     // sleep(Duration::from_secs(5)).await;

//     // mpsc_tx.clone().send(close).await;

//     // sleep(Duration::from_secs(5)).await;
//     while let Some(message) = read.next().await {      
//         match message {
//             Ok(message) => {
//                 println!("Reader Task {}", message);
//             },
//             Err(err) => println!("Reader Task error{}", err)
//         }       
//     }
//     // println!("{:?}", response);
//     // runtime.block_on(echo_future).unwrap();
//     assert_eq!(2 + 2, 4);
// }