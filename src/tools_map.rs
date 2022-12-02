use std::collections::HashMap;
use std::env::Args;
use std::io::Result;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref TOOLS_MAP: HashMap<&'static str, fn(Args) -> Result<()>> = {
        let mut m: HashMap<&'static str, fn(Args) -> Result<()>> = HashMap::new();
        m.insert("unzip", unzip::unzip);
        m
    };

}
