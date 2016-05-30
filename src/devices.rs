/*! # Device specifications

This module defines the devices on the flight computer that we wish to receive
data from. It knows about and can read and write messages for device data.
*/

extern crate byteorder;

use std::io::Cursor;
use self::byteorder::{ReadBytesExt, BigEndian};

/// Unwrapped and converted ADIS IMU data.
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

/// ADIS message size (bytes)
pub const SIZE_OF_ADIS: usize = 24;

/// ADIS message name (ASCII: ADIS)
pub const ADIS_NAME: [u8;4] = [65, 68, 73, 83];

/// Read an ADIS message from raw bytes.
///
/// Unrwap a byte array (assuming network endianess) into fields for the ADIS
/// Data type. This will also do the conversion from ADC counts to proper (SI)
/// Units.
///
/// ## Parameters
///
/// - **message_buffer**: A buffer of bytes at least as long as the ADIS
/// message (`SIZE_OF_ADIS`).
pub fn recv_adis(message_buffer: &[u8]) -> ADIS {

    let mut message = Cursor::new(message_buffer);

    // Each read from the cursor will get 2 bytes out of the message.
    // Read each field and convert to appropriate units, then return populated
    // struct.
    ADIS {
        vcc:    message.read_i16::<BigEndian>().unwrap() as f64 * VCC2VOLTS,
        gyro_x: message.read_i16::<BigEndian>().unwrap() as f64 * GYRO2DEGS,
        gyro_y: message.read_i16::<BigEndian>().unwrap() as f64 * GYRO2DEGS,
        gyro_z: message.read_i16::<BigEndian>().unwrap() as f64 * GYRO2DEGS,
        acc_x:  message.read_i16::<BigEndian>().unwrap() as f64 * ACC2G * G_0,
        acc_y:  message.read_i16::<BigEndian>().unwrap() as f64 * ACC2G * G_0,
        acc_z:  message.read_i16::<BigEndian>().unwrap() as f64 * ACC2G * G_0,
        magn_x: message.read_i16::<BigEndian>().unwrap() as f64 * MAG2T,
        magn_y: message.read_i16::<BigEndian>().unwrap() as f64 * MAG2T,
        magn_z: message.read_i16::<BigEndian>().unwrap() as f64 * MAG2T,
        temp:  (message.read_i16::<BigEndian>().unwrap() as f64 * TEMP2C) + C2K,
    }
}
