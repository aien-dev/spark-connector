#!/bin/sh
# Reproduces evidence/trust/: run the judge's bundle verifier on one real bundle from
# aien-receipts, on altered copies (must be refused), and on a bundle signed by a key that is
# not in the registry (must be refused). Usage: verify-evidence.sh AIEN_TEST_BINARY OUT_DIR
# OUT_DIR must already hold the files this script reads (see evidence/trust/README.md).
set -u
AT=${1:?path to aien-test}; E=${2:?evidence dir}
REPO=github.com/aien-dev/aegis-runtime
SHA=f85dbe3910dd2c29faff74a3a96d657531ca1320
PUB=$E/aegis-runtime.verify.pub
REQ=$(grep -v '^#' "$E/aegis-runtime.always" | paste -sd, -)
run() { # label, expected exit (0 or nonzero), then the command
  label=$1; want=$2; shift 2
  out=$("$@" 2>&1); rc=$?
  echo "### $label"; echo "\$ $*"; echo "$out"; echo "exit code: $rc"
  if [ "$want" = 0 ] && [ $rc -ne 0 ]; then echo "UNEXPECTED: wanted exit 0"; BAD=1; fi
  if [ "$want" != 0 ] && [ $rc -eq 0 ]; then echo "UNEXPECTED: wanted a refusal"; BAD=1; fi
  echo
}
BAD=0
T=$(mktemp -d)
run "1. original bundle, project key" 0 "$AT" verify-bundle "$E/bundle.original.json" --repo $REPO --commit $SHA --pubkey "$PUB" --require "$REQ"
# 2. one gate verdict flipped PASS -> FAIL in the file, signature left alone
sed '0,/"verdict":"PASS"/s//"verdict":"FAIL"/' "$E/bundle.original.json" > "$T/flip.json"
run "2. altered copy: first PASS changed to FAIL" 1 "$AT" verify-bundle "$T/flip.json" --repo $REPO --commit $SHA --pubkey "$PUB" --require "$REQ"
# 3. one hex digit of a receipt digest changed
sed '0,/"receipt":"7/s//"receipt":"6/' "$E/bundle.original.json" > "$T/digit.json"
run "3. altered copy: one receipt digit changed" 1 "$AT" verify-bundle "$T/digit.json" --repo $REPO --commit $SHA --pubkey "$PUB" --require "$REQ"
# 4. real bundle, asked about a different commit
run "4. original bundle presented for a different commit" 1 "$AT" verify-bundle "$E/bundle.original.json" --repo $REPO --commit 0000000000000000000000000000000000000000 --pubkey "$PUB" --require "$REQ"
# 5-8 use a throwaway key (a stranger's node). The signer insists the executor id is the
# fingerprint (sha256 of the public key bytes) of the key that signs, so the id is replaced too.
OLDID=$(grep -o '"executor":{[^}]*' "$E/bundle.original.json" | grep -o '"id":"[0-9a-f]*"' | cut -d'"' -f4)
"$AT" keygen "$T/stranger.key" | sed -n 's/^pubkey //p' > "$T/stranger.pub"
NEWID=$(tr -d '\n' < "$T/stranger.pub" | xxd -r -p | sha256sum | cut -d' ' -f1)
sed "s/$OLDID/$NEWID/" "$E/bundle.original.json" > "$T/stranger.unsigned.json"
"$AT" bundle-sign "$T/stranger.unsigned.json" --key "$T/stranger.key" --out "$T/resigned.json" >/dev/null
run "5. same content signed by an unregistered key, checked against the registered key" 1 "$AT" verify-bundle "$T/resigned.json" --repo $REPO --commit $SHA --pubkey "$PUB" --require "$REQ"
run "6. the same bundle checked against the stranger's own key (the file is well formed; only the registry decides whether that key counts)" 0 "$AT" verify-bundle "$T/resigned.json" --repo $REPO --commit $SHA --pubkey "$T/stranger.pub" --require "$REQ"
# 7. a signed extra member in the executor block (the judge version) is accepted by the verifier as it is today
JC=10b4f851d6b082df6c2c477fbab700b551b854be
sed "s/\"class\":\"spark\"/\"class\":\"spark\",\"judge_commit\":\"$JC\"/" "$T/stranger.unsigned.json" > "$T/jc.unsigned.json"
"$AT" bundle-sign "$T/jc.unsigned.json" --key "$T/stranger.key" --out "$T/jc.json" >/dev/null
run "7. test-key bundle that also carries executor.judge_commit, signed" 0 "$AT" verify-bundle "$T/jc.json" --repo $REPO --commit $SHA --pubkey "$T/stranger.pub" --require "$REQ"
sed "s/$JC/0000000000000000000000000000000000000000/" "$T/jc.json" > "$T/jc.tampered.json"
run "8. same bundle with judge_commit edited after signing" 1 "$AT" verify-bundle "$T/jc.tampered.json" --repo $REPO --commit $SHA --pubkey "$T/stranger.pub" --require "$REQ"
rm -rf "$T"
[ $BAD -eq 0 ] && echo "RESULT: every case behaved as the policy needs" || echo "RESULT: A CASE DID NOT BEHAVE AS EXPECTED"
exit $BAD
