# ⚡ rushfetch

> A fast system information utility written in Rust. Rush in like a hurricane, grab the info!

**rushfetch** is a modern utility for displaying your system's characteristics in a clean, customizable format. Written in Rust for maximum performance.

## Features

- **BLAZING fast** - Written in Rust, compiles to native code
- **Customizable themes** - Full color control with ready presets (Gruvbox, Dracula, Nord, etc.)
- **Multiple languages** - Support for English and Russian
- **Modular categories** - Enable/disable entire categories or individual fields
- **Custom fields** - Add your own commands and show what you want
- **Minimal dependencies** - Only the essentials
- **Self-documenting config** - No need to read boring docs

## Screenshots
![plot](./screens/rushfetch_1.png)
```

                                           rejzi@mac
                   -`                      ─────────
                  .o+`                     
                 `oooo.                    󰍛 System
                `+oooo:                      OS ........... EndeavourOS
               `+oooooo:                     Kernel ....... 6.18.7-zen1-1-zen
               -+oooooo+:                    Arch ......... x86_64
             `/:-:++oooo+:                 
            `/++++/+++++++:                󰘚 Hardware
           `/++++++++++++++:                 Host ......... mac
          `/+++rustooooooooo\`               CPU .......... Intel(R) Core(TM) i5-2415M CPU @ 2.30GHz
         ./ooosssso++osssssso+`            
        .oossssso-````/ossssss+`           󰓅 Resources
       -osssssso.      :ssssssso.            RAM .......... 7143 MB / 7846 MB
      :osssssss/        osssso+++.           Swap ......... 2022 MB / 2047 MB
     /ossssssss/        +ssssooo/-           Disk ......... 126 GB / 147 GB
   `/ossssso+/:-        -:/+osssso+-       
  `+sso+:-`                 `.-/+oso:      󰆍 Environment
 `++:.                           `-/+/       Uptime ....... 18h 51m
 .`                                 `        Shell ........ fish
                                             Terminal ..... xterm-kitty
                                             DE / WM ...... KDE
                                           
                                           󰆾 Custom
                                             Packages ....... 1853
                                             Flatpaks ....... 28
                                             Snaps .......... 18
```


## Installation

### Arch-based (AUR)

```bash
yay -S rushfetch
# or
paru -S rushfetch
```

### From source

```bash
git clone https://github.com/Rejzi-dich/rushfetch.git
cd rushfetch
cargo build --release
sudo cp target/release/rushfetch /usr/bin/
sudo mkdir -p /etc/rushfetch
sudo cp config.toml /etc/rushfetch/
```

## Usage

Just run:
```bash
rushfetch
```

### First setup

Copy the default config to customize:
```bash
mkdir -p ~/.config/rushfetch
cp /etc/rushfetch/config.toml ~/.config/rushfetch/
```

Then edit `~/.config/rushfetch/config.toml` - everything is described there with examples!

## Configuration

rushfetch uses a self-documenting TOML config. Here's a snippet:

```toml
# Choose language
language = "russian"  # or "english"

# Configure colors
[theme]
primary   = "green"   # Icons and category titles
secondary = "white"   # Field names
accent    = "green"   # user@hostname
text      = "white"   # Values
separator = "white"   # separator

# Enable/disable categories
[[categories]]
label = "system"
enabled = true
subcategories = ["os", "kernel", "arch"]

# Add your own fields
[[custom_fields]]
label = "Packages"
command = "pacman -Q | wc -l"
```

See [full config example](config.toml) with all options!

## Popular Themes

Presets are available in the config:
- **Gruvbox** - Retro colors
- **Dracula** - Dark vampire theme
- **Nord** - Arctic, northern palette
- **Solarized** - Precise colors for machines and humans
- **Monokai** - Smooth and pleasant
- **Tokyo Night** - Dark theme in Tokyo night style

Just uncomment the desired theme in the config!

## Language Support

### English mode
```
OS .......... Arch Linux
CPU ......... AMD Ryzen 9
Memory ...... 16 GB / 32 GB
```

### Russian mode
```toml
language = "russian"
```
```
ОС .......... Arch Linux
Проц ........ AMD Ryzen 9
Память ...... 12 GB / 32 GB
```

## Custom Field Examples

Add whatever you want:

```toml
# Package count
[[custom_fields]]
label   = "Packages"
command = "pacman -Q | wc -l"

# Battery status
[[custom_fields]]
label   = "Battery"
command = "acpi | awk '{print $4}' | tr -d ','"

# Now playing
[[custom_fields]]
label   = "Playing"
command = "playerctl metadata --format '{{ artist }} - {{ title }}'"

# GTK theme
[[custom_fields]]
label   = "GTK theme"
command = "gsettings get org.gnome.desktop.interface gtk-theme"
```

There are more examples in config.toml!

## Architecture

rushfetch is built with clean modular architecture:

- **SystemInfo** - Hardware and system information collection
- **Localizer** - Multi-language support
- **Config** - Type-safe configuration via serde
- **Theme** - Flexible color system

Everything is written in BLAZING Rust with proper error handling.

## 🤝 Contribute, Comrade!

Contributions are welcome! You can:
- Report bugs
- Suggest features
- Make pull requests
- Improve documentation

as soon as the author sober up, he will definitely answer you!

## Plans!
- make it possible to create categories for custom fields
- add ability to add custom fields to built-in categories
- add more built-in fields
- add more built-in ascii arts for other distributions and systems
- publish on termux repository

## 📜 License

GPL-3.0 - See [LICENSE](LICENSE) for details.

## Acknowledgments

Inspired by:
- **neofetch** - The original system info tool (rest in peace)
- **fastfetch** - Fast C implementation
- **Rust** - For making this possible

## Support

- Bugs: [GitHub Issues](https://github.com/Rejzi-dich/rushfetch/issues)
- Discussions: [GitHub Discussions](https://github.com/Rejzi-dich/rushfetch/discussions)

---

**Made with ❤️ on ⚡Rust**

*Rush in fast, grab the info, look cool.*
