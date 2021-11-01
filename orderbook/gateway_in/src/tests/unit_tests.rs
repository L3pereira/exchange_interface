use tokio::{
    sync::{broadcast, mpsc}
};
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::protocol::Message;
use pretty_assertions::assert_eq;
use crate::settings::{ReaderSettings, WriterSettings};
use super::mocks::MockWebSocketStream;

#[tokio::test]
async fn test_reader_task() {
    // super::setup();
    let (r_sender, r_receiver) = broadcast::channel(3);
    let (w_sender, _) = broadcast::channel(3);
    let _ = r_sender.send(Message::Text("Msg 1".to_string()));
    let _ = r_sender.send(Message::Text("Msg 2".to_string()));
    let _ = r_sender.send(Message::Text("Msg 3".to_string()));
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
async fn test_writer_task() {
    // super::setup();
    
    let (_, r_receiver) =  broadcast::channel(3);
    let (w_sender, mut w_receiver)=  broadcast::channel(3); 
    let stream = MockWebSocketStream::new(r_receiver, w_sender);
    let (writer, _) = stream.split();
    let (output_tx_ch, input_rx_ch): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(20);
    
    let settings = WriterSettings::new("BNBBTC".to_string(), writer, input_rx_ch);
    
    tokio::spawn(crate::writer_task(settings));
    let _ = output_tx_ch.send(Message::Text("Msg 1".to_string())).await;
    let _ = output_tx_ch.send(Message::Text("Msg 2".to_string())).await;
    let _ = output_tx_ch.send(Message::Text("Msg 3".to_string())).await;
    
    assert_eq!(w_receiver.recv().await, Ok(Message::Text("Msg 1".to_string())));
    assert_eq!(w_receiver.recv().await, Ok(Message::Text("Msg 2".to_string())));
    assert_eq!(w_receiver.recv().await, Ok(Message::Text("Msg 3".to_string())));
}


