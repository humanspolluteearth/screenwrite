# Dump Your Thoughts (Focus-Write)

A frameless, GPU-accelerated minimalist writing environment for Arch Linux, inspired by the [Monkeytype](https://monkeytype.com) aesthetic. It is designed to remove all distractions, leaving only you and your thoughts.

---

## Philosophy

Most writing apps are cluttered with toolbars, sidebars, and status indicators. **Dump Your Thoughts** is built on the principle of *Dynamic Focus*:
- **Typewriter Fade**: Only your active line is fully visible. Surrounding lines fade into the background as they move further from your cursor.
- **Centered Workspace**: In Focused mode, your active line remains vertically centered, creating a consistent focal point.
- **Retro Feedback**: Optional mechanical keyboard audio feedback to provide tactile-like confirmation of your progress.

---

## Features

- **Minimalist UI**: No window borders, no buttons, just text.
- **Dynamic Opacity**: Active line is 100% opaque. Previous/next lines fade using a configurable decay formula.
- **Multiple View Modes**: 
  - `Focused`: Typewriter-style scrolling with line fading.
  - `Full`: Standard reading mode with full visibility.
- **Command Overlay**: A Vim-like command system triggered by `:` or `Ctrl+;`.
- **Theming**: Built-in support for Default (Dark), Sepia, E-Ink (High Contrast), Night, and Amoled themes.
- **PDF Export**: Export your writings to professional PDF documents directly from the app.
- **Session Stats**: View words written, elapsed time, and WPM (Words Per Minute) upon quitting.
- **Customizable**: Full control over font size, line height, column width, and decay rates via YAML.

---

## Hotkeys

| Key | Action |
|-----|--------|
| `:` or `Ctrl+;` | Open Command Overlay |
| `Ctrl+S` | Quick save to current file |
| `Ctrl+Shift+S` | Save As... |
| `Ctrl+O` | Open file dialog |
| `Ctrl+Z` / `Ctrl+Y` | Undo / Redo |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Copy / Cut / Paste |
| `Ctrl+A` | Select All |
| `Ctrl+T` | Cycle through Themes |
| `Ctrl+F` | Cycle through installed Fonts |
| `Ctrl+` + `+/-` | Increase/Decrease Font Size |
| `Tab` | Insert 4 spaces |
| `Esc` | Close Command Overlay / Clear status message |

---

## Commands

Enter these into the command overlay (`:`):

| Command | Action |
|---------|--------|
| `:w` | Save to current file (defaults to `~/focus.txt`) |
| `:w <path>` | Save to a specific path |
| `:e <path>` | Open a specific file |
| `:all` | Toggle between Focused and Full view modes |
| `:pdf` | Export current buffer to PDF |
| `:theme <name>` | Switch theme (default, sepia, eink, night, amoled) |
| `:font <name>` | Set font name (e.g., `:font JetBrains Mono`) |
| `:size <num>` | Set font size |
| `:lh <num>` | Set line height multiplier (e.g., `1.8`) |
| `:width <num>` | Set max column width in pixels |
| `:decay <0-1>` | Set line fade decay rate (higher = faster fade) |
| `:reset` | Reset all settings to default |
| `:q` / `:quit` | Show session summary and prepare to quit |

---

## Installation

### Dependencies

On Arch Linux:
```bash
sudo pacman -S fontconfig libxkbcommon vulkan-icd-loader ttf-jetbrains-mono
```

### From Source

1. Clone the repository:
   ```bash
   git clone git@github.com:humaneediedofanxiety/dump-your-thoughts.git
   cd dump-your-thoughts
   ```
2. Build and run:
   ```bash
   cargo run --release
   ```

### Arch Linux (PKGBUILD)

```bash
makepkg -si
```

---

## Configuration

The configuration is stored in `~/.config/focus-write/config.yaml`.

```yaml
theme: Default
font_size: 22.0
line_height: 1.8
max_width: 800.0
decay_rate: 0.35
min_opacity: 0.0
font_name: null
default_save_path: "/home/user/focus.txt"
```
