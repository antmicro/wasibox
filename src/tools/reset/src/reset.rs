/*
* Copyright (c) 2026 Antmicro <www.antmicro.com>
*
* SPDX-License-Identifier: Apache-2.0
*/

use std::{
    env::Args,
    fs::OpenOptions,
    io::{self, Write},
};

const RESET_PATH: &str = "/proc/sys/reset";

pub fn reset(_args: Args) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).open(RESET_PATH)?;

    file.write_all(b"1")?;
    Ok(())
}
