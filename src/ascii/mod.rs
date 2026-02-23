pub mod builtin;

use crate::config::AsciiConfig;
use crate::utils::unicode_str_width;
use std::fs;

pub fn load_ascii_art(cfg: &AsciiConfig) -> Vec<String> {
    if !cfg.enabled { return vec![]; }

    if let Some(path) = &cfg.file {
        if let Ok(content) = fs::read_to_string(path) {
            let lines: Vec<&str> = content.lines().collect();
            let max_width = lines.iter()
                .map(|l| unicode_str_width(l))
                .max()
                .unwrap_or(0);

            let effective_width = cfg.width.max(max_width);
            
            return lines.iter().map(|l| {
                let s = l.to_string();
    
                if unicode_str_width(&s) < effective_width {
                    let padding = effective_width - unicode_str_width(&s);
                    format!("{}{}", s, " ".repeat(padding))
                } else {
                    let mut result = String::new();
                    let mut current_width = 0;

                    for c in s.chars() {
                        let char_width = if c as u32 <= 0x7F { 1 } else { 2 };
                        if current_width + char_width > effective_width {
                            break;
                        }
                        result.push(c);
                        current_width += char_width;
                    }

                    result
                }
            }).collect();
        }
    }
    builtin::builtin_ascii(cfg.width, cfg.distro.as_deref())
}
