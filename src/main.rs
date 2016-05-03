#![doc(html_root_url = "https://natronics.github.io/rust-fc/")]

//! # rust-fc
//!
//! A minimal clone of PSAS's [av3-fc](https://github.com/psas/av3-fc) rocket
//! flight computer executive process written in Rust for fun.

extern crate byteorder;

use std::net::UdpSocket;
use std::net::SocketAddr;
use std::io::Error;
use std::io::Cursor;
use byteorder::{ReadBytesExt, BigEndian};

const PSAS_LISTEN_UDP_PORT: u16 = 36000;
const PSAS_ADIS_PORT: u16 = 35020;

/// Gravity
const G_0: f64 = 9.80665;

/// Listen on the data port for messages
///
/// Binds to the PSAS data port and listens for messages. Listening is blocking
/// because if we're not receiving data, we do nothing (sleep).
pub fn demux_udp() -> Result<(), Error> {

    println!("Listening UDP ...");

    // bind our socket to the PSAS data port:
    let socket = try!(UdpSocket::bind(("0.0.0.0", PSAS_LISTEN_UDP_PORT)));

    // A buffer to put data in from the port. At least size of MTU (1500 bytes)
    let mut buf: [u8; 1500] = [0; 1500];

    // Read from the port (blocking!)
    // buf gets filled and we get the number of bytes read along with and
    // address that the message came from
    let (num_recv_bytes, recv_addr) = try!(socket.recv_from(&mut buf));
    println!("Ping! Received {} bytes from port {}", num_recv_bytes, recv_addr.port());

    // Send it to the sequenced packet receiver
    sequence_recv(recv_addr, buf);

    Ok(())
}


/// Receive a packet off the wire and get it's sequence number.
///
/// We'll keep track of what packet types we've received and their sequence
/// numbers. If a packed arrives out of order we log it, but otherwise throw it
/// away.
pub fn sequence_recv(recv_addr: SocketAddr, raw_bytes: [u8; 1500]) {

    // The sequence number is the first 4 bytes
    let mut buf = Cursor::new(&raw_bytes[..4]);
    let seqn = buf.read_u32::<BigEndian>().unwrap();

    println!("    Packet Sequence number: {}", seqn);

    // The message is the rest
    let message = &raw_bytes[4..];

    // The PSAS system message types are defined by the port they're sent from
    match recv_addr.port() {
        PSAS_ADIS_PORT => recv_adis(message),
        _ => { ; } // Default case: do nothing
    }


}

/// Receives an ADIS message
///
/// Unrwap a byte array assuming network endian into fields in the ADIS Data
/// type.
pub fn recv_adis(buffer: &[u8]) {

    println!("    Packet type: ADIS");

    // Convert fields:

    // VCC
    let mut buf = Cursor::new(&buffer[..2]);
    let mut vcc: f64 = buf.read_i16::<BigEndian>().unwrap() as f64;
    vcc = vcc * 0.002418;

    // Gyro X
    let mut buf = Cursor::new(&buffer[2..4]);
    let mut gyro_x: f64 = buf.read_i16::<BigEndian>().unwrap() as f64;
    gyro_x = gyro_x * 0.05;

    // Accel X
    let mut buf = Cursor::new(&buffer[8..10]);
    let mut accel_x: f64 = buf.read_i16::<BigEndian>().unwrap() as f64;
    accel_x = accel_x * 0.00333 * G_0;

    println!("    VCC: {}", vcc);
    println!("    Gyro X: {}", gyro_x);
    println!("    Accel X: {}", accel_x);

}

fn main() {
    println!("Launch!");

    demux_udp().unwrap();
}
