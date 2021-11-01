use std::collections::HashMap;
use anyhow::Result;
use url::Url;
use serde::{Deserialize, Deserializer};
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::*;

#[derive(Deserialize)]
#[derive(Clone, Debug)]
pub struct BitstampConfiguration {
    #[serde(deserialize_with = "to_url")]
    websocket_base_url: Url,

    #[serde(deserialize_with = "to_url")]
    snapshot_base_url: Url,

    // #[serde(deserialize_with = "to_upper_vec")]
    symbols: Vec<String>,

}


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitstampConfig{
    pub websocket_url: Url,
    pub websocket_payloads: HashMap<String, Message>,
    pub snapshot_urls: HashMap<String, Url>,
    pub symbols: Vec<String>
}

impl<'de> Deserialize<'de> for BitstampConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bitstamp_config: BitstampConfiguration = Deserialize::deserialize(deserializer)?;

        let mut snapshot_hashmap: HashMap<Symbol, Url> = HashMap::new();
        let mut websocket_payloads: HashMap<Symbol, Message> = HashMap::new();

        
        let path = bitstamp_config.snapshot_base_url.path();
        for symbol in bitstamp_config.symbols.iter(){
            let mut new_snapshot_url = bitstamp_config.snapshot_base_url.clone();
            new_snapshot_url.set_path(symbol);
            new_snapshot_url.set_path(&format!("{}/{}", path, symbol));
            snapshot_hashmap.insert(symbol.clone(), new_snapshot_url);

            let payload_message =  format!("{{\"event\": \"bts:subscribe\", \"data\": {{ \"channel\": \"order_book_{}\" }} }}", symbol);
            websocket_payloads.insert(symbol.clone(), Message::Text(payload_message)); 
        }
 

        let config = BitstampConfig{
            websocket_url: bitstamp_config.websocket_base_url,
            websocket_payloads: websocket_payloads,
            snapshot_urls: snapshot_hashmap,
            symbols: bitstamp_config.symbols

        };
        Ok(config)

    }
}