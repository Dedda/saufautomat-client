extern crate serde_json;

use std::io;
use std::collections::HashMap;
use std::time::Duration;
use serialport::SerialPortInfo;

mod parse;
mod serial;

#[derive(Debug, PartialEq)]
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

fn main() {
    println!("Looking for Arduino via USB...");
    if let Some(arduino_port) = serial::find_arduino() {
        println!("Found on port: {}", arduino_port.port_name);
        let bevs = com_loop(arduino_port).unwrap();
        println!("Found drinks in {} files! Yay!", bevs.len());
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
    println!("Receiving dump...");
    loop {
        match port.read(serial_buf.as_mut_slice()) {
            Ok(t) => {
                let s = std::str::from_utf8(&serial_buf[..t]).unwrap_or("");
                buffer.push_str(s);
                received_anything = true;
                if let Some(pos) = buffer.find("%$") {
                    let buffer = &buffer[..pos+2];
                    let _ = port.write("exit\n".as_bytes());
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