//! Git repository status detection.
//!
//! Shells out to `git` (no libgit2 dependency). Returns `None` when not inside
//! a git repo or when `git` is unavailable.

use std::process::Command;

/// Snapshot of the current git repo state.
pub struct GitInfo {
    pub branch: String,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub ahead: usize,
    pub behind: usize,
}

/// Query git status for `cwd`. Returns `None` if not in a git repo.
pub fn git_info(cwd: &str) -> Option<GitInfo> {
    if cwd.is_empty() {
        return None;
    }
    let in_repo = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(cwd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !in_repo {
        return None;
    }

    let branch = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(cwd)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    let status_out = Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=normal"])
        .current_dir(cwd)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let (mut staged, mut modified, mut untracked) = (0usize, 0usize, 0usize);
    for line in status_out.lines() {
        if line.len() < 2 {
            continue;
        }
        let mut chars = line.chars();
        let x = chars.next().unwrap_or(' ');
        let y = chars.next().unwrap_or(' ');
        if x == '?' && y == '?' {
            untracked += 1;
        } else {
            if x != ' ' && x != '?' {
                staged += 1;
            }
            if y != ' ' && y != '?' {
                modified += 1;
            }
        }
    }

    let (ahead, behind) = Command::new("git")
        .args(["rev-list", "--left-right", "--count", "@{u}...HEAD"])
        .current_dir(cwd)
        .output()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout).to_string();
            let v: Vec<&str> = s.split_whitespace().collect();
            if v.len() == 2 {
                (v[1].parse().unwrap_or(0), v[0].parse().unwrap_or(0))
            } else {
                (0, 0)
            }
        })
        .unwrap_or((0, 0));

    Some(GitInfo { branch, staged, modified, untracked, ahead, behind })
}
