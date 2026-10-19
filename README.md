# spark-connector

Connect your NVIDIA DGX Spark to the AIEN merge pipeline as a volunteer judge.

**Status: pre-alpha, 2026-10-19. Nothing here is proven yet.** Every claim in this repository
must link to a file under `evidence/`. If it does not, it is a plan, not a fact.

## Why
Every pull request in the AIEN repositories is verified by one machine, the project's Spark,
before it merges. That one machine is the bottleneck. A community Spark running this connector
checks the same commits with the same judge (`aien-test`, from aien-sovereign-core) and publishes
a signed receipt. Two independent receipts on the same commit let the desk merge without waiting
for the project's Spark.

## What it is
A Unix-style service: one binary, one systemd unit, one key.
- `spark-connector join`: makes a key pair, prints the public key for the operator to register.
- `spark-connector run`: polls the AIEN repositories for pull requests, runs the judge on each
  head in a sandbox user (no secrets, no network during the run), signs the bundle, and
  publishes it.
- Pull only. A volunteer Spark never accepts inbound connections and never needs a token with
  write access to AIEN code.

## Streams (see docs/PLAN.md)
| stream | directory | branch |
|---|---|---|
| connector (binary, install, sandbox, publish) | `connector/` | `connector/v0` |
| trust (key registry, verdict policy, desk integration) | `trust/` | `trust/v0` |

License: AGPL-3.0.
