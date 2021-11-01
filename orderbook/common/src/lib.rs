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
pub mod binance_config_utils;
pub mod bitstamp_config_utils;
#[cfg(test)]
mod tests;

use std::{
    str::FromStr,
    collections::BTreeMap,
    io::Read,
    fmt,
    fs::File
};
use url::Url;
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, de::Error};

pub use binance_config_utils::*;
pub use bitstamp_config_utils::*;


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


#[derive(Deserialize)]
#[derive(Clone, Debug)]
pub struct ExchangesConfig {
    pub binance: BinanceConfig,
    pub bitstamp: BitstampConfig,
    pub grpc_server: String,
    pub web_server: String,
    pub client_websocket: String,
}


fn to_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Url::parse(s).map_err(D::Error::custom)
}

fn to_upper_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Vec<String> = Deserialize::deserialize(deserializer)?;
    Ok(s.into_iter().map(|x| x.to_uppercase()).collect())
}

pub fn setup_config(path: &str) -> Result<ExchangesConfig> {
    // let task_name = "--setup_config--";
    let mut file = File::open(path)?;
    // let mut file = match File::open(path){
    //     Ok(file) => file,
    //     Err(err) => {
    //         log::error!("Error in {:?}\nFile open:\n{:?}", task_name, err);
    //         panic!("Error in {:?}\nFile open:\n{:?}", task_name, err)
    //     }
    // };
    let mut buff = String::new(); 
    file.read_to_string(&mut buff)?;
    // if let Err(err) = file.read_to_string(&mut buff){
    //     log::error!("Error in {:?}\nFile Read To String :\n{:?}", task_name, err);
    //     panic!("Error in {:?}\nFile Read To String :\n{:?}", task_name, err)

    // };

    // let config = match WebAppConfig::new(buff){
    //     Ok(config) => config,
    //     Err(err) => {
    //         log::error!("Error in {:?}\nDeserialize config :\n{:?}", task_name, err);
    //         panic!("Error in {:?}\nDeserialize config :\n{:?}", task_name, err);
    //     }
    // };
    let config: ExchangesConfig = serde_json::from_str(&buff)
        .context("JSON was not well-formatted config binance")?;

    Ok(config)
}