# Audit scope

Written for an outside, independent security audit. Revision described:
`c257974530764e10e8769091ce3a88793024403f` on `main`, 2026-10-26. Everything below is taken from
the repository's own files at that revision; items marked NOT_RUN were not executed while writing
this document.

## What this repository is

A proposed service that would let a volunteer NVIDIA DGX Spark run AIEN's pinned judge
(`aien-test`, from aien-sovereign-core) on pull request heads and publish independent signed
receipts to aien-dev/aien-receipts. Status: pre-alpha, 2026-10-19, "Nothing here is proven yet."
On `main` the repository holds `README.md`, `AGENTS.md`, `docs/PLAN.md`, `LICENSE` and empty
`connector/`, `trust/` and `evidence/` directories (each a `.keep` file). There is no code, no
build file and no dependency. Work in progress lives on branches `connector/v0` and `trust/v0`,
which this document does not describe.

## What to review, in priority order

1. `docs/PLAN.md`, connector stream: key generation through `aien-test bundle-sign` (Ed25519);
   the poll loop over open pull request heads; running the judge as a sandbox user with network
   off; signing and publishing `<repo>/<sha>/bundle.<node>.json` by pull request to
   aien-receipts; `install.sh` that builds the judge from a pinned commit and creates the sandbox
   user and systemd unit.
2. `docs/PLAN.md`, trust stream: the proposed `NODES.md` registry (public key, operator, device
   identifier, approver), the proposed desk policy (project Spark READY, or two registered nodes
   READY on the exact head with `aien-test verify-bundle` passing on both), and the threat notes.
3. `README.md`: the pull-only rule and the statement that no two-node substitution is in force.
4. Branches `connector/v0` and `trust/v0`, when code lands there (not on `main`).
5. There are no CI workflows in this repository; AGENTS.md states `main` has no required checks.

## Trust boundaries

Planned, not built:

- Untrusted: pull request code run by the judge on a volunteer box; other volunteer nodes.
- Trusted: the pinned judge commit; the maintainer who approves registry entries; the project's
  own Spark, which would spot re-verify.
- A volunteer Spark would never accept inbound connections, and its GitHub token would write only
  to its own fork of aien-receipts.

## Threat model summary

- Assets: the volunteer's machine and secrets; the integrity of receipts; AIEN code (never
  writable by a node token).
- Attackers considered (from the plan's threat notes): a malicious pull request running on a
  volunteer box; a malicious node; a stale judge.
- Defended (proposed): sandbox user, no secrets, network off during the run; registry with
  maintainer approval; a proposed two-node rule and spot re-verification; the pinned judge
  commit in every bundle and a proposed refusal of bundles from an unpinned or older judge.
- Not defended / explicitly out of scope: nothing is implemented, so no defence is in force.
  Using volunteer verdicts for any merge or Alpha decision requires a separate maintainer policy
  decision and is not active.

## Build from source at a pin

There is nothing to build on `main` (AGENTS.md). The planned Rust crate `spark-connector` and
`install.sh` are not on `main`.

```sh
git clone https://github.com/aien-dev/spark-connector.git
cd spark-connector
git checkout c257974530764e10e8769091ce3a88793024403f
```

## Run every test

There are no tests on `main`. The only documented check is the dash check from AGENTS.md:

```sh
grep -rnP '[\x{2013}\x{2014}]' --include='*.md' .
```

- Expected: no output.
- Running the connector needs a physical DGX Spark, a sandbox user and the `aien-test` judge;
  NOT_RUN. The command above: NOT_RUN while writing this document.

## Known limitations and NOT_RUN

- `spark-connector join` and `spark-connector run` are not implemented on `main`.
- No evidence file exists under `evidence/`; by the README's own rule, every claim here is a plan.
- The proposed registry and desk policy are not in force.

## Dependencies and provenance

- None on `main`. The plan names Rust, the `aien-test` judge built from aien-sovereign-core at a
  pinned commit, and GitHub for publishing.
- Licence: AGPL-3.0 (`LICENSE`).

## Evidence

- `evidence/` is empty (`evidence/.keep`). The plan calls for a full run under a second node key
  and an `aien-test verify-bundle` run, logged there.
- Planned node bundles in `aien-dev/aien-receipts` would be `<repo>/<sha>/bundle.<node>.json`.
  Whether any exist is unknown from the repository. Receipts are append-only and never edited.
