// ──────────────────────────────────────────────────────────────
// rushfetch - fast, minimal system info tool
// Reads system data directly from /proc and /sys - no shell spawning
// for built-in fields. Shell is only used for user-defined custom_fields.
// ──────────────────────────────────────────────────────────────

use colored::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use unicode_width::{UnicodeWidthStr, UnicodeWidthChar};

// ─── Config Structures ───────────────────────────────────────────────────

/// All built-in info fields. Serde uses lowercase/snake_case names in TOML.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum InfoField {
    Os,
    Kernel,
    Arch,
    Host,
    Cpu,
    Gpu,
    Memory,
    Swap,
    Disk,
    Uptime,
    Shell,
    Terminal,
    De,
    LocalIp,
    PublicIp,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BuiltinCategory {
    System,
    Hardware,
    Resources,
    Environment,
    Network,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CategoryConfig {
    pub category: BuiltinCategory,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Override the displayed fields and their order.
    /// If omitted, the default set for this category is used.
    #[serde(default)]
    pub fields: Vec<InfoField>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CustomField {
    pub label: String,
    pub command: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AsciiConfig {
    /// Enable ASCII art on the left side
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Path to a plain-text ASCII file. If None, built-in distro art is used.
    pub file: Option<String>,
    /// Override distro detection. If set, use this distro's built-in art.
    pub distro: Option<String>,
    /// Width (columns) reserved for the ASCII block. Default 20.
    #[serde(default = "default_ascii_width")]
    pub width: usize,
    /// Color for the ASCII art
    #[serde(default = "default_accent_color")]
    pub color: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Theme {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub text: String,
    pub separator: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    English,
    Russian,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub ascii: AsciiConfig,
    /// Show category/field icons
    #[serde(default = "default_true")]
    pub show_icons: bool,
    #[serde(default = "default_categories")]
    pub categories: Vec<CategoryConfig>,
    #[serde(default)]
    pub custom_fields: Vec<CustomField>,
}

// ─── Defaults ────────────────────────────────────────────────────────────

fn default_true() -> bool {
    true
}
fn default_ascii_width() -> usize {
    20
}
fn default_accent_color() -> String {
    "bright_cyan".to_string()
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: "bright_yellow".to_string(),
            secondary: "bright_cyan".to_string(),
            accent: "bright_magenta".to_string(),
            text: "bright_white".to_string(),
            separator: "bright_black".to_string(),
        }
    }
}

impl Default for AsciiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            file: None,
            distro: None,
            width: 20,
            color: "bright_cyan".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::default(),
            theme: Theme::default(),
            ascii: AsciiConfig::default(),
            show_icons: true,
            categories: default_categories(),
            custom_fields: vec![],
        }
    }
}

fn default_categories() -> Vec<CategoryConfig> {
    vec![
        CategoryConfig {
            category: BuiltinCategory::System,
            enabled: true,
            fields: vec![],
        },
        CategoryConfig {
            category: BuiltinCategory::Hardware,
            enabled: true,
            fields: vec![],
        },
        CategoryConfig {
            category: BuiltinCategory::Resources,
            enabled: true,
            fields: vec![],
        },
        CategoryConfig {
            category: BuiltinCategory::Environment,
            enabled: true,
            fields: vec![],
        },
        CategoryConfig {
            category: BuiltinCategory::Network,
            enabled: false,
            fields: vec![],
        },
    ]
}

// ─── Default fields per category ─────────────────────────────────────────

fn default_fields(cat: BuiltinCategory) -> &'static [InfoField] {
    match cat {
        BuiltinCategory::System => &[InfoField::Os, InfoField::Kernel, InfoField::Arch],
        BuiltinCategory::Hardware => &[InfoField::Host, InfoField::Cpu, InfoField::Gpu],
        BuiltinCategory::Resources => {
            &[InfoField::Memory, InfoField::Swap, InfoField::Disk]
        }
        BuiltinCategory::Environment => &[
            InfoField::Uptime,
            InfoField::Shell,
            InfoField::Terminal,
            InfoField::De,
        ],
        BuiltinCategory::Network => &[InfoField::LocalIp, InfoField::PublicIp],
    }
}

// ─── Localization ────────────────────────────────────────────────────────

fn localize_category(cat: BuiltinCategory, lang: Language) -> &'static str {
    match lang {
        Language::English => match cat {
            BuiltinCategory::System => "System",
            BuiltinCategory::Hardware => "Hardware",
            BuiltinCategory::Resources => "Resources",
            BuiltinCategory::Environment => "Environment",
            BuiltinCategory::Network => "Network",
        },
        Language::Russian => match cat {
            BuiltinCategory::System => "Система",
            BuiltinCategory::Hardware => "Железо",
            BuiltinCategory::Resources => "Ресурсы",
            BuiltinCategory::Environment => "Окружение",
            BuiltinCategory::Network => "Сеть",
        },
    }
}

fn localize_field(field: InfoField, lang: Language) -> &'static str {
    match lang {
        Language::English => match field {
            InfoField::Os => "OS",
            InfoField::Kernel => "Kernel",
            InfoField::Arch => "Arch",
            InfoField::Host => "Host",
            InfoField::Cpu => "CPU",
            InfoField::Gpu => "GPU",
            InfoField::Memory => "RAM",
            InfoField::Swap => "Swap",
            InfoField::Disk => "Disk",
            InfoField::Uptime => "Uptime",
            InfoField::Shell => "Shell",
            InfoField::Terminal => "Terminal",
            InfoField::De => "DE / WM",
            InfoField::LocalIp => "Local IP",
            InfoField::PublicIp => "Public IP",
        },
        Language::Russian => match field {
            InfoField::Os => "ОС",
            InfoField::Kernel => "Ядро",
            InfoField::Arch => "Архитектура",
            InfoField::Host => "Имя ПК",
            InfoField::Cpu => "Проц",
            InfoField::Gpu => "Видеокарта",
            InfoField::Memory => "Память",
            InfoField::Swap => "Своп",
            InfoField::Disk => "Диск",
            InfoField::Uptime => "Аптайм",
            InfoField::Shell => "Шелл",
            InfoField::Terminal => "Терминал",
            InfoField::De => "ДЕ / ВМ",
            InfoField::LocalIp => "Локал IP",
            InfoField::PublicIp => "Внешний IP",
        },
    }
}

fn category_icon(cat: BuiltinCategory) -> &'static str {
    match cat {
        BuiltinCategory::System => "󰍛 ",
        BuiltinCategory::Hardware => "󰘚 ",
        BuiltinCategory::Resources => "󰓅 ",
        BuiltinCategory::Environment => "󰆍 ",
        BuiltinCategory::Network => "󰀂 ",
    }
}

// ─── Fast system info - NO shell for built-in fields ─────────────────────

/// All data collected once at startup.
pub struct SysData {
    pub os: Option<String>,
    pub kernel: Option<String>,
    pub arch: &'static str,
    pub host: Option<String>,
    pub cpu: Option<String>,
    pub gpu: Option<String>,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub swap_used_mb: u64,
    pub swap_total_mb: u64,
    pub disk_used_gb: u64,
    pub disk_total_gb: u64,
    pub uptime_secs: u64,
    pub shell: Option<String>,
    pub terminal: Option<String>,
    pub de: Option<String>,
    pub local_ip: Option<String>,
}

impl SysData {
    pub fn collect() -> Self {
        // Collect everything in parallel where possible using scoped threads
        let (memory_used_mb, memory_total_mb, swap_used_mb, swap_total_mb) = read_meminfo();
        let (disk_used_gb, disk_total_gb) = read_disk_root();

        Self {
            os: read_os_pretty_name(),
            kernel: read_kernel_version(),
            arch: std::env::consts::ARCH,
            host: read_hostname(),
            cpu: read_cpu_model(),
            gpu: detect_gpu(),
            memory_used_mb,
            memory_total_mb,
            swap_used_mb,
            swap_total_mb,
            disk_used_gb,
            disk_total_gb,
            uptime_secs: read_uptime(),
            shell: shell_name(),
            terminal: env::var("TERM").ok(),
            de: env::var("XDG_CURRENT_DESKTOP")
                .or_else(|_| env::var("DESKTOP_SESSION"))
                .ok(),
            local_ip: read_local_ip(),
        }
    }

    pub fn get(&self, field: InfoField) -> Option<String> {
        match field {
            InfoField::Os => self.os.clone(),
            InfoField::Kernel => self.kernel.clone(),
            InfoField::Arch => Some(self.arch.to_string()),
            InfoField::Host => self.host.clone(),
            InfoField::Cpu => self.cpu.clone(),
            InfoField::Gpu => self.gpu.clone(),
            InfoField::Memory => Some(format!(
                "{} MB / {} MB",
                self.memory_used_mb, self.memory_total_mb
            )),
            InfoField::Swap => {
                if self.swap_total_mb == 0 {
                    Some("N/A".to_string())
                } else {
                    Some(format!(
                        "{} MB / {} MB",
                        self.swap_used_mb, self.swap_total_mb
                    ))
                }
            }
            InfoField::Disk => Some(format!(
                "{} GB / {} GB",
                self.disk_used_gb, self.disk_total_gb
            )),
            InfoField::Uptime => Some(format_uptime(self.uptime_secs)),
            InfoField::Shell => self.shell.clone(),
            InfoField::Terminal => self.terminal.clone(),
            InfoField::De => self.de.clone(),
            InfoField::LocalIp => self.local_ip.clone(),
            InfoField::PublicIp => fetch_public_ip(),
        }
    }
}

// ─── `/proc` and `/sys` readers - zero shell invocations ─────────────────

fn read_lines_from(path: &str) -> Option<Vec<String>> {
    let file = fs::File::open(path).ok()?;
    let reader = io::BufReader::new(file);
    Some(reader.lines().filter_map(|l| l.ok()).collect())
}

fn read_os_pretty_name() -> Option<String> {
    let lines = read_lines_from("/etc/os-release")?;
    for line in &lines {
        if line.starts_with("PRETTY_NAME=") {
            return Some(
                line["PRETTY_NAME=".len()..]
                    .trim_matches('"')
                    .to_string(),
            );
        }
    }
    None
}

fn read_kernel_version() -> Option<String> {
    fs::read_to_string("/proc/version")
        .ok()
        .and_then(|s| {
            // "Linux version 6.x.y-..." - extract up to the first space after "version "
            let after = s.strip_prefix("Linux version ")?;
            Some(after.split_whitespace().next()?.to_string())
        })
}

fn read_hostname() -> Option<String> {
    fs::read_to_string("/proc/sys/kernel/hostname")
        .ok()
        .map(|s| s.trim().to_string())
}

fn read_cpu_model() -> Option<String> {
    let lines = read_lines_from("/proc/cpuinfo")?;
    for line in &lines {
        if line.starts_with("model name") {
            if let Some(val) = line.splitn(2, ':').nth(1) {
                // Compact: remove excessive spaces
                let compact = val
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                return Some(compact);
            }
        }
    }
    // ARM / other: try "Hardware"
    for line in &lines {
        if line.starts_with("Hardware") {
            if let Some(val) = line.splitn(2, ':').nth(1) {
                return Some(val.trim().to_string());
            }
        }
    }
    None
}

fn detect_gpu() -> Option<String> {
    // Try /sys/class/drm first - no shell needed
    let drm = Path::new("/sys/class/drm");
    if drm.exists() {
        if let Ok(entries) = fs::read_dir(drm) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let s = name.to_string_lossy();
                if s.starts_with("card") && !s.contains('-') {
                    let vendor_path = entry.path().join("device/vendor");
                    let model_path = entry.path().join("device/product");
                    if let (Ok(vendor), Ok(model)) = (
                        fs::read_to_string(&vendor_path),
                        fs::read_to_string(&model_path),
                    ) {
                        let v = vendor.trim();
                        let m = model.trim();
                        if !m.is_empty() {
                            return Some(format!("{} {}", v, m));
                        }
                    }
                    // Try uevent for GPU name
                    let uevent_path = entry.path().join("device/uevent");
                    if let Ok(ue) = fs::read_to_string(&uevent_path) {
                        for l in ue.lines() {
                            if l.starts_with("PCI_ID=") {
                                // not that useful without a database - fall through
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    // Fallback: read /proc/bus/pci/devices and match known GPU classes (0x0300)
    // This is still pure file I/O, no shell.
    if let Some(lines) = read_lines_from("/proc/bus/pci/devices") {
        for line in &lines {
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() >= 2 {
                let class_vendor = cols[1]; // e.g. "03000012"
                if class_vendor.starts_with("0300") || class_vendor.starts_with("0302") {
                    // We have a VGA device but no name from this file.
                    // Return a generic marker so the field appears rather than vanishing.
                    return Some("(detected, install lspci for details)".to_string());
                }
            }
        }
    }
    None
}

fn read_meminfo() -> (u64, u64, u64, u64) {
    let mut total = 0u64;
    let mut available = 0u64;
    let mut swap_total = 0u64;
    let mut swap_free = 0u64;

    if let Some(lines) = read_lines_from("/proc/meminfo") {
        for line in &lines {
            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("MemTotal:") => total = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0),
                Some("MemAvailable:") => {
                    available = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0)
                }
                Some("SwapTotal:") => {
                    swap_total = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0)
                }
                Some("SwapFree:") => {
                    swap_free = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0)
                }
                _ => {}
            }
        }
    }

    let used = total.saturating_sub(available) / 1024;
    let total_mb = total / 1024;
    let swap_used = swap_total.saturating_sub(swap_free) / 1024;
    let swap_total_mb = swap_total / 1024;

    (used, total_mb, swap_used, swap_total_mb)
}

fn read_disk_root() -> (u64, u64) {
    use std::mem::MaybeUninit;
    let path = std::ffi::CString::new("/").unwrap();
    let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
    unsafe {
        if libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) == 0 {
            let s = stat.assume_init();
            let total = s.f_blocks * s.f_frsize / (1024 * 1024 * 1024);
            let free = s.f_bfree * s.f_frsize / (1024 * 1024 * 1024);
            return (total.saturating_sub(free), total);
        }
    }
    (0, 0)
}

fn read_uptime() -> u64 {
    fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| {
            s.split_whitespace()
                .next()
                .and_then(|v| v.parse::<f64>().ok())
        })
        .map(|f| f as u64)
        .unwrap_or(0)
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    match days {
        0 => format!("{}h {}m", hours, mins),
        _ => format!("{}d {}h {}m", days, hours, mins),
    }
}

fn shell_name() -> Option<String> {
    env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(|s| s.to_string()))
}

fn read_local_ip() -> Option<String> {
    // Read from /proc/net/route - find default gateway interface, then get its IP
    // from /proc/net/fib_trie or /proc/net/if_inet6 - pure file I/O.
    let route = fs::read_to_string("/proc/net/route").ok()?;
    let mut iface = None;
    for line in route.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 2 && cols[1] == "00000000" {
            iface = Some(cols[0].to_string());
            break;
        }
    }
    let iface = iface?;

    // Read IP from /proc/net/fib_trie - find LOCAL entries for our interface
    // Simpler: read /proc/net/if_inet6 or just do getifaddrs via libc
    local_ip_for_iface(&iface)
}

fn local_ip_for_iface(iface: &str) -> Option<String> {
    use std::net::Ipv4Addr;
    // /proc/net/fib_trie is complex; use /proc/net/arp as alternative for default route IPs
    // Cleanest pure-rust approach: parse /proc/net/if_inet6 for IPv6 or read from
    // /sys/class/net/<iface>/... - no IP address file exists there.
    // Use libc getifaddrs instead:
    unsafe {
        let mut addrs: *mut libc::ifaddrs = std::ptr::null_mut();
        if libc::getifaddrs(&mut addrs) != 0 {
            return None;
        }
        let mut cur = addrs;
        let mut result = None;
        while !cur.is_null() {
            let a = &*cur;
            if !a.ifa_name.is_null() && !a.ifa_addr.is_null() {
                let name = std::ffi::CStr::from_ptr(a.ifa_name)
                    .to_string_lossy();
                if name == iface && (*a.ifa_addr).sa_family as i32 == libc::AF_INET {
                    let sin = a.ifa_addr as *const libc::sockaddr_in;
                    let ip_u32 = u32::from_be((*sin).sin_addr.s_addr);
                    result = Some(Ipv4Addr::from(ip_u32).to_string());
                    break;
                }
            }
            cur = a.ifa_next;
        }
        libc::freeifaddrs(addrs);
        result
    }
}

fn fetch_public_ip() -> Option<String> {
    // Public IP requires network - only requested if field is in config
    // Uses a simple TCP connection - no shell
    use std::io::{Read, Write};
    use std::net::TcpStream;
    let mut stream = TcpStream::connect("ifconfig.me:80").ok()?;
    stream
        .write_all(b"GET /ip HTTP/1.0\r\nHost: ifconfig.me\r\nUser-Agent: rushfetch\r\n\r\n")
        .ok()?;
    let mut body = String::new();
    stream.read_to_string(&mut body).ok()?;
    body.split("\r\n\r\n").nth(1).map(|s| s.trim().to_string())
}

// ─── ASCII Art ───────────────────────────────────────────────────────────

/// Built-in minimal distro art. First checks config, then detects from /etc/os-release ID field.
fn builtin_ascii(_width: usize, distro_override: Option<&str>) -> Vec<String> {
    let id = if let Some(distro) = distro_override {
        distro.to_lowercase()
    } else {
        fs::read_to_string("/etc/os-release")
            .unwrap_or_default()
            .lines()
            .find(|l| l.starts_with("ID="))
            .map(|l| l["ID=".len()..].trim_matches('"').to_lowercase())
            .unwrap_or_default()
    };

    let art: &[&str] = match id.as_str() {
        "arch" | "endeavouros" => &[
            "                                     ",
            "                  -`                 ",
            "                 .o+`                ",
            "                `oooo.                ",
            "               `+oooo:               ",
            "              `+oooooo:              ",
            "              -+oooooo+:             ",
            "            `/:-:++oooo+:            ",
            "           `/++++/+++++++:           ",
            "          `/++++++++++++++:          ",
            "         `/+++rustooooooooo\\`        ",
            "        ./ooosssso++osssssso+`       ",
            "       .oossssso-````/ossssss+`      ",
            "      -osssssso.      :ssssssso.     ",
            "     :osssssss/        osssso+++.    ",
            "    /ossssssss/        +ssssooo/-    ",
            "  `/ossssso+/:-        -:/+osssso+-  ",
            " `+sso+:-`                 `.-/+oso: ",
            "`++:.                           `-/+/",
            ".`                                 ` ",
            "                                     ",
        ],
        "apple" | "macos" | "macbook" | "yablocoder" => &[
            "                              ",
            "                    c.'       ",
            "                 ,xNMM.       ",
            "               .OMMMMo        ",
            "               lMM\"           ",
            "     .;loddo:.  .olloddol;.   ",
            "   cKMMMMMMMMMMNWMMMMMMMMMM0: ",
            " .KMMMMMMMMMMMMMMMMMMMMMMMWd. ",
            " XMMMMMMMMMMMMMMMMMMMMMMMX.   ",
            ";MMMMMMMMMMMMMMMMMMMMMMMM:    ",
            ":MMMMMMMMMMMMMMMMMMMMMMMM:    ",
            ".MMMMMMMMMMMMMMMMMMMMMMMMX.   ",
            " kMMMMMMMMMMMMMMMMMMMMMMMMWd. ",
            " 'XMMMMMMMMMMMMMMMMMMMMMMMMMMk",
            "  'XMMMMMMMMMMMMMMMMMMMMMMMMK.",
            "    kMMMMMMMMMMMMMMMMMMMMMMd  ",
            "     ;KMMMMMMMWXXWMMMMMMMk.   ",
            "       \"cooc*\"    \"*coo'\"     ",
            "                              ",
        ],
        _ => &[
            "                           ",
            "                           ",
            "                           ",
            " ⠀⠀⠀⠰⡿⠿⠛⠛⠻⠿⣷           ",
            "⠀⠀⠀⠀⠀⠀⣀⣄⡀⠀⠀⠀⠀⢀⣀⣀⣤⣄⣀⡀ ",
            "⠀⠀⠀⠀⠀⢸⣿⣿⣷⠀⠀⠀⠀⠛⠛⣿⣿⣿⡛⠿⠷",
            "⠀⠀⠀⠀⠀⠘⠿⠿⠋⠀⠀⠀⠀⠀⠀⣿⣿⣿⠇   ",
            "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠁    ",
            "⠀⠀⠀⠀⣿⣷⣄⠀⢶⣶⣷⣶⣶⣤⣀        ",
            "⠀⠀⠀⠀⣿⣿⣿⠀⠀⠀⠀⠀⠈⠙⠻⠗       ",
            "⠀⠀⠀⣰⣿⣿⣿⠀⠀⠀⠀⢀⣀⣠⣤⣴⣶⡄     ",
            "⠀⣠⣾⣿⣿⣿⣥⣶⣶⣿⣿⣿⣿⣿⠿⠿⠛⠃     ",
            "⢰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡄           ",
            "⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡁           ",
            "⠈⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁           ",
            "⠀⠀⠛⢿⣿⣿⣿⣿⣿⣿⡿⠟            ",
            "⠀⠀⠀⠀⠀⠉⠉⠉                 ",
            "                           ",
            "                           ",
            "   why do I need Linux?    ",
        ],
    };

    // Find the maximum width in the ASCII art
    let max_width = art.iter()
        .map(|l| l.width())
        .max()
        .unwrap_or(0);

    art.iter().map(|l| {
        let s = l.to_string();
        // pad to max width using unicode width
        if s.width() < max_width {
            let padding = max_width - s.width();
            format!("{}{}", s, " ".repeat(padding))
        } else {
            s.clone()
        }
    }).collect()
}

fn load_ascii_art(cfg: &AsciiConfig) -> Vec<String> {
    if !cfg.enabled {
        return vec![];
    }
    if let Some(path) = &cfg.file {
        if let Ok(content) = fs::read_to_string(path) {
            let lines: Vec<&str> = content.lines().collect();
            // Find the maximum width in the ASCII art
            let max_width = lines.iter()
                .map(|l| l.width())
                .max()
                .unwrap_or(0);
            
            // Use the larger of config width or actual art width
            let effective_width = cfg.width.max(max_width);
            
            return lines.iter().map(|l| {
                let s = l.to_string();
                // pad to effective width using unicode width
                if s.width() < effective_width {
                    let padding = effective_width - s.width();
                    format!("{}{}", s, " ".repeat(padding))
                } else {
                    // Truncate by unicode width, not by char count
                    let mut result = String::new();
                    let mut current_width = 0;
                    for c in s.chars() {
                        let char_width = c.width().unwrap_or(0);
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
    builtin_ascii(cfg.width, cfg.distro.as_deref())
}


// ─── Color helper ────────────────────────────────────────────────────────

fn colorize<'a>(text: &'a str, color: &str) -> ColoredString {
    match color {
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

// ─── Rendering ───────────────────────────────────────────────────────────

struct Renderer<'a> {
    config: &'a Config,
    data: &'a SysData,
    ascii_lines: Vec<String>,
}

impl<'a> Renderer<'a> {
    fn new(config: &'a Config, data: &'a SysData) -> Self {
        let ascii_lines = if config.ascii.enabled {
            load_ascii_art(&config.ascii)
        } else {
            vec![]
        };
        Self { config, data, ascii_lines }
    }

    /// Build all info lines (strings, pre-colored) that appear on the right
    fn build_info_lines(&self) -> Vec<String> {
        let lang = self.config.language;
        let theme = &self.config.theme;
        let mut lines: Vec<String> = Vec::new();

        let username = whoami_username();
        let hostname = whoami_hostname();
        let header = format!("{}@{}", username, hostname);
        let separator = "─".repeat(header.len());

        lines.push(format!(
            "{}",
            colorize(&header, &theme.accent).bold()
        ));
        lines.push(format!(
            "{}",
            colorize(&separator, &theme.separator)
        ));
        lines.push(String::new());

        for cat_cfg in &self.config.categories {
            if !cat_cfg.enabled {
                continue;
            }
            let cat = cat_cfg.category;
            let cat_name = localize_category(cat, lang);
            let icon = if self.config.show_icons {
                category_icon(cat)
            } else {
                ""
            };

            lines.push(format!(
                "{}{}",
                colorize(icon, &theme.primary),
                colorize(cat_name, &theme.primary).bold()
            ));

            let fields: &[InfoField] = if cat_cfg.fields.is_empty() {
                default_fields(cat)
            } else {
                &cat_cfg.fields
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

        // Custom fields
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
            // No ascii - just print info lines directly
            println!();
            for line in &info_lines {
                println!(" {}", line);
            }
            println!();
            return;
        }

        // Calculate actual ASCII width from the loaded art
        let actual_ascii_width = self.ascii_lines.iter()
            .map(|l| l.width())
            .max()
            .unwrap_or(0);
        
        let ascii_w = actual_ascii_width + 2; // +2 for padding
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

// ─── Shell execution (_ONLY_ blyat for custom_fields) ────────────────────

fn shell_exec(cmd: &str) -> Option<String> {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

// ─── whoami without crate (direct env / proc reads) ──────────────────────

fn whoami_username() -> String {
    env::var("USER")
        .or_else(|_| env::var("LOGNAME"))
        .unwrap_or_else(|_| {
            fs::read_to_string("/proc/self/loginuid")
                .ok()
                .unwrap_or_else(|| "user".to_string())
                .trim()
                .to_string()
        })
}

fn whoami_hostname() -> String {
    fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "localhost".to_string())
}

// ─── Config loading ──────────────────────────────────────────────────────

fn load_config() -> Config {
    let paths: Vec<String> = vec![
        env::var("HOME")
            .map(|h| format!("{}/.config/rushfetch/config.toml", h))
            .unwrap_or_default(),
        "/etc/rushfetch/config.toml".to_string(),
    ];

    for path in &paths {
        if path.is_empty() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(path) {
            match toml::from_str::<Config>(&content) {
                Ok(cfg) => return cfg,
                Err(e) => {
                    eprintln!("rushfetch: config parse error in {}: {}", path, e);
                }
            }
        }
    }

    Config::default()
}

// ─── Entry point ─────────────────────────────────────────────────────────

fn main() {
    let config = load_config();
    let data = SysData::collect();
    let renderer = Renderer::new(&config, &data);
    renderer.render();
}
