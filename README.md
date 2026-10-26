# spark-connector

A proposed way for a volunteer NVIDIA DGX Spark to produce independent AIEN verification receipts.

**Status: pre-alpha, 2026-10-19. Nothing here is proven yet.** Every claim in this repository
must link to a file under `evidence/`. If it does not, it is a plan, not a fact.

## Why
AIEN's current component workflow treats GitHub pull-request checks as advisory and uses a
signed full Spark bundle to qualify an `alpha/*` tag on a commit already on `main`
(see [aienux's workflow](https://github.com/aien-dev/aienux/blob/main/.github/workflows/aien-verification.yml)).
An open PR or merged commit is not an Alpha result. The proposed connector would let
community Sparks run the pinned judge (`aien-test`, from aien-sovereign-core) and publish
independent receipts. Whether or how those receipts affect a future desk or merge policy
requires separate review and approval; no two-node substitution is in force here.

## What it is
A proposed Unix-style service: one binary, one systemd unit, one key. The following commands
are not implemented on `main`:
- `spark-connector join`: would make a key pair and print the public key for registration.
- `spark-connector run`: would poll for pull requests, run the judge on each head in a
  sandbox user (no secrets, no network during the run), sign the bundle and publish it.
- Pull only. A volunteer Spark never accepts inbound connections and never needs a token with
  write access to AIEN code.

## Streams (see docs/PLAN.md)
| stream | directory | branch |
|---|---|---|
| connector (binary, install, sandbox, publish) | `connector/` | `connector/v0` |
| trust (key registry, verdict policy, desk integration) | `trust/` | `trust/v0` |

License: AGPL-3.0.
