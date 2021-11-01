use std::time::{Duration, Instant};
use actix::prelude::*;
use actix_web_actors::ws;
use actix_web::Result;
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use crate::orderbook::Summary;
use actix::Message as ActixMessage;
// use serde::{Serialize, Deserialize};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize, ActixMessage)]
#[rtype(result = "()")]
pub struct OutputData{

    pub summary: Summary
    
}

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


pub struct FeedActor;

impl Actor for FeedActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_async::<SystemBroker, OutputData>(ctx);
     
    }
}

impl Handler<OutputData> for FeedActor {
    type Result = ();

    fn handle(&mut self, item: OutputData, _ctx: &mut Self::Context) {
  
        self.issue_async::<SystemBroker, _>(item);
    }
}


pub struct WsSession  {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
}

impl Actor for WsSession  {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        self.subscribe_async::<SystemBroker, OutputData>(ctx);


    }
}

impl Handler<OutputData> for WsSession {
    type Result = ();

    fn handle(&mut self, item: OutputData, ctx: &mut Self::Context) {

        let json = serde_json::to_string(&item.summary).unwrap_or("#r{}".to_owned());
        ctx.text(json);
    }
}
/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession  {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        // process websocket messages
        // println!("WS: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                if text == "PONG"{
                    // println!("WS PONG");
                    self.hb = Instant::now();
                }
                else{
                    ctx.text(text);
                }
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl WsSession  {
    pub fn new() -> Self {
        Self { hb: Instant::now() }
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
            // println!("WS PING");
            ctx.text("PING")
            // ctx.ping(b"");
        });
    }
}