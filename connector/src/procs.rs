//! Is another judge already working on this head on this machine?
//!
//! The judge keeps every head's worktree at the fixed path
//! `/var/lib/aien-run/wt/<repo>/<sha>` and removes a leftover one before it
//! starts. Two judges on one head on one machine (for example the project's
//! own worker and a node on the project's Spark) would delete each other's
//! checkout. So before starting, the node looks through /proc for any
//! `aien-test worker --once <repo> <sha>` and waits for a later round if one
//! is running.

use std::fs;

/// True when `argv` is `... worker ... --once REPO SHA ...`.
pub fn is_judge_of(argv: &[String], repo: &str, sha: &str) -> bool {
    argv.iter().any(|a| a == "worker")
        && argv
            .windows(3)
            .any(|w| w[0] == "--once" && w[1] == repo && w[2] == sha)
}

/// The process ids of judges already running on `repo` `sha`.
pub fn judges_of(repo: &str, sha: &str) -> Vec<u32> {
    let mut found = Vec::new();
    let Ok(rd) = fs::read_dir("/proc") else {
        return found;
    };
    for e in rd.flatten() {
        let Some(pid) = e.file_name().to_str().and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        let Ok(raw) = fs::read(e.path().join("cmdline")) else {
            continue;
        };
        let argv: Vec<String> = raw
            .split(|b| *b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .collect();
        if is_judge_of(&argv, repo, sha) {
            found.push(pid);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Vec<String> {
        s.split(' ').map(String::from).collect()
    }

    #[test]
    fn matches_only_the_same_head() {
        let a = v("aien-test worker --once omega abc --state-dir /s");
        assert!(is_judge_of(&a, "omega", "abc"));
        assert!(!is_judge_of(&a, "omega", "abd"));
        assert!(!is_judge_of(&a, "physics", "abc"));
        assert!(!is_judge_of(
            &v("aien-test verify abc --repo omega"),
            "omega",
            "abc"
        ));
    }
}
