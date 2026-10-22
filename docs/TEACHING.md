# Learn the connector boundary

At `46be239a8ae61593257adb6d7983a258a6f65e59`, this is a design repository, not a runnable connector. I want a second, independently keyed judge so a merge can rely on two receipts for the same head. [The plan](../docs/PLAN.md) describes `join`, polling, a network-isolated sandbox, signing and publication, and a separately governed trust registry. The default branch has only [`connector/.keep`](../connector/.keep) and [`trust/.keep`](../trust/.keep). Neither command, a key registry, nor an install unit exists here. Issue [#1](https://github.com/aien-dev/spark-connector/issues/1) tracks implementation. No claim of an existing two-judge verdict follows from this document.

The intended boundary matters: the volunteer should publish through its own receipts fork, while a trusted desk checks the exact head, registered key and pinned judge. Its token should not write AIEN code. This is a design choice in `docs/PLAN.md`, not a tested security property. A second node could collude or run a stale judge; the plan calls for registration, two receipts and spot checks, but those defenses still need implementation and evidence.

## Fresh checkout exercise (offline, no Spark or key)

Prerequisites: Git and Python 3. Checkout the exact default-branch source pin above, then run from repository root:

```sh
python3 - <<'PYEX'
from pathlib import Path
p=Path('docs/PLAN.md').read_text()
assert 'join' in p and 'sandbox' in p and 'two registered nodes' in p
print('PLAN PRESENT')
assert not Path('connector/Cargo.toml').exists()
assert not Path('trust/NODES.md').exists()
print('CONNECTOR UNAVAILABLE')
PYEX
```

Expected: `PLAN PRESENT`, `CONNECTOR UNAVAILABLE`, exit 0. This checks the source-to-plan boundary only. A real positive result needs a merged binary, sandbox unit, registered key, two signed bundles and an independently checked verdict. The missing files are the negative case; do not run `join` or `run` from this snapshot. [Replay receipt](teaching-replay-2026-10-22.txt).


### Parent fresh-checkout replay

A separate coordination replay is retained in [the parent receipt](parent-replay-2026-10-22.txt). Its scope and observed exit are explicit; source review and final desk qualification remain separate. A recorded absence or expected failing baseline is not a passing implementation.
