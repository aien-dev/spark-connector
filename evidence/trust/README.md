# Evidence: does the receipt checker refuse what it should?

Date: 2026-10-19. Everything here is reproduced by `trust/verify-evidence.sh`.

- The checker is `aien-test verify-bundle`, built from aien-sovereign-core `main` at commit
  `10b4f851d6b082df6c2c477fbab700b551b854be` (`cargo build --release`, tools/aien-test).
- The real receipt is `bundle.original.json`, copied unchanged from aien-receipts:
  `aegis-runtime/f85dbe3910dd2c29faff74a3a96d657531ca1320/bundle.json`, signed by the project's Spark.
- `aegis-runtime.verify.pub` and `aegis-runtime.always` are the public key and the required-check
  list read from aegis-runtime's `.aien/` on its default branch on the same day.
- `verify-bundle.log` is the full output.

| case | what was checked | result |
|---|---|---|
| 1 | the real receipt | PASS, exit 0 |
| 2 | one check's verdict edited from PASS to FAIL | refused, BAD_SIGNATURE |
| 3 | one character of a receipt digest edited | refused, BAD_SIGNATURE |
| 4 | the real receipt offered for a different commit | refused, COMMIT_MISMATCH |
| 5 | same content signed by a key nobody registered, checked against the registered key | refused, UNKNOWN_KEY |
| 6 | that same file checked against its own key | PASS (so the file alone proves nothing; the registry decides which keys count) |
| 7 | a test-key receipt carrying an extra signed `executor.judge_commit` member | PASS (the checker already tolerates the extra member) |
| 8 | case 7 with `judge_commit` edited after signing | refused, BAD_SIGNATURE |

Limits, stated plainly:
- Cases 5 to 8 use a throwaway key made for the test. They are not community-node receipts: no
  community node exists yet. The connector stream's receipts have not landed on the branch
  `connector/v0` as of this run.
- Case 7 shows the checker accepts a `judge_commit` member. It does not show the judge writes one
  today: the real receipt in case 1 has no such member (see `trust/VERDICT_POLICY.md`).
- The checker was run on this machine only. Nothing was run on a second machine.
