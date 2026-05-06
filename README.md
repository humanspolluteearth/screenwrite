# Focus-Write

A frameless, GPU-accelerated minimalist writing environment for Arch Linux, inspired by the [Monkeytype](https://monkeytype.com) aesthetic.

## Features

- **Typewriter Fade**: The active line is 100% opaque. Previous/next lines fade using the formula: `Opacity = max(0.08, 1.0 - (distance × decay_rate))`
- **Focused View** (default): Active line stays at vertical center, all others fade
- **Full View**: Standard readable view with `:all` command
- **Command Overlay**: Press `:` to enter commands
- **Caret Blink**: Monkeytype-style blinking bar caret in `#e2b714`
- **Zero network overhead**: No telemetry, no update checks

---

## Hotkeys

| Key | Action |
|-----|--------|
| `:w` | Save to current file (defaults to `~/focus.txt`) |
| `:w <path>` | Save to a specific path |
| `:e <path>` / `:o <path>` | Open a file |
| `:all` | Toggle between Focused and Full view |
| `:q` / `:quit` | Quit the application |
| `Ctrl+S` | Quick save |
| `Ctrl+O` | Open the file dialog (command overlay) |
| `Esc` | Close command overlay |
| `Arrow Keys` | Navigate cursor |
| `Home` / `End` | Jump to line start/end |
| `Tab` | Insert 4 spaces |

---

## YAML Configuration

The config file lives at `~/.config/focus-write/config.yaml` and is auto-generated on first run.

```yaml
background: "#0d0d0d"
text_color: "#e0e0e0"
caret_color: "#e2b714"
font_size: 22.0
line_height: 1.8
max_width: 800.0
decay_rate: 0.15
min_opacity: 0.08
```

### Config Schema

| Key | Type | Description |
|-----|------|-------------|
| `background` | Hex color | Window background |
| `text_color` | Hex color | Base text color |
| `caret_color` | Hex color | Caret/active highlight color |
| `font_size` | Float | Font size in pixels |
| `line_height` | Float multiplier | Line spacing multiplier (e.g. `1.8`) |
| `max_width` | Float | Max text column width in pixels |
| `decay_rate` | Float (0.0–1.0) | How fast lines fade. Higher = faster fade |
| `min_opacity` | Float (0.0–1.0) | Floor opacity for the most distant lines |

---

## Building

### Requirements

- `rustup` with stable toolchain
- `libfontconfig` (for font loading)
- `libxkbcommon` (for keyboard)
- `libvulkan` or `libGL` (GPU backend)

On Arch Linux:
```bash
sudo pacman -S fontconfig libxkbcommon vulkan-icd-loader
```

### Build & Run

```bash
cargo build --release
./target/release/focus-write
```

Or install via the PKGBUILD:

```bash
makepkg -si
```

---

## Font

Focus-Write requires **JetBrains Mono**. Install it with:

```bash
sudo pacman -S ttf-jetbrains-mono
```
