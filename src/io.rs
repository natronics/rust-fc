//! # rust-fc/IO
//!
//! Help with network and file-writing

extern crate byteorder;

use std::net::UdpSocket;
use std::io::Cursor;
use self::byteorder::{ReadBytesExt, BigEndian};

/// Ports for data
const PSAS_LISTEN_UDP_PORT: u16 = 36000;

/// Expected port for ADIS messages
pub const PSAS_ADIS_PORT: u16 = 35020;


/// Flight Computer IO.
pub struct FC {

    pub fc_listen_socket: UdpSocket,
}

impl Default for FC {
    fn default () -> FC {
        match UdpSocket::bind(("0.0.0.0", PSAS_LISTEN_UDP_PORT)) {
            Ok(socket) => { return FC {fc_listen_socket: socket} },
            Err(e) => { panic!(e) },
        }
    }
}

impl FC {

    /// Listen on the PSAS_LISTEN socket.
    ///
    /// # Returns:
    /// Received message SEQN, received message origin port.
    pub fn listen(&self, message: &mut [u8]) -> Option<(u32, u16)> {

        // A buffer to put data in from the port.
        // Should at least be the size of MTU (1500 bytes).
        let mut message_buffer: [u8; 1500] = [0; 1500];

        // Read from the socket (blocking!)
        // message_buffer gets filled and we get the number of bytes read
        // along with and address that the message came from
        match self.fc_listen_socket.recv_from(&mut message_buffer) {
            Ok((num_recv_bytes, recv_addr)) => {

                // We got data!
                println!("Received {} bytes from port {}", num_recv_bytes, recv_addr.port());

                // First 4 bytes are the sequence number
                let mut buf = Cursor::new(&message_buffer[..4]);
                let seqn = buf.read_u32::<BigEndian>().unwrap();

                println!("  SEQN: {}", seqn);

                // Rest of the bytes may be part of a message
                message.clone_from_slice(&message_buffer[4..]);

                Some((seqn, recv_addr.port()))
            },
            Err(e) => { None },  // continue
        }
    }
}
