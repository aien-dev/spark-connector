# spark-connector (proposed volunteer DGX Spark verifier)

> spark-connector is planned to let an owner of an NVIDIA DGX Spark check AIEN revisions with the project's judge (`aien-test`, from aien-sovereign-core) and publish an independent signed receipt. This is not a current merge gate or a replacement for Spark Alpha qualification. It is pre-alpha. On `main` the repository is a plan and a skeleton: `README.md`, `docs/PLAN.md`, and empty `connector/`, `trust/` and `evidence/` directories. Licensed AGPL-3.0.

Shared rules: read https://www.aienos.com/agents/RULES.md first. This file adds what is specific to this repository; where the two differ, this file wins here.

## Scope

- Applies to the whole repository.
- Two streams, each with its own directory and branch, as `README.md` and `docs/PLAN.md` define:
  - `connector/` on branch `connector/v0`: the binary, installer, sandbox and publishing.
  - `trust/` on branch `trust/v0`: the key registry format, the verdict policy and the threat notes.
- Work in progress lives on those branches and is not merged into `main`. Read the branch you are changing, not `main`, before editing a stream.
- `evidence/` holds the logs that back claims. `README.md` sets the rule: a claim that does not link a file under `evidence/` is a plan, not a fact.

## Installation

```bash
git clone https://github.com/aien-dev/spark-connector.git
cd spark-connector
```

`main` has no code, build files or dependencies, so there is nothing to build. `docs/PLAN.md` plans a Rust crate named `spark-connector` and an `install.sh` for a fresh Spark; neither is on `main`.

## Usage

- The commands `spark-connector join` and `spark-connector run` described in `README.md` are planned, not available from `main`.
- Running the connector needs a physical NVIDIA DGX Spark, a sandbox user, and the `aien-test` judge built from aien-sovereign-core. Do not run any part of it on another machine or claim it ran without a log under `evidence/`.
- Check Markdown for forbidden dashes before committing:

```bash
grep -rnP '[\x{2013}\x{2014}]' --include='*.md' .
```

## Conventions

- Keep each change inside one stream's directory and on that stream's branch. Changes that join the streams go through one reviewed merge, as `docs/PLAN.md` says.
- Every claim of a working feature links its evidence file under `evidence/`. State the status plainly; the project calls itself pre-alpha and so should any new text.
- Design constraints from `README.md` and `docs/PLAN.md` that changes must keep:
  - Pull only: a volunteer Spark never accepts inbound connections.
  - The node's GitHub token can write only to its own fork of aien-receipts, never to AIEN code.
  - The judge runs as a sandbox user with no secrets and no network during the run.
  - A node publishes its own bundle file and never overwrites the project's bundle.
- Plain direct English. No em dashes or en dashes.

## Safety and approvals

- Changes to the proposed node registry (`NODES.md` in aien-receipts, per `docs/PLAN.md`) and any policy that would use volunteer verdicts for merges need the maintainer's explicit approval. No two-node merge substitution is active on `main`.
- Do not weaken the proposed two-node condition, the pinned judge commit, the sandbox, or the network-off condition to get a run through.
- Never commit private keys, tokens, or node identifiers that are not meant to be public.
- `main` has no required checks or reviews configured. Still work on a branch and merge through a pull request; never force-push.

## Further reading

- `README.md`: why the connector exists and what it is.
- `docs/PLAN.md`: the two streams, their deliverables and their evidence.
- `LICENSE`: GNU Affero General Public License version 3.
