use std::io;
use std::env;

use clap::{Arg, Command};

pub fn kill(args: env::Args) -> io::Result<()> {
    let name = env::args().next().unwrap_or(env!("CARGO_PKG_NAME").to_string());

    let matches = Command::new(name)
        .version(env!("CARGO_PKG_VERSION"))
        .no_binary_name(true)
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::new("pid")
                .help("Point process that will receive signal")
                .value_name("PID")
                .required(true)
        ).get_matches_from(args);

    let pid = if let Some(pid_str) = matches.value_of("pid") {
        if let Ok(pid) = pid_str.parse::<i32>() {
            pid
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Cannot parse pid!"));
        }
    } else {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Cannot get pid!"));
    };

    if let Err(e) = wasi_ext_lib::kill(pid, wasi_ext_lib::SIGNAL_KILL) {
        return Err(io::Error::from_raw_os_error(e));
    }

    Ok(())
}
