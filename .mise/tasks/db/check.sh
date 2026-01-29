#!/usr/bin/env bash
#MISE quiet=true

set -euo pipefail

file="/tmp/check"

cargo run --quiet -- cache check >"$file"
cat "$file"
