use std::io;
use std::env;

use wasibox::tools_map::TOOLS_MAP;

fn main() -> io::Result<()> {
    let mut args = env::args();
    match args.next() {
        Some(s) => {
            if let Some(x) = TOOLS_MAP.get(
                &if s == env!("CARGO_PKG_NAME") || s == format!("{}.wasm", env!("CARGO_PKG_NAME")) {
                    if let Some(tool) = args.next() {
                        tool
                    } else {
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing tool name"));
                    }
                } else { s }[..]
            ) {
                x(args)
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "No such tool"))
            }
        }
        None => {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing command line arguments"))
        }
    }
}
