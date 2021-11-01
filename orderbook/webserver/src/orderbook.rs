tonic::include_proto!("orderbook");
use serde::ser::{Serialize, Serializer, SerializeStruct};

// #[derive(Clone, PartialEq, ::prost::Message)]
// pub struct Summary {
//     #[prost(double, tag = "1")]
//     pub spread: f64,
//     #[prost(message, repeated, tag = "2")]
//     pub bids: ::prost::alloc::vec::Vec<Level>,
//     #[prost(message, repeated, tag = "3")]
//     pub asks: ::prost::alloc::vec::Vec<Level>,
// }
// #[derive(Clone, PartialEq, ::prost::Message)]
// pub struct Level {
//     #[prost(string, tag = "1")]
//     pub exchange: ::prost::alloc::string::String,
//     #[prost(double, tag = "2")]
//     pub price: f64,
//     #[prost(double, tag = "3")]
//     pub amount: f64,
// }
impl Serialize for Summary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Summary", 3)?;
        state.serialize_field("spread", &self.spread)?;
        state.serialize_field("bids", &self.bids)?;
        state.serialize_field("asks", &self.asks)?;
        state.end()
    }
}
impl Serialize for Level {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Level", 3)?;
        state.serialize_field("exchange", &self.exchange)?;
        state.serialize_field("price", &self.price)?;
        state.serialize_field("amount", &self.amount)?;
        state.end()
    }
}