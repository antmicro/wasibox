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

use wasi_ext_lib::{mknod, spawn, Redirect};

use serde::{Deserialize, Serialize};

mod services;

const FIFO_PATH: &str = "/dev/init.fifo";
const LOG_PATH: &str = "/tmp/init.log";

#[derive(Deserialize, Serialize)]
struct SpawnArgs {
    cmd: String,
    args: Vec<String>,
}

#[derive(Deserialize, Serialize)]
enum Operation {
    Start(String),
    Stop(String),
    Spawn(SpawnArgs),
}

struct Init {
    pub(crate) service_manager: services::ServiceManager,
}

impl Init {
    fn new() -> Self {
        Self {
            service_manager: services::ServiceManager::new(),
        }
    }

    fn main_loop(&mut self) -> io::Result<()> {
        let mut buf = [0u8; 8192];

        mknod(FIFO_PATH, -1).map_err(io::Error::from_raw_os_error)?;

        let mut sock = fs::OpenOptions::new().read(true).open(FIFO_PATH)?;
        let mut log = fs::OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(LOG_PATH)?;

        loop {
            let size = sock.read(&mut buf)?;
            let operation = match serde_json::from_slice(&buf[..size]) {
                Ok(v) => v,
                Err(e) => {
                    _ = log.write(format!("{:?}\n", e).as_bytes())?;
                    continue;
                }
            };

            match operation {
                Operation::Start(name) => {
                    let service =
                        if let Some(service) = self.service_manager.services.get_mut(&name) {
                            service
                        } else {
                            continue;
                        };

                    if service.pid < 0 {
                        service.spawn().unwrap();
                    }
                }
                Operation::Stop(name) => {
                    let service = if let Some(service) = self.service_manager.services.get(&name) {
                        service
                    } else {
                        continue;
                    };

                    if service.pid > 0 {
                        wasi_ext_lib::kill(service.pid, wasi::SIGNAL_KILL)
                            .map_err(io::Error::from_raw_os_error)?;
                    }
                }
                Operation::Spawn(spawn_args) => {
                    let stdin_path = "/dev/spawn_stdin";
                    let stdout_path = "/dev/spawn_stdout";
                    let stderr_path = "/dev/spawn_stderr";

                    mknod(stdin_path, -1).map_err(io::Error::from_raw_os_error)?;
                    mknod(stdout_path, -1).map_err(io::Error::from_raw_os_error)?;
                    mknod(stderr_path, -1).map_err(io::Error::from_raw_os_error)?;

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
                            Redirect::Read(0, String::from(stdin_path)),
                            Redirect::Append(1, String::from(stdout_path)),
                            Redirect::Append(2, String::from(stderr_path)),
                        ],
                    )
                    .map_err(io::Error::from_raw_os_error)?;
                }
            }
        }
    }
}

pub fn init(_args: env::Args) -> io::Result<()> {
    let mut init = Init::new();
    init.service_manager.load_services()?;
    init.service_manager.spawn_services()?;
    init.main_loop()?;
    Ok(())
}
