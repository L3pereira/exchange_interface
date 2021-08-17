#![deny(
    //   missing_docs,
      trivial_casts,
      trivial_numeric_casts,
      unsafe_code,
      unused_import_braces,
      unused_qualifications,
      warnings
  )]
 

//! This crate provides common types for gateway_in crates and order_book_server

use std::fmt;
use std::{
    str::FromStr,
    collections::BTreeMap
};
use anyhow::Result;
use rust_decimal::Decimal;



/// ErrCode
pub type ErrCode = i32 ;
/// ErrMsg
pub type ErrMsg = String;
/// Tick Symbol of an intrument
pub type Symbol = String;
/// Price of the instrument
pub type Price = Decimal;
/// Volume at level
pub type Volume = Decimal;
/// the moment in time when the data arrives
pub type FirstUpdateIdTimestamp = u64;
/// A margin where the snapshot data is valid
pub type LastUpdateIdTimestamp =u64 ;
/// Timestamp
pub type Timestamp = u64;

/// Exchange
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

impl fmt::Display for Exchange {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
            Exchange::Binance => write!(f, "Binance"),
            Exchange::Bitstamp => write!(f, "Bitstamp"),

       }
    }
}
/// ErrorMessage
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

/// DepthData
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepthData {
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub first_update_id_timestamp: FirstUpdateIdTimestamp,
    pub last_update_id_timestamp: LastUpdateIdTimestamp,
    pub bid_to_update: BTreeMap<Price, Volume>,
    pub ask_to_update: BTreeMap<Price, Volume>

}
 /// SnapshotData
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotData {
    pub exchange: Exchange, 
    pub symbol: Symbol,
    pub timestamp: Timestamp,
    pub bid_to_update: BTreeMap<Price, Volume>,
    pub ask_to_update: BTreeMap<Price, Volume>

}

