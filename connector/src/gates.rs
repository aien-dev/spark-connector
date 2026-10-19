//! Which gates of a head need the chip.
//!
//! The sandbox shim gives every `host` and `qemu` check a fresh network
//! namespace with only loopback, but `gb10` checks keep the host network
//! (they share the chip tooling's /tmp and IPC). A volunteer node promises
//! "no network during the run", so v0 refuses any head whose gate files
//! require `gb10` or `nvrm`: those heads stay with the project's Spark.
//! The scan reads the gate files straight from the git object store at the
//! head (nothing is checked out or run).

use crate::sh;
use std::path::Path;

/// The pools a gate file names under `requires:`, either as a block list
/// (`requires:` then `  - host` lines) or inline (`requires: [gb10, host]`).
pub fn requires(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_block = false;
    for line in text.lines() {
        let t = line.trim_end();
        if let Some(rest) = t.strip_prefix("requires:") {
            let rest = rest.trim();
            if rest.is_empty() {
                in_block = true;
            } else {
                in_block = false;
                let inner = rest.trim_start_matches('[').trim_end_matches(']');
                out.extend(
                    inner
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty()),
                );
            }
            continue;
        }
        if in_block {
            let s = t.trim_start();
            if let Some(item) = s.strip_prefix("- ") {
                out.push(item.trim().trim_matches('"').to_string());
            } else if !s.is_empty() && !s.starts_with('#') {
                in_block = false;
            }
        }
    }
    out
}

/// True when a gate file needs the chip (`gb10` or `nvrm`).
pub fn needs_chip(text: &str) -> bool {
    requires(text).iter().any(|r| r == "gb10" || r == "nvrm")
}

/// The gate name (`gate: NAME`) of a gate file, or the path when missing.
pub fn gate_name(text: &str, path: &str) -> String {
    text.lines()
        .find_map(|l| l.strip_prefix("gate:").map(|n| n.trim().to_string()))
        .unwrap_or_else(|| path.to_string())
}

/// The chip gates in the tree of `sha` in `clone` (empty: the node may judge it).
pub fn chip_gates(clone: &Path, sha: &str) -> Result<Vec<String>, String> {
    let list = sh::git(clone, &["ls-tree", "-r", "--name-only", sha])?;
    let mut chip = Vec::new();
    for path in list.lines().filter(|p| p.ends_with(".gate")) {
        let text = sh::git(clone, &["show", &format!("{sha}:{path}")])?;
        if needs_chip(&text) {
            chip.push(gate_name(&text, path));
        }
    }
    Ok(chip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_list_host_only_is_not_chip() {
        let g = "gate: AEGIS-TEST\nmanifest_version: 2\nrequires:\n  - host\ninputs:\n  - src/\n";
        assert_eq!(requires(g), vec!["host"]);
        assert!(!needs_chip(g));
        assert_eq!(gate_name(g, "x.gate"), "AEGIS-TEST");
    }

    #[test]
    fn block_list_with_gb10_is_chip_and_inputs_are_not_pools() {
        let g = "gate: E2E\nrequires:\n  - host\n  # the chip\n  - gb10\ninputs:\n  - nvrm/\n";
        assert_eq!(requires(g), vec!["host", "gb10"]);
        assert!(needs_chip(g));
    }

    #[test]
    fn inline_list_and_nvrm() {
        assert!(needs_chip("requires: [host, nvrm]\n"));
        assert!(!needs_chip("requires: [\"qemu\"]\n"));
        assert!(requires("gate: X\n").is_empty());
    }
}
