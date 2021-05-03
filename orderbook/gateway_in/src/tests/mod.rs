mod mocks;


#[cfg(test)]
mod unit_tests;

#[cfg(test)]
mod binance_tests;

#[cfg(test)]
mod bitstamp_tests;
use std::sync::Once;
static INIT: Once = Once::new();
pub fn setup() -> () { 
    INIT.call_once(|| {
        log4rs::init_file(crate::LOG_CONFIG_PATH, Default::default()).unwrap();
    });
}