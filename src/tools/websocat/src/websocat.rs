/*
 * Copyright (c) 2022-2024 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::mem;
use std::os::fd::AsRawFd;

use clap::Parser;

#[derive(Parser)]
#[command(no_binary_name = true)]
struct CliArgs {
    url: String,
}

const TOKEN_STDIN: u64 = 0;
const TOKEN_SOCKET: u64 = 1;

pub fn websocat(args: env::Args) -> io::Result<()> {
    let cli = CliArgs::parse_from(args);
    let mut buf = [0u8; 8192];

    let minor = fs::OpenOptions::new()
        .write(true)
        .open("/dev/ws0")?
        .write(cli.url.as_bytes())?;

    let mut connection_device = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(format!("/dev/ws0s{}", minor))?;

    let subs = [
        wasi::Subscription {
            userdata: TOKEN_SOCKET,
            u: wasi::SubscriptionU {
                tag: wasi::EVENTTYPE_FD_READ.raw(),
                u: wasi::SubscriptionUU {
                    fd_read: wasi::SubscriptionFdReadwrite {
                        file_descriptor: connection_device.as_raw_fd() as u32,
                    },
                },
            },
        },
        wasi::Subscription {
            userdata: TOKEN_STDIN,
            u: wasi::SubscriptionU {
                tag: wasi::EVENTTYPE_FD_READ.raw(),
                u: wasi::SubscriptionUU {
                    fd_read: wasi::SubscriptionFdReadwrite {
                        file_descriptor: io::stdin().as_raw_fd() as u32,
                    },
                },
            },
        },
    ];

    let mut events: [wasi::Event; 2] = unsafe { mem::zeroed() };

    loop {
        let count = unsafe {
            match wasi::poll_oneoff(subs.as_ptr(), events.as_mut_ptr(), subs.len()) {
                Ok(n) => n,
                Err(e) => {
                    return Err(io::Error::from_raw_os_error(e.raw() as i32));
                }
            }
        };

        if count == 0 {
            continue;
        }

        for event in events[0..count].iter() {
            match event.userdata {
                TOKEN_SOCKET => {
                    if event.type_ == wasi::EVENTTYPE_FD_READ {
                        let len = connection_device.read(&mut buf)?;
                        _ = io::stdout().write(&buf[..len])?;
                    } else {
                        eprintln!("Connection closed by the server");
                        return Ok(());
                    }
                }
                _ => {
                    let len = io::stdin().read(&mut buf)?;
                    _ = connection_device.write(&buf[..len])?;
                }
            }
        }
        events = unsafe { mem::zeroed() };
    }
}
