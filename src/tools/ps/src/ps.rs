/*
* Copyright (c) 2026 Antmicro <www.antmicro.com>
*
* SPDX-License-Identifier: Apache-2.0
*/

use std::{env::Args, fs, io};

const MAJ_HTERM: u32 = 1;

fn format_time(ms: u64) -> String {
    let total_seconds = ms / 1000;

    let seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) % 60;
    let hours = (total_seconds / 3600) % 24;
    let days = total_seconds / 3600 / 24;

    if days > 0 {
        format!("{:02}-{:02}:{:02}:{:02}", days, hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

pub fn ps(_args: Args) -> io::Result<()> {
    println!("    PID TTY          TIME CMD");

    fs::read_dir("/proc")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().is_dir()
                && entry
                    .file_name()
                    .to_str()
                    .map(|s| s.parse::<u32>().is_ok())
                    .unwrap_or(false)
        })
        .for_each(|proc| {
            let stat = match fs::read_to_string(proc.path().join("stat")) {
                Ok(stat) => stat,
                Err(_) => return,
            };

            let data: Vec<&str> = stat.split(" ").collect();

            let pid = data[0];
            let cmd = data[1].trim_matches(|c| c == '(' || c == ')');

            let tty = data[6].parse::<u32>().unwrap_or_default();

            let time = match data[13].parse::<u64>() {
                Ok(time) => format_time(time),
                Err(_) => String::from("--:--:--"),
            };

            let tty_major = (tty >> 8) & 0xFF;
            let tty_minor = ((tty >> 12) & 0xFFF00) | (tty & 0xFF);

            let tty_human = if tty_major == MAJ_HTERM {
                format!("ttyH{}", tty_minor)
            } else {
                String::from("?")
            };

            println!("{:>7} {:<6} {:>10} {}", pid, tty_human, time, cmd);
        });

    Ok(())
}
