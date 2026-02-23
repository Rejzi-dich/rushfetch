use std::env;
use std::fs;

pub fn shell_exec(cmd: &str) -> Option<String> {
    std::process::Command::new("sh")
        .arg("-c").arg(cmd).output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn whoami_username() -> String {
    env::var("USER") // TODO: исправить потом. заметка 218 в таблице 2
        .or_else(|_| env::var("LOGNAME"))
        .unwrap_or_else(|_| {
            fs::read_to_string("/proc/self/loginuid")
                .ok().unwrap_or_else(|| "user".to_string())
                .trim().to_string()
        })
}

pub fn whoami_hostname() -> String {
    fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "localhost".to_string())
}

pub fn shell_name() -> Option<String> {
    env::var("SHELL")
        .ok().and_then(|s| s.rsplit('/')
        .next().map(|s| s.to_string()))
}
