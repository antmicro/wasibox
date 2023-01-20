/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::io;
use std::fs;
use std::env::Args;
use std::path::PathBuf;

use clap::{Arg, ArgAction, command};

use zip::ZipArchive;

pub fn unzip(args: Args) -> io::Result<()> {
    let matches = command!()
        .no_binary_name(true)
        .arg(
            Arg::new("quiet")
                .help("quiet mode")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)

        )
        .arg(
            Arg::new("FILE")
                .help("Archive to decompress")
                .required(true)
                .index(1)
        ).get_matches_from(args);
    let filepath: PathBuf = PathBuf::from(matches.get_one::<String>("FILE").unwrap());
    // TODO: for some reason it is always true, it may be a bug in clap, maybe we should downgrade
    let quiet = matches.get_flag("quiet");
    if !filepath.is_file() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Cannot find or open {}", filepath.display())));
    }
    if let Ok(archive) = &mut ZipArchive::new(fs::File::open(filepath)?) {
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let output_path = file.enclosed_name().to_owned().unwrap();
            if file.name().ends_with('/') {
                if !quiet { println!("creating dir {}", output_path.display()); }
                fs::create_dir_all(&output_path).unwrap();
                continue;
            }
            if let Some(parent) = output_path.parent() {
                if !parent.exists() {
                    if !quiet { println!("creating dir {}", parent.display()); }
                    fs::create_dir_all(&parent)?;
                }
            }
            if !quiet {
                println!("decompressing {}",
                    file.enclosed_name().unwrap().display()
                );
            }
            let mut output_file = fs::File::create(&output_path).unwrap();
            io::copy(&mut file, &mut output_file).unwrap();
            if !quiet {
                println!("decompressing {} done.",
                    file.enclosed_name().unwrap().display()
                );
            }
        }
    }
    Ok(())
}
