/*
 | Project: Rush fetch (rushfetch)
 | Description: blazing fast tool for outputting information on Linux systems on blazing rast
 | Authors: Rejzi-dich
 |
 | SPDX-License-Identifier: GPL-3.0-or-later
 | Website: None
 | Copyright: Rejzi-dich
 */

use colored::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

// функция для подсчёта ширины символов
fn unicode_str_width(s: &str) -> usize {
    s.chars().map(|c| {
        let code = c as u32;
        // ASCII ширина 1 символ
        if code <= 0x7F { 1 }
        // китайские/корейские/японские иероглифы и другие жирные символы - 2 ширины  
        else if 
            (0x1100..=0x115F).contains(&code) ||
            (0x2E80..=0x2EFF).contains(&code) ||
            (0x3000..=0x30FF).contains(&code) ||
            (0x4E00..=0x9FFF).contains(&code) ||
            (0xAC00..=0xD7AF).contains(&code) ||
            (0xF900..=0xFAFF).contains(&code) ||
            (0xFF00..=0xFFEF).contains(&code) { 2 }
        // Эмодзи - 2 ширины - блять, кто использует эмодзи в артах терминала. это же ужасно выглядит. перестанте
        // поэтому я запрещаю вам использовать их в терминале
        else if (0x1F000..=0x1FAFF).contains(&code) { 222 }
        // Остальное - 1 ширина
        else { 1 }
    }).sum()
}

// Конфигурационные структуры
// Поля информации
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InfoField {
    Kernel,     Os,     Arch, Host,
    Memory,     Swap,   Disk,
    Terminal,   Shell,  De,
    PublicIp,   LocalIp,
    Cpu,        Gpu,
    Uptime,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BuiltinCategory {
    System, Hardware,
    Res, Env, Net,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CategoryConfig {
    pub category: BuiltinCategory,

    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub fields: Vec<InfoField>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CustomField {
    pub label:   String,
    pub command: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AsciiConfig {
    pub file: Option<String>,
    // указывает встренный асции какого дистриубтива отображать. если не указть будет опеределять сам
    pub distro: Option<String>,
    #[serde(default = "default_true")]          pub enabled: bool,
    #[serde(default = "default_ascii_width")]   pub width:   usize,
    #[serde(default = "default_accent_color")]  pub color:   String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Theme {
    pub primary:    String,
    pub secondary:  String,
    pub accent:     String,
    pub text:       String,
    pub separator:  String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    English, Russian,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    #[serde(default)] pub language: Language,
    #[serde(default)] pub theme:    Theme,
    #[serde(default)] pub ascii:    AsciiConfig,
    #[serde(default)] pub custom_fields: Vec<CustomField>,
    #[serde(default = "default_true")]       pub show_icons: bool,
    #[serde(default = "default_categories")] pub categories: Vec<CategoryConfig>,
}

fn default_true()         -> bool   { true }
fn default_ascii_width()  -> usize  { 20 }
fn default_accent_color() -> String { "bright_cyan".to_string() }

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary:    "bright_yellow".to_string(),
            secondary:  "bright_cyan".to_string(),
            accent:     "bright_magenta".to_string(),
            text:       "bright_white".to_string(),
            separator:  "bright_black".to_string(),
        }
    }
}

impl Default for AsciiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            file:   None,
            distro: None,
            width:  20,
            color:  "bright_cyan".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language:       Language::default(),
            theme:          Theme::default(),
            ascii:          AsciiConfig::default(),
            show_icons:     true,
            categories:     default_categories(),
            custom_fields:  vec![],
        }
    }
}

fn default_categories() -> Vec<CategoryConfig> {
    vec![
        CategoryConfig {
            category:   BuiltinCategory::System,
            enabled:    true,
            fields:     vec![],
        },
        CategoryConfig {
            category:   BuiltinCategory::Hardware,
            enabled:    true,
            fields:     vec![],
        },
        CategoryConfig {
            category:   BuiltinCategory::Res,
            enabled:    true,
            fields:     vec![],
        },
        CategoryConfig {
            category:   BuiltinCategory::Env,
            enabled:    true,
            fields:     vec![],
        },
        CategoryConfig {
            category:   BuiltinCategory::Net,
            enabled:    false,
            fields:     vec![],
        },
    ]
}

fn default_fields(category: BuiltinCategory) -> &'static [InfoField] {
    match category {
        BuiltinCategory::System => &[
            InfoField::Os,      InfoField::Kernel,  InfoField::Arch
        ],
        BuiltinCategory::Hardware => &[
            InfoField::Host,    InfoField::Cpu,     InfoField::Gpu
        ],
        BuiltinCategory::Res => &[
            InfoField::Memory,  InfoField::Swap,    InfoField::Disk
        ],
        BuiltinCategory::Env => &[
            InfoField::Uptime,  InfoField::Shell,
            InfoField::Terminal,InfoField::De,
        ],
        BuiltinCategory::Net => &[
            InfoField::LocalIp, InfoField::PublicIp
        ],
    }
}

fn localize_category(category: BuiltinCategory, lang: Language) -> &'static str {
    match lang {
        Language::English => match category {
            BuiltinCategory::System     => "System",
            BuiltinCategory::Hardware   => "Hardware",
            BuiltinCategory::Res        => "Resources",
            BuiltinCategory::Env        => "Environment",
            BuiltinCategory::Net        => "Network",
        },
        Language::Russian => match category {
            BuiltinCategory::System     => "Система",
            BuiltinCategory::Hardware   => "Железо",
            BuiltinCategory::Res        => "Ресурсы",
            BuiltinCategory::Env        => "Окружение",
            BuiltinCategory::Net        => "Сеть",
        },
    }
}

fn localize_field(field: InfoField, lang: Language) -> &'static str {
    match lang {
        Language::English => match field {
            InfoField::Os           => "OS",
            InfoField::Kernel       => "Kernel",
            InfoField::Arch         => "Arch",
            InfoField::Host         => "Host",
            InfoField::Cpu          => "CPU",
            InfoField::Gpu          => "GPU",
            InfoField::Memory       => "RAM",
            InfoField::Swap         => "Swap",
            InfoField::Disk         => "Disk",
            InfoField::Uptime       => "Uptime",
            InfoField::Shell        => "Shell",
            InfoField::Terminal     => "Terminal",
            InfoField::De           => "DE / WM",
            InfoField::LocalIp      => "Local IP",
            InfoField::PublicIp     => "Public IP",
        },
        Language::Russian => match field {
            InfoField::Os           => "ОС",
            InfoField::Kernel       => "Ядро",
            InfoField::Arch         => "Архитектура",
            InfoField::Host         => "Имя ПК",
            InfoField::Cpu          => "Проц",
            InfoField::Gpu          => "Гпу",
            InfoField::Memory       => "Память",
            InfoField::Swap         => "Своп",
            InfoField::Disk         => "Диск",
            InfoField::Uptime       => "Аптайм",
            InfoField::Shell        => "Шелл",
            InfoField::Terminal     => "Терминал",
            InfoField::De           => "ДЕ / ВМ",
            InfoField::LocalIp      => "Локал IP",
            InfoField::PublicIp     => "Внешний IP",
        },
    }
}

fn category_icon(category: BuiltinCategory) -> &'static str {
    match category {
        // тут нордовские иконки, в некоторых местах это может выглядеть как квадратики.
        BuiltinCategory::System     => "󰍛 ", 
        BuiltinCategory::Hardware   => "󰘚 ",
        BuiltinCategory::Res        => "󰓅 ",
        BuiltinCategory::Env        => "󰆍 ",
        BuiltinCategory::Net        => "󰀂 ",
    }
}

pub struct SysData {
    pub os:                 Option<String>,
    pub kernel:             Option<String>,
    pub arch:             &'static str,
    pub host:               Option<String>,
    pub cpu:                Option<String>,
    pub gpu:                Option<String>,
    pub memory_used_mb:     u64,
    pub memory_total_mb:    u64,
    pub swap_used_mb:       u64,
    pub swap_total_mb:      u64,
    pub disk_used_gb:       u64,
    pub disk_total_gb:      u64,
    pub uptime_secs:        u64,
    pub shell:              Option<String>,
    pub terminal:           Option<String>,
    pub de:                 Option<String>,
    pub local_ip:           Option<String>,
}

// для медленных операций
static PUBLIC_IP_CACHE: OnceLock<Option<String>> = OnceLock::new();
static GPU_CACHE: OnceLock<Option<String>> = OnceLock::new();

// Глобальный кэш для статических данных (которые редко меняются)
struct CachedData {
    os_pretty_name: Option<String>,
    hostname: Option<String>,
    cpu_model: Option<String>,
    kernel_version: Option<String>,
    os_release_id: Option<String>,
}

static CACHED_DATA: OnceLock<CachedData> = OnceLock::new();

// TTL кэш для динамических данных (память, диск)
struct TtlCache<T> {
    data: T,
    timestamp: Instant,
    ttl: Duration,
}

impl<T> TtlCache<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            timestamp: Instant::now(),
            ttl,
        }
    }
    
    fn get(&self) -> Option<&T> {
        if self.timestamp.elapsed() < self.ttl {
            Some(&self.data)
        } else { None }
    }
    
    fn is_valid(&self) -> bool {
        self.timestamp.elapsed() < self.ttl
    }
}

static MEMORY_CACHE: OnceLock<TtlCache<(u64, u64, u64, u64)>> = OnceLock::new();
static DISK_CACHE:   OnceLock<TtlCache<(u64, u64)>>           = OnceLock::new();

impl SysData {
    pub fn collect() -> Self {
        // Запускаем тяжёлые операции параллельно
        let memory_handle   = thread::spawn(read_meminfo);
        let disk_handle     = thread::spawn(read_disk_root);
        let uptime_handle   = thread::spawn(read_uptime);

        // Быстрые операции выполняем последовательно
        let terminal    = env::var("TERM").ok();
        let shell       = shell_name();
        let arch        = std::env::consts::ARCH;
        let kernel      = read_kernel_version();
        let host        = read_hostname();
        let cpu         = read_cpu_model();
        let os          = read_os_pretty_name();
        let local_ip    = read_local_ip();

        // получаем результаты из параллльных потоков
        let (memory_used_mb, memory_total_mb, 
             swap_used_mb, swap_total_mb) = memory_handle.join().unwrap();
        let (disk_used_gb, disk_total_gb) = disk_handle.join().unwrap();

        let uptime_secs = uptime_handle.join().unwrap();

        Self {
            uptime_secs, terminal,
            kernel, shell,  cpu,
            arch,   host,   os,
            local_ip,
            memory_used_mb,
            memory_total_mb,
            swap_used_mb,
            swap_total_mb,
            disk_used_gb,
            disk_total_gb,
            gpu: None, // Будет загружено лениво
            de: env::var("XDG_CURRENT_DESKTOP")
                .or_else(|_| env::var("DESKTOP_SESSION")).ok(),
        }
    }

    pub fn get(&self, field: InfoField) -> Option<String> {
        match field {
            InfoField::Os       => self.os.clone(),
            InfoField::Kernel   => self.kernel.clone(),
            InfoField::Arch     => Some(self.arch.to_string()),
            InfoField::Host     => self.host.clone(),
            InfoField::Cpu      => self.cpu.clone(),
            InfoField::Gpu      => {
                // Ленивая загрузка - только когда надо
                GPU_CACHE.get_or_init(|| detect_gpu()).clone()
            },
            InfoField::Memory   => Some(format!(
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
            InfoField::Uptime   => Some(format_uptime(self.uptime_secs)),
            InfoField::Shell    => self.shell.clone(),
            InfoField::Terminal => self.terminal.clone(),
            InfoField::De       => self.de.clone(),
            InfoField::LocalIp  => self.local_ip.clone(),
            InfoField::PublicIp => {
                // Ленивая загрузка айпи
                PUBLIC_IP_CACHE.get_or_init(|| fetch_public_ip()).clone()
            },
        }
    }
}

// Zero-copy оптимизация для чтения файлов
fn read_file_cow(path: &str) -> Option<Cow<'static, str>> {
    fs::read_to_string(path)
        .ok()
        .map(|s| Cow::Owned(s))
}

fn read_lines_from(path: &str) -> Option<Vec<String>> {
    let file    = fs::File::open(path).ok()?;
    let reader  = io::BufReader::new(file);
    Some(reader.lines().filter_map(|l| l.ok()).collect())
}

// Инициализация глобального кэша
fn init_cached_data() -> &'static CachedData {
    CACHED_DATA.get_or_init(|| {
        let os_release = read_file_cow("/etc/os-release").unwrap_or_default();
        
        // Парсим OS release
        let mut os_pretty_name = None;
        let mut os_release_id  = None;
        
        for line in os_release.lines() {
            if line.starts_with("PRETTY_NAME=") {
                os_pretty_name = Some(
                    line["PRETTY_NAME=".len()..]
                        .trim_matches('"')
                        .to_string()
                );
            } else if line.starts_with("ID=") {
                os_release_id = Some(
                    line["ID=".len()..]
                        .trim_matches('"')
                        .to_lowercase()
                );
            }
        }
        
        // Читаем остальные данные
        let hostname = fs::read_to_string("/proc/sys/kernel/hostname")
            .ok().map(|s| s.trim().to_string());
            
        let kernel_version = fs::read_to_string("/proc/version")
            .ok().and_then(|s| {
                s.strip_prefix("Linux version ")
                    .and_then(|after| after.split_whitespace().next())
                    .map(|v| v.to_string())
            });
            
        let cpu_model = read_cpu_model_from_proc();
        
        CachedData {
            os_pretty_name,
            hostname,
            cpu_model,
            kernel_version,
            os_release_id,
        }
    })
}

// Zero-copy оптимизированное чтение цпу
fn read_cpu_model_from_proc() -> Option<String> {
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if let Some(model_start) = line.find("model name") {
                if let Some(colon_pos) = line[model_start..].find(':') {
                    let model_value = &line[model_start + colon_pos + 1..];
                    let trimmed = model_value.trim();
                    if !trimmed.is_empty() {
                        let mut result = String::with_capacity(trimmed.len());
                        let mut in_word = false;
                        
                        for ch in trimmed.chars() {
                            if ch.is_whitespace() {
                                if in_word {
                                    result.push(' ');
                                    in_word = false;
                                }
                            } else {
                                result.push(ch);
                                in_word = true;
                            }
                        }
                        return Some(result.trim().to_string());
                    }
                }
            }
        }
        
        // Fallback для ARMов
        for line in content.lines() {
            if let Some(hardware_start) = line.find("Hardware") {
                if let Some(colon_pos) = line[hardware_start..].find(':') {
                    let hardware_value = &line[hardware_start + colon_pos + 1..];
                    let trimmed = hardware_value.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
    }
    None
}

fn read_os_pretty_name() -> Option<String> { Some(init_cached_data().os_pretty_name.clone()?) }
fn read_kernel_version() -> Option<String> { Some(init_cached_data().kernel_version.clone()?) }
fn get_os_release_id()   -> Option<String> { Some(init_cached_data().os_release_id.clone()?) }
fn read_cpu_model()      -> Option<String> { Some(init_cached_data().cpu_model.clone()?) }
fn read_hostname()       -> Option<String> { Some(init_cached_data().hostname.clone()?) }

fn detect_gpu_drm() -> Option<String> {
    // пробуем открыть drm
    let drm = Path::new("/sys/class/drm");
    if drm.exists() {
        if let Ok(entries) = fs::read_dir(drm) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let s = name.to_string_lossy();
                if s.starts_with("card") && !s.contains('-') {
                    let vendor_path = entry.path().join("device/vendor");
                    let model_path  = entry.path().join("device/product");
                    if let (Ok(vendor), Ok(model)) = (
                        fs::read_to_string(&vendor_path),
                        fs::read_to_string(&model_path),
                    ) {
                        let (v, m) = (vendor.trim(), model.trim());

                        if !m.is_empty() {
                            return Some(format!("{} {}", v, m));
                        }
                    }
                    let uevent_path = entry.path().join("device/uevent");
                    if let Ok(ue) = fs::read_to_string(&uevent_path) {
                        for l in ue.lines() {
                            if l.starts_with("PCI_ID=") {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
fn detect_gpu_pci() -> Option<String> {
    // Fallback на pci шину
    if let Some(lines) = read_lines_from("/proc/bus/pci/devices") {
        for line in &lines {
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() >= 2 {
                let class_vendor = cols[1];
                if class_vendor.starts_with("0300") || class_vendor.starts_with("0302") {
                    return Some("(detected, install lspci for details)".to_string());
                }
            }
        }
    }

    None
}

fn detect_gpu() -> Option<String> {
    detect_gpu_drm().or_else(detect_gpu_pci)
}

fn read_meminfo() -> (u64, u64, u64, u64) {
    // Проверяем ttl кэш
    if let Some(cached) = MEMORY_CACHE.get() {
        if cached.is_valid() {
            return *cached.get().unwrap()
        }
    }

    let (mut total,      mut available, 
         mut swap_total, mut swap_free) =
        (0u64, 0u64, 0u64, 0u64);

    // Zero-copy чтение файла
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            let mut parts = line.split_ascii_whitespace();
            match parts.next() {
                Some("MemTotal:")       => total        = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0),
                Some("MemAvailable:")   => available    = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0),
                Some("SwapTotal:")      => swap_total   = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0),
                Some("SwapFree:")       => swap_free    = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0),
                _ => {}
            }
        }
    }

    let used            = total.saturating_sub(available) / 1024;
    let total_mb        = total / 1024;
    let swap_used       = swap_total.saturating_sub(swap_free) / 1024;
    let swap_total_mb   = swap_total / 1024;

    let result = (used, total_mb, swap_used, swap_total_mb);

    // Кэшируем результат
    MEMORY_CACHE.set(TtlCache::new(result, Duration::from_secs(120))).ok();

    result
}

fn read_disk_root() -> (u64, u64) {
    // Проверяем TTL кэш (5 секунд для диска)
    if let Some(cached) = DISK_CACHE.get() {
        if cached.is_valid() {
            return *cached.get().unwrap();
        }
    }

    use std::mem::MaybeUninit;
    let path = std::ffi::CString::new("/").unwrap();
    let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
    let result = unsafe {
        if libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) == 0 {
            let s = stat.assume_init();
            let total = s.f_blocks * s.f_frsize / (1024 * 1024 * 1024);
            let free  = s.f_bfree * s.f_frsize /  (1024 * 1024 * 1024);
            (total.saturating_sub(free), total)
        } else { (0, 0) }
    };

    // Кэшируем результат на 5 секунд
    DISK_CACHE.set(TtlCache::new(result, Duration::from_secs(300))).ok();
    result
}

fn read_uptime() -> u64 {
    fs::read_to_string("/proc/uptime")
        .ok().and_then(|s| {
            s.split_whitespace().next()
                .and_then(|v| v.parse::<f64>().ok())
        })
        .map(|f| f as u64).unwrap_or(0)
}

fn format_uptime(secs: u64) -> String {
    let days  =  secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins  = (secs % 3600 ) / 60;
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

    local_ip_for_iface(&iface)
}

fn local_ip_for_iface(iface: &str) -> Option<String> {
    use std::net::Ipv4Addr;

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
                    let sin     = a.ifa_addr as *const libc::sockaddr_in;
                    let ip_u32  = u32::from_be((*sin).sin_addr.s_addr);

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

fn builtin_ascii(_width: usize, distro_override: Option<&str>) -> Vec<String> {
    let id = if let Some(distro) = distro_override {
        distro.to_lowercase()
    } else {
        get_os_release_id().unwrap_or_default()
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
        "arch-mini" => &[
            "                  ",
            "        /\\        ",
            "       /  \\       ",
            "      /    \\      ",
            "     _\\     \\     ",
            "    /        \\    ",
            "   /          \\   ",
            "  /     __   \\_\\  ",
            " /     /  \\     \\ ",
            "/__,--'    '--,__\\",
            "                  ",
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

    // Находим максимальную ширину в ASCII art
    let max_width = art.iter()
        .map(|l| unicode_str_width(l))
        .max()
        .unwrap_or(0);

    art.iter().map(|l| {
        let s = l.to_string();
        // дополняем до максимальной ширины
        if unicode_str_width(&s) < max_width {
            let padding = max_width - unicode_str_width(&s);
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
            // Находим максимальную ширину в ASCII art
            let max_width = lines.iter()
                .map(|l| unicode_str_width(l))
                .max()
                .unwrap_or(0);
            
            // Используем большую из ширин: конфига или реальной
            let effective_width = cfg.width.max(max_width);
            
            return lines.iter().map(|l| {
                let s = l.to_string();
                // дополняем до эффективной ширины
                if unicode_str_width(&s) < effective_width {
                    let padding = effective_width - unicode_str_width(&s);
                    format!("{}{}", s, " ".repeat(padding))
                } else {
                    // Обрезаем по ширине символа
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
    builtin_ascii(cfg.width, cfg.distro.as_deref())
}

fn colorize<'a>(text: &'a str, color: &str) -> ColoredString {
    match color {
        "black"             => text.black(),
        "red"               => text.red(),
        "green"             => text.green(),
        "yellow"            => text.yellow(),
        "blue"              => text.blue(),
        "magenta"           => text.magenta(),
        "cyan"              => text.cyan(),
        "white"             => text.white(),
        "bright_black"      => text.bright_black(),
        "bright_red"        => text.bright_red(),
        "bright_green"      => text.bright_green(),
        "bright_yellow"     => text.bright_yellow(),
        "bright_blue"       => text.bright_blue(),
        "bright_magenta"    => text.bright_magenta(),
        "bright_cyan"       => text.bright_cyan(),
        "bright_white"      => text.bright_white(),
        _ => text.white(),
    }
}

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

        for category_cfg in &self.config.categories {
            if !category_cfg.enabled {
                continue;
            }
            let category = category_cfg.category;
            let category_name = localize_category(category, lang);
            let icon = if self.config.show_icons {
                category_icon(category)
            } else {
                ""
            };

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
            .max()
            .unwrap_or(0);
        
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
                Err(e)  => {
                    eprintln!("rushfetch: config parse error in {}: {}", path, e);
                }
            }
        }
    }

    Config::default()
}

fn main() {
    let config   = load_config();
    let data     = SysData::collect();
    let renderer = Renderer::new(&config, &data);

    renderer.render();
}
