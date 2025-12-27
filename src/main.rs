// Cargo.toml dependencies:
// [dependencies]
// serde = { version = "1.0", features = ["derive"] }
// toml = "0.8"
// sysinfo = "0.30"
// whoami = "1.4"
// colored = "2.1"

use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use sysinfo::{Disks, System};

// ============================================================================
// Configuration Structures
// ============================================================================

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
struct Config {
    language: Language,
    theme: Theme,
    categories: Vec<CategoryConfig>,
    custom_fields: Vec<CustomField>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Language {
    English,
    Russian,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Theme {
    primary: String,
    secondary: String,
    accent: String,
    text: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct CategoryConfig {
    name: String,
    #[serde(default)]
    subcategories: Vec<String>,
    #[serde(default = "default_true")]
    enabled: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct CustomField {
    name: String,
    command: String,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::English,
            theme: Theme::default(),
            categories: default_categories(),
            custom_fields: vec![],
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: "bright_yellow".to_string(),
            secondary: "bright_cyan".to_string(),
            accent: "bright_magenta".to_string(),
            text: "bright_white".to_string(),
        }
    }
}

fn default_categories() -> Vec<CategoryConfig> {
    vec![
        CategoryConfig {
            name: "system".to_string(),
            subcategories: vec!["os".to_string(), "kernel".to_string(), "arch".to_string()],
            enabled: true,
        },
        CategoryConfig {
            name: "hardware".to_string(),
            subcategories: vec!["host".to_string(), "cpu".to_string(), "gpu".to_string()],
            enabled: true,
        },
        CategoryConfig {
            name: "resources".to_string(),
            subcategories: vec!["memory".to_string(), "swap".to_string(), "disk".to_string()],
            enabled: true,
        },
        CategoryConfig {
            name: "environment".to_string(),
            subcategories: vec!["uptime".to_string(), "shell".to_string(), "terminal".to_string(), "de".to_string()],
            enabled: true,
        },
        CategoryConfig {
            name: "network".to_string(),
            subcategories: vec!["local_ip".to_string(), "public_ip".to_string()],
            enabled: false,
        },
    ]
}

// ============================================================================
// Localization
// ============================================================================

struct Localizer {
    language: Language,
}

impl Localizer {
    fn new(language: Language) -> Self {
        Self { language }
    }

    fn get<'a>(&self, key: &'a str) -> &'a str {
        match self.language {
            Language::English => self.get_english(key),
            Language::Russian => self.get_russian(key),
        }
    }

    fn get_english<'a>(&self, key: &'a str) -> &'a str {
        match key {
            "system" => "System",
            "hardware" => "Hardware",
            "resources" => "Res",
            "environment" => "Env",
            "network" => "Net",
            "os" => "OS",
            "kernel" => "Kernel",
            "arch" => "Arch",
            "host" => "Host",
            "cpu" => "CPU",
            "gpu" => "GPU",
            "memory" => "RAM",
            "swap" => "Swap",
            "disk" => "Disk",
            "uptime" => "Uptime",
            "shell" => "Shell",
            "terminal" => "Terminal",
            "de" => "DE/WM",
            "local_ip" => "Local IP",
            "public_ip" => "Public IP",
            _ => key,
        }
    }

    fn get_russian<'a>(&self, key: &'a str) -> &'a str {
        match key {
            "system" => "Система",
            "hardware" => "Железо",
            "resources" => "Ресы",
            "environment" => "Окружение",
            "network" => "Сеть",
            "os" => "ОСь",
            "kernel" => "Ядро",
            "arch" => "Архитектура",
            "host" => "Имя ПиСи",
            "cpu" => "Проц",
            "gpu" => "Видюха",
            "memory" => "Память",
            "swap" => "Своп",
            "disk" => "Диск",
            "uptime" => "Аптайм",
            "shell" => "Шелл",
            "terminal" => "Консось",
            "de" => "ДЕ / ВМ",
            "local_ip" => "Локал IP",
            "public_ip" => "Внешний IP",
            _ => key,
        }
    }
}

// ============================================================================
// System Information Gathering
// ============================================================================

struct SystemInfo {
    sys: System,
    disks: Disks,
}

impl SystemInfo {
    fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        
        Self { sys, disks }
    }

    fn get_info(&mut self, subcategory: &str) -> Option<String> {
        match subcategory {
            "os" => Some(format!(
                "{} {}",
                System::name()?,
                System::os_version()?
            )),
            "kernel" => System::kernel_version(),
            "arch" => Some(std::env::consts::ARCH.to_string()),
            "host" => Some(whoami::devicename()),
            "cpu" => {
                let cpu = self.sys.cpus().first()?;
                Some(cpu.brand().to_string())
            }
            "gpu" => self.get_gpu_info(),
            "memory" => {
                let used = self.sys.used_memory() / 1024 / 1024;
                let total = self.sys.total_memory() / 1024 / 1024;
                Some(format!("{} MB / {} MB", used, total))
            }
            "swap" => {
                let used = self.sys.used_swap() / 1024 / 1024;
                let total = self.sys.total_swap() / 1024 / 1024;
                if total > 0 {
                    Some(format!("{} MB / {} MB", used, total))
                } else {
                    Some("N/A".to_string())
                }
            }
            "disk" => {
                let disk = self.disks.first()?;
                let used = (disk.total_space() - disk.available_space()) / 1024 / 1024 / 1024;
                let total = disk.total_space() / 1024 / 1024 / 1024;
                Some(format!("{} GB / {} GB", used, total))
            }
            "uptime" => {
                let uptime = System::uptime();
                let days = uptime / 86400;
                let hours = (uptime % 86400) / 3600;
                let mins = (uptime % 3600) / 60;
                
                if days > 0 {
                    Some(format!("{}d {}h {}m", days, hours, mins))
                } else {
                    Some(format!("{}h {}m", hours, mins))
                }
            }
            "shell" => env::var("SHELL")
                .ok()
                .and_then(|s| s.split('/').last().map(|s| s.to_string())),
            "terminal" => env::var("TERM").ok(),
            "de" => self.get_desktop_environment(),
            "local_ip" => self.get_local_ip(),
            "public_ip" => self.get_public_ip(),
            _ => None,
        }
    }

    fn get_gpu_info(&self) -> Option<String> {
        execute_command("lspci | grep -i vga | cut -d':' -f3")
            .filter(|s| s != "N/A")
    }

    fn get_desktop_environment(&self) -> Option<String> {
        env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| env::var("DESKTOP_SESSION"))
            .ok()
    }

    fn get_local_ip(&self) -> Option<String> {
        execute_command("hostname -I | awk '{print $1}'")
            .filter(|s| s != "N/A")
    }

    fn get_public_ip(&self) -> Option<String> {
        execute_command("curl -s ifconfig.me")
            .filter(|s| s != "N/A")
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

fn execute_command(cmd: &str) -> Option<String> {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn load_config() -> Config {
    // Try to find config in priority order:
    // 1. ~/.config/rushfetch/config.toml (user config)
    // 2. /etc/rushfetch/config.toml (system config)
    // 3. Default settings
    
    let user_config = env::var("HOME")
        .map(|h| format!("{}/.config/rushfetch/config.toml", h))
        .ok();
    
    let system_config = "/etc/rushfetch/config.toml";
    
    // Try to load user config
    if let Some(path) = user_config {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str(&content) {
                return config;
            }
        }
    }
    
    // If not found, try system config
    if let Ok(content) = fs::read_to_string(system_config) {
        if let Ok(config) = toml::from_str(&content) {
            return config;
        }
    }
    
    // If nothing found, use defaults
    Config::default()
}

fn get_icon(category: &str) -> &str {
    match category {
        "system" => "◉",
        "hardware" => "◈",
        "network" => "◎",
        _ => "◆",
    }
}

fn apply_color(text: &str, color_name: &str) -> ColoredString {
    match color_name {
        "black" => text.black(),
        "red" => text.red(),
        "green" => text.green(),
        "yellow" => text.yellow(),
        "blue" => text.blue(),
        "magenta" => text.magenta(),
        "cyan" => text.cyan(),
        "white" => text.white(),
        "bright_black" => text.bright_black(),
        "bright_red" => text.bright_red(),
        "bright_green" => text.bright_green(),
        "bright_yellow" => text.bright_yellow(),
        "bright_blue" => text.bright_blue(),
        "bright_magenta" => text.bright_magenta(),
        "bright_cyan" => text.bright_cyan(),
        "bright_white" => text.bright_white(),
        _ => text.white(),
    }
}

// ============================================================================
// Display Logic
// ============================================================================

fn print_info(config: &Config, info: &mut SystemInfo) {
    let localizer = Localizer::new(config.language);
    let username = whoami::username();
    let hostname = whoami::devicename();

    // Header
    let header = format!("{}@{}", username, hostname);
    println!("\n {}\n", apply_color(&header, &config.theme.accent).bold());

    // Categories
    for category in &config.categories {
        if !category.enabled {
            continue;
        }

        let cat_name = localizer.get(&category.name);
        let icon = get_icon(&category.name);
        
        println!(
            " {} {}",
            apply_color(icon, &config.theme.primary).bold(),
            apply_color(cat_name, &config.theme.primary).bold()
        );

        // Subcategories
        for subcat in &category.subcategories {
            if let Some(value) = info.get_info(subcat) {
                let label = localizer.get(subcat);
                println!(
                    "   {} {}",
                    apply_color(&format!("{:.<14}", label), &config.theme.secondary),
                    apply_color(&value, &config.theme.text).bold()
                );
            }
        }
        println!();
    }

    // Custom fields
    if !config.custom_fields.is_empty() {
        println!(
            " {} {}\n",
            apply_color("◆", &config.theme.primary).bold(),
            apply_color("CUSTOM", &config.theme.primary).bold()
        );

        for custom in &config.custom_fields {
            let value = execute_command(&custom.command).unwrap_or_else(|| "N/A".to_string());
            println!(
                "   {} {}",
                apply_color(&format!("{:.<14}", custom.name), &config.theme.secondary),
                apply_color(&value, &config.theme.text).bold()
            );
        }
        println!();
    }
}

// ============================================================================
// Entry Point
// ============================================================================

fn main() {
    let config = load_config();
    let mut info = SystemInfo::new();
    
    print_info(&config, &mut info);
}