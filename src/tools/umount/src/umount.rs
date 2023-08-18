/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env;
use std::fs::File;
use std::io::{self, BufRead};

use clap::Parser;

#[derive(Parser)]
#[clap(no_binary_name = true)]
struct CliArgs {
    #[arg(short, long)]
    all: bool,
    #[clap(required_unless_present("all"))]
    mount_point: Option<String>,
}

pub fn umount(args: env::Args) -> io::Result<()> {
    let cli = CliArgs::parse_from(args);
    if cli.all {
        io::BufReader::new(File::open("/proc/self/mountinfo")?)
            .lines()
            .for_each(|line| {
                let line = if let Ok(ln) = line {
                    ln
                } else {
                    return;
                };

                let mount_point = line.split(' ').next().unwrap();
                if mount_point == "/" {
                    return;
                }

                if let Err(e) = wasi_ext_lib::umount(mount_point) {
                    eprintln!("Could not umount {} - system error {}", mount_point, e);
                }
            });
        Ok(())
    } else if let Some(point) = cli.mount_point {
        wasi_ext_lib::umount(&point).map_err(io::Error::from_raw_os_error)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No mount point specified",
        ))
    }
}
