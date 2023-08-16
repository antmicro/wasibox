/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::fs::File;
use std::io;
use std::io::BufRead;
use std::env;

use clap::Parser;

#[derive(Parser)]
#[clap(no_binary_name(true))]
struct CliArgs {
    #[arg(short, long)]
    types: Option<String>,
    #[arg(short, long)]
    options: Option<String>,
    source: String,
    target: Option<String>
}

pub fn mount(args: env::Args) -> io::Result<()> {
    if args.len() == 0 {
        io::BufReader::new(File::open("/proc/self/mountinfo")?)
            .lines()
            .for_each(|line| {
                println!("{}", line.unwrap());
            });
        return Ok(());
    }

    let args = CliArgs::parse_from(args);

    let source: String;
    let target: String;

    if let Some(t) = args.target {
        source = args.source;
        target = t;
    } else {
        target = args.source;
        source = String::from("");
    }

    wasi_ext_lib::mount(
        &source,
        &target,
        &args.types.unwrap_or("".to_string()),
        0u64,
        &args.options.unwrap_or("".to_string()),
    ).map_err(io::Error::from_raw_os_error)
}
