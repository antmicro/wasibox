/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env;
use std::io;
use std::process;

pub use wasibox::tools_map::AppletType;
use wasibox::tools_map::TOOLS_MAP;

fn get_applet(args: &mut env::Args) -> Result<&AppletType, &'static str> {
    let command = if let Some(cmd) = args.next() {
        if cmd == env!("CARGO_PKG_NAME") || cmd == format!("{}.wasm", env!("CARGO_PKG_NAME")) {
            if let Some(cmd_) = args.next() {
                cmd_
            } else {
                return Err("Missing tool name");
            }
        } else {
            cmd
        }
    } else {
        return Err("Missing command line arguments");
    };

    if let Some(applet) = TOOLS_MAP.get(&command[..]) {
        Ok(applet)
    } else {
        Err("No such tool")
    }
}

fn main() -> io::Result<()> {
    let mut args = env::args();

    match get_applet(&mut args) {
        Ok(applet) => applet(args),
        Err(e) => {
            eprintln!("{}\n", e);
            eprintln!("USAGE: {} [tool [arguments]]", env!("CARGO_PKG_NAME"));
            eprintln!("   or: tool [arguments]\n");
            eprintln!("Available tools are: {:?}", TOOLS_MAP.keys());
            process::exit(1);
        }
    }
}
