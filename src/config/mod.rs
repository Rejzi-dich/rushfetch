pub mod defaults;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InfoField {
    Kernel,    Os,       Arch, Host,
    Memory,    Swap,     Disk,
    Terminal,  Shell,    De,
    PublicIp,  LocalIp,
    Cpu,       Gpu,
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

    #[serde(default = "defaults::default_true")]
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
    pub distro: Option<String>,

    #[serde(default = "defaults::default_true")]          pub enabled: bool,
    #[serde(default = "defaults::default_ascii_width")]   pub width:   usize,
    #[serde(default = "defaults::default_accent_color")]  pub color:   String,
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
    // TODO: добавить "йазыг падонкафф"
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    #[serde(default)] pub language: Language,
    #[serde(default)] pub theme:    Theme,
    #[serde(default)] pub ascii:    AsciiConfig,
    #[serde(default)] pub custom_fields: Vec<CustomField>,
    #[serde(default = "defaults::default_true")]       pub show_icons: bool,
    #[serde(default = "defaults::default_categories")] pub categories: Vec<CategoryConfig>,
}

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
            categories:     defaults::default_categories(),
            custom_fields:  vec![],
        }
    }
}

pub fn load_config() -> Config {
    use std::env;
    use std::fs;

    let paths: Vec<String> = vec![
        env::var("HOME")
            .map(|h| format!("{}/.config/rushfetch/config.toml", h))
            .unwrap_or_default(),
        "/etc/rushfetch/config.toml".to_string(),
    ];

    for path in &paths {
        if path.is_empty() { continue; }

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
