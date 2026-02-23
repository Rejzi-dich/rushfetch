pub mod colors;

use crate::config::{Config, InfoField};
use crate::data::SysData;
use crate::ui::colors::colorize;
use crate::localization::{localize_category, localize_field, category_icon};
use crate::config::defaults::default_fields;
use crate::ascii::load_ascii_art;
use crate::utils::{shell_exec, unicode_str_width, whoami_username, whoami_hostname};
use colored::*;

pub struct Renderer<'a> {
    config: &'a Config,
    data:   &'a SysData,

    ascii_lines: Vec<String>,
}

impl<'a> Renderer<'a> {
    pub fn new(config: &'a Config, data: &'a SysData) -> Self {
        let ascii_lines = if config.ascii.enabled {
            load_ascii_art(&config.ascii)
        } else { vec![] };

        Self { config, data, ascii_lines }
    }

    fn build_info_lines(&self) -> Vec<String> {
        let lang = self.config.language;
        let theme = &self.config.theme;
        let mut lines: Vec<String> = Vec::new();

        let username = whoami_username();
        let hostname = whoami_hostname();
        let header = format!("{}@{}", username, hostname);
        let separator = "─".repeat(header.len());

        lines.push(format!(
            "{}", colorize(&header, &theme.accent).bold()
        ));
        lines.push(format!(
            "{}", colorize(&separator, &theme.separator)
        ));
        lines.push(String::new());

        for category_cfg in &self.config.categories {
            if !category_cfg.enabled { continue; }

            let category = category_cfg.category;
            let category_name = localize_category(category, lang);
            let icon = if self.config.show_icons {
                category_icon(category)
            } else { "" };

            lines.push(format!(
                "{}{}",
                colorize(icon, &theme.primary),
                colorize(category_name, &theme.primary).bold()
            ));

            let fields: &[InfoField] = if category_cfg.fields.is_empty() {
                default_fields(category)
            } else {
                &category_cfg.fields
            };

            for &field in fields {
                if let Some(value) = self.data.get(field) {
                    let label = localize_field(field, lang);
                    let dot_label = format!("{:.<14}", format!("{} ", label));
                    lines.push(format!(
                        "  {} {}",
                        colorize(&dot_label, &theme.secondary),
                        colorize(&value, &theme.text).bold()
                    ));
                }
            }
            lines.push(String::new());
        }

        // Кастомные поля
        if !self.config.custom_fields.is_empty() {
            let icon = if self.config.show_icons { "󰆾 " } else { "" };
            lines.push(format!(
                "{}{}",
                colorize(icon, &theme.primary),
                colorize("Custom", &theme.primary).bold()
            ));

            for custom in &self.config.custom_fields {
                let value = shell_exec(&custom.command).unwrap_or_else(|| "N/A".to_string());
                let dot_label = format!("{:.<16}", format!("{} ", custom.label));
                lines.push(format!(
                    "  {} {}",
                    colorize(&dot_label, &theme.secondary),
                    colorize(&value, &theme.text).bold()
                ));
            }
            lines.push(String::new());
        }

        lines
    }

    pub fn render(&self) {
        let info_lines = self.build_info_lines();

        if self.ascii_lines.is_empty() {
            println!();
            for line in &info_lines {
                println!(" {}", line);
            }
            println!();
            return;
        }

        // Считаем реальную ширину ASCII art
        let actual_ascii_width = self.ascii_lines.iter()
            .map(|l| unicode_str_width(l))
            .max().unwrap_or(0);
        
        let ascii_w = actual_ascii_width + 1; // 1 на отступ
        let gap = "  ";
        let total = self.ascii_lines.len().max(info_lines.len());

        println!();
        for i in 0..total {
            let art = self.ascii_lines.get(i).map(|s| s.as_str()).unwrap_or("");
            let info = info_lines.get(i).map(|s| s.as_str()).unwrap_or("");

            let colored_art = colorize(art, &self.config.ascii.color).bold();
            print!(" {:<width$}{}", colored_art, gap, width = ascii_w);
            println!("{}", info);
        }
        println!();
    }
}
