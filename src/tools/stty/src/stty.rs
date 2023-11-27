/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env;
use std::io;
use std::os::fd::AsRawFd;

use termios::tcflag_t;
use wasi_ext_lib::{tcgetattr, tcgetwinsize, tcsetattr, termios, Fd, TcsetattrAction};

fn get_size() -> io::Result<(usize, usize)> {
    match tcgetwinsize(io::stdin().as_raw_fd() as Fd) {
        Ok(size) => Ok((size.ws_row as usize, size.ws_col as usize)),
        Err(e) => Err(io::Error::from_raw_os_error(e)),
    }
}

fn print_termios(termios_p: &termios::termios) -> io::Result<()> {
    fn is_flag_set(field: tcflag_t, flag: tcflag_t) -> &'static str {
        if (field & flag) != 0 {
            ""
        } else {
            "-"
        }
    }

    fn get_csn(field: tcflag_t) -> &'static str {
        match field & termios::CSIZE {
            termios::CS5 => "cs5",
            termios::CS6 => "cs6",
            termios::CS7 => "cs7",
            termios::CS8 => "cs8",
            _ => unreachable!(),
        }
    }

    let (rows, columns) = get_size()?;
    println!("rows {rows}; columns {columns};");

    // termios.c_iflag
    println!(
        "{}ignbrk {}brkint {}ignpar {}parmrk {}inpck {}istrip {}inlcr {}igncr {}icrnl {}iuclc {}ixon {}ixany {}ixoff {}imaxbel {}iutf8",
        is_flag_set(termios_p.c_iflag, termios::IGNBRK),
        is_flag_set(termios_p.c_iflag, termios::BRKINT),
        is_flag_set(termios_p.c_iflag, termios::IGNBRK),
        is_flag_set(termios_p.c_iflag, termios::PARMRK),
        is_flag_set(termios_p.c_iflag, termios::INPCK),
        is_flag_set(termios_p.c_iflag, termios::ISTRIP),
        is_flag_set(termios_p.c_iflag, termios::INLCR),
        is_flag_set(termios_p.c_iflag, termios::IGNCR),
        is_flag_set(termios_p.c_iflag, termios::ICRNL),
        is_flag_set(termios_p.c_iflag, termios::IUCLC),
        is_flag_set(termios_p.c_iflag, termios::IXON),
        is_flag_set(termios_p.c_iflag, termios::IXANY),
        is_flag_set(termios_p.c_iflag, termios::IXOFF),
        is_flag_set(termios_p.c_iflag, termios::IMAXBEL),
        is_flag_set(termios_p.c_iflag, termios::IUTF8)
    );

    // termios.c_oflag
    println!(
        "{}opost {}olcuc {}onlcr {}ocrnl {}onocr {}onlret {}ofill {}ofdel",
        is_flag_set(termios_p.c_oflag, termios::OPOST),
        is_flag_set(termios_p.c_oflag, termios::OLCUC),
        is_flag_set(termios_p.c_oflag, termios::ONLCR),
        is_flag_set(termios_p.c_oflag, termios::OCRNL),
        is_flag_set(termios_p.c_oflag, termios::ONOCR),
        is_flag_set(termios_p.c_oflag, termios::ONLRET),
        is_flag_set(termios_p.c_oflag, termios::OFILL),
        is_flag_set(termios_p.c_oflag, termios::OFDEL)
    );

    // termios.c_cflag
    println!(
        "{} {}cstopb {}cread {}parenb {}parodd {}hupcl {}clocal",
        get_csn(termios_p.c_cflag),
        is_flag_set(termios_p.c_cflag, termios::CSTOPB),
        is_flag_set(termios_p.c_cflag, termios::CREAD),
        is_flag_set(termios_p.c_cflag, termios::PARENB),
        is_flag_set(termios_p.c_cflag, termios::PARODD),
        is_flag_set(termios_p.c_cflag, termios::HUPCL),
        is_flag_set(termios_p.c_cflag, termios::CLOCAL)
    );

    // termios.c_lflag
    println!(
        "{}isig {}icanon {}echo {}echoe {}echok {}echonl {}noflsh {}tostop {}iexten",
        is_flag_set(termios_p.c_lflag, termios::ISIG),
        is_flag_set(termios_p.c_lflag, termios::ICANON),
        is_flag_set(termios_p.c_lflag, termios::ECHO),
        is_flag_set(termios_p.c_lflag, termios::ECHOE),
        is_flag_set(termios_p.c_lflag, termios::ECHOK),
        is_flag_set(termios_p.c_lflag, termios::ECHONL),
        is_flag_set(termios_p.c_lflag, termios::NOFLSH),
        is_flag_set(termios_p.c_lflag, termios::TOSTOP),
        is_flag_set(termios_p.c_lflag, termios::IEXTEN)
    );

    Ok(())
}

pub fn stty(args: env::Args) -> io::Result<()> {
    let mut termios = match tcgetattr(io::stdin().as_raw_fd() as Fd) {
        Ok(termios) => termios,
        Err(e) => return Err(io::Error::from_raw_os_error(e)),
    };

    if args.len() == 0 {
        print_termios(&termios)?;
        return Ok(());
    }

    for arg in args {
        let on = !arg.starts_with('-');
        let arg_: &str = if on { &arg } else { &arg[1..] };

        match arg_ {
            "ignbrk" | "brkint" | "ignpar" | "parmrk" | "inpck" | "istrip" | "inlcr" | "igncr"
            | "icrnl" | "iuclc" | "ixon" | "ixany" | "ixoff" | "imaxbel" | "iutf8" => {
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
            "cs5" | "cs6" | "cs7" | "cs8" | "cstopb" | "cread" | "parenb" | "parodd" | "hupcl"
            | "clocal" => {
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
            "isig" | "icanon" | "echo" | "echoe" | "echok" | "echonl" | "noflsh" | "tostop"
            | "iexten" => {
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
                let (rows, columns) = get_size()?;
                println!("{rows} {columns}");
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
        &termios,
    ) {
        Err(io::Error::from_raw_os_error(e))
    } else {
        Ok(())
    }
}
