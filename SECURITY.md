# Security policy

## Reporting a vulnerability

Report privately. Do not open a public issue, pull request or discussion about it.

1. GitHub: open this repository's Security tab and choose "Report a vulnerability". This
   creates a private advisory that only the maintainers can see. If that option is not shown,
   private vulnerability reporting has not been enabled for this repository yet; use email.
2. Email: drake@aienos.com with the subject `[spark-connector] security report`.

Include the commit or release you tested, the component (path), steps to reproduce, and what an
attacker gains. If the report involves a secret, say where it was exposed and do not send the
secret itself. Encrypted mail is not required; do not post proof-of-concept payloads publicly.

## Response targets

This project has one maintainer; the targets below are best effort and stated honestly.

- Acknowledgement: within 3 working days.
- Triage and severity: within 7 days of acknowledgement.
- Fix or mitigation for confirmed issues: aim for 30 days for high and critical, 90 days
  otherwise, with a status update at least every 14 days until closed.
- Disclosure: coordinated. An advisory is published with credit (unless you ask otherwise)
  once the fix is on the default branch. Default embargo 90 days, negotiable.

## Scope

On `main` this repository is a plan with no code. In scope:

- Design flaws in `docs/PLAN.md` and `README.md`: the pull-only model, the sandbox user with no
  secrets and no network during a judge run, the node token limited to its own fork of
  aien-receipts, one bundle file per node, and the proposed two-node desk policy.
- Code on the stream branches `connector/v0` and `trust/v0`, once it exists, reported against the
  branch and commit you tested.

Out of scope: the judge `aien-test` (report to aien-sovereign-core); aien-receipts itself.

## Supported versions

Only the default branch `main`; there is no release. Fixes land on the default branch only.

## Status

Pre-alpha; nothing is proven yet and no volunteer receipt affects any merge or Alpha decision
(README). There is no certification and no bug bounty. See AUDIT_SCOPE.md for what to review and
how to build and test it.
