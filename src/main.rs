//! rust-my-claude — Claude Code powerline statusline renderer.
//!
//! # Dual-mode dispatch
//!
//! - **Render mode** (default): JSON is piped on stdin and no subcommand is
//!   given. Reads the statusline JSON, renders a Powerline statusline, prints
//!   it, and exits 0. Empty or garbage stdin → renders with zero-value data,
//!   still exits 0.
//! - **CLI mode**: first argument matches a known subcommand verb → handled by
//!   `cli::run(args)`.

use std::io::{self, Read};

mod cli;
mod components;
mod config;
mod git;
mod input;
mod render;
mod themes;
mod util;

use input::Input;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // CLI mode: dispatch subcommands before touching stdin.
    if let Some(cmd) = args.first()
        && cli::is_subcommand(cmd)
    {
        std::process::exit(cli::run(&args));
    }

    // Render mode: read stdin, parse JSON, render, print.
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw).ok();
    let input: Input = serde_json::from_str(&raw).unwrap_or_default();
    let cfg = config::load();
    let output = render_statusline(&cfg, &input);
    print!("{output}");
}

/// Render a full two-line statusline string from config + input.
pub fn render_statusline(cfg: &config::Config, input: &Input) -> String {
    // Collect pills with ordering metadata.
    let mut tagged: Vec<Vec<(i64, usize, render::Pill)>> = vec![Vec::new(), Vec::new()];

    for (source_idx, (name, cc)) in cfg.components.iter().enumerate() {
        if cc.disabled {
            continue;
        }
        let line_idx = (cc.line() as usize).saturating_sub(1).min(1);
        let order = cc.order();
        let pills = components::build(name, cc, input, &cfg.style);
        for pill in pills {
            tagged[line_idx].push((order, source_idx, pill));
        }
    }

    // Sort each line by (order, source_idx) for stable ordering.
    for line in &mut tagged {
        line.sort_by_key(|(order, src, _)| (*order, *src));
    }

    let pill_lines: Vec<Vec<render::Pill>> = tagged
        .into_iter()
        .map(|v| v.into_iter().map(|(_, _, p)| p).collect())
        .collect();

    render::render_lines(&pill_lines, &cfg.style)
}
