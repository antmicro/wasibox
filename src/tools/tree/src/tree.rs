/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env::Args;
use std::{fs, io};

use clap::Parser;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

fn visit_dirs(
    dir: &Path,
    current_path: &PathBuf,
    cli: &CliArgs,
    cb: &dyn Fn(&PathBuf, &DirEntry),
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !cli.all
                && path
                    .as_path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with('.')
            {
                continue;
            }
            cb(current_path, &entry);
            if path.is_dir() {
                visit_dirs(&path, &current_path.join(entry.file_name()), cli, cb)?;
            }
        }
    }
    Ok(())
}

#[derive(Parser)]
#[command(no_binary_name = true)]
struct CliArgs {
    #[arg(short, long)]
    all: bool,
    files: Vec<String>,
}

pub fn tree(args: Args) -> io::Result<()> {
    let cli = CliArgs::parse_from(args);

    if cli.files.is_empty() {
        for dir in &cli.files {
            visit_dirs(
                Path::new(&dir),
                &PathBuf::new(),
                &cli,
                &|current_path: &PathBuf, entry: &DirEntry| {
                    println!("{}", current_path.join(entry.file_name()).display());
                },
            )?;
        }
    } else {
        visit_dirs(
            Path::new("."),
            &PathBuf::new(),
            &cli,
            &|current_path: &PathBuf, entry: &DirEntry| {
                println!("{}", current_path.join(entry.file_name()).display());
            },
        )?;
    }

    Ok(())
}
