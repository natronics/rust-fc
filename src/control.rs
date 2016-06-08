/*! # Vehicle Controls

Using control the rocket by filtering the vehicle state vector.

For [PSAS Launch-12](https://github.com/psas/Launch-12) there was only one
controllable device: a roll angle actuator (small twist-able set of fins). The
fin angle is computed using the output of a PID loop that is then normalized to
velocity.
*/

use state;

/// K_p, the proportional constant for PID loop
const KP: f64 = 5.0;

/// K_i, the integral constant for PID loop
const KI: f64 = 0.01;

/// K_d, the derivative constant for PID loop
const KD: f64 = 0.0;

/// The target control value (0 deg/s roll rate, in this case)
const PID_TARGET: f64 = 0.0;

/// Maximum integrator value we tolerate
const MAX_INTEGRATOR: f64 =  10000.0;

/// Minimum integrator value we tolerate
const MIN_INTEGRATOR: f64 = -10000.0;

/// Controller
pub struct Control {

    /// Current PID Integral
    pub integral: f64,

    /// Last error for PID Derivative
    pub last_error: f64,
}

/// Initialize PID loop to zeros.
impl Default for Control {
    fn default () -> Control {
        Control {
            integral: 0.0,
            last_error: 0.0,
        }
    }
}

impl Control {

    /// A PID loop controller.
    ///
    /// This is expected to be called every time there is a new State Vector
    ///
    /// ## Parameters:
    ///
    /// - **state**: State vector to operate on
    ///
    pub fn pid(&mut self, state: &state::State) {

        // Determine the error by taking the difference of the target and the
        // current value
        let error = PID_TARGET - state.roll_rate;

        // Proportional stage
        let proportional = KP * error;

        // Integral stage
        let integral = KI * self.integral;

        // Derivative stage
        let derivative = KD * (error - self.last_error);

        // Output of the PID controller
        let correction = proportional + integral + derivative;


        // Store values for next loop:
        self.last_error = error;

        // Add the error to the integral stage
	    self.integral += error;

        // Integrator clamping, helps dampen a run-away system
        if self.integral > MAX_INTEGRATOR {
            self.integral = MAX_INTEGRATOR;
        }
        else if self.integral < MIN_INTEGRATOR {
            self.integral = MIN_INTEGRATOR;
        }

	    // Look normalized fin angle based on requested angular acceleration
	    //double output = estimate_alpha(correction, *state);
    }
}
