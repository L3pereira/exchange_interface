use std::collections::HashMap;
use anyhow::Result;
use url::Url;
use serde::{Deserialize, Deserializer};
use crate::*;

#[derive(Deserialize)]
#[derive(Clone, Debug)]
struct BinanceConfiguration {

    #[serde(deserialize_with = "to_url")]
    websocket_base_url: Url,

    #[serde(deserialize_with = "to_url")]
    snapshot_base_url: Url,

    #[serde(deserialize_with = "to_upper_vec")]
    symbols: Vec<String>,

    websocket_rate_ms: u32,

    snapshot_depth: u32,

}


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinanceConfig{
    pub websocket_rate_ms: u32,
    pub snapshot_urls: HashMap<Symbol, Url>,
    pub websocket_urls: HashMap<Symbol, Url>,
    pub snapshot_depth: u32,
    pub symbols: Vec<String>
}

impl<'de> Deserialize<'de> for BinanceConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let binance_config: BinanceConfiguration = Deserialize::deserialize(deserializer)?;

        let mut snapshot_hashmap: HashMap<Symbol, Url> = HashMap::new();
        let mut symbol_websocket_url_hashmap: HashMap<Symbol, Url> = HashMap::new();

        for symbol in binance_config.symbols.iter(){
            let symbol_lower_case = symbol.to_lowercase();
            let mut symbol_snapshot_url = binance_config.snapshot_base_url.clone();
            symbol_snapshot_url.set_query(Some(&format!("symbol={}&limit={}", symbol, binance_config.snapshot_depth)));
            snapshot_hashmap.insert(symbol.clone(), symbol_snapshot_url);

            let mut symbol_websocket_url = binance_config.websocket_base_url.clone();
            symbol_websocket_url.set_path(&format!("/ws/{}@depth@{}ms", symbol_lower_case.clone(), &binance_config.websocket_rate_ms));
            symbol_websocket_url_hashmap.insert(symbol.clone(), symbol_websocket_url); 
        }

        let config = BinanceConfig{
            websocket_urls: symbol_websocket_url_hashmap,
            websocket_rate_ms: binance_config.websocket_rate_ms,
            snapshot_urls: snapshot_hashmap,
            snapshot_depth: binance_config.snapshot_depth,
            symbols: binance_config.symbols

        };

        Ok(config)

    }
}