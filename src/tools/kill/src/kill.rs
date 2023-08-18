/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env;
use std::io;

use clap::Parser;

#[derive(Parser)]
#[command(no_binary_name = true)]
struct CliArgs {
    pid: i32,
}

pub fn kill(args: env::Args) -> io::Result<()> {
    let cli = CliArgs::parse_from(args);

    if let Err(e) = wasi_ext_lib::kill(cli.pid, wasi_ext_lib::SIGNAL_KILL) {
        return Err(io::Error::from_raw_os_error(e));
    }

    Ok(())
}
