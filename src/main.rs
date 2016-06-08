mod devices;
mod io;
mod state;
mod control;

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

    // New controller
    let mut controller: control::Control = Default::default();

    // Track the sequence number for an ADIS message
    let mut adis_seqn_expected = 0;


    // The Flight Computer. Loop forever.
    loop {

        // Wait for a message from the network
        // Then match it to the message type based on the port it came from
        if let Some((seqn, recv_port, recv_time, message)) = flight_computer.listen() {
            match recv_port {

                // Message from ADIS IMU
                // =====================
                // When we get new IMU data we log it. If it's a new message then we
                // update the state and send new data over the telemetry channel
                io::PSAS_ADIS_PORT => {

                    // We expect monotonically increasing sequence numbers.
                    // Anything received out of order is ignored. Real time
                    // systems can't do anything with stale data!
                    if seqn < adis_seqn_expected {

                        // Out of order packet! Log it
                        let mut seqerror = io::SequenceError {
                            port: recv_port,
                            expected: adis_seqn_expected,
                            received: seqn,
                        };
                        flight_computer.log_message(&seqerror.as_message(), io::SEQE_NAME, recv_time, io::SIZE_OF_SEQE).unwrap();
                    }
                    // As long as we have *new* data it's okay
                    if seqn >= adis_seqn_expected {

                        // but if it's from the future it's still an error, Log it.
                        if seqn > adis_seqn_expected {
                            let mut seqerror = io::SequenceError {
                                port: recv_port,
                                expected: adis_seqn_expected,
                                received: seqn,
                            };
                            flight_computer.log_message(&seqerror.as_message(), io::SEQE_NAME, recv_time, io::SIZE_OF_SEQE).unwrap();
                        }

                        // Unpack binary message into proper values with units
                        let adis = devices::recv_adis(&message);

                        // Since this is IMU data, we need to update the state vector
                        state.update_imu(recv_time, adis);

                        // Do control based on new state
                        controller.pid(&state);

                        // Log ADIS and STAT. Send ADIS out over telemetry
                        flight_computer.log_message(&message, devices::ADIS_NAME, recv_time, devices::SIZE_OF_ADIS).unwrap();
                        flight_computer.log_message(&state.as_message(), state::STATE_NAME, recv_time, state::SIZE_OF_STATE).unwrap();
                        flight_computer.telemetry(&message, devices::ADIS_NAME, recv_time, devices::SIZE_OF_ADIS);

                        // Update sequence number counter
                        adis_seqn_expected = seqn + 1;
                    }
                },


                // Unknown Message Type
                // ====================
                // We don't know what message this is, skip it.
                _ => { ; }
            }
        }
    }
}
