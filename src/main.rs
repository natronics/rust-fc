mod devices;
mod io;

/// State: position and velocity
pub struct State {
    x: f64,
    v: f64,
}

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
    let flight_comptuer = io::FC { ..Default::default() };

    // Initialize state
    let mut state = State { x: 0.0, v: 0.0 };

    let mut last_adis_message = 0;

    let mut message: [u8; 1496] = [0;1496];
    match flight_comptuer.listen(&mut message) {
        Some((seqn, recv_port)) => {
            match recv_port {
                io::PSAS_ADIS_PORT => {
                    last_adis_message = seqn;
                    let adis = devices::recv_adis(&message);
                    println!("  VCC: {}", adis.vcc);
                },
                _ => { ; }
            }
        },
        None => { ; }  // Oh well. Keep listening.
    }
}
