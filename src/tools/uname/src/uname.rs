/*
 * Copyright (c) 2022-2025 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env::Args;
use std::io;

use clap::Parser;

#[derive(Parser)]
#[command(no_binary_name = true)]
struct CliArgs {}

pub fn uname(_args: Args) -> io::Result<()> {
    match wasi_ext_lib::uname() {
        Ok(info) => {
            println!("{}", info);
            Ok(())
        }
        Err(code) => Err(io::Error::from_raw_os_error(code)),
    }
}
