//! Running other programs. Every call is logged to stderr with its arguments,
//! so the node log shows exactly what the node did.

use std::path::Path;
use std::process::{Command, Stdio};

/// Run `program args` (in `cwd` when given); stdout on success, an error with
/// stderr otherwise.
pub fn run(cwd: Option<&Path>, program: &str, args: &[&str]) -> Result<String, String> {
    eprintln!("spark-connector: $ {program} {}", args.join(" "));
    let mut cmd = Command::new(program);
    cmd.args(args).stdin(Stdio::null());
    if let Some(d) = cwd {
        cmd.current_dir(d);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("cannot run {program}: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(format!(
            "{program} {} failed ({}): {}",
            args.first().copied().unwrap_or(""),
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// `git -C dir args`.
pub fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let d = dir.display().to_string();
    let mut all = vec!["-C", d.as_str()];
    all.extend_from_slice(args);
    run(None, "git", &all)
}
