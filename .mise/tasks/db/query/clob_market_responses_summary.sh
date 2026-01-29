#!/usr/bin/env bash
#MISE quiet=true

set -euo pipefail

fjall list clob_market_responses --kind value "$@" | jq "{market_slug, question, description}"
