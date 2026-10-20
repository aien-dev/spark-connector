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
A Unix-style service: one binary, one key, and a plain restart loop started at boot (no service manager unit).
- `spark-connector join`: makes a key pair, prints the public key for the operator to register.
- `spark-connector run`: polls the AIEN repositories for pull requests, runs the judge on each
  head in a sandbox user (no secrets, no network during the run), signs the bundle, and
  publishes it.
- Pull only. A volunteer Spark never accepts inbound connections and never needs a token with
  write access to AIEN code.

**First node run (2026-10-20):** a test node on the project's Spark judged a real pull request and
its receipt passed the project's own checker next to the project's receipt for the same commit
([evidence/connector/README.md](evidence/connector/README.md)). What the connector promises, and
what it does not: [connector/GUARANTEES.md](connector/GUARANTEES.md).

## Run a node
You need a DGX Spark with its normal NVIDIA software (DGX OS), a GitHub account, and a little
time at the keyboard. Disk space and run time per pull request are not measured yet.

1. **Fork the receipts shelf.** Open https://github.com/aien-dev/aien-receipts and press **Fork**.
   This gives you your own copy, the only place your node will ever write.
2. **Make a key for the node that can only touch that fork.** On GitHub: your picture, then
   Settings, Developer settings, Personal access tokens, **Fine-grained tokens**, Generate new token.
   Repository access: **Only select repositories**, pick your `aien-receipts` fork.
   Permissions: **Contents: Read and write**. Copy the token it shows you.
   (The node refuses to start with a key that could change AIEN code:
   [refusals.log](evidence/connector/refusals.log), case 1.)
3. **Give the key to your Spark.** In a terminal on the Spark type
   `sudo apt-get install -y git gh` and press Enter, then `gh auth login --with-token`, press
   Enter, paste the token, press Enter, then Ctrl+D. Nothing is printed when it works.
4. **Get the connector.** Type
   `git clone https://github.com/aien-dev/spark-connector.git && cd spark-connector`.
5. **Install.** Type `connector/install.sh`. It asks for your password once, sets up the locked-down
   sandbox user, builds the project's judge from source at the pinned version, and makes the node's
   key. It ends by printing a block that starts with `## spark-` and lists your public key.
   Running it again later changes nothing that is already right
   ([install-run3-idempotent.log](evidence/connector/install-run3-idempotent.log)).
6. **Ask to join.** Open a pull request on `aien-dev/aien-receipts` that adds that block to
   `NODES.md` (the format is in `trust/NODES.md`). Until Drake accepts it, your receipts are
   advisory only.
7. **Start the node.** Type `connector/install.sh --start`. The node starts now and again after
   every reboot. It works at the lowest priority, so your own work on the Spark comes first.
8. **Check on it.** Type `spark-connector status`. It shows what it judged, what passed, and
   each receipt it published to your fork.
9. **Hand in receipts.** GitHub does not let a fork-only key open pull requests on the project, so
   each receipt waits on your fork with a link in `spark-connector status`. Open the link and press
   **Create pull request** ([publish-final.log](evidence/connector/publish-final.log)).
10. **Stop.** Type `crontab -e`, delete the line with `spark-connector-supervise`, save, then
    reboot (or stop the `spark-connector-supervise` process). Tell the project to mark your node
    as retired in `NODES.md`.

## Streams (see docs/PLAN.md)
| stream | directory | branch |
|---|---|---|
| connector (binary, install, sandbox, publish) | `connector/` | `connector/v0` |
| trust (key registry, verdict policy, desk integration) | `trust/` | `trust/v0` |

License: AGPL-3.0.
