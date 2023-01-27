mod constants;
mod compression;

use std::fs;
use std::path::Path;
use std::io;
use std::io::Read;
use std::env;

use tar::{Builder, Archive};
use clap::{ArgMatches, ArgGroup, Arg, Command};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;

use compression::Compression;

fn extract_stream<R: io::Read + 'static>(stream: R, comp: Compression) -> io::Result<Box<dyn io::Read>> {
    match comp {
        Compression::Gzip => {
            let decoder = GzDecoder::new(stream);
            if let None = decoder.header() {
                Err(io::Error::new(io::ErrorKind::InvalidInput, "not in gzip format"))
            } else {
                Ok(Box::new(decoder))
            }
        },
        Compression::Bzip => {
            let mut decoder = BzDecoder::new(stream);
            // bzip2 lib doesn't implement reading headers so we read 0 bytes to
            // make the decoder check if it's header is coorect
            if let Err(_) = decoder.read(&mut Vec::new()) {
                Err(io::Error::new(io::ErrorKind::InvalidInput, "not in bzip2 format"))
            } else {
                Ok(Box::new(decoder))
            }
        },
        Compression::None => Ok(Box::new(stream))
    }
}

fn encode_stream<R: io::Write + 'static>(stream: R, comp: Compression) -> Box<dyn io::Write> {
    match comp {
        Compression::Gzip => Box::new(GzEncoder::new(stream, flate2::Compression::best())),
        Compression::Bzip => Box::new(BzEncoder::new(stream, bzip2::Compression::best())),
        Compression::None => Box::new(stream)
    }
}

struct Context {
    file: Option<String>,
    create: bool,
    extract: bool,
    verbose: bool,
    compression: Compression,
    files: Vec<String>,
}

impl From<ArgMatches> for Context {
    fn from(m: ArgMatches) -> Self {
        Context {
            file: m.value_of("file").map(str::to_string),
            create: m.is_present("create"),
            extract: m.is_present("extract"),
            verbose: m.is_present("verbose"),
            compression: Compression::from_cli(&m),
            files: if let Some(v) = m.values_of("FILES") { v.map(String::from).collect() } else { Vec::new() },
        }
    }
}

pub fn tar(args: env::Args) -> io::Result<()> {
    #[cfg(target_os = "wasi")]
    let _ = wasi_ext_lib::chdir(
        &match wasi_ext_lib::getcwd() {
            Ok(p) => p,
            Err(_e) => String::from("/")
        });
    let name = env::args().next().unwrap_or(env!("CARGO_PKG_NAME").to_string());
    let matches = Command::new(name)
        .version(env!("CARGO_PKG_VERSION"))
        .author("Antmicro <www.antmicro.com>")
        .arg(
            Arg::new("file")
                .help("Use archive file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .takes_value(true)
        )
        .arg(
            Arg::new("create")
                .help("Create a new archive")
                .short('c')
                .long("create")
                .takes_value(false)
        )
        .arg(
            Arg::new("extract")
                .help("Extract files from an archive")
                .short('x')
                .long("extract")
                .takes_value(false)
        )
        .arg(
            Arg::new("verbose")
                .help("Verbosely list files processed")
                .short('v')
                .long("verbose")
                .takes_value(false)
        )
        .arg(
            Arg::new(constants::GZ_MATCH)
                .help("Filter the archive through gzip")
                .short('z')
                .long(constants::GZ_MATCH)
                .takes_value(false)
        )
        .arg(
            Arg::new(constants::BZ2_MATCH)
                .help("Filter the archive through bzip")
                .short('j')
                .long(constants::BZ2_MATCH)
                .takes_value(false)
        )
        .group(
            ArgGroup::with_name("action")
                .args(&["extract", "create"])
                .required(true)
        )
        .group(
            ArgGroup::with_name("compressions")
                .args(&[constants::GZ_MATCH, constants::BZ2_MATCH])
                .required(false)
        )
        .arg(
            Arg::new("FILES")
                .multiple_values(true)
                .help("File to compress or decompress destination")
                .index(1)
        ).get_matches_from(args);
    let context = Context::from(matches);
    if context.extract {
        let (input_stream, compression): (Box<dyn io::Read>, Compression) = if let Some(path) = &context.file {
            (Box::new(fs::File::open(&path)?),
            if let Compression::None = context.compression {
                Compression::from_extensions(&Path::new(&path))?
            } else {
                context.compression
            })
        } else {
            (Box::new(io::stdin()),
            context.compression)
        };
        // TODO: if stream extraction failed, the program should return 2
        // using ? operator makes the program exit with 1
        untar(&mut extract_stream(input_stream, compression)?, &context)?;
    } else if context.create {
        let (output_stream, compression): (Box<dyn io::Write>, Compression) = if let Some(path) = &context.file {
            (Box::new(fs::File::create(&path)?),
            if let Compression::None = context.compression {
                Compression::from_extensions(&Path::new(&path))?
            } else {
                context.compression
            })
        } else {
            #[cfg(target_os = "wasi")] {
                let args = format!("{{\"args\": \"[{}]\"}}", constants::STDOUT);
                if let Ok(r) = fs::read_link(format!("/!{{\
                    \"command\": \"isatty\",\
                    \"buf_len\": {},\
                    \"buf_ptr\": \"{:?}\"}}", &args.len(), &args.as_ptr())) {
                    let result = r.to_str().unwrap().trim_matches(char::from(0)).to_string();
                    let (err, tty) = result.split_once("\x1b").unwrap();
                    if err == "0" && tty == "0" {
                        (Box::new(io::stdout()), context.compression)
                    } else {
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Refusing to write to stdout"));
                    }
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Refusing to write to stdout"));
                }
            }
            #[cfg(not(target_os = "wasi"))] {
                if atty::is(atty::Stream::Stdout) {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Refusing to write to stdout"));
                } else {
                    (Box::new(io::stdout()), context.compression)
                }
            }
        };
        create_tar(&mut encode_stream(output_stream, compression), &context)?;
    }
    Ok(())
}

fn untar<R: ?core::marker::Sized + io::Read>(stream: &mut R, con: &Context) -> io::Result<()> {
    let mut archive = Archive::new(stream);
    for entry in archive.entries()? {
        let mut entry = entry?;
        if con.verbose { println!("{}", entry.path().unwrap().display()); }
        entry.unpack_in(".")?;
    }
    Ok(())
}

fn walk_dir<F: FnMut(&Path) -> io::Result<()>>(path: &Path, action: &mut F) -> io::Result<()> {
    action(path)?;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            walk_dir(&entry?.path(), action)?;
        }
    }
    Ok(())
}

fn create_tar<R: ?core::marker::Sized + io::Write>(stream: &mut R, con: &Context) -> io::Result<()> {
    let mut builder = Builder::new(stream);
    for f in &con.files {
        let path = Path::new(f);
        walk_dir(path, &mut |p: &Path| -> io::Result<()> {
            if con.verbose { println!("{}", p.display()); }
            builder.append_path(p)
        })?;
    }
    builder.finish()?;
    Ok(())
}
