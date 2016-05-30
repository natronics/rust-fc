mod devices;
mod io;
mod state;

fn main() {
    println!(r#"
 Rust-FC: PSAS Flight Computer rewrite in Rust
 : Copyright (C) 2016 Nathan Bergey

----------------------------------------------------------------
 This program comes with ABSOLUTELY NO WARRANTY;
 for details please visit http://www.gnu.org/licenses/gpl.html.

 This is free software, and you are welcome to redistribute it
 under certain conditions; For details, please visit
 http://www.gnu.org/licenses/gpl.html.
----------------------------------------------------------------

"#);

    // Set up a Flight Computer IO state
    let mut flight_comptuer: io::FC = Default::default();

    // New state vector
    let mut state: state::State = Default::default();

    let mut last_adis_message = 0;

    loop {

        // What the message is from matches on Port
        match flight_comptuer.listen() {
            Some((seqn, recv_port, recv_time, message)) => {
                match recv_port {

                    // Message from ADIS IMU
                    // When we get new IMU data we log it. If it's a new message then we
                    // update the state and send new data over the telemetry channel
                    io::PSAS_ADIS_PORT => {

                        flight_comptuer.log_message(&message, devices::ADIS_NAME, recv_time, devices::SIZE_OF_ADIS).unwrap();

                        if seqn == (last_adis_message + 1) {

                            // unpack message into values
                            let adis = devices::recv_adis(&message);

                            // update state
                            state.update_imu(recv_time, adis);

                            // log state, send ADIS over telemetry
                            flight_comptuer.log_message(&state.as_message(), state::STATE_NAME, recv_time, state::SIZE_OF_STATE).unwrap();
                            flight_comptuer.telemetry(&message, devices::ADIS_NAME, recv_time, devices::SIZE_OF_ADIS);
                        }
                        last_adis_message = seqn;
                    },
                    _ => { ; }
                }
            },
            None => { ; }  // Oh well. Keep listening.
        }
    }
}
