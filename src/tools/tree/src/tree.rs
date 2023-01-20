/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::{fs, io};
use std::env::Args;

use clap::{Arg, Command, ArgMatches, Values};
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

static OPT_ALL: &str = "all";
static FILES: &str = "files";

fn visit_dirs(dir: &Path, current_path: &PathBuf, matches: &ArgMatches, cb: &dyn Fn(&PathBuf, &DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !matches.is_present(OPT_ALL)
                && path.as_path().file_name().unwrap().to_str().unwrap().starts_with(".") { continue; }
            cb(&current_path, &entry);
            if path.is_dir() {
                visit_dirs(&path, &current_path.join(entry.file_name()), matches, cb)?;
            }
        }
    }
    Ok(())
}

pub fn tree(args: Args) -> io::Result<()> {
    let matches = Command::new("tree")
        .no_binary_name(true)
        .version("0.1")
        .about("Display all files and directories recursively")
        .arg(
            Arg::new(OPT_ALL)
                .short('a')
                .long(OPT_ALL)
                .takes_value(false)
                .help("list all files")
        )
        .arg(
            Arg::new(FILES).multiple_values(true)
        ).get_matches_from(args);

    let dirs = matches.values_of(FILES).map_or_else(|| vec!["."], Values::collect);
    for dir in dirs {
        visit_dirs(Path::new(&dir), &PathBuf::new(), &matches, &|current_path: &PathBuf, entry: &DirEntry| {
            println!("{}", current_path.join(entry.file_name()).display());
        })?;
    }

    Ok(())
}
