/*
 * Copyright (c) 2022-2025 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env::Args;
use std::io;

use clap::ArgGroup;
use clap::Parser;

#[derive(Parser)]
#[command(no_binary_name = true)]
#[command(group(
    ArgGroup::new("name_type")
        .args(&["href", "protocol", "host", "port", "pathname", "search", "hash", "origin", "user_agent"])
        .multiple(false)
))]
struct CliArgs {
    #[arg(short = 'H', long)]
    href: bool,

    #[arg(short = 'p', long)]
    protocol: bool,

    #[arg(short = 't', long)]
    host: bool,

    #[arg(short = 'P', long)]
    port: bool,

    #[arg(short = 'n', long)]
    pathname: bool,

    #[arg(short = 's', long)]
    search: bool,

    #[arg(short = 'S', long)]
    hash: bool,

    #[arg(short = 'o', long)]
    origin: bool,

    #[arg(short = 'u', long)]
    user_agent: bool,
}

pub fn uname(args: Args) -> io::Result<()> {
    let cli = CliArgs::parse_from(args);

    let name_type = if cli.href {
        wasi_ext_lib::NameType::Href
    } else if cli.protocol {
        wasi_ext_lib::NameType::Protocol
    } else if cli.host {
        wasi_ext_lib::NameType::Host
    } else if cli.port {
        wasi_ext_lib::NameType::Port
    } else if cli.pathname {
        wasi_ext_lib::NameType::Pathname
    } else if cli.search {
        wasi_ext_lib::NameType::Search
    } else if cli.hash {
        wasi_ext_lib::NameType::Hash
    } else if cli.origin {
        wasi_ext_lib::NameType::Origin
    } else if cli.user_agent {
        wasi_ext_lib::NameType::UserAgent
    } else {
        wasi_ext_lib::NameType::Href
    };

    match wasi_ext_lib::uname(name_type) {
        Ok(info) => {
            println!("{}", info);
            Ok(())
        }
        Err(code) => Err(io::Error::from_raw_os_error(code)),
    }
}
