/*! # State Vector

Tracking state of the rocket.
*/
extern crate byteorder;

use std::time;
use self::byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};
use devices;

/// Current State Vector.
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

/// Initialize State Vector
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
    pub fn update_imu(&mut self, time: time::Duration, imu: devices::ADIS) {

        // Copy of last state to use in integration
        let t_last = self.time;
        let a_last = self.acc_up;
        //let v_last = self.vel_up;

        // Copy new data
        self.time = (time.as_secs() * 1000000000) + time.subsec_nanos() as u64;

        // Subtract gravity!!!
        self.acc_up = imu.acc_x - 9.8;

        // Compute and update integrals
        let t_seconds = (self.time - t_last) as f64 / 1e9;
        self.vel_up += (t_seconds * (self.acc_up + a_last)) / 2.0;

    }

    /// Return a copy of this struct as a byte array.
    pub fn as_message(&mut self) -> Vec<u8> {
        let mut message = Vec::with_capacity(50);

        // Time
        let mut t = Vec::with_capacity(8);
        t.write_u64::<BigEndian>(self.time).unwrap();
        message.extend_from_slice(&t);

        // Acc_up
        let mut v = Vec::with_capacity(8);
        v.write_f64::<BigEndian>(self.acc_up).unwrap();
        message.extend_from_slice(&v);

        // Vel_up
        let mut v = Vec::with_capacity(8);
        v.write_f64::<BigEndian>(self.vel_up).unwrap();
        message.extend_from_slice(&v);

        // Pack each field from this struct in the message
        message
    }
}      
