#!/usr/bin/env bash
#MISE quiet=true
#USAGE arg "<question>"

set -euo pipefail

question=${usage_question?}

cache_subdir="clob_market_responses_summaries_by_question"
cache="$MISE_ORIGINAL_CWD/.cache/query/$cache_subdir/$question.jsonl"
mkdir -p "$(dirname "$cache")"

if [ -e "$cache" ]; then
  cat "$cache"
else
  mise run db:query:clob_market_responses_summaries | fx "?.question.includes('$question')" ".market_slug" | tee "$cache"
fi
