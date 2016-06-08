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
    let mut flight_computer: io::FC = Default::default();

    // New state vector
    let mut state: state::State = Default::default();

    // Track the sequence number for an ADIS message
    let mut last_adis_message = 0;


    // The Flight Computer. Loop forever.
    loop {

        // Wait for a message from the network
        // Then match it to the message type based on the port it came from
        match flight_computer.listen() {
            Some((seqn, recv_port, recv_time, message)) => {
                match recv_port {

                    // Message from ADIS IMU
                    // =====================
                    // When we get new IMU data we log it. If it's a new message then we
                    // update the state and send new data over the telemetry channel
                    io::PSAS_ADIS_PORT => {

                        // We expect monotonically increasing sequence numbers.
                        // Anything received out of order is ignored. Real time
                        // systems can't do anything with stale data!
                        if seqn == (last_adis_message + 1) {

                            // Unpack binary message into proper values with units
                            let adis = devices::recv_adis(&message);

                            // Since this is IMU data, we need to update the state vector
                            state.update_imu(recv_time, adis);

                            // Log updated state
                            flight_computer.log_message(&state.as_message(), state::STATE_NAME, recv_time, state::SIZE_OF_STATE).unwrap();

                            // Log ADIS message and send it out over telemetry
                            flight_computer.log_message(&message, devices::ADIS_NAME, recv_time, devices::SIZE_OF_ADIS).unwrap();
                            flight_computer.telemetry(&message, devices::ADIS_NAME, recv_time, devices::SIZE_OF_ADIS);
                        }

                        // Update sequence number counter
                        last_adis_message = seqn;
                    },


                    // Unknown Message Type
                    // ====================
                    // We don't know what message this is, skip it.
                    _ => { ; }                  
                }
            },
            None => { ; }  // Oh well. Keep listening.
        }
    }
}
