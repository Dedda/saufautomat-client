extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate serialport;
#[macro_use]
extern crate rouille;

use std::io;
use std::collections::HashMap;
use std::time::Duration;
use serialport::SerialPortInfo;
use serde::Serialize;
use yatl::{duration_to_human_string, Timer};

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
        let bevs = com_loop(arduino_port).unwrap();
        println!("Found drinks in {} files! Yay!", bevs.len());
        let count: usize = bevs.iter().map(|(_, v)| v.iter().map(|b| b.count).sum::<usize>()).sum();
        println!("This adds up to {} drinks. WOW!", count);

        server::serve(bevs);
    } else {
        println!("Could not find any connected Arduino boards");
    }
}

fn com_loop(port: SerialPortInfo) -> Result<HashMap<String, Vec<Beverage>>, parse::Error> {
    let mut port = serialport::open(&port.port_name).unwrap();
    let _ = port.set_timeout(Duration::from_millis(500));
    println!("Connected with baud: {}", port.baud_rate().unwrap());
    let mut buffer = String::new();
    let mut serial_buf: Vec<u8> = vec![0; 1000];
    let mut received_anything = false;
    println!("Waiting for Arduino...");
    loop {
        match port.read(serial_buf.as_mut_slice()) {
            Ok(t) => {
                let s = std::str::from_utf8(&serial_buf[..t]).unwrap_or("");
                buffer.push_str(s);
                received_anything = true;
            },
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                if received_anything && buffer.lines().last().unwrap_or("").starts_with("/>") {
                    break;
                }
            },
            Err(e) => eprintln!("{:?}", e),
        }

    }
    println!("Requesting dump");
    match port.write("dump\n".as_bytes()) {
        Ok(_) => {},
        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
        Err(e) => eprintln!("{:?}", e),
    }
    buffer = String::new();
    received_anything = false;
    print!("Receiving dump... ");
    let mut timer = Timer::new();
    let _ = timer.start();
    loop {
        match port.read(serial_buf.as_mut_slice()) {
            Ok(t) => {
                let s = std::str::from_utf8(&serial_buf[..t]).unwrap_or("");
                buffer.push_str(s);
                received_anything = true;
                if let Some(pos) = buffer.find("%$") {
                    let buffer = &buffer[..pos+2];
                    let _ = port.write("exit\n".as_bytes());
                    let dur = timer.lap().unwrap();
                    println!(" OK [{}] [{} bytes]", duration_to_human_string(&dur), buffer.as_bytes().len());
                    return parse::parse(&buffer);
                }
            },          
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                if received_anything {
                    break;
                }
            },
            Err(e) => eprintln!("{:?}", e),
        }

    }
    Err(parse::Error::CustomFormat("communication problems".into()))
}