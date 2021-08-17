#![deny(
    //  missing_docs, // not compatible with big_array
      trivial_casts,
      trivial_numeric_casts,
      unsafe_code,
      unused_import_braces,
      unused_qualifications,
      warnings
  )]

pub mod settings;
pub mod exchanges_services;

#[cfg(test)]
mod tests;

use anyhow::{Context, Result};
use url::Url;
use settings::{ReaderSettings, WriterSettings};

use futures_util::{
    SinkExt, 
    StreamExt,
    stream::Stream,
    sink::Sink
};

use tokio_tungstenite::{
    tungstenite::protocol::Message,
    tungstenite::error::Error as WsError
};


async fn reader_task<S>(mut settings: ReaderSettings<S>) 
    where S: Stream<Item=Result<Message, WsError>> + Unpin
{
    let task_name = "--Reader Task--";
    log::info!("{:?} Init", task_name);   
    while let Some(message) = settings.websocket_reader.next().await {      
        match message {
            Ok(message) => {
                log::trace!("{:?}:\n{:?}", task_name, message);
                if let Err(err) = settings.output_tx_ch.send(message) {
                    log::error!("Error in {:?}\noutput_tx_ch:\n{:?}", task_name, err);
                    break;
                }
            },
            Err(err) => log::error!("Error in {:?}:\nReading message from stream:\n{:?}", task_name, err)
        }       
    }
    log::info!("{:?} End", task_name);
}

async fn writer_task<S>(mut settings: WriterSettings<S>) 
    where S: Sink<Message, Error= WsError>  + Unpin
{
    let task_name = "--Writer Task--";
    log::info!("{:?} Init", task_name);
    while let Some(message) = settings.input_rx_ch.recv().await { 
        if let  Err(err) = settings.websocket_writer.send(message.clone()).await {
            log::error!("Error in {:?}:\nSending message to stream:\n{:?}", task_name, err)
        }
        else{
            log::trace!("{:?}:\n{:?}", task_name, message);
        }
    }
    log::info!("{:?} End", task_name);

}

async fn get_snapshot(url: Url) -> Result<String>{
    let client = reqwest::Client::new();
    let request = client.get(url).send().await.context("Request snapshot error")?;
    let body = request.text().await.context("Request (body) snapshot error")?;
    Ok(body)
}




