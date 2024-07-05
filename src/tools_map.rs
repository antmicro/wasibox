/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env::Args;
use std::io::Result;

pub type AppletType = fn(Args) -> Result<()>;

lazy_static! {
    pub static ref TOOLS_MAP: HashMap<&'static str, AppletType> = {
        let mut m: HashMap<&'static str, AppletType> = HashMap::new();
        m.insert("unzip", unzip::unzip);
        m.insert("hexdump", hexdump::hexdump);
        m.insert("imgcat", imgcat::imgcat);
        m.insert("purge", purge::purge);
        m.insert("tree", tree::tree);
        m.insert("tar", tar_wasi::tar);
        m.insert("kill", kill::kill);
        m.insert("stty", stty::stty);
        m.insert("mount", mount::mount);
        m.insert("umount", umount::umount);
        m.insert("wget", wget::wget);
        m.insert("mknod", mknod::mknod);
        m
    };
}
