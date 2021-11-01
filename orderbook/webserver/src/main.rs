#![deny(
    //  missing_docs, // not compatible with big_array
      trivial_casts,
      trivial_numeric_casts,
      unsafe_code,
      unused_import_braces,
      unused_qualifications,
      warnings
  )]
mod actix_actors;
mod orderbook;

use actix::prelude::*;
use lazy_static::lazy_static;
use actix_files as fs;
use actix_web_actors::ws;
use common::*;
use anyhow::Result;
// use actix_files::NamedFile;
// use std::path::PathBuf;
use actix_web::{web, App, HttpRequest, Error, HttpResponse, HttpServer, Result as ActixResult, middleware::Logger};
use actix_actors::{FeedActor, WsSession, OutputData};
use log::warn;
use std::sync::Once;
use tonic::Request;
use orderbook::{Summary, Empty};
use orderbook::orderbook_aggregator_client::OrderbookAggregatorClient;

const CONFIG_PATH: &str = "../config.json"; 
const LOG_CONFIG_PATH: &str = "log_config.yaml";
const PATH_SERVER: &str = "./www/dist/";
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

async fn ws_stream_rates(req: HttpRequest, stream: web::Payload) -> ActixResult<HttpResponse, Error> {
    warn!("Websocket Called");
    let resp = ws::start(WsSession::new(), &req, stream);
    resp
}

async fn data_stream(mut stream: tonic::Streaming<Summary>, feed_actor_addr: Addr<FeedActor>) -> Result<()> {
    warn!("Stream Called");
       
    while let Some(data) = stream.message().await? {
   
        let output_data = OutputData{
            summary: data
        };

        feed_actor_addr.do_send(output_data);
      
    }

    Ok(())
}

#[rustfmt::skip]
#[actix_web::main(flavor = "multi_thread")]
async fn main() ->  std::io::Result<()> {
    setup_log();
    std::env::set_var("RUST_LOG", "warn");
    std::env::set_var("RUST_BACKTRACE", "1");
    // env_logger::init();
    warn!("Server INIT");
    let web_server_add = CONFIG.web_server.clone();
    
    let grpc_server_add = format!("http://{}", CONFIG.grpc_server.clone());
    let mut client = match OrderbookAggregatorClient::connect(grpc_server_add).await {
        Ok(cli) => cli,
        Err(err) => {
            log::error!("\n{:?}", err);
            panic!("\n{:?}", err);
        }
    };
    let stream = match client.book_summary(Request::new(Empty{})).await {
        Ok(cli) => cli.into_inner(),
        Err(err) => {
            log::error!("\n{:?}", err);
            panic!("\n{:?}", err);
        }
    };
    let feed_actor_addr = FeedActor.start();
    tokio::spawn(data_stream(stream, feed_actor_addr));

    HttpServer::new(|| {
        let logger = Logger::default();
        App::new().wrap(logger)
        // .route("/index", web::get().to(index))
        .route("/rates", web::get().to(ws_stream_rates))
        .service(fs::Files::new("/", PATH_SERVER).index_file("index.html"))
    })
    .bind(web_server_add.clone())?
    .run()
    .await
}
