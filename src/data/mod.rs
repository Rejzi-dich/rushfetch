pub mod collectors;

use crate::config::InfoField;
use crate::utils::shell_name;
use collectors::*;

use std::env;
use std::thread;

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
            uptime_secs, terminal, local_ip,
            kernel,      shell,    cpu,
            arch,        host,     os,

            memory_used_mb, memory_total_mb,
            swap_used_mb,   swap_total_mb,
            disk_used_gb,   disk_total_gb,

            gpu: None,

            de: env::var("XDG_CURRENT_DESKTOP")
                .or_else(|_| env::var("DESKTOP_SESSION")).ok(),
        }
    }

    pub fn get(&self, field: InfoField) -> Option<String> {
        match field {
            InfoField::Os       => self.os.as_ref().cloned(),
            InfoField::Kernel   => self.kernel.as_ref().cloned(),
            InfoField::Arch     => Some(self.arch.to_string()),
            InfoField::Host     => self.host.as_ref().cloned(),
            InfoField::Cpu      => self.cpu.as_ref().cloned(),
            InfoField::Gpu      => detect_gpu(),
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
            InfoField::Shell    => self.shell.as_ref().cloned(),
            InfoField::Terminal => self.terminal.as_ref().cloned(),
            InfoField::De       => self.de.as_ref().cloned(),
            InfoField::LocalIp  => self.local_ip.as_ref().cloned(),
            InfoField::PublicIp => fetch_public_ip(),
        }
    }
}
