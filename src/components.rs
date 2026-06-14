//! Maps a component config table + parsed JSON input into zero or more `Pill`s.
//!
//! Each known component name has a dedicated `build_*` function that returns
//! `Vec<Pill>` (empty when the "show when" condition is not met). Unknown
//! component names produce no pills (silently ignored).

use std::collections::HashMap;

use crate::config::{ComponentConfig, Style};
use crate::git::git_info;
use crate::input::Input;
use crate::render::Pill;
use crate::util::{fmt_countdown, fmt_date, fmt_duration, render_format, short_dir, FormatValue};

// ─── Dynamic git bg colors ────────────────────────────────────────────────────
const GIT_BG_CLEAN: u8 = 71;
const GIT_BG_STAGED: u8 = 35;
const GIT_BG_DIRTY: u8 = 178;

// ─── Default gauge track colors ───────────────────────────────────────────────
const TRACK_CTX: u8 = 54;
const TRACK_5H: u8 = 89;
const TRACK_7D: u8 = 24;

// ─── Dispatch ────────────────────────────────────────────────────────────────

/// Build pills for a named component. Returns `Vec<Pill>` (may be empty when
/// the show-condition is not met). Unknown names return `vec![]`.
pub fn build(name: &str, cc: &ComponentConfig, input: &Input, style: &Style) -> Vec<Pill> {
    if cc.disabled {
        return vec![];
    }
    match name {
        "model" => build_model(cc, input, style),
        "directory" => build_directory(cc, input),
        "project_dir" => build_project_dir(cc, input),
        "git" => build_git(cc, input),
        "repo" => build_repo(cc, input),
        "context" => build_context(cc, input, style),
        "context_tokens" => build_context_tokens(cc, input),
        "cost" => build_cost(cc, input),
        "duration" => build_duration(cc, input),
        "limits" => build_limits(cc, input, style),
        "limit_5h" => build_one_limit(cc, input, style, Window::FiveHour),
        "limit_7d" => build_one_limit(cc, input, style, Window::SevenDay),
        "vim" => build_vim(cc, input),
        "thinking" => build_thinking(cc, input),
        "effort" => build_effort(cc, input),
        "pr" => build_pr(cc, input),
        "session" => build_session(cc, input),
        "output_style" => build_output_style(cc, input),
        "version" => build_version(cc, input),
        _ => vec![],
    }
}

// ─── Component builders ───────────────────────────────────────────────────────

fn build_model(cc: &ComponentConfig, input: &Input, _style: &Style) -> Vec<Pill> {
    let name = if !input.model.display_name.is_empty() {
        &input.model.display_name
    } else if !input.model.id.is_empty() {
        &input.model.id
    } else {
        return vec![];
    };
    let icon = cc.icon_or("\u{2733}"); // ✳
    let fg = cc.fg_or(231);
    let bg = cc.bg_or(24);
    let mut vars = HashMap::new();
    vars.insert("name", FormatValue::Str(name.clone()));
    let text = render_format(cc.format_or("{name}"), &vars);
    vec![Pill::new(icon, text, fg, bg)]
}

fn build_directory(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let cwd = &input.workspace.current_dir;
    if cwd.is_empty() {
        return vec![];
    }
    let icon = cc.icon_or("\u{f07b}"); //
    let fg = cc.fg_or(235);
    let bg = cc.bg_or(208);
    let dir = short_dir(cwd);
    let mut vars = HashMap::new();
    vars.insert("dir", FormatValue::Str(dir.clone()));
    let text = render_format(cc.format_or("{dir}"), &vars);
    vec![Pill::new(icon, text, fg, bg)]
}

fn build_project_dir(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let proj = match &input.workspace.project_dir {
        Some(p) if !p.is_empty() => p,
        _ => return vec![],
    };
    // Only show if it differs from current_dir.
    if proj == &input.workspace.current_dir {
        return vec![];
    }
    let icon = cc.icon_or("\u{f07c}"); //
    let fg = cc.fg_or(250);
    let bg = cc.bg_or(238);
    let dir = short_dir(proj);
    let mut vars = HashMap::new();
    vars.insert("project_dir", FormatValue::Str(dir));
    let text = render_format(cc.format_or("{project_dir}"), &vars);
    vec![Pill::new(icon, text, fg, bg)]
}

fn build_git(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let cwd = &input.workspace.current_dir;
    if cwd.is_empty() {
        return vec![];
    }
    let g = match git_info(cwd) {
        Some(g) => g,
        None => return vec![],
    };
    // Dynamic bg unless the user set an explicit one.
    let bg = cc.bg.unwrap_or(if g.staged > 0 {
        GIT_BG_STAGED
    } else if g.modified > 0 || g.untracked > 0 {
        GIT_BG_DIRTY
    } else {
        GIT_BG_CLEAN
    });
    let icon = cc.icon_or("\u{e0a0}"); //
    let fg = cc.fg_or(235);
    let mut t = if g.branch.is_empty() {
        "(detached)".to_string()
    } else {
        g.branch.clone()
    };
    if g.staged > 0 {
        t.push_str(&format!(" +{}", g.staged));
    }
    if g.modified > 0 {
        t.push_str(&format!(" ~{}", g.modified));
    }
    if g.untracked > 0 {
        t.push_str(&format!(" ?{}", g.untracked));
    }
    if g.ahead > 0 {
        t.push_str(&format!(" \u{2191}{}", g.ahead));
    }
    if g.behind > 0 {
        t.push_str(&format!(" \u{2193}{}", g.behind));
    }
    vec![Pill::new(icon, t, fg, bg)]
}

fn build_repo(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let r = match &input.workspace.repo {
        Some(r) if !r.name.is_empty() => r,
        _ => return vec![],
    };
    let icon = cc.icon_or(""); // empty default (devicon placeholder)
    let fg = cc.fg_or(250);
    let bg = cc.bg_or(238);
    let label = format!("{}/{}", r.owner, r.name);
    let mut vars = HashMap::new();
    vars.insert("owner", FormatValue::Str(r.owner.clone()));
    vars.insert("name", FormatValue::Str(r.name.clone()));
    let text = render_format(cc.format_or("{owner}/{name}"), &vars);
    // If the format resolves to the same as the label, use that; otherwise
    // use the formatted text (handles `{name}`-only formats etc.)
    let _ = label; // consumed by the vars above
    vec![Pill::new(icon, text, fg, bg)]
}

fn build_context(cc: &ComponentConfig, input: &Input, style: &Style) -> Vec<Pill> {
    let pct = input.context_window.used_percentage.unwrap_or(0.0);
    let in_tok = input.context_window.total_input_tokens;
    let ctx_size = input.context_window.context_window_size;

    let icon = cc.icon_or("\u{f0e4}"); //
    let fg = cc.fg_or(231);
    let bg = cc.bg_or(99);
    let track = cc.track.unwrap_or(TRACK_CTX);

    let mut vars = HashMap::new();
    vars.insert("pct", FormatValue::Float(pct));
    vars.insert("in_tok", FormatValue::Str(format!("{}", in_tok / 1000)));
    vars.insert(
        "ctx_size",
        FormatValue::Str(format!("{}", ctx_size / 1000)),
    );

    let text = render_format(cc.format_or("{pct:.0}% · {in_tok}k/{ctx_size}k"), &vars);

    if cc.gauge_or(true) {
        vec![Pill::gauge(icon, text, pct, bg, track, fg, style)]
    } else {
        vec![Pill::new(icon, text, fg, bg)]
    }
}

fn build_context_tokens(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let in_tok = input.context_window.total_input_tokens;
    let out_tok = input.context_window.total_output_tokens;
    if in_tok == 0 {
        return vec![];
    }
    let icon = cc.icon_or("\u{f1de}"); //
    let fg = cc.fg_or(250);
    let bg = cc.bg_or(238);
    let mut vars = HashMap::new();
    vars.insert("in_tok", FormatValue::Str(format!("{}", in_tok / 1000)));
    vars.insert(
        "out_tok",
        FormatValue::Str(format!("{}", out_tok / 1000)),
    );
    let text = render_format(cc.format_or("{in_tok}k↑ {out_tok}k↓"), &vars);
    vec![Pill::new(icon, text, fg, bg)]
}

fn build_cost(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let cost = input.cost.total_cost_usd;
    let dur_ms = input.cost.total_duration_ms;
    if cost == 0.0 && dur_ms == 0 {
        return vec![];
    }
    let icon = cc.icon_or("\u{f155}"); //
    let fg = cc.fg_or(231);
    let bg = cc.bg_or(94);
    let mut vars = HashMap::new();
    vars.insert("cost", FormatValue::Float(cost));
    vars.insert("duration", FormatValue::Str(fmt_duration(dur_ms)));
    let text = render_format(cc.format_or("${cost:.4} · {duration}"), &vars);
    vec![Pill::new(icon, text, fg, bg)]
}

fn build_duration(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let dur_ms = input.cost.total_duration_ms;
    if dur_ms == 0 {
        return vec![];
    }
    let icon = cc.icon_or("\u{f017}"); //
    let fg = cc.fg_or(250);
    let bg = cc.bg_or(238);
    let mut vars = HashMap::new();
    vars.insert("duration", FormatValue::Str(fmt_duration(dur_ms)));
    let text = render_format(cc.format_or("{duration}"), &vars);
    vec![Pill::new(icon, text, fg, bg)]
}

/// Which rate-limit window to render.
enum Window {
    FiveHour,
    SevenDay,
}

/// Render one rate-limit window into a single pill using `cc`'s style fields.
/// Shared by `[limits]` (both windows, same colours) and the independent
/// `[limit_5h]` / `[limit_7d]` components.
fn build_one_limit(
    cc: &ComponentConfig,
    input: &Input,
    style: &Style,
    window: Window,
) -> Vec<Pill> {
    let rl = match &input.rate_limits {
        Some(rl) => rl,
        None => return vec![],
    };
    let (w, label, def_bg, def_track, def_icon) = match window {
        Window::FiveHour => (&rl.five_hour, "5h", 168, TRACK_5H, "\u{f06d}"), //
        Window::SevenDay => (&rl.seven_day, "7d", 33, TRACK_7D, "\u{f073}"),  //
    };
    let w = match w {
        Some(w) => w,
        None => return vec![],
    };
    let pct = w.used_percentage.unwrap_or(0.0);
    let mut t = format!("{label} {pct:.0}%");
    if let Some(at) = w.resets_at {
        match window {
            Window::FiveHour => t.push_str(&format!(" \u{21ba}{}", fmt_countdown(at))),
            Window::SevenDay => t.push_str(&format!(" {}", fmt_date(at))),
        }
    }
    let fg = cc.fg_or(231);
    let bg = cc.bg_or(def_bg);
    let track = cc.track.unwrap_or(def_track);
    let icon = cc.icon_or(def_icon);
    let pill = if cc.gauge_or(true) {
        Pill::gauge(icon, t, pct, bg, track, fg, style)
    } else {
        Pill::new(icon, t, fg, bg)
    };
    vec![pill]
}

/// `[limits]` — both windows in one block (they share `cc`'s colours).
fn build_limits(cc: &ComponentConfig, input: &Input, style: &Style) -> Vec<Pill> {
    let mut pills = build_one_limit(cc, input, style, Window::FiveHour);
    pills.extend(build_one_limit(cc, input, style, Window::SevenDay));
    pills
}

fn build_vim(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let mode = match &input.vim {
        Some(v) if !v.mode.is_empty() && v.mode != "INSERT" => v.mode.clone(),
        _ => return vec![],
    };
    let icon = cc.icon_or("\u{e62b}"); //
    let fg = cc.fg_or(231);
    let bg = cc.bg_or(30);
    let mut vars = HashMap::new();
    vars.insert("mode", FormatValue::Str(mode));
    let text = render_format(cc.format_or("{mode}"), &vars);
    vec![Pill::new(icon, text, fg, bg)]
}

fn build_thinking(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let enabled = input.thinking.as_ref().map(|t| t.enabled).unwrap_or(false);
    if !enabled {
        return vec![];
    }
    let icon = cc.icon_or("\u{f0eb}"); //
    let fg = cc.fg_or(235);
    let bg = cc.bg_or(168);
    vec![Pill::new(icon, "thinking", fg, bg)]
}

fn build_effort(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let level = match &input.effort {
        Some(e) if !e.level.is_empty() => e.level.clone(),
        _ => return vec![],
    };
    let icon = cc.icon_or("\u{f085}"); //
    let fg = cc.fg_or(235);
    let bg = cc.bg_or(178);
    let mut vars = HashMap::new();
    vars.insert("level", FormatValue::Str(level));
    let text = render_format(cc.format_or("{level}"), &vars);
    vec![Pill::new(icon, text, fg, bg)]
}

fn build_pr(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let pr = match &input.pr {
        Some(p) if p.number.is_some() => p,
        _ => return vec![],
    };
    let number = pr.number.unwrap();
    let icon = cc.icon_or("\u{f407}"); //
    let fg = cc.fg_or(231);
    let bg = cc.bg_or(141);
    let mut vars = HashMap::new();
    vars.insert("number", FormatValue::Str(number.to_string()));
    vars.insert("review_state", FormatValue::Str(pr.review_state.clone()));
    let text = render_format(cc.format_or("#{number} {review_state}"), &vars);
    vec![Pill::new(icon, text, fg, bg)]
}

fn build_session(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let session = match &input.session_name {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return vec![],
    };
    let icon = cc.icon_or("\u{f292}"); //
    let fg = cc.fg_or(250);
    let bg = cc.bg_or(238);
    let mut vars = HashMap::new();
    vars.insert("session", FormatValue::Str(session));
    let text = render_format(cc.format_or("{session}"), &vars);
    vec![Pill::new(icon, text, fg, bg)]
}

fn build_output_style(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let name = match &input.output_style {
        Some(os) if !os.name.is_empty() => os.name.clone(),
        _ => return vec![],
    };
    let icon = cc.icon_or("\u{f1fc}"); //
    let fg = cc.fg_or(250);
    let bg = cc.bg_or(238);
    let mut vars = HashMap::new();
    vars.insert("name", FormatValue::Str(name));
    let text = render_format(cc.format_or("{name}"), &vars);
    vec![Pill::new(icon, text, fg, bg)]
}

fn build_version(cc: &ComponentConfig, input: &Input) -> Vec<Pill> {
    let ver = match &input.version {
        Some(v) if !v.is_empty() => v.clone(),
        _ => return vec![],
    };
    let icon = cc.icon_or("\u{f126}"); //
    let fg = cc.fg_or(250);
    let bg = cc.bg_or(238);
    let mut vars = HashMap::new();
    vars.insert("version", FormatValue::Str(ver));
    let text = render_format(cc.format_or("v{version}"), &vars);
    vec![Pill::new(icon, text, fg, bg)]
}
