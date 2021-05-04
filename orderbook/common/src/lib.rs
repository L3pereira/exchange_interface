// use futures_util::{
//     SinkExt, 
//     StreamExt,
//     stream::{SplitSink, SplitStream}
// };
// use tokio_tungstenite::{accept_async, client_async, connect_async, tungstenite::protocol::Message, WebSocketStream};
// use tokio::{
//     io::{AsyncRead, AsyncWrite},
//     net::{TcpListener, TcpStream},
//     sync::{oneshot, watch, broadcast, mpsc}
// };
use std::{
    str::FromStr,
    collections::{BTreeMap}
};
// use derive_more::{Display, Add, Sub, Mul, Div, Sum, Product};
use rust_decimal::Decimal;

//#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Display, Add, Sub, Mul, Div, Sum)]
pub type ErrCode = i32 ;

//#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Display)]
pub type ErrMsg = String;

//#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Display)]
pub type Symbol = String;

// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Display, Add, Sub, Mul, Div, Sum)]
pub type Price = Decimal;

//#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Display, Add, Sub, Mul, Div, Sum)]
pub type Volume = Decimal; 

//#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Display, Add, Sub, Mul, Div, Sum)]
pub type FirstUpdateIdTimestamp = u64;

//#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Display, Add, Sub, Mul, Div, Sum)]
pub type LastUpdateIdTimestamp =u64 ;

pub type Timestamp = u64;


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Exchange {
     Binance, Bitstamp
}
impl FromStr for Exchange {
    type Err = (); 
    fn from_str(input: &str) -> Result<Exchange, Self::Err> {
        match input {
            "Binance"  =>  Ok(Exchange::Binance),
            "binance"  =>  Ok(Exchange::Binance),
            "Bitstamp"  => Ok(Exchange::Bitstamp),
            "bitstamp"  => Ok(Exchange::Bitstamp),
            _      => Err(()),
        }
    }
}
// #[derive(Clone, PartialEq, Eq, Display)]
// #[display(fmt = "ErrorMessage {{code: {}, message: {} }}", code, message)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorMessage{
    pub code: ErrCode,
    pub message: ErrMsg
}
impl ErrorMessage{
    pub fn new(code: ErrCode, message: ErrMsg) -> Self {
        ErrorMessage{
            code:code,
            message: message
        }
    }
}

 
// #[derive(Clone, Display)]
// #[display(fmt = "DepthData {{}}")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepthData {
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub first_update_id_timestamp: FirstUpdateIdTimestamp,
    pub last_update_id_timestamp: LastUpdateIdTimestamp,
    pub bid_to_update: BTreeMap<Price, Volume>,
    pub ask_to_update: BTreeMap<Price, Volume>

}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotData {
    pub exchange: Exchange, 
    pub symbol: Symbol,
    pub timestamp: Timestamp,
    pub bid_to_update: BTreeMap<Price, Volume>,
    pub ask_to_update: BTreeMap<Price, Volume>

}
