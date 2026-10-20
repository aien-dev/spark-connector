#!/usr/bin/env bash
# verify-two-receipts.sh AIEN_TEST_BIN [WORK_DIR]
# Re-checks the two receipts on one head from public sources only:
#   A. the project's Spark: aien-dev/aien-receipts main, aegis-runtime/<sha>/bundle.json, key = aegis-runtime main .aien/verify.pub
#   B. the test node spark-b87b-node2: branch node-8d750f9ae2d4086e/aegis-runtime-bfa6b636f33f of drakestapleton/aien-receipts,
#      aegis-runtime/<sha>/bundle.8d750f9ae2d4086e.json, key = NODE_PUB below (printed by spark-connector join)
# plus negative controls: each receipt under the other key, a one-byte tamper, and the wrong commit. Each line says
# what exit code it expects (0 PASS, 1 FAIL, 2 NOT_RUN, 3 REJECTED) and what it got.
set -uo pipefail
BIN=${1:?usage: verify-two-receipts.sh AIEN_TEST_BIN [WORK_DIR]}
W=${2:-$(mktemp -d)}
SHA=bfa6b636f33f44a68a48d8f560c069fd1f99882d
NODE_PUB=8d750f9ae2d4086e5b67370e684f606aa8aa8755af600069aa85ffce4376b206
BRANCH=node-8d750f9ae2d4086e/aegis-runtime-bfa6b636f33f
REPO=github.com/aien-dev/aegis-runtime
mkdir -p "$W"
curl -fsSL "https://raw.githubusercontent.com/aien-dev/aien-receipts/main/aegis-runtime/$SHA/bundle.json" -o "$W/project.json"
curl -fsSL "https://raw.githubusercontent.com/drakestapleton/aien-receipts/$BRANCH/aegis-runtime/$SHA/bundle.8d750f9ae2d4086e.json" -o "$W/node.json"
curl -fsSL "https://raw.githubusercontent.com/aien-dev/aegis-runtime/main/.aien/verify.pub" -o "$W/project.pub"
REQ=$(curl -fsSL "https://raw.githubusercontent.com/aien-dev/aegis-runtime/main/.aien/always" | grep -v '^#' | grep -v '^$' | paste -sd, -)
sed '0,/"verdict":"PASS"/s//"verdict":"FAIL"/' "$W/node.json" > "$W/node.tampered.json"
echo "sha256 project.json $(sha256sum < "$W/project.json" | cut -c1-64)"
echo "sha256 node.json    $(sha256sum < "$W/node.json" | cut -c1-64)"
echo "project key $(tr -d '\n' < "$W/project.pub")"
echo "node key    $NODE_PUB"
echo "required    $REQ"
fail=0
check() { # check LABEL EXPECTED FILE PUBKEY [COMMIT]
  local out rc
  out=$("$BIN" verify-bundle "$3" --repo "$REPO" --commit "${5:-$SHA}" --pubkey "$4" --require "$REQ" 2>&1); rc=$?
  printf '%-58s expect %s got %s  %s\n' "$1" "$2" "$rc" "$out"
  [[ $rc == "$2" ]] || fail=1
}
check "A. project receipt, project key" 0 "$W/project.json" "$W/project.pub"
check "B. node receipt, node key" 0 "$W/node.json" "$NODE_PUB"
check "C. node receipt under the project key (refused)" 3 "$W/node.json" "$W/project.pub"
check "D. project receipt under the node key (refused)" 3 "$W/project.json" "$NODE_PUB"
check "E. node receipt with verdict flipped (refused)" 3 "$W/node.tampered.json" "$NODE_PUB"
check "F. node receipt for another commit (refused)" 3 "$W/node.json" "$NODE_PUB" 0000000000000000000000000000000000000000
grep -o '"executor":{[^}]*}' "$W/node.json" | sed 's/^/node executor block: /'
[[ $fail == 0 ]] && echo "ALL 6 AS EXPECTED" || echo "MISMATCH"
exit "$fail"
