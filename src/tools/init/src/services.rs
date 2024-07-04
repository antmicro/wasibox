use std::fs;
use std::io;

use std::collections::HashMap;

use serde::Deserialize;

use wasi_ext_lib::{spawn, Redirect};

#[derive(Deserialize, Debug)]
pub(crate) struct Service {
    pub(crate) stdin: String,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) cmd: String,
    pub(crate) args: Vec<String>,
}

pub(crate) struct ServiceManager {
    services: Vec<Service>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self { services: vec![] }
    }

    fn read_services() -> io::Result<impl std::iter::Iterator<Item = io::Result<Service>>> {
        Ok(
            fs::read_dir("/etc/init.d")?.map(|entry| -> io::Result<Service> {
                if let Ok(service) = serde_json::from_reader(fs::File::open(entry?.path())?) {
                    Ok(service)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Invalid init service",
                    ))
                }
            }),
        )
    }

    pub fn load_services(&mut self) -> io::Result<()> {
        self.services = Self::read_services()?.flatten().collect::<Vec<Service>>();
        Ok(())
    }

    pub fn spawn_services(&self) -> io::Result<()> {
        self.services
            .iter()
            .map(|service| -> io::Result<()> {
                spawn(
                    &service.cmd,
                    &service
                        .args
                        .iter()
                        .map(|arg| arg.as_str())
                        .collect::<Vec<&str>>(),
                    &HashMap::new(),
                    true,
                    &[
                        Redirect::Read(0, service.stdin.clone()),
                        Redirect::Append(1, service.stdout.clone()),
                        Redirect::Append(2, service.stderr.clone()),
                    ],
                )
                .map_err(io::Error::from_raw_os_error)?;

                Ok(())
            })
            .filter(|r| r.is_err())
            .for_each(|err| eprintln!("{:?}", err));
        Ok(())
    }
}
