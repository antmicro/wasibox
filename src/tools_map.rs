/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env::Args;
use std::io::Result;

lazy_static! {
    pub static ref TOOLS_MAP: HashMap<&'static str, fn(Args) -> Result<()>> = {
        let mut m: HashMap<&'static str, fn(Args) -> Result<()>> = HashMap::new();
        m.insert("unzip", unzip::unzip);
        m.insert("hexdump", hexdump::hexdump);
        m.insert("imgcat", imgcat::imgcat);
        m.insert("purge", purge::purge);
        m.insert("tree", tree::tree);
        m.insert("tar", tar_wasi::tar);
        m
    };
}
