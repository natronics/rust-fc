/*! # State Vector

Tracking state of the rocket.
*/
use devices;

/// Current State Vector.
pub struct State {

    /// Exact time (seconds from boot) that state-vector is valid for.
    pub time: f64,

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
            time: 0.0,
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
    pub fn update_imu(&mut self, imu: devices::ADIS) {

        self.acc_up = imu.acc_x;
    }
}      
