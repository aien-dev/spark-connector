#!/usr/bin/env bash
# install.sh: set up a DGX Spark (Ubuntu, the NVIDIA DGX OS image) as a spark-connector node. Idempotent:
# every step checks before it acts, so running it again repairs or upgrades and changes nothing that is right.
#
#   connector/install.sh [--fork OWNER/aien-receipts] [--name NODE_NAME] [--start] [--reinstall-sandbox]
#
# Run it as the normal user who will own the node (not root). It asks for your sudo password once.
# What it does, in order:
#   1. system packages: git, gh (GitHub CLI), build tools, acl, iproute2 (apt, only what is missing)
#   2. Rust toolchain 1.98.1 through rustup (only if missing; the same toolchain the judge pins)
#   3. checks that gh is logged in (the token you made for your fork; see README "Run a node")
#   4. the judge: aien-sovereign-core at JUDGE_COMMIT below, checked out at <node home>/judge and built from source
#   5. the sandbox: user aien-run, /usr/local/sbin/aien-run-exec and its sudoers line, installed by the judge's
#      own tools/aien-local-ci/install.sh at that same commit (the project Spark's sandbox, byte for byte). Skipped
#      when the identical shim and the user are already there, unless --reinstall-sandbox.
#   6. the connector binary, built from this checkout, copied to ~/.local/bin
#   7. spark-connector join: the node key (made by the judge's own keygen), node.conf, and the registration block
#   8. with --start only: a cron @reboot line for spark-connector-supervise, and starts it now
# Node home: $SPARK_CONNECTOR_HOME or ~/.spark-connector. Nothing outside it, ~/.local/bin, the crontab (with
# --start) and the sandbox paths of step 5 is touched.
set -euo pipefail

# The judge every node runs: aien-sovereign-core main, 2026-10-19 (merge of #685). Bump with a pull request here.
JUDGE_COMMIT=10b4f851d6b082df6c2c477fbab700b551b854be
JUDGE_URL=https://github.com/aien-dev/aien-sovereign-core.git
ARCH_URL=https://github.com/aien-dev/aien-architecture.git
TOOLCHAIN=1.98.1

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
HOME_DIR=${SPARK_CONNECTOR_HOME:-$HOME/.spark-connector}
fork=""; name=""; start=0; resandbox=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --fork) fork=${2:?--fork needs OWNER/aien-receipts}; shift 2 ;;
    --name) name=${2:?--name needs a name}; shift 2 ;;
    --start) start=1; shift ;;
    --reinstall-sandbox) resandbox=1; shift ;;
    *) sed -n '2,20p' "$0"; exit 2 ;;
  esac
done
say() { printf '[spark-connector install] %s\n' "$*"; }
die() { printf '[spark-connector install] STOP: %s\n' "$*" >&2; exit 1; }

[[ $(id -u) -ne 0 ]] || die "run as your normal user, not root (it uses sudo where it must)"
[[ $(uname -m) == aarch64 ]] || die "this is for the DGX Spark (aarch64); this machine is $(uname -m)"
sudo -n true 2>/dev/null || { say "sudo needs your password once:"; sudo -v; }

# 1. packages
need=()
for p in git gh build-essential acl iproute2 util-linux ca-certificates curl; do
  dpkg -s "$p" >/dev/null 2>&1 || need+=("$p")
done
if [[ ${#need[@]} -gt 0 ]]; then
  say "installing ${need[*]}"
  sudo apt-get update -q && sudo apt-get install -y -q "${need[@]}"
else
  say "packages present"
fi

# 2. Rust
export PATH="$HOME/.cargo/bin:$PATH"
if [[ ! -x $HOME/.cargo/bin/rustup ]]; then
  say "installing rustup (no default toolchain)"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain none
fi
if ! rustup toolchain list | grep -q "^$TOOLCHAIN-"; then
  rustup toolchain install "$TOOLCHAIN" --profile minimal -c rustfmt,clippy
fi
say "rust $TOOLCHAIN present"

# 3. GitHub login (the fork-only token). The node refuses to run with a token that can push to project repos.
gh auth status >/dev/null 2>&1 || die "gh is not logged in. Make the fork-only token (README, Run a node, step 3), then: gh auth login --with-token"
login=$(gh api user --jq .login)
gh auth setup-git >/dev/null 2>&1 || true
[[ -n $fork ]] || fork="$login/aien-receipts"
gh api "repos/$fork" --jq .fork 2>/dev/null | grep -qx true \
  || die "$fork is not a fork of aien-dev/aien-receipts yet. Fork it on github.com (README, Run a node, step 2), then run this again"
say "GitHub: $login, publishing through $fork"

# 4. the judge at the pinned commit (its own checkout, never the clones the judge reads)
mkdir -p "$HOME_DIR/clones"
J=$HOME_DIR/judge
if [[ ! -d $J/.git ]]; then
  git clone -q --no-checkout "$JUDGE_URL" "$J"
  git -C "$J" checkout -q --detach "$JUDGE_COMMIT"
elif [[ ! -e $J/Cargo.toml ]]; then # a clone that never got its checkout (an interrupted first run)
  git -C "$J" cat-file -e "$JUDGE_COMMIT^{commit}" 2>/dev/null || git -C "$J" fetch -q origin
  git -C "$J" checkout -q -f --detach "$JUDGE_COMMIT"
else
  [[ -z $(git -C "$J" status --porcelain --untracked-files=no) ]] || die "$J has local changes; move them away first"
  git -C "$J" cat-file -e "$JUDGE_COMMIT^{commit}" 2>/dev/null || git -C "$J" fetch -q origin
  git -C "$J" checkout -q --detach "$JUDGE_COMMIT"
fi
[[ $(git -C "$J" rev-parse HEAD) == "$JUDGE_COMMIT" ]] || die "judge checkout is not at $JUDGE_COMMIT"
(cd "$J" && nice -n 19 cargo build -q --release --locked -p aien-test)
say "judge built: $J/target/release/aien-test at $JUDGE_COMMIT"

# 5. the sandbox (the judge's own installer at the same commit)
A=$HOME_DIR/clones/aien-architecture
[[ -d $A/.git ]] || git clone -q "$ARCH_URL" "$A"
shim_ok=0
if getent passwd aien-run >/dev/null && cmp -s "$J/tools/aien-local-ci/aien-run-exec" /usr/local/sbin/aien-run-exec \
   && sudo -n -l /usr/local/sbin/aien-run-exec >/dev/null 2>&1; then shim_ok=1; fi
if [[ $shim_ok == 1 && $resandbox == 0 ]]; then
  say "sandbox present: user aien-run and /usr/local/sbin/aien-run-exec identical to the judge's copy (skipped)"
else
  say "installing the sandbox with the judge's tools/aien-local-ci/install.sh"
  AIEN_OPERATOR=$(id -un) AIEN_ARCH_CLONE=$A AIEN_TOOLCHAIN="$TOOLCHAIN-aarch64-unknown-linux-gnu" \
    bash "$J/tools/aien-local-ci/install.sh"
fi

# 6. the connector
(cd "$here" && nice -n 19 cargo build -q --release --locked --offline 2>/dev/null || nice -n 19 cargo build -q --release --locked)
mkdir -p "$HOME/.local/bin"
install -m 0755 "$here/target/release/spark-connector" "$HOME/.local/bin/spark-connector"
install -m 0755 "$here/spark-connector-supervise" "$HOME/.local/bin/spark-connector-supervise"
say "installed ~/.local/bin/spark-connector"

# 7. join (never rewrites an existing node.conf or key)
args=(join --home "$HOME_DIR" --fork "$fork" --judge-commit "$JUDGE_COMMIT" --judge-src "$J")
[[ -n $name ]] && args+=(--name "$name")
if [[ -f $HOME_DIR/node.conf ]] && ! grep -qx "judge_commit=$JUDGE_COMMIT" "$HOME_DIR/node.conf"; then
  sed -i "s/^judge_commit=.*/judge_commit=$JUDGE_COMMIT/" "$HOME_DIR/node.conf"
  say "node.conf: judge_commit updated to $JUDGE_COMMIT (open a NODES.md change for the new judge commit)"
fi
"$HOME/.local/bin/spark-connector" "${args[@]}"

# 8. start (only when asked)
if [[ $start == 1 ]]; then
  line="@reboot SPARK_CONNECTOR_HOME=$HOME_DIR $HOME/.local/bin/spark-connector-supervise"
  if ! crontab -l 2>/dev/null | grep -qxF "$line"; then
    { crontab -l 2>/dev/null || true; echo "$line"; } | crontab -
    say "cron @reboot line added"
  fi
  SPARK_CONNECTOR_HOME=$HOME_DIR setsid nohup "$HOME/.local/bin/spark-connector-supervise" >/dev/null 2>&1 < /dev/null &
  say "node started; see: spark-connector status --home $HOME_DIR"
else
  say "not started. Send the block above as your registration request; start later with: connector/install.sh --start"
fi
