use colored::*;

pub fn colorize<'a>(text: &'a str, color: &str) -> ColoredString {
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
