//! Shared formatting utilities.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Format a millisecond duration as e.g. "3m20s" or "45s".
pub fn fmt_duration(ms: u64) -> String {
    let s = ms / 1000;
    let m = s / 60;
    if m > 0 {
        format!("{m}m{:02}s", s % 60)
    } else {
        format!("{s}s")
    }
}

/// Format a Unix-epoch reset point as a countdown: "1h30m", "45m", "now".
pub fn fmt_countdown(resets_at: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if resets_at <= now {
        return "now".into();
    }
    let d = resets_at - now;
    let h = d / 3600;
    let m = (d % 3600) / 60;
    if h > 0 {
        format!("{h}h{m:02}m")
    } else {
        format!("{m}m")
    }
}

/// Format an epoch-seconds reset point as a local calendar date, e.g. "Jun 19".
/// Shells out to `date` so the local timezone is respected; falls back to the
/// countdown form if `date` is unavailable or fails.
pub fn fmt_date(resets_at: u64) -> String {
    Command::new("date")
        .args(["-d", &format!("@{resets_at}"), "+%b %-d"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| fmt_countdown(resets_at))
}

/// Shorten a directory path: replace home with `~`, keep at most the last 2
/// path segments (e.g. `~/projects/foo/bar` → `foo/bar`).
pub fn short_dir(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let p = if !home.is_empty() && path.starts_with(&home) {
        path.replacen(&home, "~", 1)
    } else {
        path.to_string()
    };
    let parts: Vec<&str> = p.trim_end_matches('/').split('/').collect();
    let n = parts.len();
    if n <= 2 {
        p
    } else {
        format!("{}/{}", parts[n - 2], parts[n - 1])
    }
}

/// Minimal `{name}` / `{name:.Nf}` template renderer.
///
/// Substitutes tokens of the form `{key}` (display) or `{key:.Nf}` (float
/// with N decimal places) from `vars`. Unknown tokens are left verbatim.
pub fn render_format(template: &str, vars: &std::collections::HashMap<&str, FormatValue>) -> String {
    let mut out = String::with_capacity(template.len() * 2);
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '{' {
            out.push(c);
            continue;
        }
        // Collect up to the matching `}`.
        let mut tok = String::new();
        let mut closed = false;
        for ch in chars.by_ref() {
            if ch == '}' {
                closed = true;
                break;
            }
            tok.push(ch);
        }
        if !closed {
            // No closing brace — output literally.
            out.push('{');
            out.push_str(&tok);
            continue;
        }
        // Parse key and optional :.Nf suffix.
        if let Some(dot_pos) = tok.find(":.") {
            let key = &tok[..dot_pos];
            let fmt_spec = &tok[dot_pos + 2..]; // e.g. "0f" or "4f"
            if let Some(fv) = vars.get(key) {
                let precision: usize = fmt_spec
                    .trim_end_matches('f')
                    .parse()
                    .unwrap_or(2);
                out.push_str(&fv.fmt_float(precision));
            } else {
                out.push('{');
                out.push_str(&tok);
                out.push('}');
            }
        } else {
            let key = &tok;
            if let Some(fv) = vars.get(key.as_str()) {
                out.push_str(&fv.to_string());
            } else {
                out.push('{');
                out.push_str(key);
                out.push('}');
            }
        }
    }
    out
}

/// A format variable value (string or float).
pub enum FormatValue {
    Str(String),
    Float(f64),
}

impl FormatValue {
    pub fn fmt_float(&self, precision: usize) -> String {
        match self {
            FormatValue::Float(f) => format!("{:.prec$}", f, prec = precision),
            FormatValue::Str(s) => s.clone(),
        }
    }
}

impl std::fmt::Display for FormatValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatValue::Str(s) => write!(f, "{s}"),
            FormatValue::Float(v) => write!(f, "{v}"),
        }
    }
}
