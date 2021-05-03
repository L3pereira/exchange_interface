use std::{
    collections::{BTreeMap, VecDeque},
    pin::Pin
};
use futures_util::{
    stream::{Stream},
    sink::{Sink},
    task::{Context, Poll}
};
use tokio::{
    sync::{oneshot, watch, broadcast, mpsc},
    time::{sleep, Duration}
};
use tokio_tungstenite::{
    tungstenite::{
        protocol::Message,
        error:: {
            Error as WsError,
            CapacityError
        }
    },  
};
#[derive(Debug)]
pub struct MockWebSocketStream{
    pub r_buffer: broadcast::Receiver<Message>,
    pub w_buffer: broadcast::Sender<Message>
}

impl MockWebSocketStream{
    pub fn new(r_buffer:  broadcast::Receiver<Message>, 
        w_buffer: broadcast::Sender<Message>) -> Self {
        MockWebSocketStream{
            r_buffer: r_buffer,
            w_buffer: w_buffer
        }
    }
}

impl Stream for MockWebSocketStream
{
    type Item = Result<Message, WsError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Check to see if we've finished counting or not.

        match self.r_buffer.try_recv(){
            Ok(value) =>  {
                Poll::Ready(Some(Ok(value)))
            },
            Err(err) => { 
                Poll::Ready(None) } 
        }
    }
}

impl Sink<Message> for MockWebSocketStream
{
    type Error = WsError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        match  self.w_buffer.send(item) {
            Ok(val) => Ok(()),
            Err(err) =>  Err(Self::Error::Capacity(CapacityError ::TcpBufferFull))
        } 
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
