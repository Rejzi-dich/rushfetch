# 🚀 rushfetch

> Fast system information tool written in Rust. Rush like the wind, fetch like a champion!

**rushfetch** is a modern system information utility that displays your system specs in a clean, customizable format. Written in Rust for maximum performance and safety.

## ✨ Features

- ⚡ **Blazing Fast** - Written in Rust, compiled to native code
- 🎨 **Customizable Themes** - Full color control with popular presets (Gruvbox, Dracula, Nord, etc.)
- 🌍 **Multiple Languages** - English and Russian support
- 📦 **Modular Categories** - Enable/disable entire categories or specific subcategories
- 🛠️ **Custom Fields** - Add your own commands and display anything you want
- 🎯 **Minimal Dependencies** - Just the essentials
- 📝 **Self-Documenting Config** - No need to read boring documentation

## 📸 Screenshots

```
 user@hostname

 ◉ SYSTEM
   OS .............. Arch Linux 6.7.2
   Kernel .......... 6.7.2-arch1-1
   Arch ............ x86_64

 ◈ HARDWARE
   Host ............ ThinkPad X1 Carbon
   CPU ............. AMD Ryzen 9 5900X
   GPU ............. NVIDIA GeForce RTX 3080

 ◫ RESOURCES
   RAM ............. 8192 MB / 16384 MB
   Disk ............ 256 GB / 512 GB
```

## 🚀 Installation

### Arch Linux (AUR)

```bash
yay -S rushfetch
# or
paru -S rushfetch
```

### From Source

```bash
git clone https://github.com/yourusername/rushfetch.git
cd rushfetch
cargo build --release
sudo cp target/release/rushfetch /usr/bin/
sudo mkdir -p /etc/rushfetch
sudo cp config.toml /etc/rushfetch/
```

## 🎮 Usage

Simply run:
```bash
rushfetch
```

### First Time Setup

Copy the default config to customize:
```bash
mkdir -p ~/.config/rushfetch
cp /etc/rushfetch/config.toml ~/.config/rushfetch/
```

Then edit `~/.config/rushfetch/config.toml` - it's fully documented with examples!

## ⚙️ Configuration

rushfetch uses a self-documenting TOML config file. Here's a quick taste:

```toml
# Choose your language
language = "english"  # or "russian"

# Customize colors
[theme]
primary = "bright_yellow"    # Category icons and headers
secondary = "bright_cyan"    # Field labels
accent = "bright_magenta"    # user@hostname
text = "bright_white"        # Values

# Enable/disable categories
[[categories]]
name = "system"
enabled = true
subcategories = ["os", "kernel", "arch"]

# Add custom fields
[[custom_fields]]
name = "Packages"
command = "pacman -Q | wc -l"
```

See the [full config example](config.toml) for all options!

## 🎨 Popular Themes

The config includes presets for:
- **Gruvbox** - Retro groove colors
- **Dracula** - Dark vampire theme
- **Nord** - Arctic, north-bluish color palette
- **Solarized** - Precision colors for machines and people
- **Monokai** - Smooth and pleasant
- **Tokyo Night** - A dark theme inspired by Tokyo at night

Just uncomment the theme you want in the config!

## 🌐 Language Support

### English Mode
```
OS .............. Arch Linux
CPU ............. AMD Ryzen 9
Memory .......... 16 GB / 32 GB
```

### Russian Mode (неформальный стиль)
```toml
language = "russian"
```
```
ОСь ............. Arch Linux
Процессор ....... AMD Ryzen 9
Память .......... 16 GB / 32 GB
```

## 🛠️ Custom Fields Examples

Add anything you want to display:

```toml
# Package count
[[custom_fields]]
name = "Packages"
command = "pacman -Q | wc -l"

# Battery status
[[custom_fields]]
name = "Battery"
command = "acpi | awk '{print $4}' | tr -d ','"

# Current playing song
[[custom_fields]]
name = "Now Playing"
command = "playerctl metadata --format '{{ artist }} - {{ title }}'"

# GTK theme
[[custom_fields]]
name = "GTK Theme"
command = "gsettings get org.gnome.desktop.interface gtk-theme"
```

See config.toml for 20+ more examples!

## 🏗️ Architecture

rushfetch is built with clean, modular architecture:

- **SystemInfo** - Hardware and system information gathering
- **Localizer** - Multi-language support
- **Config** - Type-safe configuration with serde
- **Theme** - Flexible color system

All written in idiomatic Rust with proper error handling.

## 🤝 Contributing

Contributions are welcome! Feel free to:
- Report bugs
- Suggest features
- Submit pull requests
- Improve documentation

## 📜 License

GPL-3.0 - See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

Inspired by:
- **neofetch** - The OG system info tool (RIP)
- **fastfetch** - Fast C implementation
- **Rust** - For making this possible

## 📞 Support

- 🐛 Issues: [GitHub Issues](https://github.com/yourusername/rushfetch/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/yourusername/rushfetch/discussions)

---

**Made with ❤️ and Rust**

*Rush in, fetch fast, look good.*