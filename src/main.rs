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
        let mut message: [u8; 1496] = [0;1496];
        match flight_comptuer.listen(&mut message) {
            Some((seqn, recv_port)) => {
                match recv_port {
                    io::PSAS_ADIS_PORT => {

                        flight_comptuer.log_message(&message, devices::ADIS_NAME, devices::SIZE_OF_ADIS).unwrap();

                        // Only process if correct data:
                        if seqn == (last_adis_message + 1) {

                            let adis = devices::recv_adis(&message);
                            state.update_imu(adis);
                            flight_comptuer.telemetry(&message, devices::ADIS_NAME, devices::SIZE_OF_ADIS);
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
