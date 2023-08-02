/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

pub const GZ_MATCH: &str = "gzip";
pub const BZ2_MATCH: &str = "bzip2";

// #[cfg(target_os = "wasi")]
// pub const STDIN: u32 = 0;
#[cfg(target_os = "wasi")]
pub const STDOUT: u32 = 1;
// #[cfg(target_os = "wasi")]
// pub const STDERR: u32 = 2;
