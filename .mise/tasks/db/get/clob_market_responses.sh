#!/usr/bin/env bash
#MISE quiet=true

set -euo pipefail

fjall get clob_market_responses "$@" | jq "del(.notifications_enabled, .tags, .rewards, .image, .icon)"
