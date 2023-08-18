/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::CliCompression;

use std::ffi::OsStr;
use std::io;
use std::path::Path;

#[derive(Copy, Clone, Debug)]
pub enum Compression {
    Gzip,
    Bzip,
    None,
}

impl TryFrom<&CliCompression> for Compression {
    type Error = ();

    fn try_from(comp: &CliCompression) -> Result<Compression, Self::Error> {
        // TODO: is there a way to do it without if-else chain
        if comp.gzip && comp.bzip2 {
            Err(())
        } else if comp.gzip {
            Ok(Compression::Gzip)
        } else if comp.bzip2 {
            Ok(Compression::Bzip)
        } else {
            Ok(Compression::None)
        }
    }
}
impl Compression {
    pub fn from_extensions(path: &Path) -> io::Result<Compression> {
        match path.extension().and_then(OsStr::to_str) {
            Some("bz2") => Ok(Compression::Bzip),
            Some("gz") => Ok(Compression::Gzip),
            Some("tar") | None => Ok(Compression::None),
            Some(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unknown compression",
            )),
        }
    }
}
