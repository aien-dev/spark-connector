# NODES.md: the registry of community Sparks

This file describes the registry, a list of the community Sparks whose receipts the project
counts. The registry itself lives in the receipts repository, `aien-dev/aien-receipts`, as a
file at the top level called `NODES.md`. Because it lives next to the receipts, the desk reads
the receipt and the registry from the same place.

## The rule (it decides everything else)
**Only Drake merges changes to the registry.** Not the desk, not an agent, not the node's own
operator. A pull request that touches `NODES.md` is held for Drake and for nobody else.

A receipt signed by a key that is not in the registry is **advisory only**. It can be read and
re-checked by anyone. It never counts toward a merge.

## Why a registry at all
A signed receipt proves two things: the file was not changed after signing, and the holder of
one particular key signed it. It does not prove the key belongs to an honest Spark running the
real judge. In the test run in `evidence/trust/verify-bundle.log`, case 6, a receipt signed by a
brand new throwaway key passes the checker against its own key. Only the registry turns "some
key" into "a node the project has agreed to count."

## One entry per node
Each node is one block in `NODES.md`, in this exact shape:

```
## <node-name>
- status: active            # active | suspended | revoked
- public key: <64 lower-case hex characters, from `spark-connector join`>
- executor id: <64 hex characters: the sha256 of the public key bytes, as written in every receipt>
- operator: <GitHub handle of the person responsible>
- gpu: <GPU UUID from `nvidia-smi -L`>
- judge commit: <40-hex commit of aien-sovereign-core the node's judge was built from>
- date added: <YYYY-MM-DD>
- approved by: <Drake, with the link to the merged registry pull request>
- notes: <free text, optional>
```

Field notes:
- **public key** is what the desk hands to `aien-test verify-bundle --pubkey`. The desk always
  takes the key from the registry, never from the bundle (a bundle names its own key, which
  proves nothing).
- **executor id** must equal the `executor.id` in the node's receipts. The desk checks both.
- **gpu** ties a node name to one physical card. A second Spark is a second node with its own key.
  This is a statement by the operator, not something the receipt proves (see `THREATS.md`,
  section 2).
- **judge commit** is what the operator says the node runs *right now*. It is a statement for
  humans and a record for audits; the check the desk relies on is the one inside the receipt
  (`VERDICT_POLICY.md`, "Stale judge"). When a node upgrades its judge, it opens a registry
  change to update this line, and Drake merges it like any other change.

## Joining
1. The operator runs `spark-connector join` on their Spark. It makes a key and prints the public key.
2. The operator opens a pull request on the receipts repository adding one block with
   `status: active` left for Drake to set. The pull request description says who they are and
   why they are running a node.
3. Drake reads it and merges it, or does not. Until it is merged, the node's receipts are
   advisory.

## Revocation line
Drake changes `status` to `revoked` (or `suspended` while a question is open) and adds a line
under `notes` giving the date and the reason. The change takes effect when it merges.

The desk rule that follows (full text in `VERDICT_POLICY.md`):
- A receipt from a `suspended` or `revoked` node no longer counts, for new merges or old.
- The desk lists every head merged on the strength of that node's receipt, and has the project's
  Spark re-check each one.
- Revoked keys are never deleted from the file and never reused. The block stays as the record.

## What the registry does not hold
No secrets, no private keys, no tokens, no personal details beyond the GitHub handle the
operator chose to publish.
