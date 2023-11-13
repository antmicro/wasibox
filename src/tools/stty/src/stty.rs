/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env;
use std::io;
use std::os::fd::AsRawFd;

use wasi_ext_lib::{tcgetattr, tcsetattr, tcgetwinsize, termios, Fd, TcsetattrAction};

pub fn stty(args: env::Args) -> io::Result<()> {
    let mut termios = match tcgetattr(io::stdin().as_raw_fd() as Fd) {
        Ok(termios) => termios,
        Err(e) => return Err(io::Error::from_raw_os_error(e)),
    };

    for arg in args {
        let on = !arg.starts_with('-');
        let arg_: &str = if on { &arg } else { &arg[1..] };

        match arg_ {
            "ignbrk" | "brkint" | "ignpar" | "parmrk" | "inpck" | "istrip" | "inlcr" | "igncr" | "icrnl" | "iuclc" | "ixon" | "ixany" | "ixoff" | "imaxbel" | "iutf8" => {
                let flag = match arg_ {
                    "ignbrk" => termios::IGNBRK,
                    "brkint" => termios::BRKINT,
                    "ignpar" => termios::IGNPAR,
                    "parmrk" => termios::PARMRK,
                    "inpck" => termios::INPCK,
                    "istrip" => termios::ISTRIP,
                    "inlcr" => termios::INLCR,
                    "igncr" => termios::IGNCR,
                    "icrnl" => termios::ICRNL,
                    "iuclc" => termios::IUCLC,
                    "ixon" => termios::IXON,
                    "ixany" => termios::IXANY,
                    "ixoff" => termios::IXOFF,
                    "imaxbel" => termios::IMAXBEL,
                    "iutf8" => termios::IUTF8,
                    _ => unreachable!(),
                };

                if on {
                    termios.c_iflag |= flag;
                } else {
                    termios.c_iflag &= !flag;
                }
            }
            "opost" | "olcuc" | "onlcr" | "ocrnl" | "onocr" | "onlret" | "ofill" | "ofdel" => {
                let flag = match arg_ {
                    "opost" => termios::OPOST,
                    "olcuc" => termios::OLCUC,
                    "onlcr" => termios::ONLCR,
                    "ocrnl" => termios::OCRNL,
                    "onocr" => termios::ONOCR,
                    "onlret" => termios::ONLRET,
                    "ofill" => termios::OFILL,
                    "ofdel" => termios::OFDEL,
                    _ => unreachable!(),
                };

                if on {
                    termios.c_oflag |= flag;
                } else {
                    termios.c_oflag &= !flag;
                }
            }
            "cs5" | "cs6" | "cs7" | "cs8" | "cstopb" | "cread" | "parenb" | "parodd" | "hupcl" | "clocal" => {
                let flag = match arg_ {
                    "cs5" => termios::CS5,
                    "cs6" => termios::CS6,
                    "cs7" => termios::CS7,
                    "cs8" => termios::CS8,
                    "cstopb" => termios::CSTOPB,
                    "cread" => termios::CREAD,
                    "parenb" => termios::PARENB,
                    "parodd" => termios::PARODD,
                    "hupcl" => termios::HUPCL,
                    "clocal" => termios::CLOCAL,
                    _ => unreachable!(),
                };

                if on {
                    termios.c_cflag |= flag;
                } else {
                    termios.c_cflag &= !flag;
                }
            }
            "isig" | "icanon" | "echo" | "echoe" | "echok" | "echonl" | "noflsh" | "tostop" | "iexten" => {
                let flag = match arg_ {
                    "isig" => termios::ISIG,
                    "icanon" => termios::ICANON,
                    "echo" => termios::ECHO,
                    "echoe" => termios::ECHOE,
                    "echok" => termios::ECHOK,
                    "echonl" => termios::ECHONL,
                    "noflsh" => termios::NOFLSH,
                    "tostop" => termios::TOSTOP,
                    "iexten" => termios::IEXTEN,
                    _ => unreachable!(),
                };

                if on {
                    termios.c_lflag |= flag;
                } else {
                    termios.c_lflag &= !flag;
                }
            }
            "size" => {
                match tcgetwinsize(io::stdin().as_raw_fd() as Fd) {
                    Ok(size) => println!("Rows: {}, columns: {}", size.ws_row, size.ws_col),
                    Err(e) => return Err(io::Error::from_raw_os_error(e))
                }
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid argument",
                ));
            }
        }
    }
    
    if let Err(e) = tcsetattr(
        io::stdin().as_raw_fd() as Fd,
        TcsetattrAction::TCSANOW,
        &termios
    ) {
        Err(io::Error::from_raw_os_error(e))
    } else {
        Ok(())
    }
}
