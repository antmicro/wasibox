use std::io;
use std::fs;
use std::path::PathBuf;
use std::env::Args;
use std::collections::HashMap;

fn traverse(path: &PathBuf, paths: &mut Vec<String>) -> io::Result<()>{
    if let Ok(a) = fs::read_dir(path) {
        for entry in a {
            let entry = entry?;
            let path = entry.path();
            let filename = String::from(path.to_str().unwrap());
            if entry.file_type()?.is_dir() {
                traverse(&entry.path(), paths)?;
            }
            paths.push(filename);
        }
    }
    Ok(())
}

pub fn purge(mut _args: Args) -> io::Result<()> {
    // remove all mounting points before purging
    wasi_ext_lib::spawn("/usr/bin/umount", &["-a"], &HashMap::new(), false, Vec::new()).unwrap();


    println!("Removing /filesystem-initiated");
    let _ = fs::remove_file("/filesystem-initiated");

    println!("Starting purge...");
    let mut files: Vec<String> = vec![];
    traverse(&PathBuf::from("/"), &mut files)?;

    for i in files {
        let path_obj = PathBuf::from(&i);
        println!("Removing {}", i);
        if let Err(e) = if path_obj.is_dir() {
            fs::remove_dir(path_obj)
        } else {
            fs::remove_file(path_obj)
        } {
            eprintln!("Could not remove {}: {:?}", i, e);
        }
    }
    Ok(())
}
