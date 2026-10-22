<!-- github-native-navigation -->
[Source](https://github.com/aien-dev/spark-connector) · [Walkthrough](docs/TEACHING.md) · [Replay receipt](docs/teaching-replay-2026-10-22.txt) · [Issues](https://github.com/aien-dev/spark-connector/issues)
<!-- /github-native-navigation -->

# spark-connector

## How it works and how to reproduce

I trace one actual source or design boundary in [the worked lesson](docs/TEACHING.md), with a pinned fresh-checkout exercise and [raw replay receipt](docs/teaching-replay-2026-10-22.txt). The lesson states what remains unimplemented or unverified at this revision.

Connect your NVIDIA DGX Spark to the AIEN merge pipeline as a volunteer judge.

**Status: pre-alpha, 2026-10-19. Nothing here is proven yet.** Every claim in this repository
must link to a file under `evidence/`. If it does not, it is a plan, not a fact.

## Why
Every pull request in the AIEN repositories is verified by one machine, the project's Spark,
before it merges. That one machine is the bottleneck. The proposed volunteer connector would check the same commit with the same judge ([`aien-test`](https://github.com/aien-dev/aien-sovereign-core-mv/tree/main/tools/aien-test)) and publish a signed receipt. The two-receipt merge route remains a plan until implementation and independent verification.

## Proposed interface
The plan calls for a Unix-style service: one binary, one systemd unit, one key. No connector binary is merged here.
- `spark-connector join`: would make a key pair and print the public key for the operator to register.
- `spark-connector run`: would poll the AIEN repositories for pull requests, run the judge on each
  head in a sandbox user (no secrets, no network during the run), sign the bundle, and
  publish it.
- Pull only. A volunteer Spark never accepts inbound connections and never needs a token with
  write access to AIEN code.

## Streams (see docs/PLAN.md)
| stream | directory | branch |
|---|---|---|
| connector (binary, install, sandbox, publish) | `connector/` | `connector/v0` |
| trust (key registry, verdict policy, desk integration) | `trust/` | `trust/v0` |

License: AGPL-3.0.
