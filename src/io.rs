/*! # Input and Output

This module holds a single struct (`FC`) that keeps what would otherwise be global
state of the running process. All the open file handles and sockets are stored
here as well as helper functions to properly initialize the program and all the
necessary packing and work to receive, log, and send any data.


In many ways this is the guts of the flight computer.

*/


extern crate byteorder;

use std::net::UdpSocket;
use std::net::SocketAddrV4;
use std::net::Ipv4Addr;
use std::io::Error;
use std::io::Cursor;
use std::fs::File;
use std::io::Write;
use std::time;


use self::byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

/// Ports for data
const PSAS_LISTEN_UDP_PORT: u16 = 36000;

/// Port for outgoing telemetry
const PSAS_TELEMETRY_UDP_PORT: u16 = 35001;

/// Expected port for ADIS messages
pub const PSAS_ADIS_PORT: u16 = 35020;

/// Maximum size of single telemetry packet
const P_LIMIT: usize = 1432;

/// Size of PSAS Packet header
const HEADER_SIZE: usize = 12;

/// Message name (ASCII: SEQN)
const SEQN_NAME: [u8;4] = [83, 69, 81, 78];

/// Sequence Error message size (bytes)
pub const SIZE_OF_SEQE: usize = 10;

/// Sequence Error message name (ASCII: SEQE)
pub const SEQE_NAME: [u8;4] = [83, 69, 81, 69];


/// Flight Computer IO.
///
/// Internally holds state for this implementation of the flight computer.
/// This includes time (nanosecond counter from startup), a socket for
/// listening for incoming data, a socket for sending data over the telemetry
/// link, a log file, a running count of telemetry messages sent and a buffer
/// for partly built telemetry messages.
///
/// To initialize use the Default trait:
///
/// # Example
///
/// ```no_run
/// use rust_fc::io;
///
/// let mut flight_computer: io::FC = Default::default();
/// ```
pub struct FC {

    /// Instant we started
    boot_time: time::Instant,

    /// Socket to listen on for messages.
    fc_listen_socket: UdpSocket,

    /// Socket to send telemetry.
    telemetry_socket: UdpSocket,

    /// File to write data to.
    fc_log_file: File,

    /// Current count of telemetry messages sent.
    sequence_number: u32,

    /// Buffer of messages to build a telemetry Packet.
    telemetry_buffer: Vec<u8>,
}


// Reusable code for packing header into bytes
fn pack_header(name: [u8; 4], time: time::Duration, message_size: usize) -> [u8; HEADER_SIZE] {

    let mut buffer = [0u8; HEADER_SIZE];
    {
        let mut header = Cursor::<&mut [u8]>::new(&mut buffer);

        // Fields:
        // ID (Four character code)
        header.write(&name).unwrap();

        // Timestamp, 6 bytes nanoseconds from boot
        let nanos: u64 = (time.as_secs() * 1000000000) + time.subsec_nanos() as u64;
        let mut time_buffer = [0u8; 8];
        {
            let mut t = Cursor::<&mut [u8]>::new(&mut time_buffer);
            t.write_u64::<BigEndian>(nanos).unwrap();
        }
        // Truncate to 6 least significant bytes
        header.write(&time_buffer[2..8]).unwrap();

        // Size:
        header.write_u16::<BigEndian>(message_size as u16).unwrap();
    }
    buffer
}


impl Default for FC {
    fn default () -> FC {

        // Boot time
        let boot_time = time::Instant::now();

        let fc_listen_socket: UdpSocket;
        let telemetry_socket: UdpSocket;
        let fc_log_file: File;

        // Try and open listen socket
        match UdpSocket::bind(("0.0.0.0", PSAS_LISTEN_UDP_PORT)) {
            Ok(socket) => { fc_listen_socket = socket; },
            Err(e) => { panic!(e) },
        }

        // Try and open telemetry socket
        match UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => { telemetry_socket = socket; },
            Err(e) => { panic!(e) },
        }


        // Try and open log file, loop until we find a name that's not taken
        let mut newfilenum = 0;
        loop {
            let filename = format!("logfile-{:03}", newfilenum);
            match File::open(filename) {
                // If this works, keep going
                Ok(_) => { newfilenum += 1; },
                // If this fails, make a new file
                Err(_) => { break; }
            }
        }

        // We got here, so open the file
        match File::create(format!("logfile-{:03}", newfilenum)) {
            Ok(file) => { fc_log_file = file; },
            Err(e) => { panic!(e) },
        }

        // Put first sequence number (always 0) in the telemetry buffer.
        let mut telemetry_buffer = Vec::with_capacity(P_LIMIT);
        telemetry_buffer.extend_from_slice(&[0, 0, 0, 0]);

        // Initialise
        let mut fc = FC {
            boot_time: boot_time,
            fc_listen_socket: fc_listen_socket,
            telemetry_socket: telemetry_socket,
            fc_log_file: fc_log_file,
            sequence_number: 0,
            telemetry_buffer: telemetry_buffer,
        };

        // Write log header
        fc.log_message(&[0, 0, 0, 0], SEQN_NAME, time::Duration::new(0, 0), 4).unwrap();
 
        fc
    }
}


impl FC {

    /// Listen for messages from the network.
    ///
    /// This makes a blocking `read` call on the `fc_listen_socket`, waiting
    /// for any message from the outside world. Once received, it will deal
    /// with the sequence numbers in the header of the data and write the raw
    /// message to the passed in buffer.
    ///
    /// #  Returns:
    ///
    /// An Option containing a tuple of data from the socket. The tuple
    /// contains:
    ///
    /// - **Sequence Number**: Sequence number from the header of the data packet
    /// - **Port**: Which port the data was sent _from_
    /// - **Time**: The time that the message was received
    /// - **Message**: A message buffer that has raw bytes off the wire
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rust_fc::io;
    ///
    /// let mut flight_computer: io::FC = Default::default();
    ///
    /// if let Some((seqn, recv_port, recv_time, message)) = flight_computer.listen() {
    ///     // Do something here with received data
    /// }
    /// ```
    pub fn listen(&self) -> Option<(u32, u16, time::Duration, [u8; P_LIMIT - 4])> {

        // A buffer to put data in from the port.
        // Should at least be the size of telemetry message.
        let mut message_buffer = [0u8; P_LIMIT];

        // Read from the socket (blocking!)
        // message_buffer gets filled and we get the number of bytes read
        // along with and address that the message came from
        match self.fc_listen_socket.recv_from(&mut message_buffer) {
            Ok((_, recv_addr)) => {

                // Get time for incoming data
                let recv_time = time::Instant::now().duration_since(self.boot_time);

                // First 4 bytes are the sequence number
                let mut buf = Cursor::new(&message_buffer[..4]);
                let seqn = buf.read_u32::<BigEndian>().unwrap();

                // Rest of the bytes may be part of a message
                let mut message = [0u8; P_LIMIT - 4];
                message.clone_from_slice(&message_buffer[4..P_LIMIT]);

                Some((seqn, recv_addr.port(), recv_time, message))
            },
            Err(_) => { None },  // continue
        }
    }

    /// Log a message to disk.
    ///
    /// All data we care about can be encoded as a "message". The original code
    /// defined messages as packed structs in C. The reasoning was to be as
    /// space-efficient as reasonably possible given that we are both disk-size
    /// and bandwidth constrained.
    ///
    /// This function takes a message (as an array of bytes) and writes it to
    /// disk.
    ///
    /// ## Parameters:
    ///
    /// - **message**: Byte array containing packed message
    /// - **name**: Byte array of the name for this message
    /// - **time**: Time of message
    /// - **message_size**: How many bytes to copy from the message array
    ///
    /// ## Returns:
    ///
    /// A Result with any errors. But we hope to never deal with a failure here
    /// (Greater care was taken in the original flight computer to not crash
    /// because of disk errors).
    pub fn log_message(&mut self, message: &[u8], name: [u8; 4], time: time::Duration, message_size: usize) -> Result<(), Error> {

        // Header:
        let header = pack_header(name, time, message_size);
        try!(self.fc_log_file.write(&header));

        // message:
        try!(self.fc_log_file.write(&message[0..message_size]));

        Ok(())
    }



    /// Send a message to the ground.
    ///
    /// All data we care about can be encoded as a "message". The original code
    /// defined messages as packed structs in C. The reasoning was to be as
    /// space-efficient as reasonably possible given that we are both disk-size
    /// and bandwidth constrained.
    ///
    /// This function takes a message (as an array of bytes) and queues it to
    /// be send out over the network once will fill the maximum size of a UDP
    /// packet.
    ///
    /// ## Parameters
    ///
    /// - **message**: Byte array containing packed message
    /// - **name**: Byte array of the name for this message
    /// - **time**: Time of message
    /// - **message_size**: How many bytes to copy from the message array
    ///
    pub fn telemetry(&mut self, message: &[u8], name: [u8; 4], time: time::Duration, message_size: usize) {

        // If we won't have room in the current packet, flush
        if (self.telemetry_buffer.len() + HEADER_SIZE + message_size) > P_LIMIT {
            self.flush_telemetry();
        }

        // Header:
        let header = pack_header(name, time, message_size);
        self.telemetry_buffer.extend_from_slice(&header);

        // Message:
        self.telemetry_buffer.extend_from_slice(&message[0..message_size]);
    }

    /// This will actually send the now full and packed telemetry packet,
    /// and set us up for the next one.
    fn flush_telemetry(&mut self) {

        // Push out the door
        let telemetry_addr: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), PSAS_TELEMETRY_UDP_PORT);

        // When did we send this packet
        let send_time = time::Instant::now().duration_since(self.boot_time);

        self.telemetry_socket.send_to(&self.telemetry_buffer, telemetry_addr).unwrap();

        // Increment SEQN
        self.sequence_number += 1;

        // Start telemetry buffer over
        self.telemetry_buffer.clear();

        // Prepend with next sequence number
        let mut seqn = Vec::with_capacity(4);
        seqn.write_u32::<BigEndian>(self.sequence_number).unwrap();
        self.telemetry_buffer.extend_from_slice(&mut seqn);

        // Keep track of sequence numbers in the flight computer log too
        self.log_message(&seqn, SEQN_NAME, send_time, 4).unwrap();
    }
}

/// A sequence error message.
///
/// When we miss a packet or get an out of order packet we should log that
/// for future analysis. This stores a error and builds a log-able message.
///
/// # Example
///
/// ```
/// use rust_fc::io;
///
/// let seqerror = io::SequenceError {
///     port: 35020,
///     expected: 12345,
///     received: 12349,
/// };
///
/// // Now you can log or send the message somewhere
/// ```
pub struct SequenceError {

    /// Which port the packet error is from
    pub port: u16,

    /// Expected packet sequence number
    pub expected: u32,

    /// Actual received packet sequence number
    pub received: u32,
}


impl SequenceError {

    /// Return a copy of this struct as a byte array.
    ///
    /// The PSAS file and message type is always a byte array with big-endian
    /// representation of fields in a struct.
    pub fn as_message(&self) -> [u8; SIZE_OF_SEQE] {
        let mut buffer = [0u8; SIZE_OF_SEQE];
        {
            let mut message = Cursor::<&mut [u8]>::new(&mut buffer);

            // Struct Fields:
            message.write_u16::<BigEndian>(self.port).unwrap();
            message.write_u32::<BigEndian>(self.expected).unwrap();
            message.write_u32::<BigEndian>(self.received).unwrap();
        }
        buffer
    }
}
