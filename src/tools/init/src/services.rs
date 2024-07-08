use std::fs;
use std::io;

use std::collections::HashMap;

use serde::Deserialize;

use wasi_ext_lib::{spawn, Redirect};

#[derive(Deserialize, Debug)]
pub(crate) struct Service {
    pub(crate) name: String,
    pub(crate) stdin: String,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) cmd: String,
    pub(crate) args: Vec<String>,

    #[serde(skip_deserializing)]
    pub(crate) pid: i32,
}

impl Service {
    pub fn spawn(&mut self) -> io::Result<()> {
        let (_, pid) = spawn(
            &self.cmd,
            &self
                .args
                .iter()
                .map(|arg| arg.as_str())
                .collect::<Vec<&str>>(),
            &HashMap::new(),
            true,
            &[
                Redirect::Read(0, self.stdin.clone()),
                Redirect::Append(1, self.stdout.clone()),
                Redirect::Append(2, self.stderr.clone()),
            ],
        )
        .map_err(io::Error::from_raw_os_error)?;

        self.pid = pid;
        Ok(())
    }
}

pub(crate) struct ServiceManager {
    pub(crate) services: HashMap<String, Service>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    fn read_services() -> io::Result<impl std::iter::Iterator<Item = io::Result<Service>>> {
        Ok(
            fs::read_dir("/etc/init.d")?.map(|entry| -> io::Result<Service> {
                if let Ok(mut service) =
                    serde_json::from_reader::<fs::File, Service>(fs::File::open(entry?.path())?)
                {
                    service.pid = -1;
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
        self.services = Self::read_services()?
            .flatten()
            .map(|service| (service.name.clone(), service))
            .collect::<HashMap<String, Service>>();
        Ok(())
    }

    pub fn spawn_services(&mut self) -> io::Result<()> {
        self.services
            .values_mut()
            .map(|service| service.spawn())
            .filter(|r| r.is_err())
            .for_each(|err| eprintln!("{:?}", err));
        Ok(())
    }
}
