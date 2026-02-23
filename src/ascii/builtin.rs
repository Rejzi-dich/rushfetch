use crate::data::collectors::get_os_release_id;
use crate::utils::unicode_str_width;

pub fn builtin_ascii(_width: usize, distro_override: Option<&str>) -> Vec<String> {
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
            "                   ",
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
        // TODO: больше артов для: EOS, омарчи, кака ос, фрибдсм, бубунту, федор а
    };

    let max_width = art.iter()
        .map(|l| unicode_str_width(l))
        .max().unwrap_or(0);

    art.iter().map(|l| {
        let s = l.to_string();
        if unicode_str_width(&s) < max_width {
            let padding = max_width - unicode_str_width(&s);
            format!("{}{}", s, " ".repeat(padding))
        } else { s.clone() }
    }).collect()
}
