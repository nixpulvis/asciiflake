#!/usr/bin/env bash
# Run one step of quineflake and show the result. The first run makes the
# quine from the metaquine, and each run after that has the quine print the
# next one in its place. Delete target/quineflake.rs to start over.
set -euo pipefail
cd "$(dirname "$0")/.."

quine=target/quineflake.rs
next=$(mktemp)
trap 'rm -f "$next"' EXIT

# Write to a temporary file first so a failed build leaves the quine alone.
if [[ -e $quine ]]; then
    cargo +nightly --quiet -Zscript "$quine" > "$next"
else
    cargo run --quiet --bin metaquineflake > "$next"
    mkdir -p "$(dirname "$quine")"
fi
cat "$next" > "$quine"
chmod +x "$quine"
cat "$quine"
