extern crate i2cdev;

use std::env;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

use reqwest::blocking::Client;

const ADDR: u16 = 0x40;

fn get_temp(buf: [u8; 3]) -> i16 {
    let value = u16::from_be_bytes([buf[0], buf[1]]);
    let value_conv = f32::from(value) * 175.72 / 65536.0 - 46.85;
    let value_f = value_conv * (9.0 / 5.0) + 32.0;
    value_f.round() as i16
}

fn get_humid(buf: [u8; 3]) -> u8 {
    let value = u16::from_be_bytes([buf[0], buf[1]]);
    let value_conv = (f32::from(value) * 125.0 / 65536.0 - 6.0).min(100.0);
    value_conv.round() as u8
}

fn main() -> Result<(), LinuxI2CError> {
    let mut dev_path = String::new();
    let mut sensor_id = String::new();
    let mut url = String::new();
    for (ind, arg) in env::args().enumerate() {
        match ind {
            1 => { dev_path.push_str(&arg) },
            2 => { sensor_id.push_str(&arg) },
            3 => { url.push_str(&arg) },
            _ => (),
        }
    }
    
    let mut dev = LinuxI2CDevice::new(dev_path, ADDR)?;
    let mut read_buffer = [0; 3];

    dev.write(&[0xF3])?;
    thread::sleep(Duration::from_millis(20));
    dev.read(&mut read_buffer)?;

    let temp = get_temp(read_buffer);

    dev.write(&[0xF5])?;
    thread::sleep(Duration::from_millis(20));
    dev.read(&mut read_buffer)?;

    let humid = get_humid(read_buffer);

    /*
    println!("Temp:  {}", temp);
    println!("Humid: {}", humid);
    */

    let mut form = HashMap::new();
    form.insert("id", sensor_id);
    form.insert("temp", temp.to_string());
    form.insert("humid", humid.to_string());

    let client = Client::new();
    let resp = client
        .post(&url)
        .form(&form)
        .send();

    match resp {
        Ok(_) => (),
        Err(e) => println!("[ERROR] {}", e),
    }

    Ok(())
}
