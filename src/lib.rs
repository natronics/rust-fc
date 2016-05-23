/*! # rust-fc

A minimal clone of PSAS's [av3-fc](https://github.com/psas/av3-fc) rocket
flight computer executive process written in Rust for fun.

## Motivation

Rust is a newer language designed to be a fast systems programing language with
guaranteed memory safety. I wanted to play with Rust and build something
somewhat non-trivial to get to know the language and it's features.

For many years I've helped [PSAS](http://psas.pdx.edu) design, build, and fly
high power experimental rockets. Over the years we've built a fairly complex
"space program" including lots of pieces of code, from simulation and design,
to flight worthy firmware.

Here's what I'll re-write in Rust:


## The PSAS Flight Computer

The central program that controls and orchestrates the flight of the PSAS
rocket is a single program written in C executed on a small computer in the
rocket running Linux.

The rocket hardware and flight computer design was always a fast moving target
as we tried different modular systems, iterating on what works and throwing
away what doesn't. So there was never any formal specification document.
Instead we'll use the most recent ([Launch-12](https://github.com/psas/Launch-12))
implementation as our guide.


### Flight Computer Architecture: Overview

The rocket flight computer stack is actually made up of lots of computers. There
are several STM32 microcontrollers that do hard-real-time work like talking to
sensors or managing power. There is a central computer (an Intel Atom
processor) running Linux with almost nothing installed except our main flight
computer code. All inter-module communication is over a dedicated Ethernet
network on the rocket.

A basic diagram looks like this:

                ROCKET                                              GROUND
    
     ---------------      Ethernet     +------+     WiFi      ----------------
    | Sensor/Device | ===============> |  FC  | ~ ~ ~ ~ ~ ~> | Ground station |
     ---------------                   +------+               ----------------
                                          || 
     ---------------------     Ethernet   ||
    | Actuator (Controls) | <=============++
     --------------------- 

The main purpose of the Flight Computer then is to

 1. **Read in data** from the network
 2. **Log data** to local file system for analysis later
 3. Efficiently pack and then **relay current data to the ground** (over WiFi)
 4. **Update the current state vector** by integrating IMU sensor data
 5. **Send control messages** out to actuators on the rocket by applying a control filter to the state vector


## What I'll Copy In Rust

The full flight computer was built over many years with features like a
[code-generated `main` function](https://github.com/psas/elderberry) that
writes an event loop to control the program flow, and pieces of code for
consistent ARM and SAFE commands (the flight computer could disable the
ignition circuit on certain error conditions before launch), and a PID
roll-stabilization control system.

I don't feel like it's necessary to duplicated _every_ single piece of that
functionality here, so instead I'll try for a minimum viable product that:

 - Receives messages (over a network) of one type: the IMU data
 - Tracks state in 1 dimension (vertical height and speed only)
 - Log data to disk
 - Send telemetry back over a network
 - Send some kind of 'control' message based on current state

Specifically there is no event loop and no commands to interpret, so it should
be much simpler to implement.


## The Rust Implementation

There are a few modules for keeping similar code together:

 - A **devices** module that will know how to read IMU data from an array of bytes
 - An **io** module that will keep track of all sockets and file handlers. This is the main interface for reading and writing data
 - A **state** module that will track state
 - And a **control** module that compute a control signal

This is not too different than how we divided up the original C flight
computer.

The `main` function will simply initialize the modules and then loop forever
listening for data.
*/

extern crate byteorder;

pub mod devices;
pub mod io;
