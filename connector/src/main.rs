//! spark-connector: join | run | status. See lib.rs for the node layout.

use serde_json::Value;
use spark_connector::gates;
use spark_connector::node::{
    executor_id, prefix, registration_block, valid_pubkey, valid_repo, valid_sha, Config, Home,
    RECEIPTS_REPO,
};
use spark_connector::procs;
use spark_connector::receipts;
use spark_connector::sh;
use std::fs::{self, File, OpenOptions};
use std::io::Write;

use std::process::{Command, Stdio};
use std::time::Duration;

const USAGE: &str = "usage:
  spark-connector join   [--home DIR] [--name NAME] [--fork OWNER/aien-receipts] [--judge-commit SHA40] [--judge-src DIR]
  spark-connector run    [--home DIR] [--once REPO SHA] [--iterations N] [--no-publish] [--rejudge] [--project-spark-test]
  spark-connector status [--home DIR]
The node home defaults to $SPARK_CONNECTOR_HOME or ~/.spark-connector.";

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn has(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn utc_now() -> String {
    sh::run(None, "date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("join") => report(join(&args[1..])),
        Some("run") => report(run(&args[1..])),
        Some("status") => report(status(&args[1..])),
        _ => {
            eprintln!("{USAGE}");
            2
        }
    };
    std::process::exit(code);
}

fn report(r: Result<(), String>) -> i32 {
    match r {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("spark-connector: {e}");
            1
        }
    }
}

fn read_pubkey(home: &Home) -> Result<String, String> {
    let pk = fs::read_to_string(home.pubkey()).map_err(|e| {
        format!(
            "cannot read {}: {e} (run spark-connector join)",
            home.pubkey().display()
        )
    })?;
    let pk = pk.trim().to_string();
    if !valid_pubkey(&pk) {
        return Err(format!(
            "{} is not a 64-hex public key",
            home.pubkey().display()
        ));
    }
    Ok(pk)
}

// ---------------------------------------------------------------- join

fn join(args: &[String]) -> Result<(), String> {
    let home = Home::resolve(flag(args, "--home").as_deref())?;
    home.make_dirs()?;
    let operator = sh::run(None, "gh", &["api", "user", "--jq", ".login"])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    // Settings: written once; join never rewrites an existing node.conf.
    if !home.conf().exists() {
        let name = flag(args, "--name").unwrap_or_else(|| {
            let h = sh::run(None, "hostname", &[]).unwrap_or_default();
            format!("spark-{}", h.trim())
        });
        let fork = flag(args, "--fork").unwrap_or_else(|| format!("{operator}/{RECEIPTS_REPO}"));
        let commit = flag(args, "--judge-commit").ok_or(
            "first join needs --judge-commit SHA40 (install.sh passes the pinned judge commit)",
        )?;
        let src = flag(args, "--judge-src")
            .unwrap_or_else(|| home.root.join("judge").display().to_string());
        let text = format!(
            "# spark-connector node settings (written by join {})\nname={name}\nfork={fork}\npoll=300\njudge_src={src}\njudge_commit={commit}\n",
            utc_now()
        );
        Config::parse(&text, &home)?;
        fs::write(home.conf(), text).map_err(|e| format!("cannot write node.conf: {e}"))?;
    }
    let cfg = Config::load(&home)?;
    // The key: made once by the judge's own keygen (same ed25519 as every bundle).
    if !home.key().exists() {
        let k = home.key().display().to_string();
        let out = sh::run(None, &cfg.judge_bin.display().to_string(), &["keygen", &k])?;
        let pk = out
            .lines()
            .find_map(|l| l.strip_prefix("pubkey "))
            .ok_or("aien-test keygen printed no pubkey line")?
            .trim()
            .to_string();
        fs::write(home.pubkey(), format!("{pk}\n"))
            .map_err(|e| format!("cannot write node.pub: {e}"))?;
    }
    let pk = read_pubkey(&home)?;
    let gpu = sh::run(None, "nvidia-smi", &["-L"])
        .ok()
        .and_then(|s| {
            s.lines().find_map(|l| {
                l.split("UUID: ")
                    .nth(1)
                    .map(|u| u.trim_end_matches(')').to_string())
            })
        })
        .unwrap_or_else(|| "unknown (nvidia-smi -L gave no UUID)".to_string());
    let date = utc_now().get(..10).unwrap_or("").to_string();
    let block = registration_block(&cfg.name, &pk, &operator, &gpu, &cfg.judge_commit, &date)?;
    println!("node home   {}", home.root.display());
    println!("public key  {pk}");
    println!("executor id {}", executor_id(&pk)?);
    println!(
        "receipts    <repo>/<sha>/bundle.{}.json via {}",
        prefix(&pk),
        cfg.fork
    );
    println!();
    println!("Registration request: add this block to NODES.md in aien-dev/aien-receipts with a pull request.");
    println!("Until Drake merges it, this node's receipts are advisory only.");
    println!();
    print!("{block}");
    Ok(())
}

// ---------------------------------------------------------------- run

/// What happened to one head (printed with Debug in the node log).
#[derive(Debug)]
#[allow(dead_code)] // the fields are read through Debug only
enum Outcome {
    /// Another judge on this machine is on the same head: try a later round.
    Busy(Vec<u32>),
    /// Not judged by this node, for good (why).
    Skipped(String),
    /// Judged: done line written; publication result.
    Judged(String),
}

fn run(args: &[String]) -> Result<(), String> {
    let home = Home::resolve(flag(args, "--home").as_deref())?;
    let cfg = Config::load(&home)?;
    home.make_dirs()?;
    let pk = read_pubkey(&home)?;
    // One node = one poller: the lock is held for the whole run.
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(home.lock())
        .map_err(|e| format!("cannot open {}: {e}", home.lock().display()))?;
    if lock.try_lock().is_err() {
        return Err(format!(
            "another spark-connector run already holds {}; one node runs one poller",
            home.lock().display()
        ));
    }
    let _ = lock.set_len(0);
    let _ = writeln!(&lock, "{} {}", std::process::id(), utc_now());
    check_judge(&cfg)?;
    check_token(&cfg, has(args, "--project-spark-test"))?;
    let publish = !has(args, "--no-publish");

    if let Some(i) = args.iter().position(|a| a == "--once") {
        let repo = args.get(i + 1).ok_or("--once needs REPO SHA")?;
        let sha = args.get(i + 2).ok_or("--once needs REPO SHA")?;
        // A head this node already judged is not judged again: finish its
        // publication if one is pending, else show the earlier result.
        if home.done(repo, sha).exists() && !has(args, "--rejudge") {
            let done = fs::read_to_string(home.done(repo, sha)).unwrap_or_default();
            println!("{repo} {sha}: already judged by this node: {}", done.trim());
            if publish && home.outbox(repo, sha).exists() {
                println!("{repo} {sha}: {}", publish_one(&home, &cfg, &pk, repo, sha));
            }
            return Ok(());
        }
        let o = judge_head(&home, &cfg, &pk, repo, sha, None, publish)?;
        println!("{repo} {sha}: {o:?}");
        return match o {
            Outcome::Judged(_) => Ok(()),
            other => Err(format!("{repo} {sha} not judged: {other:?}")),
        };
    }
    let iterations: u64 = flag(args, "--iterations")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut round = 0u64;
    loop {
        round += 1;
        if publish {
            flush_outbox(&home, &cfg, &pk);
        }
        for repo in &cfg.repos {
            let heads = match open_heads(&cfg.owner, repo) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("spark-connector: {repo}: cannot list pull requests: {e}");
                    continue;
                }
            };
            for (number, sha) in heads {
                if home.done(repo, &sha).exists() {
                    continue;
                }
                match judge_head(&home, &cfg, &pk, repo, &sha, Some(number), publish) {
                    Ok(o) => println!("{} {repo} #{number} {sha}: {o:?}", utc_now()),
                    Err(e) => eprintln!("spark-connector: {repo} {sha}: {e}"),
                }
            }
        }
        if iterations != 0 && round >= iterations {
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(cfg.poll));
    }
}

/// The judge must be the pinned commit, with a clean tree.
fn check_judge(cfg: &Config) -> Result<(), String> {
    let head = sh::git(&cfg.judge_src, &["rev-parse", "HEAD"])?;
    if head.trim() != cfg.judge_commit {
        return Err(format!(
            "judge source {} is at {}, node.conf pins {}; re-run install.sh",
            cfg.judge_src.display(),
            head.trim(),
            cfg.judge_commit
        ));
    }
    let dirty = sh::git(
        &cfg.judge_src,
        &["status", "--porcelain", "--untracked-files=no"],
    )?;
    if !dirty.trim().is_empty() {
        return Err(format!(
            "judge source {} has local changes; re-run install.sh",
            cfg.judge_src.display()
        ));
    }
    if !cfg.judge_bin.is_file() {
        return Err(format!(
            "judge binary {} missing; re-run install.sh",
            cfg.judge_bin.display()
        ));
    }
    Ok(())
}

/// The node's GitHub token must not be able to push to any project
/// repository: it writes only to the node's own fork.
fn check_token(cfg: &Config, project_spark_test: bool) -> Result<(), String> {
    let mut wide = Vec::new();
    for r in cfg.repos.iter().map(String::as_str).chain([RECEIPTS_REPO]) {
        let p = sh::run(
            None,
            "gh",
            &[
                "api",
                &format!("repos/{}/{r}", cfg.owner),
                "--jq",
                ".permissions.push // false",
            ],
        )?;
        if p.trim() == "true" {
            wide.push(r.to_string());
        }
    }
    if wide.is_empty() {
        eprintln!(
            "spark-connector: token check: no push access to any {} repository (fork-only token)",
            cfg.owner
        );
        return Ok(());
    }
    if project_spark_test {
        eprintln!(
            "spark-connector: token check: WIDE TOKEN (push to {}) accepted only because --project-spark-test was given; the node still writes only to {}",
            wide.join(","),
            cfg.fork
        );
        return Ok(());
    }
    Err(format!(
        "the GitHub token can push to {}/{{{}}}; a node needs a token that can write only to its fork {} (see README, Run a node)",
        cfg.owner,
        wide.join(","),
        cfg.fork
    ))
}

/// Open pull request heads of one repository: (number, sha).
fn open_heads(owner: &str, repo: &str) -> Result<Vec<(u64, String)>, String> {
    let out = sh::run(
        None,
        "gh",
        &[
            "api",
            &format!("repos/{owner}/{repo}/pulls?state=open&per_page=50"),
            "--jq",
            ".[] | \"\\(.number) \\(.head.sha)\"",
        ],
    )?;
    Ok(out
        .lines()
        .filter_map(|l| {
            let (n, s) = l.trim().split_once(' ')?;
            let s = s.to_ascii_lowercase();
            (valid_sha(&s))
                .then(|| (n.parse().ok(), s))
                .and_then(|(n, s)| Some((n?, s)))
        })
        .collect())
}

/// Clone (once) or fetch the node's trusted copy of `repo`.
fn ensure_clone(home: &Home, owner: &str, repo: &str) -> Result<(), String> {
    let dir = home.clones().join(repo);
    if !dir.join(".git").exists() {
        let url = format!("https://github.com/{owner}/{repo}.git");
        sh::run(
            None,
            "git",
            &["clone", "-q", &url, &dir.display().to_string()],
        )?;
    } else {
        sh::git(&dir, &["fetch", "-q", "origin"])?;
    }
    Ok(())
}

/// The repositories whose checkouts the judge places beside `repo`
/// (the sibling map and pins of the pinned judge).
fn neighbours(cfg: &Config, repo: &str) -> Vec<String> {
    let mut out = vec!["aien-architecture".to_string()];
    let dir = cfg.judge_src.join("tools/aien-local-ci");
    for (file, col) in [("siblings", 2usize), ("pins", 1usize)] {
        if let Ok(t) = fs::read_to_string(dir.join(file)) {
            for l in t
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
            {
                let f: Vec<&str> = l.split_whitespace().collect();
                if f.first() == Some(&repo) && f.len() > col && valid_repo(f[col]) {
                    out.push(f[col].to_string());
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out.retain(|r| r != repo);
    out
}

fn judge_head(
    home: &Home,
    cfg: &Config,
    pk: &str,
    repo: &str,
    sha: &str,
    pr: Option<u64>,
    publish: bool,
) -> Result<Outcome, String> {
    if !valid_repo(repo) || !cfg.repos.iter().any(|r| r == repo) {
        return Err(format!("{repo:?} is not in this node's repos list"));
    }
    if !valid_sha(sha) {
        return Err(format!("{sha:?} is not a 40-hex commit"));
    }
    let busy = procs::judges_of(repo, sha);
    if !busy.is_empty() {
        return Ok(Outcome::Busy(busy));
    }
    // Trusted side: fetch the head; the chip check reads its gate files first,
    // so a head the node will not judge costs no neighbour clones.
    ensure_clone(home, &cfg.owner, repo)?;
    let clone = home.clones().join(repo);
    let refspec = match pr {
        Some(n) => format!("+refs/pull/{n}/head:refs/remotes/origin/pr/{n}"),
        None => sha.to_string(),
    };
    sh::git(&clone, &["fetch", "-q", "origin", &refspec])?;
    sh::git(&clone, &["cat-file", "-e", &format!("{sha}^{{commit}}")])?;
    let chip = gates::chip_gates(&clone, sha)?;
    if !chip.is_empty() {
        let why = format!(
            "chip gates {} (v0 nodes run no-network checks only)",
            chip.join(",")
        );
        write_done(home, repo, sha, &format!("SKIPPED {why}"))?;
        return Ok(Outcome::Skipped(why));
    }
    for n in neighbours(cfg, repo) {
        ensure_clone(home, &cfg.owner, &n)?;
    }

    // The judge, exactly as the project's Spark runs it, with the node's key
    // and state; every check runs as aien-run through aien-run-exec.
    let src = cfg.judge_src.join("tools/aien-local-ci");
    let log_path = home.log(repo, sha);
    let log = File::create(&log_path)
        .map_err(|e| format!("cannot create {}: {e}", log_path.display()))?;
    let judge = cfg.judge_bin.display().to_string();
    let state = home.judge_state().display().to_string();
    let clones = home.clones().display().to_string();
    let key = home.key().display().to_string();
    let wt = src.join("wt.sh").display().to_string();
    let sibs = src.join("siblings").display().to_string();
    let argv = [
        "-n",
        "19",
        "ionice",
        "-c3",
        &judge,
        "worker",
        "--once",
        repo,
        sha,
        "--state-dir",
        &state,
        "--clones",
        &clones,
        "--key",
        &key,
        "--sandbox",
        "user",
        "--wt-tool",
        &wt,
        "--siblings",
        &sibs,
    ];
    eprintln!(
        "spark-connector: $ nice {} (log {})",
        argv.join(" "),
        log_path.display()
    );
    let status = Command::new("nice")
        .args(argv)
        .stdin(Stdio::null())
        .stdout(log.try_clone().map_err(|e| e.to_string())?)
        .stderr(log)
        .status()
        .map_err(|e| format!("cannot start the judge: {e}"))?;
    let raw = home.judge_bundle(repo, sha);
    if !raw.is_file() {
        return Err(format!(
            "the judge wrote no bundle ({status}); see {}",
            log_path.display()
        ));
    }

    // Stamp the judge commit into the executor block and re-sign with the
    // node key (aien-test bundle-sign: same canonical bytes and ed25519).
    let mut b: Value = serde_json::from_str(
        &fs::read_to_string(&raw).map_err(|e| format!("cannot read {}: {e}", raw.display()))?,
    )
    .map_err(|e| format!("{} is not JSON: {e}", raw.display()))?;
    let exec = executor_id(pk)?;
    if b["executor"]["id"].as_str() != Some(exec.as_str()) {
        return Err(format!(
            "{} was not signed by this node's key",
            raw.display()
        ));
    }
    b["executor"]["judge_commit"] = Value::String(cfg.judge_commit.clone());
    let signed = home.signed(repo, sha);
    let unsigned = signed.with_extension("unsigned.json");
    fs::write(
        &unsigned,
        serde_json::to_vec(&b).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("cannot write {}: {e}", unsigned.display()))?;
    let s = signed.display().to_string();
    sh::run(
        None,
        &judge,
        &[
            "bundle-sign",
            &unsigned.display().to_string(),
            "--key",
            &key,
            "--out",
            &s,
        ],
    )?;
    let _ = fs::remove_file(&unsigned);

    // Independent check with the node's public key and the base's always list.
    let always = sh::git(&clone, &["show", "origin/main:.aien/always"]).unwrap_or_default();
    let require: Vec<&str> = always
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    let repository = format!("github.com/{}/{repo}", cfg.owner);
    let mut vargs = vec![
        "verify-bundle",
        s.as_str(),
        "--repo",
        repository.as_str(),
        "--commit",
        sha,
        "--pubkey",
        pk,
    ];
    let req = require.join(",");
    if !req.is_empty() {
        vargs.extend(["--require", req.as_str()]);
    }
    let v = Command::new(&judge)
        .args(&vargs)
        .output()
        .map_err(|e| format!("cannot run verify-bundle: {e}"))?;
    let vline = String::from_utf8_lossy(&v.stdout).trim().to_string();
    let vcode = v.status.code().unwrap_or(-1);
    let verdict = b["verdict"].as_str().unwrap_or("?").to_string();
    let line = format!(
        "{} {verdict} verify-bundle exit {vcode}: {vline} | judge exit {status} | signed {s}",
        utc_now()
    );
    write_done(home, repo, sha, &line)?;
    if vcode == 3 {
        return Err(format!(
            "verify-bundle REJECTED this node's own bundle: {vline}"
        ));
    }
    if !publish {
        return Ok(Outcome::Judged(format!(
            "{verdict}; not published (--no-publish)"
        )));
    }
    fs::write(home.outbox(repo, sha), format!("{verdict}\n")).map_err(|e| e.to_string())?;
    Ok(Outcome::Judged(publish_one(home, cfg, pk, repo, sha)))
}

fn write_done(home: &Home, repo: &str, sha: &str, line: &str) -> Result<(), String> {
    fs::write(home.done(repo, sha), format!("{line}\n"))
        .map_err(|e| format!("cannot write done marker: {e}"))
}

/// Publish one judged head from the outbox; returns what happened.
fn publish_one(home: &Home, cfg: &Config, pk: &str, repo: &str, sha: &str) -> String {
    let verdict = fs::read_to_string(home.outbox(repo, sha)).unwrap_or_default();
    let bytes = match fs::read(home.signed(repo, sha)) {
        Ok(b) => b,
        Err(e) => return format!("publish failed: cannot read signed bundle: {e}"),
    };
    match receipts::publish(home, cfg, repo, sha, prefix(pk), &bytes, verdict.trim()) {
        Ok(p) => {
            let _ = fs::write(home.published(repo, sha), format!("{} {p:?}\n", utc_now()));
            let _ = fs::remove_file(home.outbox(repo, sha));
            format!("published {p:?}")
        }
        Err(e) => format!("publish failed, kept in outbox: {e}"),
    }
}

fn flush_outbox(home: &Home, cfg: &Config, pk: &str) {
    let Ok(rd) = fs::read_dir(home.root.join("outbox")) else {
        return;
    };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if let Some((repo, sha)) = name.rsplit_once('.') {
            if valid_repo(repo) && valid_sha(sha) {
                println!(
                    "{} outbox {repo} {sha}: {}",
                    utc_now(),
                    publish_one(home, cfg, pk, repo, sha)
                );
            }
        }
    }
}

// ---------------------------------------------------------------- status

fn status(args: &[String]) -> Result<(), String> {
    let home = Home::resolve(flag(args, "--home").as_deref())?;
    let cfg = Config::load(&home)?;
    let pk = read_pubkey(&home)?;
    println!("node        {} ({})", cfg.name, home.root.display());
    println!("public key  {pk}");
    println!("executor id {}", executor_id(&pk)?);
    println!(
        "judge       {} at {}",
        cfg.judge_bin.display(),
        cfg.judge_commit
    );
    println!(
        "publishes   <repo>/<sha>/bundle.{}.json via {}",
        prefix(&pk),
        cfg.fork
    );
    let poller = match File::open(home.lock()) {
        Ok(f) => match f.try_lock_shared() {
            Ok(()) => "not running".to_string(),
            Err(_) => format!(
                "running ({})",
                fs::read_to_string(home.lock()).unwrap_or_default().trim()
            ),
        },
        Err(_) => "never started".to_string(),
    };
    println!("poller      {poller}");
    for (label, dir) in [
        ("judged", "done"),
        ("to publish", "outbox"),
        ("published", "published"),
    ] {
        let mut items: Vec<(std::time::SystemTime, String)> = fs::read_dir(home.root.join(dir))
            .map(|rd| {
                rd.flatten()
                    .filter_map(|e| {
                        let t = e.metadata().ok()?.modified().ok()?;
                        let body = fs::read_to_string(e.path()).unwrap_or_default();
                        Some((
                            t,
                            format!("{}  {}", e.file_name().to_string_lossy(), body.trim()),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        items.sort();
        println!("{label:<11} {}", items.len());
        for (_, l) in items.iter().rev().take(5) {
            println!("  {}", l.chars().take(240).collect::<String>());
        }
    }
    Ok(())
}
