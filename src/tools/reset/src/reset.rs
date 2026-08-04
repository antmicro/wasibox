/*
 * Copyright (c) 2026 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env;
use std::io;
use std::os::fd::AsRawFd;

use wasi_ext_lib::{tcgetattr, tcsetattr, termios, Fd, TcsetattrAction};

pub const CLEAR_ESCAPE_CODE: &str = "\x1b[2J\x1b[H";

pub fn reset(_args: env::Args) -> io::Result<()> {
    let mut term = match tcgetattr(io::stdin().as_raw_fd() as Fd) {
        Ok(t) => t,
        Err(e) => return Err(io::Error::from_raw_os_error(e)),
    };

    print!("{}", CLEAR_ESCAPE_CODE);
    term.c_lflag = termios::ISIG
        | termios::ICANON
        | termios::ECHO
        | termios::ECHOE
        | termios::ECHOK
        | termios::IEXTEN;

    if let Err(e) = tcsetattr(
        io::stdin().as_raw_fd() as Fd,
        TcsetattrAction::TCSANOW,
        &term,
    ) {
        Err(io::Error::from_raw_os_error(e))
    } else {
        Ok(())
    }
}
