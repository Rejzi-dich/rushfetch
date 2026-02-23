use crate::utils::{read_file_cow, read_lines_from};
use std::fs;
use std::path::Path;
use std::net::TcpStream;
use std::io::{Read, Write};

fn read_cpu_model_from_proc() -> Option<String> {
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if let Some(model_start) = line.find("model name") {
                if let Some(colon_pos) = line[model_start..].find(':') {
                    let model_value = &line[model_start + colon_pos + 1..];
                    let trimmed = model_value.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.split_whitespace().collect::<Vec<_>>().join(" "));
                    }
                }
            }
        }
        
        // для армовских
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

pub fn read_os_pretty_name() -> Option<String> {
    let os_release = read_file_cow("/etc/os-release")?;
    
    for line in os_release.lines() {
        if line.starts_with("PRETTY_NAME=") {
            return Some(
                line["PRETTY_NAME=".len()..]
                    .trim_matches('"')
                    .to_string()
            );
        }
    }

    None
}

pub fn read_kernel_version() -> Option<String> {
    fs::read_to_string("/proc/version")
        .ok().and_then(|s| {
            s.strip_prefix("Linux version ")
                .and_then(|after| after.split_whitespace().next())
                .map(|v| v.to_string())
        })
}

pub fn get_os_release_id() -> Option<String> {
    let os_release = read_file_cow("/etc/os-release")?;
    
    for line in os_release.lines() {
        if line.starts_with("ID=") {
            return Some(
                line["ID=".len()..]
                    .trim_matches('"')
                    .to_lowercase()
            );
        }
    }

    None
}

pub fn read_hostname() -> Option<String> {
    fs::read_to_string("/proc/sys/kernel/hostname")
        .ok().map(|s| s.trim().to_string())
}

pub fn read_cpu_model() -> Option<String> {
    read_cpu_model_from_proc()
}

fn detect_gpu_drm() -> Option<String> {
    let drm = Path::new("/sys/class/drm");
    if !drm.exists() { return None; }

    if let Ok(entries) = fs::read_dir(drm) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let s = name.to_string_lossy();

            if s.starts_with("card") && !s.contains('-') {
                // Сначала пробуем быстрый способ
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
  
                let uevent_path = entry.path().join("device/uevent");
                if let Ok(ue) = fs::read_to_string(&uevent_path) {
                    if ue.contains("PCI_ID=") {
                        return Some("(detected GPU)".to_string());
                    }
                }
                
                // Если ничего не нашли - это не ГПУ, продолжаем пооиск
                continue;
            }
        }
    }

    None
}

fn detect_gpu_pci() -> Option<String> {
    // фаллбак на pci шину
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

pub fn detect_gpu() -> Option<String> {
    detect_gpu_drm().or_else(detect_gpu_pci)
}

pub fn read_meminfo() -> (u64, u64, u64, u64) {
    let (mut total, mut available, mut swap_total, mut swap_free) = 
        (0u64, 0u64, 0u64, 0u64);

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

    let used          = total.saturating_sub(available)      / 1024;
    let total_mb      = total                                / 1024;
    let swap_used     = swap_total.saturating_sub(swap_free) / 1024;
    let swap_total_mb = swap_total                           / 1024;

    (used, total_mb, swap_used, swap_total_mb)
}

pub fn read_disk_root() -> (u64, u64) {
    use std::mem::MaybeUninit;
    let path = std::ffi::CString::new("/").unwrap();
    let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
    
    unsafe {
        if libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) == 0 {
            let s = stat.assume_init();
            let total = s.f_blocks * s.f_frsize / (1024 * 1024 * 1024);
            let free  = s.f_bfree * s.f_frsize / (1024 * 1024 * 1024);
            (total.saturating_sub(free), total)
        } else { (0, 0) }
    }
}

pub fn read_uptime() -> u64 {
    fs::read_to_string("/proc/uptime")
        .ok().and_then(|s| {
            s.split_whitespace().next()
                .and_then(|v| v.parse::<f64>().ok())
        })
        .map(|f| f as u64).unwrap_or(0)
}

pub fn format_uptime(secs: u64) -> String {
    let days  =  secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins  = (secs % 3600 ) / 60;

    match days {
        0 => format!("{}h {}m", hours, mins),
        _ => format!("{}d {}h {}m", days, hours, mins),
    }
}

pub fn read_local_ip() -> Option<String> {
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

pub fn fetch_public_ip() -> Option<String> {
    let mut stream = TcpStream::connect("ifconfig.me:80").ok()?;
    stream
        .write_all(b"GET /ip HTTP/1.0\r\nHost: ifconfig.me\r\nUser-Agent: rushfetch\r\n\r\n")
        .ok()?;
    let mut body = String::new();
    stream.read_to_string(&mut body).ok()?;
    body.split("\r\n\r\n").nth(1).map(|s| s.trim().to_string())
}
