#!/bin/bash
#
# Usage: run_test.sh <expected-file> [excluded-types]
#
# The expected file holds the output of a build with every type feature enabled. Each
# feature-gated literal or typed null prints a single line (`Float16(...)` or `Null(Float16)`), so
# excluded-types is a `|`-separated list of type names (e.g. `Float16|Uint8`) whose lines are
# dropped from the expected output for features the FFI library was built without.

set -euxo pipefail

OUT_FILE=$(mktemp)
EXPECTED_FILE=$(mktemp)
EXCLUDED_LITERALS=${2:-}
if [ -n "$EXCLUDED_LITERALS" ]; then
  grep -vE "^ *(($EXCLUDED_LITERALS)\(|Null\(($EXCLUDED_LITERALS)\)$)" "$1" > "$EXPECTED_FILE"
else
  cp "$1" "$EXPECTED_FILE"
fi
./visit_expression | tee "$OUT_FILE"
diff -s "$OUT_FILE" "$EXPECTED_FILE"
DIFF_EXIT_CODE=$?
echo "Diff exited with $DIFF_EXIT_CODE"
rm "$OUT_FILE" "$EXPECTED_FILE"
exit "$DIFF_EXIT_CODE"
