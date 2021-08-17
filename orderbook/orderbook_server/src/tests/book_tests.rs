use std::{
    str::FromStr,
    collections::BTreeMap
};
use crate::*;
use crate::aggregated_order_book::{AggregatedBook, Level};
use rust_decimal::Decimal;
use pretty_assertions::assert_eq;

#[test]
fn get_top_asks_aggregate_book_test() {

    let mut asks_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();
    let bids_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();

    let mut asks_from_exchange_2: BTreeMap<Price, Volume> =  BTreeMap::new();
    let bids_from_exchange_2: BTreeMap<Price, Volume> =  BTreeMap::new();

    asks_from_exchange_1.insert(Decimal::from_str("10.0").unwrap(), Decimal::from_str("3").unwrap());
    asks_from_exchange_1.insert(Decimal::from_str("9.0").unwrap(), Decimal::from_str("2").unwrap());
    asks_from_exchange_1.insert(Decimal::from_str("8.0").unwrap(), Decimal::from_str("5").unwrap());

    asks_from_exchange_2.insert(Decimal::from_str("11.0").unwrap(), Decimal::from_str("3").unwrap());
    asks_from_exchange_2.insert(Decimal::from_str("8.0").unwrap(), Decimal::from_str("4").unwrap());
    asks_from_exchange_2.insert(Decimal::from_str("7.0").unwrap(), Decimal::from_str("6").unwrap());

    let update1 = SnapshotData {
        exchange: Exchange::Binance,
        symbol: "BNBBTC".to_string(),
        timestamp: 1833980193,
        bid_to_update: bids_from_exchange_1,
        ask_to_update: asks_from_exchange_1
     };
     let update2 = SnapshotData {
        exchange: Exchange::Bitstamp,
        symbol: "BNBBTC".to_string(),
        timestamp: 1833980193,
        bid_to_update: bids_from_exchange_2,
        ask_to_update: asks_from_exchange_2
    };

    let mut agrregate_book_result = AggregatedBook::new();
    agrregate_book_result.update_book(update1); 
    agrregate_book_result.update_book(update2); 

    let mut asks_expected: Vec<Level> =  Vec::new();

    let bitstamp = Exchange::Bitstamp;
    let binance = Exchange::Binance;

    asks_expected.push(Level::new(binance.clone(),  Decimal::from_str("9.0").unwrap(), Decimal::from_str("2").unwrap()));
    asks_expected.push(Level::new(binance.clone(),  Decimal::from_str("8.0").unwrap(), Decimal::from_str("5").unwrap()));
    asks_expected.push(Level::new(bitstamp.clone(), Decimal::from_str("8.0").unwrap(), Decimal::from_str("4").unwrap()));
    asks_expected.push(Level::new(bitstamp.clone(), Decimal::from_str("7.0").unwrap(), Decimal::from_str("6").unwrap()));

    asks_expected.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

    let agrregate_book_top_asks_result = agrregate_book_result.get_top_asks(4);

    // println!("agrregate_book_top_asks {:?}", agrregate_book_top_asks_result);

    // println!("asks_expected {:?}", asks_expected);

    assert_eq!(asks_expected, agrregate_book_top_asks_result);

}

#[test]
fn  get_top_bids_aggregate_book_test() {

    let asks_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut bids_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();

    let asks_from_exchange_2: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut bids_from_exchange_2: BTreeMap<Price, Volume> =  BTreeMap::new();

    bids_from_exchange_1.insert(Decimal::from_str("7.0").unwrap(), Decimal::from_str("5").unwrap());
    bids_from_exchange_1.insert(Decimal::from_str("6.0").unwrap(), Decimal::from_str("5").unwrap());
    bids_from_exchange_1.insert(Decimal::from_str("5.0").unwrap(), Decimal::from_str("5").unwrap());

    bids_from_exchange_2.insert(Decimal::from_str("6.0").unwrap(), Decimal::from_str("5").unwrap());
    bids_from_exchange_2.insert(Decimal::from_str("4.0").unwrap(), Decimal::from_str("5").unwrap());
    bids_from_exchange_2.insert(Decimal::from_str("3.0").unwrap(), Decimal::from_str("5").unwrap());

    let update1 = SnapshotData {
        exchange: Exchange::Binance,
        symbol: "BNBBTC".to_string(),
        timestamp: 1833980193,
        bid_to_update: bids_from_exchange_1,
        ask_to_update: asks_from_exchange_1
     };
     let update2 = SnapshotData {
        exchange: Exchange::Bitstamp,
        symbol: "BNBBTC".to_string(),
        timestamp: 1833980193,
        bid_to_update: bids_from_exchange_2,
        ask_to_update: asks_from_exchange_2
    };

    let mut agrregate_book_result = AggregatedBook::new();
    agrregate_book_result.update_book(update1);
    agrregate_book_result.update_book(update2); 


    let mut bids_expected: Vec<Level> =  Vec::new();
    let bitstamp = Exchange::Bitstamp;
    let binance = Exchange::Binance;

    bids_expected.push(Level::new(binance.clone(), Decimal::from_str("7.0").unwrap(), Decimal::from_str("5").unwrap()));
    bids_expected.push(Level::new(binance.clone(), Decimal::from_str("6.0").unwrap(), Decimal::from_str("5").unwrap()));
    bids_expected.push(Level::new(bitstamp.clone(), Decimal::from_str("6.0").unwrap(), Decimal::from_str("5").unwrap()));
    bids_expected.push(Level::new(binance.clone(), Decimal::from_str("5.0").unwrap(), Decimal::from_str("5").unwrap()));

    bids_expected.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());

    let agrregate_book_top_bids_result = agrregate_book_result.get_top_bids(4);

    // println!("agrregate_book_top_bids {:?}", agrregate_book_top_bids_result);

    // println!("bids_expected {:?}", bids_expected);

    
    assert_eq!(bids_expected, agrregate_book_top_bids_result);
  
}