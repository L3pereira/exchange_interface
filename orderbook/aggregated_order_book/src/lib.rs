
use std::{
    str::FromStr,
    collections::{BTreeMap}
};
use rust_decimal::Decimal;
use common::{
    DepthData,
    SnapshotData,
    ErrorMessage,
    Price,
    Volume
};
#[derive(Clone, Debug)]
pub struct Book {
    pub asks: BTreeMap<Price, Volume>,
    pub bids: BTreeMap<Price, Volume>
}
impl Book {
    pub fn new() -> Self{
        Book{
            asks: BTreeMap::new(),
            bids: BTreeMap::new()
        }       
    }

    pub fn update_book(&mut self, asks: BTreeMap<Price, Volume>, bids: BTreeMap<Price, Volume>){

        for (price, volume) in  asks.into_iter(){
            if volume == Decimal::new(0, 0) {
                self.asks.remove_entry(&price);
            }
            else{
                self.asks
                .entry(price)
                .or_insert(volume);
            }
        }
        for (price, volume) in  bids.into_iter(){

            if volume == Decimal::new(0, 0) {
                self.bids.remove_entry(&price);
            }
            else{
                self.bids
                .entry(price)
                .or_insert(volume);
            }

        }
       
    }
    // pub fn best_ask_price_volume(&self) -> (Price, Volume) { 
    //     match self.asks.iter().next(){
    //         Some((price, volume)) => (price.clone(), volume.clone()),
    //         None => (Decimal::new(0, 0), Decimal::new(0, 0))
    //     }
        
    // }
    pub fn get_spread(&self) -> Option<Decimal> {
        let best_ask = self.asks.iter().next()?;
        let best_bid = self.bids.iter().next_back()?;
        Some(best_ask.0.clone() - best_bid.0.clone())
    }
}
#[derive(Clone, Debug)]
pub struct AggregatedBook {
    pub books: Vec<Book>,
}
impl AggregatedBook {
    pub fn new() -> Self{
        AggregatedBook{
            books: Vec::new(),
        }       
    }

    pub fn get_top_asks(&self, top_num: usize) -> BTreeMap<Price, Volume> {
        self.books.iter().flat_map(|book| book.clone().asks).take(top_num).collect::<BTreeMap<Price, Volume>>()       
    }
    pub fn get_top_bids(&self, top_num: usize) -> BTreeMap<Price, Volume> {
        self.books.iter().flat_map(|book| book.clone().bids).rev().take(top_num).collect::<BTreeMap<Price, Volume>>()      
    }

    pub fn get_spread(&self) -> Option<Decimal> {

        // for (price, volume) in  self.books.iter().flat_map(|book| book.clone().asks).next().collect::<BTreeMap<Price, Volume>>(){
        //     println!("{:?}", (price, volume));
        // }
        let best_ask = self.books.iter().flat_map(|book| book.clone().asks).next()?;
        let best_bid = self.books.iter().flat_map(|book| book.clone().bids).next_back()?;
        println!("{:?}", best_ask);
        Some(best_ask.0.clone() - best_bid.0.clone())
       
    }

}


#[cfg(test)]
mod tests {
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
    fn book_get_spread_test() {

        let mut asks_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();
        let mut bids_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();

        asks_from_exchange_1.insert(Decimal::from_str("1.000").unwrap(), Decimal::from_str("9.74000000").unwrap());
        asks_from_exchange_1.insert(Decimal::from_str("0.980").unwrap(), Decimal::from_str("7.45000000").unwrap());
        asks_from_exchange_1.insert(Decimal::from_str("0.880").unwrap(), Decimal::from_str("5.74000000").unwrap());

        bids_from_exchange_1.insert(Decimal::from_str("0.670").unwrap(), Decimal::from_str("0.60000000").unwrap());
        bids_from_exchange_1.insert(Decimal::from_str("0.510").unwrap(), Decimal::from_str("4.17900000").unwrap());
        bids_from_exchange_1.insert(Decimal::from_str("0.400").unwrap(), Decimal::from_str("8.15600000").unwrap());

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
        book_exchange_1.update_book(asks_from_exchange_1, bids_from_exchange_1);
        let spread = book_exchange_1.get_spread();
        assert_eq!(Some(Decimal::from_str("0.210").unwrap()), spread);
    }

    #[test]
    fn aggregate_book_get_spread_test() {

        let mut asks_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();
        let mut bids_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();

        let mut asks_from_exchange_2: BTreeMap<Price, Volume> =  BTreeMap::new();
        let mut bids_from_exchange_2: BTreeMap<Price, Volume> =  BTreeMap::new();

        asks_from_exchange_1.insert(Decimal::from_str("1.000").unwrap(), Decimal::from_str("9.74000000").unwrap());
        asks_from_exchange_1.insert(Decimal::from_str("0.980").unwrap(), Decimal::from_str("7.45000000").unwrap());
        asks_from_exchange_1.insert(Decimal::from_str("0.880").unwrap(), Decimal::from_str("5.74000000").unwrap());

        bids_from_exchange_1.insert(Decimal::from_str("0.670").unwrap(), Decimal::from_str("0.60000000").unwrap());
        bids_from_exchange_1.insert(Decimal::from_str("0.510").unwrap(), Decimal::from_str("4.17900000").unwrap());
        bids_from_exchange_1.insert(Decimal::from_str("0.400").unwrap(), Decimal::from_str("8.15600000").unwrap());

        asks_from_exchange_2.insert(Decimal::from_str("2.00").unwrap(), Decimal::from_str("15.66000000").unwrap());
        asks_from_exchange_2.insert(Decimal::from_str("1.50").unwrap(), Decimal::from_str("12.45000000").unwrap());
        asks_from_exchange_2.insert(Decimal::from_str("1.00").unwrap(), Decimal::from_str("8.45000000").unwrap());

        bids_from_exchange_2.insert(Decimal::from_str("0.900").unwrap(), Decimal::from_str("0.60000000").unwrap());
        bids_from_exchange_2.insert(Decimal::from_str("0.520").unwrap(), Decimal::from_str("2.17900000").unwrap());
        bids_from_exchange_2.insert(Decimal::from_str("0.450").unwrap(), Decimal::from_str("0.15600000").unwrap());

        let mut book_exchange_1 = Book::new();
        book_exchange_1.update_book(asks_from_exchange_1, bids_from_exchange_1);

        let mut book_exchange_2 = Book::new();
        book_exchange_2.update_book(asks_from_exchange_2, bids_from_exchange_2);

        let mut agrregate_book = AggregatedBook::new();
        agrregate_book.books.push(book_exchange_1);
        agrregate_book.books.push(book_exchange_2);
        let spread = agrregate_book.get_spread();

        assert_eq!(Some(Decimal::from_str("-0.020").unwrap()), spread);
 
    }

    #[test]
    fn aggregate_book_get_top_asks_bids_test() {

        let mut asks_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();
        let mut bids_from_exchange_1: BTreeMap<Price, Volume> =  BTreeMap::new();

        let mut asks_from_exchange_2: BTreeMap<Price, Volume> =  BTreeMap::new();
        let mut bids_from_exchange_2: BTreeMap<Price, Volume> =  BTreeMap::new();

        asks_from_exchange_1.insert(Decimal::from_str("1.000").unwrap(), Decimal::from_str("9.74000000").unwrap());
        asks_from_exchange_1.insert(Decimal::from_str("0.980").unwrap(), Decimal::from_str("7.45000000").unwrap());
        asks_from_exchange_1.insert(Decimal::from_str("0.880").unwrap(), Decimal::from_str("5.74000000").unwrap());

        bids_from_exchange_1.insert(Decimal::from_str("0.670").unwrap(), Decimal::from_str("0.60000000").unwrap());
        bids_from_exchange_1.insert(Decimal::from_str("0.510").unwrap(), Decimal::from_str("4.17900000").unwrap());
        bids_from_exchange_1.insert(Decimal::from_str("0.400").unwrap(), Decimal::from_str("8.15600000").unwrap());

        asks_from_exchange_2.insert(Decimal::from_str("2.00").unwrap(), Decimal::from_str("15.66000000").unwrap());
        asks_from_exchange_2.insert(Decimal::from_str("1.50").unwrap(), Decimal::from_str("12.45000000").unwrap());
        asks_from_exchange_2.insert(Decimal::from_str("1.00").unwrap(), Decimal::from_str("8.45000000").unwrap());

        bids_from_exchange_2.insert(Decimal::from_str("0.900").unwrap(), Decimal::from_str("0.60000000").unwrap());
        bids_from_exchange_2.insert(Decimal::from_str("0.520").unwrap(), Decimal::from_str("2.17900000").unwrap());
        bids_from_exchange_2.insert(Decimal::from_str("0.450").unwrap(), Decimal::from_str("0.15600000").unwrap());

        let mut book_exchange_1 = Book::new();
        book_exchange_1.update_book(asks_from_exchange_1, bids_from_exchange_1);

        let mut book_exchange_2 = Book::new();
        book_exchange_2.update_book(asks_from_exchange_2, bids_from_exchange_2);

        let mut agrregate_book = AggregatedBook::new();
        agrregate_book.books.push(book_exchange_1);
        agrregate_book.books.push(book_exchange_2);

        let agrregate_book_top_asks = agrregate_book.get_top_asks(2);
        let mut agrregate_book_top_asks_iter = agrregate_book_top_asks.iter();

        let agrregate_book_top_bids = agrregate_book.get_top_bids(2);
        let mut agrregate_book_top__bids_iter = agrregate_book_top_bids.iter();


        let top_ask = agrregate_book_top_asks_iter.next().unwrap();
        let top_bid = agrregate_book_top__bids_iter.next_back().unwrap();
        let to2nd_ask = agrregate_book_top_asks_iter.next().unwrap();
        let top2nd_bid = agrregate_book_top__bids_iter.next_back().unwrap();
        println!("top_asks {:?}", to2nd_ask);
        println!("top_bids {:?}", top2nd_bid);

        let top_ask_expected = (&Decimal::from_str("0.880").unwrap(), &Decimal::from_str("5.74000000").unwrap());
        let top_bid_expected = (&Decimal::from_str("0.900").unwrap(), &Decimal::from_str("0.60000000").unwrap());
        let top2nd_ask_expected = (&Decimal::from_str("0.980").unwrap(), &Decimal::from_str("7.45000000").unwrap());
        let top2nd_bid_expected = (&Decimal::from_str("0.520").unwrap(), &Decimal::from_str("2.17900000").unwrap());
        //assert_eq!(Some(Decimal::from_str("-0.020").unwrap()), spread);
        assert_eq!(top_ask_expected, top_ask);
        assert_eq!(top_bid_expected, top_bid);
        assert_eq!(top2nd_ask_expected, to2nd_ask);
        assert_eq!(top2nd_bid_expected, top2nd_bid);
  
        //assert_eq!(top_ask_expected,(top_bid_price.clone(), top_bid_volume.clone()));
        
    }
}
