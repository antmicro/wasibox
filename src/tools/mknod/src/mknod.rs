/*
 * Copyright (c) 2022-2024 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env;
use std::io;

use clap::Parser;

#[derive(Parser)]
#[clap(no_binary_name(true))]
struct CliArgs {
    path: String,
    maj: i32,
    min: Option<i32>,
}

pub fn mknod(args: env::Args) -> io::Result<()> {
    let args = CliArgs::parse_from(args);

    if args.maj < 0 {
        wasi_ext_lib::mknod(&args.path, args.maj).map_err(io::Error::from_raw_os_error)?;
    } else {
        if args.min.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Missing minor number",
            ));
        }
        wasi_ext_lib::mknod(&args.path, wasi_ext_lib::mkdev(args.maj, args.min.unwrap()))
            .map_err(io::Error::from_raw_os_error)?;
    }

    Ok(())
}
