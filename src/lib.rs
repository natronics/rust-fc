#![doc(html_root_url = "https://natronics.github.io/rust-fc/")]

//! # rust-fc
//!
//! A minimal clone of PSAS's [av3-fc](https://github.com/psas/av3-fc) rocket
//! flight computer executive process written in Rust for fun.
extern crate byteorder;

pub mod devices;
pub mod io;
