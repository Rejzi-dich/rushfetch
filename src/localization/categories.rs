use crate::config::{BuiltinCategory, InfoField, Language};

pub fn localize_category(category: BuiltinCategory, lang: Language) -> &'static str {
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

pub fn localize_field(field: InfoField, lang: Language) -> &'static str {
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

pub fn category_icon(category: BuiltinCategory) -> &'static str {
    match category {
        // тут нордовские иконки, в некоторых местах это может выглядеть как квадратики
        BuiltinCategory::System     => "󰍛 ", 
        BuiltinCategory::Hardware   => "󰘚 ",
        BuiltinCategory::Res        => "󰓅 ",
        BuiltinCategory::Env        => "󰆍 ",
        BuiltinCategory::Net        => "󰀂 ",
    }
}
