use std::{
    str::FromStr,
    collections::{BTreeMap}
};
use common::{
    DepthData,
    SnapshotData,
    ErrorMessage,
    Price,
    Volume
};
use crate::{Book, AggregatedBook};
use rust_decimal::Decimal;
use pretty_assertions::{assert_eq, assert_ne};

#[test]
fn book_update_test() {

   let mut asks_from_exchange: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut bids_from_exchange: BTreeMap<Price, Volume> =  BTreeMap::new();

    asks_from_exchange.insert(Decimal::from_str("1.000").unwrap(), Decimal::from_str("9.74000000").unwrap());
    asks_from_exchange.insert(Decimal::from_str("0.980").unwrap(), Decimal::from_str("7.45000000").unwrap());
    asks_from_exchange.insert(Decimal::from_str("0.880").unwrap(), Decimal::from_str("0").unwrap());

    bids_from_exchange.insert(Decimal::from_str("0.670").unwrap(), Decimal::from_str("0.60000000").unwrap());
    bids_from_exchange.insert(Decimal::from_str("0.510").unwrap(), Decimal::from_str("4.17900000").unwrap());
    bids_from_exchange.insert(Decimal::from_str("0.400").unwrap(), Decimal::from_str("8.15600000").unwrap()); 

    let mut asks_from_exchange_update: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut bids_from_exchange_update: BTreeMap<Price, Volume> =  BTreeMap::new();

    asks_from_exchange_update.insert(Decimal::from_str("1.000").unwrap(), Decimal::from_str("10.0000000").unwrap());
    asks_from_exchange_update.insert(Decimal::from_str("0.980").unwrap(), Decimal::from_str("7.45000000").unwrap());
    asks_from_exchange_update.insert(Decimal::from_str("0.950").unwrap(), Decimal::from_str("10.0000000").unwrap());

    bids_from_exchange_update.insert(Decimal::from_str("0.670").unwrap(), Decimal::from_str("0").unwrap());
    bids_from_exchange_update.insert(Decimal::from_str("0.510").unwrap(), Decimal::from_str("4.17900000").unwrap());
    bids_from_exchange_update.insert(Decimal::from_str("0.400").unwrap(), Decimal::from_str("8.15600000").unwrap());
    // let expected_exchange_1_data = DepthData {
    //     symbol: "bnbbtc".to_string(),
    //     first_update_id_timestamp: 1833980193,
    //     last_update_id_timestamp: 1833980193555559,
    //     bid_to_update: bids_from_exchange_1,
    //     ask_to_update: asks_from_exchange_1  
    //  };

    //  let expected_exchange_2_data = DepthData {
    //     symbol: "bnbbtc".to_string(),
    //     first_update_id_timestamp: 1833980193,
    //     last_update_id_timestamp: 1833980193555559,
    //     bid_to_update: bids_from_exchange_2,
    //     ask_to_update: asks_from_exchange_2  
    //  };
   
    let mut book_exchange_1 = Book::new();
    book_exchange_1.update_book(asks_from_exchange, bids_from_exchange);
    // println!("book_exchange_1 {:?}", book_exchange_1);
    book_exchange_1.update_book(asks_from_exchange_update, bids_from_exchange_update);
    // println!("book_exchange_update {:?}", book_exchange_1);

    let top_ask_expected = (&Decimal::from_str("0.950").unwrap(), &Decimal::from_str("10.0000000").unwrap());
    let top_bid_expected = (&Decimal::from_str("0.510").unwrap(), &Decimal::from_str("4.17900000").unwrap());
    assert_eq!(Some(top_ask_expected), book_exchange_1.asks.iter().next());
    assert_eq!(Some(top_bid_expected), book_exchange_1.bids.iter().next_back());
}

#[test]
fn get_spread_book_test() {

    let mut asks_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut bids_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();

    asks_from_exchange_1.insert(Decimal::from_str("1.000").unwrap(), Decimal::from_str("9.74000000").unwrap());
    asks_from_exchange_1.insert(Decimal::from_str("0.980").unwrap(), Decimal::from_str("7.45000000").unwrap());
    asks_from_exchange_1.insert(Decimal::from_str("0.880").unwrap(), Decimal::from_str("5.74000000").unwrap());

    bids_from_exchange_1.insert(Decimal::from_str("0.670").unwrap(), Decimal::from_str("0.60000000").unwrap());
    bids_from_exchange_1.insert(Decimal::from_str("0.510").unwrap(), Decimal::from_str("4.17900000").unwrap());
    bids_from_exchange_1.insert(Decimal::from_str("0.400").unwrap(), Decimal::from_str("8.15600000").unwrap());

    let mut book_exchange_1 = Book::new();
    book_exchange_1.update_book(asks_from_exchange_1, bids_from_exchange_1);
    let spread = book_exchange_1.get_spread();
    assert_eq!(Some(Decimal::from_str("0.210").unwrap()), spread)
}

#[test]
fn get_spread_aggregate_book_test() {

    let mut asks_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut bids_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();

    let mut asks_from_exchange_2: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut bids_from_exchange_2: BTreeMap<Price, Volume> =  BTreeMap::new();

    asks_from_exchange_1.insert(Decimal::from_str("1.000").unwrap(), Decimal::from_str("9.74000000").unwrap());
    asks_from_exchange_1.insert(Decimal::from_str("1.500").unwrap(), Decimal::from_str("7.45000000").unwrap());
    asks_from_exchange_1.insert(Decimal::from_str("0.700").unwrap(), Decimal::from_str("5.74000000").unwrap());

    bids_from_exchange_1.insert(Decimal::from_str("0.670").unwrap(), Decimal::from_str("0.60000000").unwrap());
    bids_from_exchange_1.insert(Decimal::from_str("0.510").unwrap(), Decimal::from_str("4.17900000").unwrap());
    bids_from_exchange_1.insert(Decimal::from_str("0.400").unwrap(), Decimal::from_str("8.15600000").unwrap());

    asks_from_exchange_2.insert(Decimal::from_str("2.000").unwrap(), Decimal::from_str("15.66000000").unwrap());
    asks_from_exchange_2.insert(Decimal::from_str("1.500").unwrap(), Decimal::from_str("12.45000000").unwrap());
    asks_from_exchange_2.insert(Decimal::from_str("1.000").unwrap(), Decimal::from_str("8.45000000").unwrap());

    bids_from_exchange_2.insert(Decimal::from_str("0.900").unwrap(), Decimal::from_str("0.60000000").unwrap());
    bids_from_exchange_2.insert(Decimal::from_str("0.520").unwrap(), Decimal::from_str("2.17900000").unwrap());
    bids_from_exchange_2.insert(Decimal::from_str("0.400").unwrap(), Decimal::from_str("0.15600000").unwrap());

    let mut book_exchange_1 = Book::new();
    book_exchange_1.update_book(asks_from_exchange_1, bids_from_exchange_1);

    let mut book_exchange_2 = Book::new();
    book_exchange_2.update_book(asks_from_exchange_2, bids_from_exchange_2);

    let mut agrregate_book = AggregatedBook::new();
    agrregate_book.books.push(book_exchange_1);
    agrregate_book.books.push(book_exchange_2);
    let spread = agrregate_book.get_spread();

    assert_eq!(Some(Decimal::from_str("-0.200").unwrap()), spread);

}

#[test]
fn aggregate_book_get_top_asks_bids_test() {

    let mut asks_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut bids_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();

    let mut asks_from_exchange_2: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut bids_from_exchange_2: BTreeMap<Price, Volume> =  BTreeMap::new();

    asks_from_exchange_1.insert(Decimal::from_str("10").unwrap(), Decimal::from_str("5").unwrap());
    asks_from_exchange_1.insert(Decimal::from_str("9").unwrap(), Decimal::from_str("5").unwrap());
    asks_from_exchange_1.insert(Decimal::from_str("8").unwrap(), Decimal::from_str("5").unwrap());

    bids_from_exchange_1.insert(Decimal::from_str("7").unwrap(), Decimal::from_str("5").unwrap());
    bids_from_exchange_1.insert(Decimal::from_str("6").unwrap(), Decimal::from_str("5").unwrap());
    bids_from_exchange_1.insert(Decimal::from_str("5").unwrap(), Decimal::from_str("5").unwrap());

    asks_from_exchange_2.insert(Decimal::from_str("11").unwrap(), Decimal::from_str("5").unwrap());
    asks_from_exchange_2.insert(Decimal::from_str("8").unwrap(), Decimal::from_str("5").unwrap());
    asks_from_exchange_2.insert(Decimal::from_str("7").unwrap(), Decimal::from_str("5").unwrap());

    bids_from_exchange_2.insert(Decimal::from_str("6").unwrap(), Decimal::from_str("5").unwrap());
    bids_from_exchange_2.insert(Decimal::from_str("4").unwrap(), Decimal::from_str("5").unwrap());
    bids_from_exchange_2.insert(Decimal::from_str("3").unwrap(), Decimal::from_str("5").unwrap());

    let mut book_exchange_1 = Book::new();
    book_exchange_1.update_book(asks_from_exchange_1, bids_from_exchange_1);

    let mut book_exchange_2 = Book::new();
    book_exchange_2.update_book(asks_from_exchange_2, bids_from_exchange_2);

    let mut agrregate_book_result = AggregatedBook::new();
    agrregate_book_result.books.push(book_exchange_1);
    agrregate_book_result.books.push(book_exchange_2); 

    let mut asks_expected: BTreeMap<Price, Volume> =  BTreeMap::new();
    let mut bids_expected: BTreeMap<Price, Volume> =  BTreeMap::new();
    // asks_expected.insert(Decimal::from_str("11").unwrap(), Decimal::from_str("5").unwrap());
    // asks_expected.insert(Decimal::from_str("10").unwrap(), Decimal::from_str("5").unwrap());
    asks_expected.insert(Decimal::from_str("9").unwrap(), Decimal::from_str("5").unwrap());
    asks_expected.insert(Decimal::from_str("8").unwrap(), Decimal::from_str("10").unwrap());
    asks_expected.insert(Decimal::from_str("7").unwrap(), Decimal::from_str("5").unwrap());

    bids_expected.insert(Decimal::from_str("7").unwrap(), Decimal::from_str("5").unwrap());
    bids_expected.insert(Decimal::from_str("6").unwrap(), Decimal::from_str("10").unwrap());
    bids_expected.insert(Decimal::from_str("5").unwrap(), Decimal::from_str("5").unwrap());
    // bids_expected.insert(Decimal::from_str("4").unwrap(), Decimal::from_str("5").unwrap());
    // bids_expected.insert(Decimal::from_str("3").unwrap(), Decimal::from_str("5").unwrap());

    let agrregate_book_top_asks_result = agrregate_book_result.get_top_asks(3);

    let agrregate_book_top_bids_result = agrregate_book_result.get_top_bids(3);

    println!("agrregate_book_top_asks {:?}", agrregate_book_top_asks_result);
    println!("agrregate_book_top_bids {:?}", agrregate_book_top_bids_result);
    assert_eq!(asks_expected, agrregate_book_top_asks_result);
    assert_eq!(bids_expected, agrregate_book_top_bids_result);
  
}