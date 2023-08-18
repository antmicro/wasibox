/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

mod compression;

use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;

use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use clap::{Args, Parser};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use tar::{Archive, Builder};

use compression::Compression;

fn extract_stream<R: io::Read + 'static>(
    stream: R,
    comp: Compression,
) -> io::Result<Box<dyn io::Read>> {
    match comp {
        Compression::Gzip => {
            let decoder = GzDecoder::new(stream);
            if decoder.header().is_none() {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "not in gzip format",
                ))
            } else {
                Ok(Box::new(decoder))
            }
        }
        Compression::Bzip => {
            let mut decoder = BzDecoder::new(stream);
            // bzip2 lib doesn't implement reading headers so we read 0 bytes to
            // make the decoder check if it's header is coorect
            if decoder.read(&mut Vec::new()).is_err() {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "not in bzip2 format",
                ))
            } else {
                Ok(Box::new(decoder))
            }
        }
        Compression::None => Ok(Box::new(stream)),
    }
}

fn encode_stream<R: io::Write + 'static>(stream: R, comp: Compression) -> Box<dyn io::Write> {
    match comp {
        Compression::Gzip => Box::new(GzEncoder::new(stream, flate2::Compression::best())),
        Compression::Bzip => Box::new(BzEncoder::new(stream, bzip2::Compression::best())),
        Compression::None => Box::new(stream),
    }
}

#[derive(Parser)]
#[command(no_binary_name = true)]
struct CliArgs {
    #[command(flatten)]
    compression: CliCompression,
    #[command(flatten)]
    method: CliMethod,
    #[arg(short, long)]
    file: Option<String>,
    #[arg(short, long)]
    verbose: bool,
    #[arg(short, long)]
    files: Vec<String>,
}

#[derive(Args)]
#[group(multiple = false)]
struct CliCompression {
    #[arg(short, long)]
    gzip: bool,
    #[arg(short, long)]
    bzip2: bool,
}

#[derive(Args)]
#[group(multiple = false)]
struct CliMethod {
    #[arg(short, long)]
    create: bool,
    #[arg(short = 'x', long)]
    extract: bool,
}

pub fn tar(args: env::Args) -> io::Result<()> {
    let cli = CliArgs::parse_from(args);
    let compression = Compression::try_from(&cli.compression).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "Conflicting compression flags")
    })?;
    if cli.method.extract {
        let (input_stream, compression): (Box<dyn io::Read>, Compression) =
            if let Some(path) = &cli.file {
                (
                    Box::new(fs::File::open(path)?),
                    if let Compression::None = compression {
                        Compression::from_extensions(Path::new(&path))?
                    } else {
                        compression
                    },
                )
            } else {
                (Box::new(io::stdin()), compression)
            };
        // TODO: if stream extraction failed, the program should return 2
        // using ? operator makes the program exit with 1
        untar(&mut extract_stream(input_stream, compression)?, &cli)?;
    } else if cli.method.create {
        let (output_stream, compression): (Box<dyn io::Write>, Compression) =
            if let Some(path) = &cli.file {
                (
                    Box::new(fs::File::create(path)?),
                    if let Compression::None = compression {
                        Compression::from_extensions(Path::new(&path))?
                    } else {
                        compression
                    },
                )
            } else {
                #[cfg(target_os = "wasi")]
                if let Ok(is_tty) = wasi_ext_lib::isatty(0) {
                    if is_tty {
                        (Box::new(io::stdout()), compression)
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Refusing to write to stdout",
                        ));
                    }
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Refusing to write to stdout",
                    ));
                }
                #[cfg(not(target_os = "wasi"))]
                if atty::is(atty::Stream::Stdout) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Refusing to write to stdout",
                    ));
                } else {
                    (Box::new(io::stdout()), compression)
                }
            };
        create_tar(&mut encode_stream(output_stream, compression), &cli)?;
    }
    Ok(())
}

fn untar<R: ?core::marker::Sized + io::Read>(stream: &mut R, context: &CliArgs) -> io::Result<()> {
    let mut archive = Archive::new(stream);
    for entry in archive.entries()? {
        let mut entry = entry?;
        if context.verbose {
            println!("{}", entry.path().unwrap().display());
        }
        entry.unpack_in(".")?;
    }
    Ok(())
}

fn walk_dir<F: FnMut(&Path) -> io::Result<()>>(path: &Path, action: &mut F) -> io::Result<()> {
    action(path)?;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            walk_dir(&entry?.path(), action)?;
        }
    }
    Ok(())
}

fn create_tar<R: ?core::marker::Sized + io::Write>(
    stream: &mut R,
    context: &CliArgs,
) -> io::Result<()> {
    let mut builder = Builder::new(stream);
    for f in &context.files {
        let path = Path::new(f);
        walk_dir(path, &mut |p: &Path| -> io::Result<()> {
            if context.verbose {
                println!("{}", p.display());
            }
            builder.append_path(p)
        })?;
    }
    builder.finish()?;
    Ok(())
}
