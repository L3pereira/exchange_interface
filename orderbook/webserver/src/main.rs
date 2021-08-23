
use actix::prelude::*;
use actix_files as fs;
use actix_web_actors::ws;
use actix_files::NamedFile;
use actix_web::{get, web, App, HttpRequest, Error,  http::{header, StatusCode}, HttpResponse, HttpServer, Responder, Result, middleware::Logger, guard };
use std::path::PathBuf;
use log::{warn, info, trace};
use std::time::{Duration, Instant};
use actix_web::error::{ErrorBadGateway, ErrorInternalServerError};
mod orderbook;
use tokio::{
    sync::{oneshot, watch, broadcast, mpsc},
    // time::{sleep, Duration}
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};
use orderbook::{Summary, Level ,Empty};
use orderbook::orderbook_aggregator_client::{OrderbookAggregatorClient};


const path_server: &str = "./www/dist/";
/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

fn static_files(req: HttpRequest, file: &str) -> Result<HttpResponse> {
    let url = req.url_for("static_files_resource", &["index.html"])?; // <- generate url for "foo" resource
    warn!("url {:?}", url);
    Ok(HttpResponse::Found()
        .header(header::LOCATION, url.as_str())
        .finish())
}


async fn ws_index(r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    println!("{:?}", r);
    // match OrderbookAggregatorClient::connect("http://[::1]:50051").await{
    //     Ok(client) => {
    //         let res = ws::start(MyWebSocket::new(), &r, stream);
    //         println!("{:?}", res);
    //         res
    //     },
    //     Err(err) => Ok(HttpResponse::new(StatusCode::InternalServerError))
    // }
    let client =  OrderbookAggregatorClient::connect("http://[::1]:50051").await.unwrap();

    let res = ws::start(MyWebSocket::new(client), &r, stream);
    println!("{:?}", res);
    res

}
type ClientGRPC = OrderbookAggregatorClient<tonic::transport::Channel>;
/// websocket connection is long running connection, it easier
/// to handle with an actor
struct MyWebSocket {
    client: ClientGRPC,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}
/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        // process websocket messages
        
        // let fut = async move {
        //     let mut stream = self.client.book_summary(Request::new(Empty{})).await.unwrap().into_inner();

    

        // };
        // let fut = actix::fut::wrap_future::<_, Self>(fut);
        // ctx.spawn(fut);
        println!("WS: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl MyWebSocket {
    fn new(client : ClientGRPC) -> Self {
        Self { 
            client: client,
            hb: Instant::now() 
        }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}
async fn index(req: HttpRequest) -> Result<NamedFile> {
    warn!("Just Entered index");

    let my_path = path_server.to_owned() + "index.html";

    let path: PathBuf = my_path.parse().unwrap();
    warn!("Path {:?}", path);
    Ok(NamedFile::open(path)?)
}
#[rustfmt::skip]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_LOG", "warn");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    HttpServer::new(|| {
        let logger = Logger::default();
        App::new().wrap(logger)
        // .service(actix_web_static_files::ResourceFiles::new("/", generated))

        .route("/index", web::get().to(index))
        .route("/stream", web::get().to(ws_index))
        .service(fs::Files::new("/", "./www/dist/").index_file("index.html"))
        // .service(
        //     web::resource("/{a}")
        //         .name("static_files_resource") // <- set resource name, then it could be used in `url_for`
        //         .guard(guard::Get())
        //         .to(|| HttpResponse::Ok()),
        // )        

  






            // .service(         
            // web::scope("/app")
            // .service(index))

            // .service(fs::Files::new("/", "./www/dist").show_files_listing())

            // .route("/{name}", web::get().to(greet))

    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
