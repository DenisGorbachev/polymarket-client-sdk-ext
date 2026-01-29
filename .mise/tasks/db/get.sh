#!/usr/bin/env bash
#MISE quiet=true

set -euo pipefail

fjall get "$@" | jq
