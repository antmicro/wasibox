/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::constants;

use std::io;
use std::path::Path;
use std::ffi::OsStr;

use clap::ArgMatches;

#[derive(Copy, Clone, Debug)]
pub enum Compression {
    Gzip,
    Bzip,
    None
}

impl Compression {
    pub fn from_cli(m: &ArgMatches) -> Compression {
        // TODO: is there a way to do it without if-else chain
        if m.is_present(constants::BZ2_MATCH) {
            Compression::Bzip
        } else if m.is_present(constants::GZ_MATCH) {
            Compression::Gzip
        } else {
            Compression::None
        }
    }

    pub fn from_extensions(path: &Path) -> io::Result<Compression> {
        match path.extension().and_then(OsStr::to_str) {
            Some("bz2") => Ok(Compression::Bzip),
            Some("gz") => Ok(Compression::Gzip),
            Some("tar") | None => Ok(Compression::None),
            Some(_) => Err(io::Error::new(io::ErrorKind::InvalidInput, "Unknown compression"))
        }
    }
}
