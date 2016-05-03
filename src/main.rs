#![doc(html_root_url = "https://natronics.github.io/rust-fc/")]

//! # rust-fc
//!
//! A minimal clone of PSAS's [av3-fc](https://github.com/psas/av3-fc) rocket
//! flight computer executive process written in Rust for fun.

extern crate byteorder;

use std::net::UdpSocket;
use std::net::SocketAddr;
use std::io::Error;
use std::mem;
use std::io::Cursor;
use byteorder::{ReadBytesExt, BigEndian};

const PSAS_LISTEN_UDP_PORT: u16 = 36000;
const PSAS_ADIS_PORT: u16 = 35020;

#[repr(C)]
struct ADIS16405Data {
    vcc: i16,
	gyro_x: i16,
	gyro_y: i16,
	gyro_z: i16,
	acc_x: i16,
	acc_y: i16,
	acc_z: i16,
	magn_x: i16,
	magn_y: i16,
	magn_z: i16,
	temp: i16,
	aux_adc: u16,
}
const SIZE_OF_ADISDATA: usize = 24;



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

    // The seqnunce number is the first 4 bytes
    let mut buf = Cursor::new(&raw_bytes[..4]);
    let seqn = buf.read_u32::<BigEndian>().unwrap();

    println!("    Packet SEQN: {}", seqn);

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
/// Get the message byte array into the ADIS data struct
pub fn recv_adis(buffer: &[u8]) {

    let buffer_ptr: *const u8 = buffer.as_ptr();
    let message_ptr: *const ADIS16405Data = buffer_ptr as *const _;
    let message_ref: &ADIS16405Data = unsafe { &*message_ptr };

    println!("    VCC: {}", message_ref.vcc);
    println!("    Gyro X: {}", message_ref.gyro_x);
    println!("    Accel X: {}", message_ref.acc_x);
}

fn main() {
    assert!(SIZE_OF_ADISDATA == mem::size_of::<ADIS16405Data>());

    println!("Launch!");

    demux_udp().unwrap();
}
