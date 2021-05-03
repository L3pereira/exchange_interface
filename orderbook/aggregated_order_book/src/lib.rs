#[cfg(test)]
mod tests;
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
        let mut aggregated_book: BTreeMap<Price, Volume> = BTreeMap::new();       
        for book in self.books.iter(){
            for (price, volume) in book.asks.iter() {
                aggregated_book.entry(price.clone())
                    .and_modify(|aggregated_volume| *aggregated_volume += volume.clone())
                    .or_insert(volume.clone());
            }
        }
        aggregated_book.into_iter().take(top_num).collect::<BTreeMap<Price, Volume>>()      
    }
    pub fn get_top_bids(&self, top_num: usize) -> BTreeMap<Price, Volume> {
        let mut aggregated_book: BTreeMap<Price, Volume> = BTreeMap::new();       
        for book in self.books.iter(){
            for (price, volume) in book.bids.iter() {
                aggregated_book.entry(price.clone())
                    .and_modify(|aggregated_volume| *aggregated_volume += volume.clone())
                    .or_insert(volume.clone());
            }
        }
        aggregated_book.into_iter().rev().take(top_num).collect::<BTreeMap<Price, Volume>>()
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

