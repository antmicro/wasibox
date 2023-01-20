/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::io;
use std::fs;
use std::env::Args;

pub fn hexdump(mut args: Args) -> io::Result<()> {
    let contents = fs::read(match args.next() {
        Some(f) => f,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "hexdump: help: hexdump <filename>"));
        }
    })?;
    let len = contents.len();
    let mut v = ['.'; 16];
    for j in 0..len {
        let c = contents[j] as char;
        v[j % 16] = c;
        if (j % 16) == 0 {
            print!("{:08x} ", j);
        }
        if (j % 8) == 0 {
            print!(" ");
        }
        print!("{:02x} ", c as u8);
        if (j + 1) == len || (j % 16) == 15 {
            let mut count = 16;
            if (j + 1) == len {
                count = len % 16;
                for _ in 0..(16 - (len % 16)) {
                    print!("   ");
                }
                if count < 8 {
                    print!(" ");
                }
            }
            print!(" |");
            for c in v.iter_mut().take(count) {
                if (0x20..0x7e).contains(&(*c as u8)) {
                    print!("{}", *c as char);
                    *c = '.';
                } else {
                    print!(".");
                }
            }
            println!("|");
        }
    }
    Ok(())
}
