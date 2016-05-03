Rust Flight Computer
====================

![Language: Rust](https://img.shields.io/badge/language-Rust-red.svg)
[![Build Status](https://travis-ci.org/natronics/rust-fc.svg?branch=master)](https://travis-ci.org/natronics/rust-fc)


A minimal clone of PSAS's [av3-fc][av3fc] rocket flight computer executive process written in Rust for fun.

This is a toy, built for fun to learn the Rust language.


Building
--------

You'll need the rust toolchain, check out the [official Rust docs][installrust] for help

To compile to program:

    $ cargo build


Running
-------

You can also use cargo to run the executable:

    $ cargo run

This will start the flight computer, however it will do nothing until data is feed into it. There is a small python test utility in the `test` directory that will generate a data packet and send it to the running flight computer process.

Start rust fc with `cargo run` then in another terminal run the test script:

    $ cd test
    $ ./send_data.py


Module Documentation
--------------------

Build locally with

    $ cargo doc

The pre-build docs are hosted here:

<https://natronics.github.io/rust-fc/>



[av3fc]: https://github.com/psas/av3-fc
[installrust]: https://www.rust-lang.org/downloads.html
