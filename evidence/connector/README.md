# evidence/connector: the first node run, 2026-10-20

A second node (`spark-b87b-node2`, public key `8d750f9ae2d4086e…`) was installed on the project's own
Spark (spark-b87b) next to the project's judge, and judged one real open pull request:
aegis-runtime pull request #60, head `bfa6b636f33f44a68a48d8f560c069fd1f99882d`. The project's Spark
had judged the same head on 2026-10-16. Result: two independent signed receipts on one commit, both
PASS, both re-checked from GitHub with `aien-test verify-bundle`.

| file | what it shows |
|---|---|
| `install-run1.log` | first install stopped at a check of the judge checkout (an installer bug, fixed in the next commit) |
| `install-run2.log` | full install: judge built at aien-sovereign-core `10b4f851`, existing sandbox recognised and left alone, key made, registration block printed |
| `install-run3-idempotent.log` | a third install changes nothing: key, public key and settings byte-identical |
| `run-once-aegis-runtime-bfa6b636.log` | the node judging the head: chip check, the judge command (`--sandbox user`, node key, node state), re-sign, verify, push to the fork |
| `judge-log.aegis-runtime.<sha>.log` | the judge's own output: 3 gates PASS |
| `node-receipt.aegis-runtime.<sha>.json` | the node's signed receipt, with `executor.judge_commit` |
| `verify-two-receipts.sh` | re-checks both receipts from public URLs, plus 4 refusal cases |
| `verify-two-receipts.log`, `verify-two-receipts.rerun.log` | 6 of 6 as expected, before and after the final publish |
| `sandbox-probe.log` | a check's view from inside the sandbox: user `aien-run`, only loopback, no DNS, no route out, node key unreadable; control outside reaches GitHub |
| `refusals.log` | the node refuses a wide GitHub token, a second poller, and a head whose gates need the chip |
| `publish-final.log` | publication with the final code: receipt on the fork branch, pull request link recorded |
| `status.log` | `spark-connector status` after the run |
| `supervise-test.log` | the restart loop restarts a node that exits, and a second loop on the same node home is refused |

## Limits of this run
- Same machine as the project's Spark: own key, state, clones and judge build, but shared hardware,
  sandbox and dependency cache. A truly independent receipt needs a second physical Spark.
- The GitHub token here is the project owner's (push on every repository). The node refused it
  (`refusals.log` case 1); the runs above used `--project-spark-test` on purpose.
- Opening the receipts pull request was refused by this Spark's own GitHub guard (only the PR desk
  opens pull requests here). The receipt waits on branch
  `node-8d750f9ae2d4086e/aegis-runtime-bfa6b636f33f` of `drakestapleton/aien-receipts`; the desk
  opens the pull request.
- `install.sh --start` was not run here: with this Spark's owner token the node would refuse to start every round, by design.
  The loop itself was tested with a stand-in program (`supervise-test.log`).
- Builds and tests were pure Rust at `nice -n 19 ionice -c3`, without the chip and without the quiet lock.
