use crate::config::{BuiltinCategory, CategoryConfig, InfoField};

pub fn default_true()         -> bool   { true }
pub fn default_ascii_width()  -> usize  { 20 }
pub fn default_accent_color() -> String { "bright_cyan".to_string() }

pub fn default_categories() -> Vec<CategoryConfig> {
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

pub fn default_fields(category: BuiltinCategory) -> &'static [InfoField] {
    match category {
        BuiltinCategory::System => &[
            InfoField::Os,       InfoField::Kernel, InfoField::Arch
        ],
        BuiltinCategory::Hardware => &[
            InfoField::Host,     InfoField::Cpu,    InfoField::Gpu
        ],
        BuiltinCategory::Res => &[
            InfoField::Memory,   InfoField::Swap,   InfoField::Disk
        ],
        BuiltinCategory::Env => &[
            InfoField::Uptime,   InfoField::Shell,
            InfoField::Terminal, InfoField::De,
        ],
        BuiltinCategory::Net => &[
            InfoField::LocalIp,  InfoField::PublicIp
        ],
    }
}
