use common::{
    DepthData,
    Symbol  
};
use tokio_tungstenite::{
    tungstenite::protocol::Message,
    tungstenite::error::Error as WsError
};
use tokio::sync::{mpsc, broadcast};
use futures_util::{
    stream::{SplitSink, SplitStream, Stream},
    sink::Sink
};

#[derive(Debug)]
pub struct ReaderSettings<S>
where S: Stream<Item=Result<Message, WsError>> + Unpin {
    pub symbol: Symbol,
    pub websocket_reader: SplitStream<S>,
    pub output_tx_ch: broadcast::Sender<Message>,
}
impl<S> ReaderSettings<S>
    where  S: Stream<Item=Result<Message, WsError>> + Unpin {

    pub fn new(symbol: Symbol, websocket_reader: SplitStream<S>, 
        output_tx_ch: broadcast::Sender<Message>) -> Self {
        ReaderSettings{
            symbol: symbol,
            websocket_reader: websocket_reader,
            output_tx_ch: output_tx_ch
        }
    }
}

#[derive(Debug)]
pub struct WriterSettings<S>
    where S: Sink<Message> + Unpin
{
    pub symbol: Symbol,
    pub websocket_writer: SplitSink<S, Message>,
    pub input_rx_ch: mpsc::Receiver<Message>
}
impl<S> WriterSettings<S>
    where  S: Sink<Message> + Unpin{

    pub fn new(symbol: Symbol, websocket_writer: SplitSink<S, Message>, 
        input_rx_ch:mpsc::Receiver<Message>) -> Self {
        WriterSettings{
            symbol: symbol,
            websocket_writer: websocket_writer,
            input_rx_ch: input_rx_ch,

        }
    }
}

#[derive(Debug)]
pub struct DeserializeSettings

{
    pub symbol: Symbol,
    pub input_rx_ch: broadcast::Receiver<Message>, 
    pub output_tx_ch: broadcast::Sender<DepthData>, 
    pub writer_tx_ch: mpsc::Sender<Message>
}
impl DeserializeSettings
 {

    pub fn new( 
        symbol: Symbol,
        input_rx_ch: broadcast::Receiver<Message>, 
        output_tx_ch: broadcast::Sender<DepthData>, 
        writer_tx_ch: mpsc::Sender<Message>) -> Self {
            DeserializeSettings{
                symbol: symbol,
                input_rx_ch: input_rx_ch, 
                output_tx_ch: output_tx_ch, 
                writer_tx_ch: writer_tx_ch

        }
    }
}