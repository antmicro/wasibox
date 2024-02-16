/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::os::fd::AsRawFd;

use clap::Parser;

#[derive(Parser)]
#[command(no_binary_name = true)]
struct CliArgs {
    #[arg(short('O'), long)]
    output_document: Option<String>,
    #[arg(short('S'), long)]
    server_response: bool,
    url: String,
}

pub fn wget(args: env::Args) -> io::Result<()> {
    let cli = CliArgs::parse_from(args);

    let minor = fs::OpenOptions::new()
        .write(true)
        .open("/dev/wget0")?
        .write(cli.url.as_bytes())?;
    let response_device = fs::File::open(format!("/dev/wget0r{}", minor))?;

    if cli.server_response {
        let mut http_stat: i32 = 0;
        if let Err(e) = wasi_ext_lib::ioctl(
            response_device.as_raw_fd(),
            wasi_ext_lib::WGETGS,
            Some(&mut http_stat),
        ) {
            eprintln!("Could not retreive http status: system error {}", e);
        }
        if let Err(e) =
            wasi_ext_lib::ioctl::<()>(response_device.as_raw_fd(), wasi_ext_lib::WGETRH, None)
        {
            eprintln!("Could not read http headers: system error {}", e);
        } else {
            println!("HTTP {}", http_stat);
            std::io::copy(
                &mut io::BufReader::new(&response_device),
                &mut io::BufWriter::new(std::io::stdout()),
            )?;
            println!("\n");
        }
    }

    if let Err(e) =
        wasi_ext_lib::ioctl::<()>(response_device.as_raw_fd(), wasi_ext_lib::WGETRB, None)
    {
        eprintln!("Could not read http body: system error {}", e);
    } else {
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
        let mut reader = io::BufReader::new(response_device);
        std::io::copy(&mut reader, &mut writer)?;
    }

    Ok(())
}
