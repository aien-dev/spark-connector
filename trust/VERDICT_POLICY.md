# Verdict policy: when community receipts let a head merge

This is written as a change to the desk procedure (the `pr-desk` skill, steps 3 and 4). The
desk applies it; nothing here runs by itself. Until Drake approves this document and the open
items at the end, the desk procedure stays exactly as it is today.

## Words used here
- **Head:** the newest commit on a pull request.
- **READY:** the judge's answer for a head is PASS. In `aien-test` terms: `verify-bundle`
  exits 0 and prints `AIEN Verification: PASS`. That output already folds in the plan result: a
  receipt with no required checks, or whose plan says UNMAPPED or BLOCKED, prints NOT_RUN and
  exits 2. Exit codes: 0 PASS, 1 FAIL, 2 NOT_RUN, 3 REJECTED. Anything but 0 is not READY.
- **Registered node:** a node with `status: active` in `NODES.md` in `aien-dev/aien-receipts`
  (see `trust/NODES.md`).
- **Pinned minimum judge:** the oldest judge version the repository accepts (see below).

## Change to step 3 (Judge) and step 4 (Merge)

A head is **merge-grade** when either:

**(a)** the project's Spark says READY. This is the current rule, unchanged: the commit status
`AIEN Verification (Spark)` is success on the exact head.

**or (b)** all of the following hold:
1. Two registered nodes, with two different keys and two different GPU UUIDs, each published a
   receipt for the exact head (the 40-character commit, not the branch name).
2. The desk runs `aien-test verify-bundle` on each receipt, on the desk's own machine, and
   both print PASS and exit 0:
   `aien-test verify-bundle BUNDLE --repo github.com/aien-dev/<repo> --commit <head> --pubkey <node's key from NODES.md> --require <list from .aien/always on the base branch>`
3. The key given to `--pubkey` comes from the registry, never from the receipt. The `--require`
   list comes from the repository's base branch, never from the pull request.
4. Neither node's receipt is stale (see "Stale judge").
5. No registered node has published a FAIL, or a receipt that `verify-bundle` rejects, for
   that exact head. One honest FAIL outweighs two PASS: the desk hands the failing gate line to
   the owner, as it does for the project's Spark.
6. The two receipts' `tree_digest` and `graph_digest` are equal. They are two descriptions of one
   piece of work; if they differ, the nodes did not judge the same thing.

The desk still does everything else in step 4: read the diff, `mergeStateStatus` CLEAN,
squash, no `--admin`. A merge-grade head is *eligible*; it is not merged automatically.

A receipt from an unregistered key, a suspended or revoked node, or a node whose key does not
match its registry entry is **advisory**: the desk may read it, and it counts for nothing.

## Stale judge
The receipt must say which judge made it, and the repository must say how old is too old.

- **Pinned minimum:** each repository commits `.aien/judge.min`, one line, the 40-hex commit of
  aien-sovereign-core of the oldest acceptable judge. The desk reads it from the base branch,
  as it does `.aien/verify.pub` and `.aien/always`.
- **In the receipt:** the judge's commit is a signed member `executor.judge_commit`. The
  verifier already accepts and signs over an extra member in that block (evidence case 7 and 8,
  `evidence/trust/README.md`).
- **Rule:** the desk refuses a receipt if `executor.judge_commit` is missing, or is not a commit at
  or after `.aien/judge.min` in aien-sovereign-core's history (`git merge-base --is-ancestor`).
  The registry's "judge commit" line is a record for audits; it is not part of this check.

**Gap, stated plainly:** the receipt the project's Spark publishes today has no
`judge_commit` member (see `evidence/trust/bundle.original.json`: the `executor` block holds
only `class`, `id` and `machine_digest`). Two changes are needed before rule (b) can run:
1. `aien-test` writes `executor.judge_commit` into every bundle it signs (a change in
   aien-sovereign-core, tools/aien-test, `worker.rs`); the connector stream builds the judge
   from a pinned commit and passes that commit in.
2. Each repository adds `.aien/judge.min`.
Until both land, **rule (b) is switched off** and only rule (a) applies. This is not optional:
without the judge commit in the receipt, a stale or modified judge cannot be told from the real
one.

## Spot check
The project's Spark re-verifies **one in 5** heads that were merged under rule (b), chosen by the
desk with a counter (every fifth), not by whim, plus **every one of the first 10** per node.
Re-verifying means running the judge on the exact head on the project's Spark and comparing the
verdict and gate list to the community receipts.

Reasoning for 5: the point is that a lying node expects to be caught, so the odds per head must
not be negligible, and the project's Spark must not be the bottleneck again. A fifth of
community merges means the project's Spark carries a fifth of the load these merges would
have put on it. This number is a starting policy, not a measurement: there is no data yet on
how often nodes and the project's Spark disagree. Review it after the first 50 community
merges: if there were no disagreements, Drake may raise N; any disagreement sets N back to 1
for the nodes involved until Drake says otherwise.

On a disagreement (the project's Spark says FAIL or ERROR where two nodes said PASS):
1. The desk stops community merges that involve either node and opens a registry change setting
   both to `suspended`, for Drake.
2. The desk lists every head merged on those nodes' receipts and has the project's Spark
   re-verify each.
3. A real FAIL on a merged head is handled like any bad merge: a fix PR through the same desk.

## Revoked or suspended node
Applies the line in `NODES.md`: its receipts stop counting, and the desk re-verifies on the
project's Spark every head merged on its word.

## Open items for Drake (not decided here)
1. **Who posts the merge status.** Branch protection on the core repositories requires the check
   `AIEN Verification` (a bridge to the project's commit status `AIEN Verification (Spark)`), and
   the rules forbid `--admin`. A head that is merge-grade under (b) has no such status.
   Options: (i) the desk posts a status named `AIEN Verification (Community)` after it
   has run `verify-bundle` on both receipts, and the bridge accepts either; or (ii) the
   project's Spark itself runs `verify-bundle` on the two community receipts and posts its own
   status. (ii) keeps the one writer of statuses but puts the check back on the Spark (cheap:
   it only verifies signatures, it does not rerun the gates). Not chosen here.
2. **The `judge_commit` change in `aien-test`**, and `.aien/judge.min` per repository (above).
3. **Approval of N = 5.**
