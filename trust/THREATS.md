# THREATS.md: what could go wrong, in plain words

Five sections, one for each way this can fail. For each: what could happen, what stops it, and
what is still open. Where a claim rests on the connector software, it says so: the connector
stream has not published `connector/GUARANTEES.md` yet, so those lines are marked
**to confirm against GUARANTEES.md**.

Nothing here has been tested against a real community node, because none exists yet.
What has been tested is in `evidence/trust/README.md`.

## 1. A malicious pull request, running on a volunteer's computer
**What could happen.** Anyone can open a pull request. The node downloads its newest commit and
runs the project's checks on it. Those checks are programs. A hostile one could try to read the
volunteer's files, steal their GitHub login, mine coins, attack the home network, or damage the
Spark.

**What must be true (requirements for the connector; to confirm against GUARANTEES.md):**
- The checks run as a separate, unprivileged user with no access to the volunteer's own files, and
  with no GitHub token, SSH key, or signing key in reach. The node's signing key is read only
  by the part of the connector that signs, never by the user that runs the checks.
- The checks have no network for the whole run. The project's own helper, `aien-run-exec` (excerpt: `evidence/trust/aien-run-exec.excerpt.txt`),
  does this for ordinary checks (its own network namespace with only a loopback connection). **It does
  not do it for the `gb10` profile**, the one that runs GPU checks: that profile separates
  mounts only, and the helper's header says "all devices" are allowed there. On the project's Spark
  that is a known, accepted setting. On a volunteer's box it is a gap: a GPU check from a hostile
  pull request could reach the volunteer's network. Either the connector closes the network for
  GPU checks too, or GPU checks are not run on community nodes. **Open; Drake to decide with the connector stream.**
- Memory, process count and run time are capped, and a crashed check leaves nothing running (the
  helper does this today: a cgroup limit and a stopped scope at the end).
- The checks cannot write anywhere that a later run reads: the helper makes the sandbox home
  read-only except the one worktree and one build folder for that commit.

**Still open:** a flaw in the Linux kernel or the GPU driver could let a check break out of
any user-level sandbox. The volunteer accepts that risk by running the node; `COMMUNITY.md` says so.

## 2. A dishonest or broken node
**What could happen.** A node signs PASS for code that fails. Reasons: the operator is cheating,
the operator's computer is compromised, the judge was edited, a bug, or a bad disk.

**What stops it:**
- **Registry.** Only keys Drake approved count. A receipt from any other key is advisory
  (`NODES.md`; test case 5 in `evidence/trust/verify-bundle.log`: a re-signed copy is refused against the
  registered key).
- **Two-node rule.** One node is never enough. Two different registered keys on two different
  GPUs must both say PASS on the same commit with the same `tree_digest` and `graph_digest`
  (`VERDICT_POLICY.md`, rule b). Both nodes would have to be dishonest, or both compromised.
- **Any FAIL blocks.** A single registered FAIL on the exact head stops community merging of it.
- **Spot re-verification.** The project's Spark re-runs every fifth community-merged head, and
  the first ten per node. A node that lies is caught with real odds on every merge it takes part in.
- **Revocation.** Drake marks the node `revoked` in `NODES.md`; the desk re-checks everything
  merged on its word.

**Still open:**
- Two nodes run by the same person, or two keys on one machine, would look like two. The registry
  records one GPU UUID per node and the policy asks for different UUIDs, but that is the operator's
  word; the receipt does not prove it. Mitigation: Drake knows each operator before approving, and one
  operator can hold at most one registered node unless Drake agrees otherwise.
- Receipts only vouch for the checks the repository requires. A weak check set is a weak check set
  whoever runs it.
- Spot re-verification catches a liar only some of the time. Between samples, a bad merge can happen.
  It is a deterrent plus a way of finding out, not a guarantee.

## 3. A stale or tampered judge
**What could happen.** The judge is the `aien-test` program. If a node runs an old version with
a known bug, or a copy edited to always say PASS, its receipts are worthless, and the signature
would still be valid.

**What stops it:**
- The receipt is meant to carry the judge's own commit in a signed field,
  `executor.judge_commit`, and each repository pins the oldest acceptable judge in
  `.aien/judge.min`. The desk refuses older ones (`VERDICT_POLICY.md`, "Stale judge").
- `aien-test verify-bundle` catches any edit to the receipt after signing (cases 2, 3 and 8).
- The installer builds the judge from source at a pinned commit (connector stream; to confirm
  against GUARANTEES.md). It does not download a ready-made program.

**What does NOT stop it, and is the biggest hole in this design:**
- The receipts the project's Spark publishes today do not carry a judge commit (see the `executor`
  block in `evidence/trust/bundle.original.json`). Until `aien-test` writes it, the version
  rule cannot be applied and **community merging (rule b) stays switched off.**
- Even with the field, a node's operator can write any commit they like into it and sign it. A
  forged claim of "I run the pinned judge" is only caught by the two-node rule and the spot checks.
  The field defends against honest nodes that forgot to upgrade, not against a determined liar.
- If the real judge had a bug that wrongly says PASS, every node and the project's Spark share it. That is a
  judge bug, not a trust-design problem, and is handled by fixing the judge and raising `.aien/judge.min`.

## 4. A volunteer's GitHub token
**What could happen.** The node needs some way to publish receipts. A token that can write to
AIEN code would be a way into the project if it leaked.

**What stops it (to confirm against GUARANTEES.md):**
- The node uses a token that can write **only to the operator's own fork** of
  `aien-dev/aien-receipts`, and the receipt reaches the project as a pull request from that fork.
  It has no write access to any AIEN repository.
- Reading open pull requests needs no write access at all (public repositories).
- The token never enters the sandbox user's environment.
- A leaked token can at worst spoil that one operator's fork. A pull request from the fork
  still passes through the registry check: it adds nothing the desk counts unless the receipt
  verifies under a registered key.

**Still open:** the node's signing key is more valuable than the token. If it leaks, someone else can
sign as that node until Drake revokes it. The connector keeps it in a mode-0600 file owned by
the node's user; there is no hardware protection.

## 5. Privacy: what a node sees
**What a node sees.** Only what is public: open pull requests on public aien-dev repositories,
the exact commit being judged, and the checks the project already publishes. The node never
receives private repositories, secrets, other operators' receipts it did not ask for, or any
information about who else runs a node.

**What the project sees of a node.** The registry holds the GitHub handle, the public key and
the GPU UUID the operator gave. Every receipt carries a `machine_digest` (a fingerprint of the
machine's make-up, not its name) and the signing times. The receipt does not contain the
operator's address, files, or logs. The node name appears on every receipt it signs; that is the
point, and the operator opts into it.

**Still open:** if a repository ever becomes private, nodes must not be asked to judge it.
That would need a separate decision.
