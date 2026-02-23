pub mod unicode;
pub mod shell;

pub use unicode::unicode_str_width;
pub use shell::{shell_exec, whoami_username, whoami_hostname, shell_name};

use std::borrow::Cow;
use std::fs;
use std::io::{self, BufRead};

pub fn read_file_cow(path: &str) -> Option<Cow<'static, str>> {
    fs::read_to_string(path)
        .ok().map(|s| Cow::Owned(s))
}

pub fn read_lines_from(path: &str) -> Option<Vec<String>> {
    let file    = fs::File::open(path).ok()?;
    let reader  = io::BufReader::new(file);
    Some(reader.lines().filter_map(|l| l.ok()).collect())
}
