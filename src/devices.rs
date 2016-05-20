extern crate byteorder;

use std::io::Cursor;
use self::byteorder::{ReadBytesExt, BigEndian};

/// Unwrapped ADIS data
pub struct ADIS {
    vcc: f64,
    acc_x: f64,
}

/// Gravity
const G_0: f64 = 9.80665;


const SIZE_OF_ADIS: u16 = 28;

/// Receives an ADIS message
///
/// Unrwap a byte array assuming network endian into fields in the ADIS Data
/// type.
pub fn recv_adis(message_buffer: &[u8]) {

    println!("    Packet type: ADIS");

    // Convert fields:

    // VCC
    let mut buf = Cursor::new(&message_buffer[..2]);
    let mut vcc: f64 = buf.read_i16::<BigEndian>().unwrap() as f64;
    vcc = vcc * 0.002418;

    // Gyro X
    let mut buf = Cursor::new(&message_buffer[2..4]);
    let mut gyro_x: f64 = buf.read_i16::<BigEndian>().unwrap() as f64;
    gyro_x = gyro_x * 0.05;

    // Accel X
    let mut buf = Cursor::new(&message_buffer[8..10]);
    let mut accel_x: f64 = buf.read_i16::<BigEndian>().unwrap() as f64;
    accel_x = accel_x * 0.00333 * G_0;

    let adis = ADIS {vcc: vcc, acc_x: accel_x};

    println!("    VCC: {}", vcc);
    println!("    Gyro X: {}", gyro_x);
    println!("    Accel X: {}", accel_x);

    adis;
}


