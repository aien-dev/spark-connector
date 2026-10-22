# PLAN

Two streams, two directories, two branches. One desk item merges them.

## connector/ (branch connector/v0)
Rust crate `spark-connector` (my stack: Rust; the judge itself is `aien-test` from aien-sovereign-core, built from source by the installer, pinned by commit).
- `join`: key pair (ed25519 via the same signing the judge already uses for bundles; reuse `aien-test bundle-sign`), prints the public key and a one-line registration request.
- `run`: poll loop over the repos list (the same list the project's Spark uses), for each open PR head not yet judged by this node: clone or fetch into its own clones dir, run `aien-test` through the sandbox user exactly as `/usr/local/sbin/aien-run-exec` does on the project's Spark (ship that script or its equivalent), with network off for the run, sign the bundle, publish.
- Publish: open a pull request against aien-dev/aien-receipts adding `<repo>/<sha>/bundle.<node>.json` (one file per node; never overwrites the project's own bundle), or push to a fork and open the PR. The GitHub token the node uses can only write to its own fork of aien-receipts.
- `install.sh`: one command on a fresh Spark: deps, build judge, create sandbox user, systemd unit, join.
- Evidence: a full run on the project's Spark under a second node key (so two receipts exist on one commit), logged under evidence/.

## trust/ (branch trust/v0)
- `NODES.md` registry format for aien-receipts (public key, operator, Spark serial or GPU UUID, date, who approved) and the rule that only Drake merges registry changes.
- Verdict policy for the desk: a head is merge-grade when (a) the project's Spark says READY, or (b) two registered nodes say READY on the exact head and `aien-test verify-bundle` passes on both bundles. Write it as a change to the desk procedure (a doc here; the desk applies it).
- Threat notes in plain words: a malicious PR running on a volunteer box (sandbox, no secrets, network off), a malicious node (registry, two-node rule, spot re-verification by the project's Spark), a stale judge (pinned judge commit in every bundle; desk refuses bundles from an unpinned or older judge).
- Evidence: `aien-test verify-bundle` run on a community-style bundle from the connector stream.
