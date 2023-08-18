/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::collections::HashMap;
use std::env::Args;
use std::fs;
use std::io;
use std::path::PathBuf;

fn traverse(path: &PathBuf, paths: &mut Vec<PathBuf>) -> io::Result<()> {
    if let Ok(a) = fs::read_dir(path) {
        for entry in a {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                traverse(&entry.path(), paths)?;
            }
            paths.push(entry.path());
        }
    }
    Ok(())
}

pub fn purge(mut _args: Args) -> io::Result<()> {
    // remove all mounting points before purging
    wasi_ext_lib::spawn("/usr/bin/umount", &["-a"], &HashMap::new(), false, &[]).unwrap();

    println!("Removing /filesystem-initiated");
    let _ = fs::remove_file("/filesystem-initiated");

    println!("Starting purge...");
    let mut files: Vec<PathBuf> = vec![];
    traverse(&PathBuf::from("/"), &mut files)?;

    for path_obj in files {
        println!("Removing {}", &path_obj.display());
        if let Err(e) = if path_obj.is_dir() {
            fs::remove_dir(&path_obj)
        } else {
            fs::remove_file(&path_obj)
        } {
            eprintln!("Could not remove {}: {:?}", &path_obj.display(), e);
        }
    }
    #[cfg(target_os = "wasi")]
    let _ = wasi_ext_lib::clean_inodes();
    Ok(())
}
