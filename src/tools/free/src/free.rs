/*
* Copyright (c) 2026 Antmicro <www.antmicro.com>
*
* SPDX-License-Identifier: Apache-2.0
*/

use clap::ArgGroup;
use clap::Parser;
use std::collections::HashMap;
use std::env::Args;
use std::fs;
use std::io;

const MEMINFO_PATH: &str = "/proc/meminfo";

#[derive(Parser)]
#[command(no_binary_name = true, disable_help_flag = true)]
#[command(group(
    ArgGroup::new("exponent")
        .args(&["bytes", "kilo", "mega", "giga", "tera", "peta", "kibi", "mebi", "gibi", "tebi", "pebi", "human"])
))]
struct CliArgs {
    /// show output in bytes
    #[arg(short, long)]
    bytes: bool,

    /// show output in kilobytes
    #[arg(long)]
    kilo: bool,

    /// show output in megabytes
    #[arg(long)]
    mega: bool,

    /// show output in gigabytes
    #[arg(long)]
    giga: bool,

    /// show output in terabytes
    #[arg(long)]
    tera: bool,

    /// show output in petabytes
    #[arg(long)]
    peta: bool,

    /// show output in kibibytes
    #[arg(short, long)]
    kibi: bool,

    /// show output in mebibytes
    #[arg(short, long)]
    mebi: bool,

    /// show output in gibibytes
    #[arg(short, long)]
    gibi: bool,

    /// show output in tebibytes
    #[arg(long)]
    tebi: bool,

    /// show output in pebibytes
    #[arg(long)]
    pebi: bool,

    /// show human-readable output
    #[arg(short, long)]
    human: bool,

    /// use powers of 1000 not 1024
    #[arg(long, requires = "human")]
    si: bool,

    /// display this help and exit
    #[arg(long, action = clap::ArgAction::Help)]
    help: Option<bool>,
}

fn format_to_exponent(val: u64, exponent: u32, si: bool) -> String {
    let base: u64 = if si { 10 } else { 2 };
    format!("{:11}", val / base.pow(exponent))
}

fn format_human(bytes: u64, si: bool) -> String {
    let units = ["B", "K", "M", "G", "T", "P"];
    let base = if si { 1000.0 } else { 1024.0 };
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= base && unit_idx < units.len() - 1 {
        size /= base;
        unit_idx += 1;
    }

    let mut val = if size >= 100.0 {
        format!("{:.0}{}", size, units[unit_idx])
    } else {
        format!("{:.1}{}", size, units[unit_idx])
    };

    if !si {
        val.push('i');
    }

    format!("{:>11}", val)
}

pub fn free(args: Args) -> io::Result<()> {
    let cli = CliArgs::parse_from(args);

    let contents = match fs::read_to_string(MEMINFO_PATH) {
        Ok(contents) => contents,
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "free works only with Chromium-based browsers",
                ));
            } else {
                return Err(err);
            }
        }
    };

    let mut mem_data: HashMap<String, u64> = HashMap::new();

    for line in contents.lines() {
        let mut parts = line.splitn(2, ':');

        if let (Some(key), Some(val)) = (parts.next(), parts.next()) {
            let numeric_val = val
                .split_whitespace()
                .next()
                .unwrap()
                .parse::<u64>()
                .unwrap();

            // meminfo returns info in KiB, cast it back to bytes
            let numeric_val = numeric_val * 1024;
            mem_data.insert(key.to_owned(), numeric_val);
        }
    }

    let mut exponent = 10;
    let mut si = false;

    if cli.bytes {
        exponent = 0;
    } else if cli.kilo {
        exponent = 3;
        si = true;
    } else if cli.mega {
        exponent = 6;
        si = true;
    } else if cli.giga {
        exponent = 9;
        si = true;
    } else if cli.tera {
        exponent = 12;
        si = true;
    } else if cli.peta {
        exponent = 15;
        si = true;
    } else if cli.kibi {
        exponent = 10;
    } else if cli.mebi {
        exponent = 20;
    } else if cli.gibi {
        exponent = 30;
    } else if cli.tebi {
        exponent = 40;
    } else if cli.pebi {
        exponent = 50;
    }

    if cli.si {
        si = true;
    }

    let mem_total = mem_data.get("MemTotal").unwrap();
    let mem_free = mem_data.get("MemFree").unwrap();
    let mem_available = mem_data.get("MemAvailable").unwrap();

    let mem_used = *mem_total - *mem_free;

    let (total, used, free, available) = if cli.human {
        (
            format_human(*mem_total, si),
            format_human(mem_used, si),
            format_human(*mem_free, si),
            format_human(*mem_available, si),
        )
    } else {
        (
            format_to_exponent(*mem_total, exponent, si),
            format_to_exponent(mem_used, exponent, si),
            format_to_exponent(*mem_free, exponent, si),
            format_to_exponent(*mem_available, exponent, si),
        )
    };

    println!("               total        used        free   available");
    println!("Mem:     {} {} {} {}", total, used, free, available);

    Ok(())
}
