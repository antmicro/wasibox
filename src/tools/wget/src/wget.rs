/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env;
use std::fs;
use std::io;
use std::io::Write;

use clap::Parser;

#[derive(Parser)]
#[command(no_binary_name = true)]
struct CliArgs {
    #[arg(short('O'), long)]
    output_document: Option<String>,
    url: String,
}

pub fn wget(args: env::Args) -> io::Result<()> {
    let cli = CliArgs::parse_from(args);

    let minor = fs::OpenOptions::new()
        .write(true)
        .open("/dev/wget0")?
        .write(cli.url.as_bytes())?;
    let mut writer: Box<dyn io::Write> = if let Some(s) = cli.output_document {
        if s.as_str() == "-" {
            Box::new(io::BufWriter::new(io::stdout()))
        } else {
            Box::new(io::BufWriter::new(fs::File::create(s)?))
        }
    } else {
        Box::new(io::BufWriter::new(fs::File::create(
            cli.url.rsplit('/').next().unwrap(),
        )?))
    };
    let mut reader = io::BufReader::new(fs::File::open(format!("/dev/wget0r{}", minor))?);
    std::io::copy(&mut reader, &mut writer)?;

    Ok(())
}
