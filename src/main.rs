extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate serialport;
#[macro_use]
extern crate rouille;

use serde::Serialize;

mod parse;
mod serial;
mod server;

#[derive(Debug, PartialEq, Serialize)]
pub struct Beverage {
    pub name: String,
    pub count: usize,
}

impl Beverage {
    pub fn new(name: String, count: usize) -> Self {
        Self {
            name,
            count,
        }
    }
}

pub fn main() {
    println!("Looking for Arduino via USB...");
    if let Some(arduino_port) = serial::find_arduino() {
        println!("Found on port: {}", arduino_port.port_name);
        let bevs = serial::com_loop(arduino_port).unwrap();
        println!("Found drinks in {} files! Yay!", bevs.len());
        let count: usize = bevs.iter().map(|(_, v)| v.iter().map(|b| b.count).sum::<usize>()).sum();
        println!("This adds up to {} drinks. WOW!", count);

        server::serve(bevs);
    } else {
        println!("Could not find any connected Arduino boards");
    }
}
