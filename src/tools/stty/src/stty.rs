use std::env;
use std::io;

pub fn stty(args: env::Args) -> io::Result<()> {
    if let Some(err) = args
        .map(|arg: String| -> io::Result<()> {
            let mut on = !arg.starts_with('-');

            let arg_: &str = if on { &arg  } else { &arg[1..] };

            if let Err(errno) = match arg_ {
                "raw" => wasi_ext_lib::ioctl(0, wasi_ext_lib::IoctlNum::SetRaw, Some(&mut on)),
                "echo" => wasi_ext_lib::ioctl(0, wasi_ext_lib::IoctlNum::SetRaw, Some(&mut on)),
                "size" => {
                    let mut size = [0i32; 2];
                    let result = wasi_ext_lib::ioctl(
                        0,
                        wasi_ext_lib::IoctlNum::GetScreenSize,
                        Some(&mut size),
                    );
                    if let Ok(_) = result {
                        println!("{} {}", size[0], size[1]);
                    }
                    result
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Invalid argument",
                    ));
                }
            } {
                Err(io::Error::from_raw_os_error(errno))
            } else {
                Ok(())
            }
        })
        .find(|x| x.is_err())
    {
        err
    } else {
        Ok(())
    }
}
