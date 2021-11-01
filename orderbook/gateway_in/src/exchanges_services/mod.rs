pub mod binance;
pub mod bitstamp;
use serde::{Serialize, Deserialize};
use url::Url;
use common::*;
use anyhow::Result;
use async_trait::async_trait;
use crate::settings::DeserializeSettings;
use rust_decimal::Decimal;
use tokio::sync::broadcast::{Sender, Receiver};
////////////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
struct OuterBinance {
    #[serde(alias = "s")]
    pub symbol: String,

    #[serde(alias = "U")]
    pub first_update_id_timestamp: u64,

    #[serde(alias = "u")]
    pub last_update_id_timestamp: u64,

    #[serde(alias = "b")]
    pub bid_to_update: Vec<Vec<Decimal>>,

    #[serde(alias = "a")]
    pub ask_to_update: Vec<Vec<Decimal>>
}

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OuterBinanceSnapshot { 
    #[serde(alias = "lastUpdateId")]
    pub timestamp: Timestamp,

    #[serde(alias = "bids")]
    pub bid_to_update: Vec<Vec<Decimal>>,

    #[serde(alias = "asks")]
    pub ask_to_update: Vec<Vec<Decimal>>

}

////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize)]
struct OuterBitstampNoData {
    pub event: String,

    #[serde(alias = "channel")]
    pub symbol: String
} 
#[derive(Deserialize)]
struct OuterBitstamp {
    pub event: String,

    #[serde(alias = "channel")]
    pub symbol: String,

    pub data: InnerBitstamp,
}

#[derive(Deserialize)]
struct InnerBitstamp {
    #[serde(alias = "timestamp")]
    pub first_update_id_timestamp: String,

    #[serde(alias = "microtimestamp")]
    pub last_update_id_timestamp: String,

    #[serde(alias = "bids")]
    pub bid_to_update: Vec<Vec<Decimal>>,

    #[serde(alias = "asks")]
    pub ask_to_update: Vec<Vec<Decimal>>
}

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct OuterBitstampSnapshot { 

    #[serde(alias = "microtimestamp")]
    pub micro_timestamp: String,

    #[serde(alias = "timestamp")]
    pub timestamp: String,

    #[serde(alias = "bids")]
    pub bid_to_update: Vec<Vec<Decimal>>,

    #[serde(alias = "asks")]
    pub ask_to_update: Vec<Vec<Decimal>>

}

////////////////////////////////////////////////////////////////////////////////////////
#[async_trait]
pub(crate) trait ExchangeService{

    async fn stream_management_task(deserialize_settings: DeserializeSettings);

    async fn websocket_msg_process(deserialize_settings: &mut DeserializeSettings) -> Result<()>;

    async fn snapshot_task(
        symbol: Symbol, 
        snapshot_url: Url, 
        output_rx_ch: Receiver<DepthData>, 
        output_stream_tx_ch: Sender<SnapshotData>) -> Result<()>;

    fn deserialize_stream(json_str: String) -> Result<DepthData>;

    fn deserialize_snapshot(symbol: Symbol, json_str: String) -> Result<SnapshotData>;
}

#[async_trait]
pub trait ExchangeInit{
    async fn stream_init_task(&mut self, output_stream_tx_ch: Sender<SnapshotData>) -> Result<()>;
}