# Run a node: lend your Spark to the AIEN merge pipeline

*Draft for www.aienos.com. Nothing described here is available yet: the node software is still
being built (see `docs/PLAN.md`). Every claim below that depends on it is a plan, not a fact.*

## Why run a node
Every change to AIEN's code is checked by one computer, the project's own Spark, before it can be
accepted. When that computer is busy, everything waits. If your Spark checks the same changes with
the same program and signs the result, two independent signatures on one change are enough for the
project to move on without waiting for the project's computer. You are not trusted blindly:
the project re-checks a share of what your node approves.

## What you need
- An NVIDIA DGX Spark that you are comfortable leaving on.
- A GitHub account.
- One command to install (planned; the connector stream is writing it).

## What you get
- Your node's name on every receipt it signs. Receipts are public in `aien-dev/aien-receipts`, so
  anyone can see that your Spark checked a given change and what it found.
- A hand in how fast the project can merge.

There is no payment and no access to anything private.

## What it asks of you
- Your Spark runs other people's proposed code to check it. It runs inside a separate, locked-down
  user account with no network and no access to your files, and caps on memory and time
  (planned; see `trust/THREATS.md`, section 1, for what is promised and what is still open. One open
  point is whether GPU checks will run on community nodes at all).
- Your GitHub token only ever writes to your own copy of the receipts repository.
- A person at the project (Drake) approves your node by name before its receipts count. Until then
  they are published, but only for information.
- If your node signs something wrong, it is suspended and everything it approved is re-checked.

## How it works, in four lines
1. Your node reads the list of open changes, which is public.
2. It runs the project's checks on each one in the locked-down account.
3. It signs a receipt: a small file saying what it checked and what happened.
4. It sends the receipt to the project. The project verifies the signature and counts it once
   your node is registered and a second registered node agrees.

Details: `trust/NODES.md` (the register), `trust/VERDICT_POLICY.md` (when receipts allow a merge),
`trust/THREATS.md` (what can go wrong). Test results so far: `evidence/trust/README.md`.
