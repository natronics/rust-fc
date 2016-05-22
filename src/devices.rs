//! # rust-fc/devices
//!
//! Code for reading raw data from sensors on the rocket. Currently just
//! implements the packed ADIS16405 IMU

extern crate byteorder;

use std::io::Cursor;
use self::byteorder::{ReadBytesExt, BigEndian};

/// Unwrapped and converted ADIS data.
pub struct ADIS {

    /// VCC [Volts]. The voltage coming into the IMU
    pub vcc: f64,

    /// X-axis rate-gyro [deg/s]
    pub gyro_x: f64,

    /// Y-axis rate-gyro [deg/s]
    pub gyro_y: f64,

    /// Z-axis rate-gyro [deg/s]
    pub gyro_z: f64,

    /// X-axis accelerometer [m/s²]
    pub acc_x: f64,

    /// Y-axis accelerometer [m/s²]
    pub acc_y: f64,

    /// Z-axis accelerometer [m/s²]
    pub acc_z: f64,

    /// X-axis magnetometer [Tesla]
    pub magn_x: f64,

    /// Y-axis magnetometer [Tesla]
    pub magn_y: f64,

    /// Z-axis magnetometer [Tesla]
    pub magn_z: f64,

    /// Temperature [Kelvin] of the IMU
    pub temp: f64,
}

/// Gravity
const G_0: f64 = 9.80665;

/// Conversions
const VCC2VOLTS: f64 = 0.002418;
const GYRO2DEGS: f64 = 0.05;
const ACC2G: f64 = 0.00333;
const MAG2T: f64 = 5e-8;
const TEMP2C: f64 = 0.14;
const C2K: f64 = 299.15;

/// Message size (bytes)
pub const SIZE_OF_ADIS: u16 = 28;

/// Read an ADIS message from raw bytes.
///
/// Unrwap a byte array (assuming network endianess) into fields for the ADIS
/// Data type. This will also do the conversion from ADC counts to proper (SI)
/// Units.
pub fn recv_adis(message_buffer: &[u8]) -> ADIS {

    // VCC
    let mut buf = Cursor::new(&message_buffer[..2]);
    let vcc: f64 = buf.read_i16::<BigEndian>().unwrap() as f64 * VCC2VOLTS;


    // Rate Gyroscopes ========================================================
    // Gyro X
    let mut buf = Cursor::new(&message_buffer[2..4]);
    let gyro_x: f64 = buf.read_i16::<BigEndian>().unwrap() as f64 * GYRO2DEGS;

    // Gyro Y
    let mut buf = Cursor::new(&message_buffer[4..6]);
    let gyro_y: f64 = buf.read_i16::<BigEndian>().unwrap() as f64 * GYRO2DEGS;

    // Gyro Z
    let mut buf = Cursor::new(&message_buffer[6..8]);
    let gyro_z: f64 = buf.read_i16::<BigEndian>().unwrap() as f64 * GYRO2DEGS;


    // Accelerometer ==========================================================
    // Accel X
    let mut buf = Cursor::new(&message_buffer[8..10]);
    let accel_x: f64 = buf.read_i16::<BigEndian>().unwrap() as f64 * ACC2G * G_0;

    // Accel Y
    let mut buf = Cursor::new(&message_buffer[10..12]);
    let accel_y: f64 = buf.read_i16::<BigEndian>().unwrap() as f64 * ACC2G * G_0;

    // Accel Z
    let mut buf = Cursor::new(&message_buffer[12..14]);
    let accel_z: f64 = buf.read_i16::<BigEndian>().unwrap() as f64 * ACC2G * G_0;


    // Magnetometer ===========================================================
    // Magnetometer X
    let mut buf = Cursor::new(&message_buffer[14..16]);
    let mag_x: f64 = buf.read_i16::<BigEndian>().unwrap() as f64 * MAG2T;

    // Magnetometer Y
    let mut buf = Cursor::new(&message_buffer[16..18]);
    let mag_y: f64 = buf.read_i16::<BigEndian>().unwrap() as f64 * MAG2T;

    // Magnetometer X
    let mut buf = Cursor::new(&message_buffer[18..20]);
    let mag_z: f64 = buf.read_i16::<BigEndian>().unwrap() as f64 * MAG2T;


    // IMU Temperature
    let mut buf = Cursor::new(&message_buffer[20..22]);
    let temp: f64 = (buf.read_i16::<BigEndian>().unwrap() as f64 * TEMP2C) + C2K;

    // Ignore Aux ADC.

    let adis = ADIS {
        vcc: vcc,
        gyro_x: gyro_x,
        gyro_y: gyro_y,
        gyro_z: gyro_z,
        acc_x: accel_x,
        acc_y: accel_y,
        acc_z: accel_z,
        magn_x: mag_x,
        magn_y: mag_y,
        magn_z: mag_z,
        temp: temp,
    };

    adis
}
