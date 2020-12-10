use serialport::{available_ports, SerialPortInfo, SerialPortType};
use std::collections::HashMap;
use crate::{Beverage, parse};
use std::time::Duration;
use yatl::{Timer, duration_to_human_string};
use std::io;

pub fn find_arduino() -> Option<SerialPortInfo> {
    if let Ok(ports) = available_ports() {
        for p in ports {
            if let SerialPortType::UsbPort(info) = &p.port_type {
                if info.manufacturer.as_ref().map_or("", String::as_str).to_lowercase().contains("arduino") {
                    return Some(p);
                }
            }
        }
        None
    } else {
        None
    }
}

pub fn com_loop(port: SerialPortInfo) -> Result<HashMap<String, Vec<Beverage>>, parse::Error> {
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