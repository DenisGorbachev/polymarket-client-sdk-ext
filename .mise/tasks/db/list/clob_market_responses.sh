#!/usr/bin/env bash
#MISE quiet=true

set -euo pipefail

fjall list clob_market_responses --kind value "$@" | jq -c "del(.notifications_enabled, .tags, .rewards, .image, .icon)"
