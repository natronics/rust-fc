/*! # State Vector

Tracking state of the rocket.
*/
extern crate byteorder;

use std::time;
use std::io::Cursor;
use self::byteorder::{WriteBytesExt, BigEndian};
use devices;

/// State message size (bytes)
pub const SIZE_OF_STATE: usize = 48;

/// State message name (ASCII: STAT)
pub const STATE_NAME: [u8;4] = [83, 84, 65, 84];


/// A representation for the current state vector in MKS units.
pub struct State {

    /// Exact time (nanoseconds from boot) that state-vector is valid for.
    pub time: u64,

    /// Vertical Acceleration [m/s²]
	pub acc_up: f64,

    /// Vertical velocity [m/s]
	pub vel_up: f64,

    /// Altitude above sea level [m]
	pub altitude: f64,

    /// Roll rate [deg/s]
	pub roll_rate: f64,

    /// Roll angle [deg] (zero is initial angle)
	pub roll_angle: f64,
}

/// Initialize State Vector to Launch Site.
impl Default for State {
    fn default () -> State {
        State {
            time: 0,
            acc_up: 0.0,
            vel_up: 0.0, 
            altitude: 1390.0,
            roll_rate: 0.0,
            roll_angle: 0.0,
        }
    }
}


impl State {

    /// Update the state based on new IMU data.
    ///
    /// This is expected to be called every time new data is available from
    /// a sensor.
    ///
    /// ## Parameters:
    ///
    /// - **time**: Time that new data is valid for
    /// - **imu**: New IMU data
    ///
    pub fn update_imu(&mut self, time: time::Duration, imu: devices::ADIS) {

        // Copy of last state to use in integration
        let t_last = self.time;
        let a_last = self.acc_up;
        let v_last = self.vel_up;

        // Apply new data
        self.time = (time.as_secs() * 1000000000) + time.subsec_nanos() as u64;

        // Subtract gravity!!!
        self.acc_up = imu.acc_x - 9.8;

        // Compute and update integrals
        let t_seconds = (self.time - t_last) as f64 / 1e9;
        self.vel_up += (t_seconds * (self.acc_up + a_last)) / 2.0;
        self.altitude += (t_seconds * (self.vel_up + v_last)) / 2.0;
    }

    /// Return a copy of this struct as a byte array.
    ///
    /// The PSAS file and message type is always a byte array with big-endian
    /// representation of fields in a struct.
    pub fn as_message(&mut self) -> [u8; SIZE_OF_STATE] {
        let mut buffer = [0u8; SIZE_OF_STATE];
        {
            let mut message = Cursor::<&mut [u8]>::new(&mut buffer);

            // Struct Fields:
            message.write_u64::<BigEndian>(self.time).unwrap();
            message.write_f64::<BigEndian>(self.acc_up).unwrap();
            message.write_f64::<BigEndian>(self.vel_up).unwrap();
            message.write_f64::<BigEndian>(self.altitude).unwrap();
            message.write_f64::<BigEndian>(self.roll_rate).unwrap();
            message.write_f64::<BigEndian>(self.roll_angle).unwrap();
        }
        buffer
    }
}
