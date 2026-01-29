#!/usr/bin/env bash
#MISE quiet=true

set -euo pipefail

file="/tmp/db-check"

cargo run --quiet -- cache check >"$file.tmp"
mv "$file.tmp" "$file"
cat "$file"
