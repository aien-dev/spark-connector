# What the connector guarantees (and what it does not)

For the trust stream (`trust/THREATS.md`, `trust/VERDICT_POLICY.md`) and for anyone deciding whether
to run a node. Every promise names where it is enforced and the file under `evidence/connector/`
that shows it working on a real Spark (spark-b87b, 2026-10-20, test node `spark-b87b-node2`).

## Promises to the volunteer (a hostile pull request on your Spark)

1. **Pull request code runs only as the sandbox user `aien-run`.** The node never runs a check
   itself; it calls the project's judge with `--sandbox user`, and the judge runs every check
   through `/usr/local/sbin/aien-run-exec`, the same shim the project's Spark uses, installed by the
   judge's own installer at the pinned commit. Each receipt records the shim's fingerprint
   (`sandbox_helper_sha256`), and it matches the installed shim.
   Evidence: `sandbox-probe.log` (user `aien-run`; fingerprint in all 3 receipts = installed shim),
   `run-once-aegis-runtime-bfa6b636.log` (the judge command line with `--sandbox user`).
2. **No network while a check runs.** Ordinary checks get a fresh network namespace that holds only
   loopback: no DNS, no route out. Checks that need the chip (`gb10`, `nvrm`) keep the host network
   in the shared shim, so a v0 node **does not run any head whose gate files need the chip**; it
   reads the gate files from git before anything runs and skips the head. Those heads stay with
   the project's Spark.
   Evidence: `sandbox-probe.log` (inside: only `lo`, no DNS, "Network is unreachable"; outside, the
   same commands reach GitHub), `refusals.log` case 3 (physics skipped for 5 chip gates).
   Limit: the node itself (not the checks) uses the network before and after a run: it fetches
   code and publishes the receipt.
3. **Checks cannot read the node's signing key.** The key is a mode 0600 file of the node owner;
   the judge refuses to start with a key anyone else can read. Evidence: `sandbox-probe.log`
   ("read node key: Permission denied").
4. **No secrets in the sandbox.** The shim clears the environment (only PATH, HOME, CARGO_HOME,
   RUSTUP_HOME survive) and makes the sandbox home read-only except the one checkout under test.
   Source: the header of `aien-run-exec` in aien-sovereign-core `tools/aien-local-ci/` at the
   pinned commit. Not separately tested here beyond case 1 and 3 above.
5. **Bounded.** The shim caps each check (memory, processes, CPU; timeouts for ordinary checks),
   and the node runs the judge at the lowest CPU and disk priority (`nice -n 19 ionice -c3`), so
   your own work on the Spark comes first. Source: `aien-run-exec` header; the judge command line
   in `run-once-aegis-runtime-bfa6b636.log`.

## Promises to the project (what a node can and cannot write)

6. **The node's GitHub token can write only to its own fork.** `spark-connector run` asks GitHub
   what the token may do and refuses to start if it can push to any repository in its list or to
   `aien-dev/aien-receipts`. Evidence: `refusals.log` case 1 (this Spark's owner token is refused).
   The project's own test used `--project-spark-test` to get past this on purpose; the run log says
   so on every start. Limit: GitHub reports what the token can do today; the recommended token
   (fine-grained, one repository: the fork) cannot open pull requests on aien-dev at all (GitHub
   docs: fine-grained tokens do not support contributing to public repositories where the user is
   not a member), so its receipts wait on the fork with a ready "open pull request" link.
   Evidence: `publish-final.log`.
7. **A node never overwrites the project's receipt.** It writes only
   `<repo>/<sha>/bundle.<first 16 hex of its public key>.json`, refuses to commit if anything else is
   staged, and refuses to publish if that file is already on the shelf with different bytes
   (the shelf is append only). Source: `src/receipts.rs` (with a test that the path can never be
   `bundle.json`); evidence: `run-once-aegis-runtime-bfa6b636.log` (one file added).
8. **One node runs one poller, and never two judges on one head.** The node holds `run.lock` for
   its whole run; a second run is refused (`refusals.log` case 2). Before each head it looks for
   any other judge on the same head on the machine and waits a round if there is one. It uses no
   trigger files and has its own state directory, so the shared-trigger bug (ledger L083) cannot
   happen here.
9. **The judge is the pinned one.** `run` refuses to start if the judge checkout is not at
   `judge_commit` or has local changes. The node writes `executor.judge_commit` into every receipt
   and signs it with the rest (`aien-test bundle-sign`), so `VERDICT_POLICY.md`'s stale-judge rule
   can be applied to node receipts today. Evidence: `node-receipt.aegis-runtime.<sha>.json`,
   executor block in `verify-two-receipts.log`.
10. **Anyone can re-check a receipt.** `verify-two-receipts.sh` downloads both receipts on one head
    from GitHub and checks them with `aien-test verify-bundle`: both pass under their own keys;
    each is refused under the other key, after a one-word tamper, and for another commit.
    Evidence: `verify-two-receipts.log`, `verify-two-receipts.rerun.log` (6 of 6 as expected).

## Not guaranteed (the trust stream's job)

- **That the node is honest.** A volunteer can change the code and sign anything with their key.
  A signature proves who signed, not that the real judge ran. The registry, the two-node rule and
  spot re-checks by the project's Spark (`trust/`) cover this.
- **The judge commit and GPU in a receipt or registration are the node's own statements.**
- **A pull request that adds a new Rust dependency** fails or is not run on a node, because the
  sandbox has no network and only the dependency cache copied at install time. That is never a
  PASS, but it is a false FAIL until the node's cache is refreshed by re-running `install.sh`.
- **Independence of the test node used for the evidence.** It ran on the project's own Spark: own
  key, state, clones and judge build, but the same machine, sandbox and dependency cache.
