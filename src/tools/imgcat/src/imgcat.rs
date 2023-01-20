/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::io;
use std::env::Args;

pub fn imgcat(mut args: Args) -> io::Result<()> {
    if let Some(arg) = args.next() {
        // TODO: find out why it breaks the order of prompt
        iterm2::File::read(arg)?
            .width(iterm2::Dimension::Auto)
            .height(iterm2::Dimension::Auto)
            .preserve_aspect_ratio(true)
            .show()
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "usage: imgcat <IMAGE>"))
    }
}
