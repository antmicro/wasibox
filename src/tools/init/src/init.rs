/*
 * Copyright (c) 2022-2024 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::mem;
use std::os::fd::AsRawFd;

use wasi_ext_lib::{ioctl, mknod, spawn, Redirect};

use serde::{Deserialize, Serialize};

mod services;

const FIFO_PATH: &str = "/dev/init.fifo";
const KERNEL_FIFO_PATH_READ: &str = "/dev/initr.kfifo";
const KERNEL_FIFO_PATH_WRITE: &str = "/dev/initw.kfifo";
const LOG_PATH: &str = "/tmp/init.log";

const TOKEN_KFIFO: u64 = 0;
const TOKEN_UFIFO: u64 = 1;

#[derive(Deserialize, Serialize)]
struct SpawnArgs {
    cmd: String,
    stdin: Option<String>,
    stdout: Option<String>,
    stderr: Option<String>,
    args: Vec<String>,
    kern: bool,
}

#[derive(Deserialize, Serialize)]
enum Operation {
    Start(String),
    Stop(String),
    Spawn(SpawnArgs),
}

struct Init {
    pub(crate) service_manager: services::ServiceManager,

    ufifo: Option<fs::File>,  // userspace fifo
    kfifor: Option<fs::File>, // kernel read fifo
    kfifow: Option<fs::File>, // kernel write fifo
    logfile: Option<fs::File>,
}

impl Init {
    fn new() -> Self {
        Self {
            service_manager: services::ServiceManager::new(),
            ufifo: None,
            kfifor: None,
            kfifow: None,
            logfile: None,
        }
    }

    fn setup_descriptors(&mut self) -> io::Result<()> {
        let mut one = 1;

        mknod(FIFO_PATH, -1).map_err(io::Error::from_raw_os_error)?;
        mknod(KERNEL_FIFO_PATH_READ, -1).map_err(io::Error::from_raw_os_error)?;
        mknod(KERNEL_FIFO_PATH_WRITE, -1).map_err(io::Error::from_raw_os_error)?;

        self.ufifo = Some(fs::OpenOptions::new().read(true).open(FIFO_PATH)?);
        self.kfifor = Some(
            fs::OpenOptions::new()
                .read(true)
                .open(KERNEL_FIFO_PATH_READ)?,
        );
        ioctl(
            self.kfifor.as_mut().unwrap().as_raw_fd(),
            wasi_ext_lib::FIFOSKERNW,
            Some(&mut one),
        )
        .map_err(io::Error::from_raw_os_error)?;

        self.kfifow = Some(
            fs::OpenOptions::new()
                .write(true)
                .open(KERNEL_FIFO_PATH_WRITE)?,
        );
        ioctl(
            self.kfifow.as_mut().unwrap().as_raw_fd(),
            wasi_ext_lib::FIFOSKERNR,
            Some(&mut one),
        )
        .map_err(io::Error::from_raw_os_error)?;
        self.logfile = Some(
            fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(LOG_PATH)?,
        );

        Ok(())
    }

    fn handle_operation(&mut self, operation: &Operation, iteration: i32) -> io::Result<()> {
        match operation {
            Operation::Start(name) => {
                if let Some(service) = self.service_manager.services.get_mut(name) {
                    if service.pid < 0 {
                        service.spawn()
                    } else {
                        Ok(()) // TODO: this should be an error
                    }
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Service {} not found", &name),
                    ))
                }
            }
            Operation::Stop(name) => {
                if let Some(service) = self.service_manager.services.get(name) {
                    if service.pid > 0 {
                        wasi_ext_lib::kill(service.pid, wasi::SIGNAL_KILL)
                            .map_err(io::Error::from_raw_os_error)
                    } else {
                        Ok(()) // TODO: this should be an error
                    }
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Service {} not found", &name),
                    ))
                }
            }
            Operation::Spawn(spawn_args) => {
                let paths = [
                    (
                        &spawn_args.stdin,
                        format!("{}.{}", "/dev/spawn_stdin", iteration),
                    ),
                    (
                        &spawn_args.stdout,
                        format!("{}.{}", "/dev/spawn_stdout", iteration),
                    ),
                    (
                        &spawn_args.stderr,
                        format!("{}.{}", "/dev/spawn_stderr", iteration),
                    ),
                ];

                let redirect_paths = paths
                    .iter()
                    .enumerate()
                    .map(|(i, (override_, path))| -> io::Result<&str> {
                        if override_.is_some() {
                            Ok(override_.as_ref().unwrap())
                        } else {
                            mknod(path, -1).map_err(io::Error::from_raw_os_error)?;
                            let mut one = 1;
                            let dev = fs::OpenOptions::new().open(path)?;
                            ioctl(dev.as_raw_fd(), wasi_ext_lib::FIFOSCLOSERM, Some(&mut one))
                                .map_err(io::Error::from_raw_os_error)?;
                            if spawn_args.kern {
                                ioctl(
                                    dev.as_raw_fd(),
                                    if i == 0 {
                                        wasi_ext_lib::FIFOSKERNW
                                    } else {
                                        wasi_ext_lib::FIFOSKERNR
                                    },
                                    Some(&mut one),
                                )
                                .map_err(io::Error::from_raw_os_error)?;
                            }
                            Ok(path)
                        }
                    })
                    .collect::<io::Result<Vec<&str>>>()?;

                spawn(
                    &spawn_args.cmd,
                    &spawn_args
                        .args
                        .iter()
                        .map(|arg| arg.as_str())
                        .collect::<Vec<&str>>(),
                    &HashMap::new(),
                    true,
                    &[
                        Redirect::Read(0, redirect_paths[0].to_string()),
                        Redirect::Append(1, redirect_paths[1].to_string()),
                        Redirect::Append(2, redirect_paths[2].to_string()),
                    ],
                )
                .map_err(io::Error::from_raw_os_error)?;

                Ok(())
            }
        }
    }

    fn main_loop(&mut self) -> io::Result<()> {
        let mut buf = [0u8; 8192];
        let subs = [
            wasi::Subscription {
                userdata: TOKEN_UFIFO,
                u: wasi::SubscriptionU {
                    tag: wasi::EVENTTYPE_FD_READ.raw(),
                    u: wasi::SubscriptionUU {
                        fd_read: wasi::SubscriptionFdReadwrite {
                            file_descriptor: self.ufifo.as_ref().unwrap().as_raw_fd() as u32,
                        },
                    },
                },
            },
            wasi::Subscription {
                userdata: TOKEN_KFIFO,
                u: wasi::SubscriptionU {
                    tag: wasi::EVENTTYPE_FD_READ.raw(),
                    u: wasi::SubscriptionUU {
                        fd_read: wasi::SubscriptionFdReadwrite {
                            file_descriptor: self.kfifor.as_ref().unwrap().as_raw_fd() as u32,
                        },
                    },
                },
            },
        ];
        let mut events: [wasi::Event; 2] = unsafe { mem::zeroed() };

        let mut iteration = 0;
        loop {
            let count =
                unsafe { wasi::poll_oneoff(subs.as_ptr(), events.as_mut_ptr(), subs.len()) }
                    .map_err(|e| io::Error::from_raw_os_error(e.raw() as i32))?;

            if count == 0 {
                continue;
            }

            for event in events[0..count].iter() {
                let size = if event.userdata == TOKEN_KFIFO {
                    self.kfifor.as_mut().unwrap()
                } else {
                    self.ufifo.as_mut().unwrap()
                }
                .read(&mut buf)?;

                let operation = match serde_json::from_slice(&buf[..size]) {
                    Ok(v) => v,
                    Err(e) => {
                        _ = self
                            .logfile
                            .as_mut()
                            .unwrap()
                            .write(format!("{:?}\n", e).as_bytes())?;
                        continue;
                    }
                };

                let _ = self.handle_operation(&operation, iteration);

                if event.userdata == TOKEN_KFIFO {
                    let _ = self
                        .kfifow
                        .as_mut()
                        .unwrap()
                        .write(iteration.to_string().as_bytes());
                }

                iteration += 1;
            }
        }
    }
}

pub fn init(_args: env::Args) -> io::Result<()> {
    let mut init = Init::new();
    init.setup_descriptors()?;
    init.service_manager.load_services()?;
    init.service_manager.spawn_services()?;
    init.main_loop()?;
    Ok(())
}
