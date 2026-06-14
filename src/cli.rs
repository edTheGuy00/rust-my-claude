//! CLI subcommand parsing and handlers.
//!
//! Subcommands:
//!   init / setup         Interactive theme picker + settings.json patch
//!   theme list           List bundled themes
//!   theme preview <name> ANSI preview of a theme with sample data
//!   theme set <name>     Write theme to config file (no settings.json patch)
//!   config path          Print the resolved config path
//!   help / --help / -h   Usage
//!   --version            Version

use std::io::{self, Write as _};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;
use crate::input::{
    ContextWindow, Cost, Input, Model, RateLimits, RateWindow, Thinking, Workspace,
};
use crate::themes;

// ─── Public API ───────────────────────────────────────────────────────────────

/// Returns true when the first argument matches a known subcommand verb.
pub fn is_subcommand(s: &str) -> bool {
    matches!(
        s,
        "init"
            | "setup"
            | "theme"
            | "config"
            | "help"
            | "--help"
            | "-h"
            | "--version"
            | "version"
    )
}

/// Run the CLI. Returns an exit code (0 = success, 1 = user error, 2 = I/O error).
pub fn run(args: &[String]) -> i32 {
    let cmd = args.first().map(String::as_str).unwrap_or("help");
    match cmd {
        "init" | "setup" => cmd_init(&args[1..]),
        "theme" => cmd_theme(&args[1..]),
        "config" => cmd_config(&args[1..]),
        "--version" | "version" => {
            println!("rust-my-claude {}", env!("CARGO_PKG_VERSION"));
            0
        }
        "help" | "--help" | "-h" => {
            print_usage();
            0
        }
        other => {
            eprintln!("Unknown subcommand: {other:?}");
            eprintln!();
            print_usage_to_stderr();
            1
        }
    }
}

// ─── Usage ────────────────────────────────────────────────────────────────────

fn print_usage() {
    println!("{}", USAGE);
}

fn print_usage_to_stderr() {
    eprintln!("{}", USAGE);
}

const USAGE: &str = "\
rust-my-claude — Claude Code powerline statusline renderer

USAGE:
    rust-my-claude                       Render statusline (JSON on stdin)
    rust-my-claude init                  Interactive theme picker + settings setup
    rust-my-claude init <N>              Apply theme number N non-interactively
    rust-my-claude setup                 Alias for init
    rust-my-claude theme list            List bundled themes
    rust-my-claude theme preview <name>  Preview a theme with sample data
    rust-my-claude theme set <name>      Write theme to config file
    rust-my-claude config path           Print the resolved config file path
    rust-my-claude --version             Print version
    rust-my-claude help                  Print this help";

// ─── cmd_init ────────────────────────────────────────────────────────────────

fn cmd_init(args: &[String]) -> i32 {
    let n = themes::THEMES.len();

    // Non-interactive: `init <N>` applies theme number N (1-based) directly,
    // skipping the (slow) render-every-theme preview and the prompt.
    if let Some(arg) = args.first() {
        match arg.parse::<usize>() {
            Ok(num) if num >= 1 && num <= n => return apply_theme(num - 1),
            _ => {
                eprintln!("Invalid theme number: {arg:?}. Expected 1-{n}.");
                eprintln!("Run 'rust-my-claude theme list' to see the numbered list.");
                return 1;
            }
        }
    }

    let sample = sample_input();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   rust-my-claude — theme setup");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    for (i, theme) in themes::THEMES.iter().enumerate() {
        println!("[{}] {} — {}", i + 1, theme.name, theme.description);
        if let Some(cfg) = config::parse(theme.toml) {
            let preview = render_statusline(&cfg, &sample);
            for line in preview.lines() {
                println!("    {line}");
            }
        }
        println!();
    }

    match prompt_theme_choice(n) {
        None => {
            println!("Cancelled.");
            0
        }
        Some(idx) => apply_theme(idx),
    }
}

/// Apply the theme at `idx` (0-based): write the config and patch settings.json.
/// Shared by the interactive picker and the non-interactive `init <N>` path.
fn apply_theme(idx: usize) -> i32 {
    let theme = &themes::THEMES[idx];
    if let Err(e) = write_config(theme.toml) {
        eprintln!("Error writing config: {e}");
        return 2;
    }
    if let Err(e) = patch_settings_json() {
        eprintln!("Warning: could not patch settings.json: {e}");
        // Non-fatal: config was written, just warn.
    }
    let cfg_path = config::resolve_path();
    println!();
    println!("Theme '{}' applied.", theme.name);
    println!("Config written to: {}", cfg_path.display());
    println!();
    println!("Restart Claude Code to see the updated statusline.");
    0
}

/// Prompt for a 1-based theme index. Returns `Some(0-based index)` or `None`
/// if the user cancels (q/empty) or exhausts retries.
fn prompt_theme_choice(n: usize) -> Option<usize> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for attempt in 0..3 {
        if attempt > 0 {
            println!("  (invalid input; enter a number 1-{n} or q to cancel)");
        }
        print!("Pick a theme [1-{n}] (q to cancel): ");
        stdout.flush().ok();

        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
            return None;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("q") {
            return None;
        }
        if let Ok(num) = trimmed.parse::<usize>()
            && num >= 1
            && num <= n
        {
            return Some(num - 1);
        }
    }
    eprintln!("Too many invalid inputs. Aborting.");
    None
}

// ─── cmd_theme ────────────────────────────────────────────────────────────────

fn cmd_theme(args: &[String]) -> i32 {
    let sub = args.first().map(String::as_str).unwrap_or("list");
    match sub {
        "list" => cmd_theme_list(),
        "preview" => {
            let name = match args.get(1) {
                Some(n) => n.as_str(),
                None => {
                    eprintln!("Usage: rust-my-claude theme preview <name>");
                    return 1;
                }
            };
            cmd_theme_preview(name)
        }
        "set" => {
            let name = match args.get(1) {
                Some(n) => n.as_str(),
                None => {
                    eprintln!("Usage: rust-my-claude theme set <name>");
                    return 1;
                }
            };
            cmd_theme_set(name)
        }
        other => {
            eprintln!("Unknown theme subcommand: {other:?}");
            eprintln!("Available: list, preview <name>, set <name>");
            1
        }
    }
}

fn cmd_theme_list() -> i32 {
    println!("Bundled themes:");
    for theme in themes::THEMES {
        println!("  {:12} {}", theme.name, theme.description);
    }
    0
}

fn cmd_theme_preview(name: &str) -> i32 {
    let theme = match themes::get(name) {
        Some(t) => t,
        None => {
            eprintln!(
                "Unknown theme: {name:?}. Run 'rust-my-claude theme list' to see available themes."
            );
            return 1;
        }
    };
    let cfg = match config::parse(theme.toml) {
        Some(c) => c,
        None => {
            eprintln!("Failed to parse theme {name:?}.");
            return 1;
        }
    };
    let sample = sample_input();
    let preview = render_statusline(&cfg, &sample);
    print!("{preview}");
    0
}

fn cmd_theme_set(name: &str) -> i32 {
    let theme = match themes::get(name) {
        Some(t) => t,
        None => {
            eprintln!(
                "Unknown theme: {name:?}. Run 'rust-my-claude theme list' to see available themes."
            );
            return 1;
        }
    };
    match write_config(theme.toml) {
        Ok(path) => {
            println!("Theme '{name}' written to {}.", path.display());
            0
        }
        Err(e) => {
            eprintln!("Error writing config: {e}");
            2
        }
    }
}

// ─── cmd_config ───────────────────────────────────────────────────────────────

fn cmd_config(args: &[String]) -> i32 {
    let sub = args.first().map(String::as_str).unwrap_or("path");
    match sub {
        "path" => {
            println!("{}", config::resolve_path().display());
            0
        }
        other => {
            eprintln!("Unknown config subcommand: {other:?}");
            eprintln!("Available: path");
            1
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Write TOML content to the resolved config path, backing up any existing file.
/// Returns the path on success.
fn write_config(toml_content: &str) -> Result<PathBuf, String> {
    let path = config::resolve_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if path.exists() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let backup = path.with_extension(format!("toml.bak.{ts}"));
        std::fs::copy(&path, &backup).map_err(|e| e.to_string())?;
        println!("Backed up existing config to {}.", backup.display());
    }
    std::fs::write(&path, toml_content).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Patch `~/.claude/settings.json` to set the statusLine command, backing up
/// any existing file first.
fn patch_settings_json() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let settings_path = PathBuf::from(&home).join(".claude").join("settings.json");

    // Get the absolute path to the running binary.
    let exe = std::env::current_exe()
        .and_then(|p| p.canonicalize())
        .map_err(|e| format!("cannot resolve binary path: {e}"))?;
    let exe_str = exe.to_string_lossy().to_string();

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Load or create the JSON.
    let mut json_val: serde_json::Value = if settings_path.exists() {
        let backup = settings_path.with_extension(format!("json.bak.{ts}"));
        std::fs::copy(&settings_path, &backup).map_err(|e| e.to_string())?;
        println!("Backed up settings.json to {}.", backup.display());
        let content = std::fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or(serde_json::Value::Object(Default::default()))
    } else {
        // Create parent dir if needed.
        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        serde_json::Value::Object(Default::default())
    };

    // Patch the statusLine key.
    json_val["statusLine"] = serde_json::json!({
        "type": "command",
        "command": exe_str,
        "padding": 0
    });

    let pretty = serde_json::to_string_pretty(&json_val).map_err(|e| e.to_string())?;
    let mut file = std::fs::File::create(&settings_path).map_err(|e| e.to_string())?;
    file.write_all(pretty.as_bytes()).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())?;
    println!("Patched {}.", settings_path.display());
    Ok(())
}

// ─── Sample data for previews ─────────────────────────────────────────────────

/// Realistic sample `Input` for theme previews. Uses a synthetic git branch
/// via a fixed cwd (no actual git call for a non-repo path) so preview is
/// fast and reproducible.
fn sample_input() -> Input {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Input {
        model: Model {
            display_name: "Claude Opus 4.5".into(),
            id: "claude-opus-4-5".into(),
        },
        workspace: Workspace {
            current_dir: "/home/user/projects/my-app".into(),
            project_dir: None,
            repo: None,
        },
        cost: Cost {
            total_cost_usd: 1.234,
            total_duration_ms: 200_000, // 3m20s
            total_lines_added: 0,
            total_lines_removed: 0,
        },
        context_window: ContextWindow {
            used_percentage: Some(47.0),
            context_window_size: 200_000,
            total_input_tokens: 94_000,
            total_output_tokens: 12_000,
        },
        rate_limits: Some(RateLimits {
            five_hour: Some(RateWindow {
                used_percentage: Some(31.0),
                resets_at: Some(now + 4200),
            }),
            seven_day: Some(RateWindow {
                used_percentage: Some(12.0),
                resets_at: Some(now + 432_000),
            }),
        }),
        vim: None,
        thinking: Some(Thinking { enabled: false }),
        effort: None,
        pr: None,
        session_name: None,
        output_style: None,
        version: None,
    }
}

/// Render a full statusline string for a given config + input.
/// Used internally for previews. Delegates to `crate::render_statusline`.
fn render_statusline(cfg: &config::Config, input: &Input) -> String {
    crate::render_statusline(cfg, input)
}
