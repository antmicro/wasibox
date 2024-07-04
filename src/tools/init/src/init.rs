/*
 * Copyright (c) 2022-2024 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use std::env;
use std::io;

mod services;

pub fn init(_args: env::Args) -> io::Result<()> {
    let mut service_manager = services::ServiceManager::new();
    service_manager.load_services()?;
    service_manager.spawn_services()?;
    Ok(())
}
