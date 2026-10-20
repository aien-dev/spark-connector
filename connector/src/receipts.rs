//! Publishing a node's receipt to aien-receipts, through the node's own fork.
//!
//! The shelf layout is `<repo>/<sha>/bundle.json` for the project's Spark.
//! A node adds `<repo>/<sha>/bundle.<prefix>.json` beside it (prefix = first
//! 16 hex of the node's public key), so one commit can carry any number of
//! independent receipts and a node can never overwrite the project's. The
//! node commits on a branch of its own clone, pushes the branch to its fork
//! (the only place its GitHub token can write) and opens a pull request
//! against aien-dev/aien-receipts.

use crate::node::{valid_repo, valid_sha, Config, Home, RECEIPTS_REPO};
use crate::sh;
use std::fs;

/// The shelf path of a node receipt. Refuses anything that could name the
/// project's own `bundle.json` or leave the `<repo>/<sha>/` directory.
pub fn receipt_path(repo: &str, sha: &str, prefix: &str) -> Result<String, String> {
    if !valid_repo(repo) || !valid_sha(sha) {
        return Err(format!("refusing receipt path for {repo:?} {sha:?}"));
    }
    if prefix.len() != 16
        || !prefix
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(format!(
            "node prefix must be 16 lowercase hex, got {prefix:?}"
        ));
    }
    Ok(format!("{repo}/{sha}/bundle.{prefix}.json"))
}

/// The GitHub page that opens a pull request from the fork branch.
pub fn compare_url(owner: &str, fork_owner: &str, branch: &str) -> String {
    format!("https://github.com/{owner}/{RECEIPTS_REPO}/compare/main...{fork_owner}:{RECEIPTS_REPO}:{branch}?expand=1")
}

/// The branch a receipt travels on in the fork.
pub fn branch_name(repo: &str, sha: &str, prefix: &str) -> String {
    format!("node-{prefix}/{repo}-{}", &sha[..12])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Published {
    /// The same bytes are already on the shelf's main branch.
    AlreadyOnShelf,
    /// A pull request carries it (URL).
    PullRequest(String),
    /// Pushed to the fork; the pull request could not be opened. A token
    /// limited to the fork cannot open pull requests on aien-dev (GitHub:
    /// fine-grained tokens do not support contributing to public repositories
    /// the user is not a member of), so this is the normal end state for such
    /// a node: the receipt waits on the fork branch, and `compare` is the link
    /// that opens the pull request in a browser.
    PushedOnly { compare: String, why: String },
}

fn ensure_clone(home: &Home, cfg: &Config) -> Result<(), String> {
    let dir = home.receipts();
    if !dir.join(".git").exists() {
        let url = format!("https://github.com/{}/{RECEIPTS_REPO}.git", cfg.owner);
        let d = dir.display().to_string();
        sh::run(None, "git", &["clone", "-q", &url, &d])?;
    }
    // origin = the project's shelf (read only for us); fork = where we push.
    let have = sh::git(&dir, &["remote"])?;
    if !have.lines().any(|l| l == "fork") {
        sh::git(&dir, &["remote", "add", "fork", &cfg.fork_url])?;
    } else {
        sh::git(&dir, &["remote", "set-url", "fork", &cfg.fork_url])?;
    }
    sh::git(&dir, &["fetch", "-q", "origin", "main"])?;
    Ok(())
}

/// Publish `bytes` (a signed bundle) as this node's receipt for `repo` `sha`.
pub fn publish(
    home: &Home,
    cfg: &Config,
    repo: &str,
    sha: &str,
    prefix: &str,
    bytes: &[u8],
    title_note: &str,
) -> Result<Published, String> {
    let rel = receipt_path(repo, sha, prefix)?;
    ensure_clone(home, cfg)?;
    let dir = home.receipts();
    // Already on the shelf? Same bytes: done. Different bytes: never rewrite.
    let spec = format!("origin/main:{rel}");
    if sh::git(&dir, &["cat-file", "-e", &spec]).is_ok() {
        let there = std::process::Command::new("git")
            .args(["-C", &dir.display().to_string(), "show", &spec])
            .output()
            .map_err(|e| format!("cannot read {spec}: {e}"))?
            .stdout;
        return if there == bytes {
            Ok(Published::AlreadyOnShelf)
        } else {
            Err(format!(
                "{rel} is already on the shelf with different bytes; the shelf is append only, nothing published"
            ))
        };
    }
    let branch = branch_name(repo, sha, prefix);
    sh::git(&dir, &["checkout", "-q", "-B", &branch, "origin/main"])?;
    let file = dir.join(&rel);
    fs::create_dir_all(file.parent().unwrap_or(&dir))
        .map_err(|e| format!("cannot create {}: {e}", file.display()))?;
    fs::write(&file, bytes).map_err(|e| format!("cannot write {}: {e}", file.display()))?;
    sh::git(&dir, &["add", "--", &rel])?;
    // Only this one file may change (the project's bundle.json is never touched).
    let staged = sh::git(&dir, &["diff", "--cached", "--name-only"])?;
    if staged.trim() != rel {
        return Err(format!(
            "refusing to commit: staged {staged:?}, expected only {rel}"
        ));
    }
    let who = format!("user.name=spark-connector {}", cfg.name);
    let mail = format!("user.email={prefix}@nodes.spark-connector.invalid");
    let msg = format!("node {prefix}: {repo} {sha} {title_note}");
    sh::git(&dir, &["-c", &who, "-c", &mail, "commit", "-q", "-m", &msg])?;
    sh::git(
        &dir,
        &[
            "push",
            "-q",
            "--force",
            "fork",
            &format!("{branch}:{branch}"),
        ],
    )?;
    sh::git(&dir, &["checkout", "-q", "--detach", "origin/main"])?;

    let upstream = format!("{}/{RECEIPTS_REPO}", cfg.owner);
    let head = format!("{}:{branch}", cfg.fork_owner());
    if let Ok(url) = sh::run(
        None,
        "gh",
        &[
            "pr",
            "list",
            "-R",
            &upstream,
            "--head",
            &branch,
            "--state",
            "open",
            "--json",
            "url",
            "--jq",
            ".[0].url // empty",
        ],
    ) {
        if !url.trim().is_empty() {
            return Ok(Published::PullRequest(url.trim().to_string()));
        }
    }
    let body = format!(
        "Community node receipt (spark-connector).\n\n- file: `{rel}` (new file; the project's `bundle.json` is not touched)\n- node: {} (prefix {prefix})\n- judge commit: {}\n- result: {title_note}\n\nCheck it yourself: `aien-test verify-bundle {rel} --repo github.com/{}/{repo} --commit {sha} --pubkey <the node's key from NODES.md>`. Until the node is in NODES.md this receipt is advisory only.\n",
        cfg.name, cfg.judge_commit, cfg.owner
    );
    match sh::run(
        None,
        "gh",
        &[
            "pr", "create", "-R", &upstream, "--base", "main", "--head", &head, "--title", &msg,
            "--body", &body,
        ],
    ) {
        Ok(out) => Ok(Published::PullRequest(
            out.lines().last().unwrap_or("").trim().to_string(),
        )),
        Err(e) => Ok(Published::PushedOnly {
            compare: compare_url(&cfg.owner, cfg.fork_owner(), &branch),
            why: e.lines().last().unwrap_or("").chars().take(200).collect(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const S: &str = "bfa6b636f33f44a68a48d8f560c069fd1f99882d";

    #[test]
    fn receipt_path_never_names_the_project_bundle() {
        assert_eq!(
            receipt_path("aegis-runtime", S, "0123456789abcdef").unwrap(),
            format!("aegis-runtime/{S}/bundle.0123456789abcdef.json")
        );
        assert!(receipt_path("aegis-runtime", S, "").is_err());
        assert!(receipt_path("aegis-runtime", S, "0123456789ABCDEF").is_err());
        assert!(receipt_path("aegis-runtime", S, "../../../bundle").is_err());
        assert!(receipt_path("../x", S, "0123456789abcdef").is_err());
        assert!(receipt_path("aegis-runtime", "abc", "0123456789abcdef").is_err());
        assert_eq!(
            branch_name("aegis-runtime", S, "0123456789abcdef"),
            "node-0123456789abcdef/aegis-runtime-bfa6b636f33f"
        );
        assert_eq!(
            compare_url("aien-dev", "alice", "node-0123456789abcdef/aegis-runtime-bfa6b636f33f"),
            "https://github.com/aien-dev/aien-receipts/compare/main...alice:aien-receipts:node-0123456789abcdef/aegis-runtime-bfa6b636f33f?expand=1"
        );
    }
}
