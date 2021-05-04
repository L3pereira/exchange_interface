#[cfg(test)]
mod tests;
use std::{
    str::FromStr,
    collections::{BTreeMap}
};
use rust_decimal::Decimal;
use common::*;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Level {
    pub exchange: Exchange, 
    pub price:  Price,
    pub volume: Volume,
}
impl Level {
    pub fn new(exchange: Exchange, price:  Price, volume: Volume) -> Self {
        Level{
            exchange: exchange, 
            price:  price,
            volume: volume,
        }       
    }  
}

#[derive(Clone, Debug)]
pub struct AggregatedBook {
   books: Vec<SnapshotData>,
}

impl AggregatedBook {
    pub fn new() -> Self{
        AggregatedBook{
            books: Vec::new(),
        }       
    }


    pub fn update_book(&mut self, snapshot_data: SnapshotData){
        let mut index_to_remove: Option<usize>= None;
        for (i, book) in self.books.iter().enumerate(){
            if book.exchange == snapshot_data.exchange{
                index_to_remove = Some(i);
                break;
            }
        }
        if let Some(index) = index_to_remove {
            self.books.remove(index);
        }
        self.books.push(snapshot_data)
        
    }
    pub fn remove_book(&mut self, snapshot_data: SnapshotData) {
        let mut index_to_remove: Option<usize>= None;
        for (i, book) in self.books.iter().enumerate(){
            if book.exchange == snapshot_data.exchange{
                index_to_remove = Some(i);
                break;
            }
        }
        if let Some(index) = index_to_remove {
            self.books.remove(index);
        }       
    }
    pub fn get_top_asks(&self, top_num: usize) -> Vec<Level> {
        let mut aggregated_book: Vec<Level> = Vec::new();       
        for book in self.books.iter(){
            for (price, volume) in book.ask_to_update.iter() {
                aggregated_book.push(Level{
                    exchange: book.exchange.clone(),
                    price: *price,
                    volume: *volume
                })
            }
        }
        aggregated_book.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        aggregated_book.into_iter().take(top_num).collect::<Vec<Level>>()
    }
    pub fn get_top_bids(&self, top_num: usize) -> Vec<Level> {
        let mut aggregated_book: Vec<Level> = Vec::new();       
        for book in self.books.iter(){
            for (price, volume) in book.bid_to_update.iter() {
                aggregated_book.push(Level{
                    exchange: book.exchange.clone(),
                    price: *price,
                    volume: *volume
                })
            }
        }
        aggregated_book.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
        aggregated_book.into_iter().take(top_num).collect::<Vec<Level>>()
    }

    // pub fn get_spread(&self) -> Option<Decimal> {

    //     // for (price, volume) in  self.books.iter().flat_map(|book| book.clone().asks).next().collect::<BTreeMap<Price, Volume>>(){
    //     //     println!("{:?}", (price, volume));
    //     // }
    //     let best_ask = self.books.iter().flat_map(|book| book.clone().asks).next()?;
    //     let best_bid = self.books.iter().flat_map(|book| book.clone().bids).next_back()?;
    //     println!("{:?}", best_ask);
    //     Some(best_ask.0.clone() - best_bid.0.clone())
       
    // }

}

