//! The node: its home directory, settings, identity and registration block.

use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// The repositories the project's own Spark judges (the `REPOS` list of
/// `aien-local-ci`, 2026-10-19).
pub const DEFAULT_REPOS: &str = "aien-sovereign-core,omega,aienos,physics,aien-protocols,aegis-runtime,spark-rsi,aien-architecture,benchmarks,synapse,interplane";
pub const DEFAULT_OWNER: &str = "aien-dev";
pub const RECEIPTS_REPO: &str = "aien-receipts";

/// Where everything of one node lives. One node = one home = one poller.
#[derive(Debug, Clone)]
pub struct Home {
    pub root: PathBuf,
}

impl Home {
    pub fn new(root: &Path) -> Home {
        Home {
            root: root.to_path_buf(),
        }
    }
    /// `--home DIR`, else `$SPARK_CONNECTOR_HOME`, else `~/.spark-connector`.
    pub fn resolve(flag: Option<&str>) -> Result<Home, String> {
        if let Some(d) = flag {
            return Ok(Home::new(Path::new(d)));
        }
        if let Ok(d) = std::env::var("SPARK_CONNECTOR_HOME") {
            if !d.is_empty() {
                return Ok(Home::new(Path::new(&d)));
            }
        }
        let h = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
        Ok(Home::new(&Path::new(&h).join(".spark-connector")))
    }
    pub fn key(&self) -> PathBuf {
        self.root.join("node.key")
    }
    pub fn pubkey(&self) -> PathBuf {
        self.root.join("node.pub")
    }
    pub fn conf(&self) -> PathBuf {
        self.root.join("node.conf")
    }
    pub fn lock(&self) -> PathBuf {
        self.root.join("run.lock")
    }
    pub fn judge_state(&self) -> PathBuf {
        self.root.join("judge-state")
    }
    pub fn clones(&self) -> PathBuf {
        self.root.join("clones")
    }
    pub fn receipts(&self) -> PathBuf {
        self.root.join("receipts")
    }
    pub fn signed(&self, repo: &str, sha: &str) -> PathBuf {
        self.root.join("signed").join(format!("{repo}.{sha}.json"))
    }
    pub fn log(&self, repo: &str, sha: &str) -> PathBuf {
        self.root.join("logs").join(format!("{repo}.{sha}.log"))
    }
    pub fn done(&self, repo: &str, sha: &str) -> PathBuf {
        self.root.join("done").join(format!("{repo}.{sha}"))
    }
    pub fn outbox(&self, repo: &str, sha: &str) -> PathBuf {
        self.root.join("outbox").join(format!("{repo}.{sha}"))
    }
    pub fn published(&self, repo: &str, sha: &str) -> PathBuf {
        self.root.join("published").join(format!("{repo}.{sha}"))
    }
    pub fn make_dirs(&self) -> Result<(), String> {
        for d in [
            "",
            "judge-state",
            "clones",
            "signed",
            "logs",
            "done",
            "outbox",
            "published",
        ] {
            let p = self.root.join(d);
            fs::create_dir_all(&p).map_err(|e| format!("cannot create {}: {e}", p.display()))?;
        }
        Ok(())
    }
    /// The judge's bundle for one head (its state layout, `worker.rs`).
    pub fn judge_bundle(&self, repo: &str, sha: &str) -> PathBuf {
        self.judge_state()
            .join("bundles")
            .join(repo)
            .join(sha)
            .join("bundle.json")
    }
}

/// `node.conf`: one `key=value` per line, `#` comments.
///
/// ```text
/// name=spark-alice-1                    node name for the registry
/// owner=aien-dev                        the GitHub organisation judged
/// repos=aien-protocols,aegis-runtime    repositories to poll (default: the project's list)
/// fork=alice/aien-receipts              the node's own fork of aien-receipts
/// fork_url=https://github.com/alice/aien-receipts.git   push URL (default from fork)
/// poll=300                              seconds between rounds
/// judge_src=/home/alice/.spark-connector/judge          aien-sovereign-core checkout at judge_commit
/// judge_commit=<40 hex>                 the commit the judge was built from
/// judge_bin=<judge_src>/target/release/aien-test
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub name: String,
    pub owner: String,
    pub repos: Vec<String>,
    pub fork: String,
    pub fork_url: String,
    pub poll: u64,
    pub judge_src: PathBuf,
    pub judge_commit: String,
    pub judge_bin: PathBuf,
}

impl Config {
    pub fn parse(text: &str, home: &Home) -> Result<Config, String> {
        let mut get = std::collections::BTreeMap::new();
        for (n, line) in text.lines().enumerate() {
            let l = line.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }
            let (k, v) = l
                .split_once('=')
                .ok_or_else(|| format!("node.conf line {}: expected key=value", n + 1))?;
            get.insert(k.trim().to_string(), v.trim().to_string());
        }
        let known = [
            "name",
            "owner",
            "repos",
            "fork",
            "fork_url",
            "poll",
            "judge_src",
            "judge_commit",
            "judge_bin",
        ];
        if let Some(k) = get.keys().find(|k| !known.contains(&k.as_str())) {
            return Err(format!("node.conf: unknown setting {k:?}"));
        }
        let s = |k: &str, d: &str| get.get(k).cloned().unwrap_or_else(|| d.to_string());
        let fork = s("fork", "");
        if !valid_slug(&fork) || !fork.ends_with(&format!("/{RECEIPTS_REPO}")) {
            return Err(format!(
                "node.conf: fork must be OWNER/{RECEIPTS_REPO} (your own fork), got {fork:?}"
            ));
        }
        let owner = s("owner", DEFAULT_OWNER);
        if fork.split('/').next() == Some(owner.as_str()) {
            return Err(format!(
                "node.conf: fork {fork} is the project's own receipts repository; a node publishes only through its own fork"
            ));
        }
        let repos: Vec<String> = s("repos", DEFAULT_REPOS)
            .split(',')
            .map(|r| r.trim().to_string())
            .filter(|r| !r.is_empty())
            .collect();
        if let Some(bad) = repos.iter().find(|r| !valid_repo(r)) {
            return Err(format!("node.conf: bad repository name {bad:?}"));
        }
        let poll: u64 = s("poll", "300")
            .parse()
            .map_err(|_| "node.conf: poll must be whole seconds".to_string())?;
        let judge_src = PathBuf::from(s(
            "judge_src",
            &home.root.join("judge").display().to_string(),
        ));
        let judge_commit = s("judge_commit", "");
        if !valid_sha(&judge_commit) {
            return Err("node.conf: judge_commit must be the 40-hex aien-sovereign-core commit the judge was built from (install.sh writes it)".to_string());
        }
        let judge_bin = PathBuf::from(s(
            "judge_bin",
            &judge_src
                .join("target/release/aien-test")
                .display()
                .to_string(),
        ));
        Ok(Config {
            name: s("name", "unnamed-node"),
            fork_url: s("fork_url", &format!("https://github.com/{fork}.git")),
            owner,
            repos,
            fork,
            poll: poll.max(30),
            judge_src,
            judge_commit,
            judge_bin,
        })
    }

    pub fn load(home: &Home) -> Result<Config, String> {
        let p = home.conf();
        let text = fs::read_to_string(&p).map_err(|e| {
            format!(
                "cannot read {}: {e} (run install.sh, or spark-connector join)",
                p.display()
            )
        })?;
        Config::parse(&text, home)
    }

    pub fn fork_owner(&self) -> &str {
        self.fork.split('/').next().unwrap_or("")
    }
}

/// A repository name the judge accepts (letters, digits, `.`, `_`, `-`, not
/// starting with a dot).
pub fn valid_repo(r: &str) -> bool {
    !r.is_empty()
        && r.len() <= 100
        && !r.starts_with('.')
        && r.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-')
}

/// `OWNER/REPO` with two valid names.
pub fn valid_slug(s: &str) -> bool {
    matches!(s.split_once('/'), Some((o, r)) if valid_repo(o) && valid_repo(r))
}

/// A full lowercase 40-hex commit id.
pub fn valid_sha(s: &str) -> bool {
    s.len() == 40
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// A 64-hex public key.
pub fn valid_pubkey(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// The executor id the judge writes into every bundle: sha256 of the 32 raw
/// public key bytes, hex (`bundle::fingerprint` in aien-test).
pub fn executor_id(pubkey_hex: &str) -> Result<String, String> {
    if !valid_pubkey(pubkey_hex) {
        return Err(format!("not a 64-hex public key: {pubkey_hex:?}"));
    }
    let bytes: Vec<u8> = (0..32)
        .map(|i| u8::from_str_radix(&pubkey_hex[2 * i..2 * i + 2], 16).unwrap_or(0))
        .collect();
    let d = Sha256::digest(&bytes);
    Ok(d.iter().map(|b| format!("{b:02x}")).collect())
}

/// The short name a node's receipts carry: the first 16 hex of its public key.
pub fn prefix(pubkey_hex: &str) -> &str {
    &pubkey_hex[..16.min(pubkey_hex.len())]
}

/// The registry block for trust/NODES.md (aien-receipts `NODES.md`), in the
/// exact shape the trust stream defines. `status` and `approved by` are left
/// for Drake.
pub fn registration_block(
    name: &str,
    pubkey_hex: &str,
    operator: &str,
    gpu: &str,
    judge_commit: &str,
    date: &str,
) -> Result<String, String> {
    let exec = executor_id(pubkey_hex)?;
    Ok(format!(
        "## {name}\n- status: (left for Drake)\n- public key: {pubkey_hex}\n- executor id: {exec}\n- operator: {operator}\n- gpu: {gpu}\n- judge commit: {judge_commit}\n- date added: {date}\n- approved by: (left for Drake)\n- notes: receipts published as <repo>/<sha>/bundle.{}.json\n",
        prefix(pubkey_hex)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const C: &str = "0123456789abcdef0123456789abcdef01234567";

    fn home() -> Home {
        Home::new(Path::new("/n"))
    }

    #[test]
    fn config_defaults_and_fork_rules() {
        let c = Config::parse(
            &format!("fork=alice/aien-receipts\njudge_commit={C}\n"),
            &home(),
        )
        .unwrap();
        assert_eq!(c.owner, "aien-dev");
        assert_eq!(c.repos.len(), 11);
        assert_eq!(c.fork_url, "https://github.com/alice/aien-receipts.git");
        assert_eq!(
            c.judge_bin,
            PathBuf::from("/n/judge/target/release/aien-test")
        );
        assert_eq!(c.fork_owner(), "alice");
        assert_eq!(c.poll, 300);
        // the project's own shelf is never a node's publish target
        assert!(Config::parse(
            &format!("fork=aien-dev/aien-receipts\njudge_commit={C}\n"),
            &home()
        )
        .is_err());
        assert!(Config::parse(&format!("fork=alice/other\njudge_commit={C}\n"), &home()).is_err());
        assert!(Config::parse("fork=alice/aien-receipts\n", &home()).is_err());
        assert!(Config::parse(
            &format!("fork=alice/aien-receipts\njudge_commit={C}\ntypo=1\n"),
            &home()
        )
        .is_err());
        assert!(Config::parse(
            &format!("fork=alice/aien-receipts\njudge_commit={C}\nrepos=ok,../x\n"),
            &home()
        )
        .is_err());
        let c = Config::parse(
            &format!("fork=alice/aien-receipts\njudge_commit={C}\npoll=1\nrepos=aegis-runtime\n"),
            &home(),
        )
        .unwrap();
        assert_eq!(
            (c.poll, c.repos.clone()),
            (30, vec!["aegis-runtime".to_string()])
        );
    }

    #[test]
    fn executor_id_is_sha256_of_key_bytes() {
        // sha256 of 32 zero bytes
        assert_eq!(
            executor_id(&"0".repeat(64)).unwrap(),
            "66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925"
        );
        assert!(executor_id("abc").is_err());
        assert!(executor_id(&"A".repeat(64)).is_err());
    }

    #[test]
    fn registration_block_has_every_registry_field() {
        let b =
            registration_block("n1", &"0".repeat(64), "alice", "GPU-1", C, "2026-10-20").unwrap();
        for f in [
            "## n1\n",
            "- status:",
            "- public key: 0000",
            "- executor id: 6668",
            "- operator: alice",
            "- gpu: GPU-1",
            "- judge commit: 0123",
            "- date added: 2026-10-20",
            "- approved by:",
            "bundle.0000000000000000.json",
        ] {
            assert!(b.contains(f), "{f} missing in\n{b}");
        }
    }
}
