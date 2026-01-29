#!/usr/bin/env bash
#MISE quiet=true

set -euo pipefail

cache_file_stem="clob_market_responses_summary"
cache="$MISE_ORIGINAL_CWD/.cache/query/$cache_file_stem.jsonl"
mkdir -p "$(dirname "$cache")"

if [ -e "$cache" ]; then
  cat "$cache"
else
  fjall list clob_market_responses --kind value | jq "{market_slug, question, description}" | tee "$cache"
fi
