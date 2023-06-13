use std::io;
use std::env;

pub fn stty(args: env::Args) -> io::Result<()> {
    if let Some(err) = args.map(|arg: String| -> io::Result<()> {
        let off = arg.starts_with("-");

        let arg_: &str = if off {
            &arg[1..]
        } else {
            &arg
        };

        if let Err(errno) = wasi_ext_lib::ioctl(
            0,
            match arg_ {
                "raw" => wasi_ext_lib::IoctlNum::SetRaw,
                "echo" => wasi_ext_lib::IoctlNum::SetEcho,
                _ => { return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid argument")); }
            },
            Some(&mut (off as i32))
        ) {
            Err(io::Error::from_raw_os_error(errno))
        } else {
            Ok(())
        }
    }).find(|x| { x.is_err() }) {
        err
    } else {
        Ok(())
    }
}
