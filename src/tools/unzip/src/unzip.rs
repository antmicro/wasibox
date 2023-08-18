/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env::Args;
use std::fs;
use std::io;
use std::path::PathBuf;

use clap::Parser;

use zip::ZipArchive;

#[derive(Parser)]
#[command(no_binary_name = true)]
struct CliArgs {
    #[arg(short, long)]
    quiet: bool,
    file: String,
}

pub fn unzip(args: Args) -> io::Result<()> {
    let cli = CliArgs::parse_from(args);
    let filepath: PathBuf = PathBuf::from(cli.file);
    // TODO: for some reason it is always true, it may be a bug in clap, maybe we should downgrade
    if !filepath.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Cannot find or open {}", filepath.display()),
        ));
    }
    if let Ok(archive) = &mut ZipArchive::new(fs::File::open(filepath)?) {
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let output_path = file.enclosed_name().to_owned().unwrap();
            if file.name().ends_with('/') {
                if !cli.quiet {
                    println!("creating dir {}", output_path.display());
                }
                fs::create_dir_all(output_path).unwrap();
                continue;
            }
            if let Some(parent) = output_path.parent() {
                if !parent.exists() {
                    if !cli.quiet {
                        println!("creating dir {}", parent.display());
                    }
                    fs::create_dir_all(parent)?;
                }
            }
            if !cli.quiet {
                println!("decompressing {}", file.enclosed_name().unwrap().display());
            }
            let mut output_file = fs::File::create(output_path).unwrap();
            io::copy(&mut file, &mut output_file).unwrap();
            if !cli.quiet {
                println!(
                    "decompressing {} done.",
                    file.enclosed_name().unwrap().display()
                );
            }
        }
    }
    Ok(())
}
