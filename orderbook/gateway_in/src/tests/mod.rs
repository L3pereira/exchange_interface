mod mocks;

mod unit_tests;
mod binance_tests;


mod bitstamp_tests;
// use std::sync::Once;
// const CONFIG_PATH: &str = "src/tests/config.json"; 

// static INIT: Once = Once::new();
// fn setup() -> () { 
//     INIT.call_once(|| {
//         const LOG_CONFIG_PATH: &str = "src/tests/log_config.yaml";
//         log4rs::init_file(LOG_CONFIG_PATH, Default::default()).unwrap();
//     });
// }