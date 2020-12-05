use serialport::{available_ports, SerialPortInfo, SerialPortType};

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