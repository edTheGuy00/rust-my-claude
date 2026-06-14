# rust-my-claude

A fast, themeable **status line for [Claude Code](https://docs.claude.com/en/docs/claude-code)**, written in pure Rust. Zero npm / node / jq — a single ~600 KB binary (no runtime dependencies) that reads Claude Code's session JSON on stdin and prints a colored, Powerline-style status line.

Inspired by [oh-my-posh](https://ohmyposh.dev) and [oh-my-claude](https://github.com/ssenart/oh-my-claude). Ships with **178 bundled themes** and a fully TOML-configurable component system.

```
 personal/rust-my-claude  ✳ Sonnet 4.5   master ~3 ?2   acme/widget
 62% · 124k/200k   5h 45% ↺2h19m   7d 30% Jun 19   $0.42 · 6m12s
```

---

## Gallery

A handful of the 178 bundled themes, rendered with `theme preview`:

**`powerline`** — the default, two lines, classic Powerline chain.

![powerline theme](docs/screenshots/powerline.png)

**`tokyonight`** — a popular palette port.

![tokyonight theme](docs/screenshots/tokyonight.png)

**`bubbles`** — each segment a floating rounded pill (`diamond` separator).

![bubbles theme](docs/screenshots/bubbles.png)

**`flame`** — flame-shaped Powerline dividers.

![flame theme](docs/screenshots/flame.png)

**`loaded-gruvbox`** — everything on one line: model · context · 5h · 7d · cost.

![loaded-gruvbox theme](docs/screenshots/loaded-gruvbox.png)

**`minimal`** — ASCII-safe `plain` style, no Nerd Font required (gauges render as `[####----]`).

![minimal theme](docs/screenshots/minimal.png)

**→ See [`docs/THEMES.md`](docs/THEMES.md) for a numbered preview of all 178 themes.** Each theme's number is its *config number* — apply it directly with `install.sh <N>` or `rust-my-claude init <N>` (no need to scroll the interactive picker).

> Browse them in your terminal with `rust-my-claude theme list`, and preview any one live with `rust-my-claude theme preview <name>`.

---

## Contents
- [Gallery](#gallery)
- [Prerequisites](#prerequisites)
- [Install](#install)
- [How it works](#how-it-works)
- [CLI commands](#cli-commands)
- [Configuration](#configuration)
- [Components reference](#components-reference)
- [Styles & glyphs](#styles--glyphs)
- [Themes](#themes)
- [Troubleshooting](#troubleshooting)
- [Uninstall](#uninstall)

---

## Prerequisites

### A Nerd Font (required for icons & Powerline glyphs)
Most themes use [Nerd Font](https://www.nerdfonts.com/) glyphs (folder , git branch , gauge , flame , Powerline dividers `   `, rounded caps `  `). **Your terminal must be configured to use a Nerd Font**, or these render as empty boxes (▯ "tofu").

Install one (any will do) and set it as your terminal font:

| OS | Install |
|----|---------|
| macOS | `brew install --cask font-hack-nerd-font` (or `font-jetbrains-mono-nerd-font`, `font-fira-code-nerd-font`, `font-meslo-lg-nerd-font`) |
| Arch / Manjaro | `sudo pacman -S ttf-hack-nerd` (or `ttf-jetbrains-mono-nerd`, `ttf-firacode-nerd`) |
| Debian / Ubuntu | `sudo apt install fonts-hack` then grab a patched build from [nerdfonts.com](https://www.nerdfonts.com/font-downloads), or download a `.ttf` to `~/.local/share/fonts` and run `fc-cache -f` |
| Windows | Download from [nerdfonts.com](https://www.nerdfonts.com/font-downloads), install the `.ttf`, set it in your terminal |

Then set the font in your terminal's settings (e.g. *Hack Nerd Font*, *JetBrainsMono Nerd Font*).

> Don't have / want a Nerd Font? Use an **ASCII-safe theme**: `minimal`, `matrix`, or any `plain`-style theme — they avoid Nerd Font glyphs.

### Other requirements
- **Claude Code** (the status line is invoked by it).
- A terminal with **256-color** support (virtually all modern terminals).
- **git** on `PATH` — used by the `git` segment (optional; the segment just hides if absent).
- **For `--compile` only:** the [Rust toolchain](https://rustup.rs/) (`cargo`).

---

## Install

The installer downloads or builds the binary, then launches an interactive **theme picker** that writes your config and wires up Claude Code's `settings.json` (backing up any existing one). Already know which theme you want? Pass its [number](docs/THEMES.md) to skip the picker — see [below](#pick-a-theme-up-front-skip-the-picker).

### Prebuilt binary (no Rust needed) — default
```bash
curl -fsSL https://raw.githubusercontent.com/edTheGuy00/rust-my-claude/main/install.sh | bash
# equivalently: ... | bash -s -- --bin
```

### Compile from source (requires Rust)
```bash
curl -fsSL https://raw.githubusercontent.com/edTheGuy00/rust-my-claude/main/install.sh | bash -s -- --compile
```
…or from a local clone:
```bash
git clone https://github.com/edTheGuy00/rust-my-claude.git
cd rust-my-claude
bash install.sh --compile
```

The installer has two modes plus an optional **theme number**:

| Arg | What it does |
|------|--------------|
| `--bin` | (default) Downloads the prebuilt binary for your platform from GitHub Releases into `~/.local/bin`. No Rust toolchain required. |
| `--compile` | Clones the repo (or builds your local checkout) with `cargo`, installs into `~/.local/bin`. Requires `cargo`. |
| `<N>` | Apply theme number `N` directly and **skip the interactive picker** (which renders all 178 themes and takes a few seconds). Find the number in [`docs/THEMES.md`](docs/THEMES.md) or via `rust-my-claude theme list`. |

### Pick a theme up front (skip the picker)
Browse [`docs/THEMES.md`](docs/THEMES.md), find the number you want, and pass it:
```bash
# prebuilt binary, apply theme #140
curl -fsSL https://raw.githubusercontent.com/edTheGuy00/rust-my-claude/main/install.sh | bash -s -- 140
# compile from source, apply theme #16 (bubbles)
curl -fsSL https://raw.githubusercontent.com/edTheGuy00/rust-my-claude/main/install.sh | bash -s -- --compile 16
```

> Make sure `~/.local/bin` is on your `PATH` (the installer warns if not).

### Manual setup
If you'd rather not run `init`, point Claude Code at the binary yourself in `~/.claude/settings.json`:
```json
{
  "statusLine": { "type": "command", "command": "/home/you/.local/bin/rust-my-claude", "padding": 0 }
}
```
Then create a config (see [Configuration](#configuration)) or run `rust-my-claude theme set <name>`.

---

## How it works

Claude Code runs the binary after each turn and pipes session JSON to it on stdin; whatever the binary prints becomes the status line. `rust-my-claude` is **dual-mode**:

- **Render mode** — stdin has JSON and no subcommand → render the status line and exit. (What Claude Code calls.)
- **CLI mode** — invoked with a subcommand → manage themes/config.

Missing or invalid config falls back to the bundled `powerline` theme; empty or malformed stdin never panics.

---

## CLI commands

```
rust-my-claude init                 # interactive: preview themes, pick one,
rust-my-claude setup                #   write config + patch settings.json
rust-my-claude init <N>             # apply theme number N directly (no picker)
rust-my-claude theme list           # list all bundled themes (in number order)
rust-my-claude theme preview <name> # render a theme with sample data
rust-my-claude theme set <name>     # write a theme to your config
rust-my-claude config path          # print the resolved config file path
```

> `init <N>` and `setup <N>` apply theme number `N` (1-based, matching [`docs/THEMES.md`](docs/THEMES.md) / `theme list`) without rendering the full picker.

---

## Configuration

### File location (resolution order)
1. `$RUST_MY_CLAUDE_CONFIG` — explicit path, if set
2. `$XDG_CONFIG_HOME/rust-my-claude/config.toml`
3. `~/.config/rust-my-claude/config.toml`

A **theme is just a config file**. `theme set` / `init` write one of the bundled themes to location (2)/(3); you can then edit it freely.

### The model: presence = shown
Each status-line component is its **own TOML table**. **If the table is present, the component is shown; if absent, it's hidden.** A config with only `[model]` and `[context]` shows exactly those two.

```toml
[model]
[context]
```

### `[style]` — global look
```toml
[style]
separator           = "rounded"   # powerline | rounded | plain | diamond
default_fg          = 231          # fallback text color (xterm-256, 0–255)
default_bg          = 238          # fallback pill background
gauge_red_shift     = true         # gauge hue bleeds toward red as it fills
gauge_red_threshold = 60           # ...starting at this percent
powerline_divider   = ""          # divider glyph for "powerline" mode
cap_left            = ""          # left cap for "rounded"/"diamond"
cap_right           = ""          # right cap for "rounded"/"diamond"
```

### Per-component fields
Every component table accepts:

| Field | Type | Meaning |
|-------|------|---------|
| `line` | int (1–2) | Which output line. |
| `order` | int | Left-to-right position within the line. |
| `icon` | string | Nerd Font glyph (or `""` for none). |
| `fg` | u8 (0–255) | Text color (xterm-256). |
| `bg` | u8 (0–255) | Pill background (the *filled* color for gauges). |
| `gauge` | bool | Fill the background by percentage (gauge-capable components only). |
| `track` | u8 | Gauge's unfilled "track" color. |
| `format` | string | Template; see [tokens](#components-reference). |
| `disabled` | bool | If `true`, treat as absent. |

Omitted fields fall back to per-component defaults, then `[style]` defaults.

### Example: a compact two-line config
```toml
[style]
separator = "rounded"

[directory]
line = 1
order = 0
icon = ""
fg = 235
bg = 208

[model]
line = 1
order = 1
icon = "✳"
fg = 231
bg = 24

[context]
line = 2
order = 0
icon = ""
gauge = true
bg = 99
track = 54
format = "{pct:.0}% · {in_tok}k/{ctx_size}k"

[cost]
line = 2
order = 1
icon = ""
bg = 94
format = "{cost:.2f} · {duration}"
```

---

## Components reference

Presence shows the component; it's additionally suppressed at runtime when its data is absent (e.g. `cost` only when > 0, `git` only inside a repo).

| Table | Data source (Claude Code JSON) | Gauge | Format tokens |
|-------|-------------------------------|:-----:|---------------|
| `[model]` | `model.display_name` | | `{name}` |
| `[directory]` | `workspace.current_dir` (home→`~`, last 2 segments) | | `{dir}` |
| `[project_dir]` | `workspace.project_dir` | | `{project_dir}` |
| `[git]` | local `git` (branch + `+staged ~modified ?untracked ↑ahead ↓behind`) | | — (bg is dynamic: clean/staged/dirty) |
| `[repo]` | `workspace.repo.{owner,name}` | | `{owner}`, `{name}` |
| `[context]` | `context_window.{used_percentage,total_input_tokens,context_window_size}` | ✓ | `{pct}`, `{in_tok}` (×1000), `{ctx_size}` (×1000) |
| `[context_tokens]` | `context_window.{total_input_tokens,total_output_tokens}` | | `{in_tok}`, `{out_tok}` |
| `[cost]` | `cost.{total_cost_usd,total_duration_ms}` | | `{cost}`, `{duration}` |
| `[duration]` | `cost.total_duration_ms` | | `{duration}` |
| `[limits]` | `rate_limits.{five_hour,seven_day}` — **two pills**, shared color | ✓ | — |
| `[limit_5h]` | `rate_limits.five_hour` (`%` + `↺countdown`) | ✓ | — |
| `[limit_7d]` | `rate_limits.seven_day` (`%` + reset date) | ✓ | — |
| `[vim]` | `vim.mode` (hidden in INSERT) | | `{mode}` |
| `[thinking]` | `thinking.enabled` | | — |
| `[effort]` | `effort.level` | | `{level}` |
| `[pr]` | `pr.{number,review_state}` | | `{number}`, `{review_state}` |
| `[session]` | `session_name` | | `{session}` |
| `[output_style]` | `output_style.name` | | `{name}` |
| `[version]` | `version` | | `{version}` |

> **Independent rate-limit pills:** use `[limit_5h]` and `[limit_7d]` (each its own colors) instead of the combined `[limits]` to color the two windows differently.

**Format precision:** numeric tokens accept a precision like `{pct:.0}` or `{cost:.2}`/`{cost:.2f}`. Unknown tokens are left literal. `{in_tok}` and `{ctx_size}` are already divided by 1000 — append a literal `k` (e.g. `{in_tok}k/{ctx_size}k` → `124k/200k`, or `620k/1000k` on a 1M-context model).

---

## Styles & glyphs

`[style].separator` selects the structural look; the cap/divider glyphs are configurable, so one mode covers several shapes:

| `separator` | Look |
|-------------|------|
| `rounded` | One connected chain of pills with `` dividers and rounded end caps. (Classic.) |
| `powerline` | Connected chain, flush ends. Set `powerline_divider` to `` (arrow), `` (slant), or `` (flame). |
| `diamond` (aliases `bubbles`, `chips`) | Each segment is its **own floating shape** wrapped in `cap_left`/`cap_right`, separated by gaps. Round caps `` `` → bubbles; slant `` ``; flame `` ``. |
| `plain` | No backgrounds — just color-coded text. Gauges render as `[####----] 62%`. Great for ASCII/transparent prompts and non-Nerd-Font terminals. |

**Gauges** (`gauge = true` on `context` / `limits` / `limit_5h` / `limit_7d`) fill the pill background left-to-right by percentage; with `gauge_red_shift` the filled hue bleeds toward red past `gauge_red_threshold` so a nearly-full budget reads as a warning.

---

## Themes

**178 bundled themes.** See **[`docs/THEMES.md`](docs/THEMES.md) for a numbered preview of every one.** Browse in the terminal with `rust-my-claude theme list`, preview with `theme preview <name>`, apply with `theme set <name>`, `init <N>`, or the `init` picker.

| Family | Count | What |
|--------|------:|------|
| Two-line | 65 | Originals + structural showcases (`powerline`, `bubbles`, `chips`, `cert`, `flame`, `slant`, `emodipt`, …) + 44 ports of popular oh-my-posh themes (`dracula`, `gruvbox`, `nord`, `tokyonight`, `catppuccin*`, `monokai`, `onedark`, `night-owl`, `cobalt2`, `jandedobbeleer`, `powerlevel10k-*`, …). |
| One-line (`*-1l`) | 65 | A single-line variant of every theme above (mix of minimal & compact layouts). |
| `spaced-*` | 24 | `model · context · 5h · 7d` as spaced pills, across many palettes & styles. |
| `loaded-*` | 24 | `model · git · context · 5h · 7d · cost` on one line, 5h/7d colored independently. |

### Make your own
Copy any bundled theme to your config and edit it:
```bash
rust-my-claude theme set bubbles      # writes bubbles to your config.toml
rust-my-claude config path            # find it and edit
```
Colors are [xterm-256 indices](https://www.ditig.com/256-colors-cheat-sheet) (0–255).

---

## Troubleshooting

- **Boxes / ▯ instead of icons:** your terminal isn't using a Nerd Font. Install one and set it as the terminal font (see [Prerequisites](#prerequisites)), or switch to `minimal` / `matrix` / a `plain`-style theme.
- **No status line appears:** confirm `~/.claude/settings.json` has a `statusLine.command` pointing at the binary, and that the path exists. Trigger any interaction in Claude Code (it re-runs after each turn).
- **`~/.local/bin` not on PATH:** add `export PATH="$HOME/.local/bin:$PATH"` to your shell profile.
- **Wrong colors:** ensure your terminal advertises 256 colors (`echo $TERM` → something like `xterm-256color`).
- **Test it by hand:**
  ```bash
  echo '{"model":{"display_name":"Sonnet"},"workspace":{"current_dir":"'"$HOME"'"},"context_window":{"used_percentage":42}}' | rust-my-claude
  ```

---

## Uninstall

```bash
rm ~/.local/bin/rust-my-claude
rm -rf ~/.config/rust-my-claude
```
Then remove the `statusLine` block from `~/.claude/settings.json` (a timestamped backup was saved next to it when you ran `init`).

---

## Credits & license

Inspired by [oh-my-posh](https://ohmyposh.dev) (theme palettes & structural styles) and [oh-my-claude](https://github.com/ssenart/oh-my-claude). MIT licensed.
