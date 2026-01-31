#!/usr/bin/env bash
#MISE quiet=true

set -euo pipefail

fjall get clob_markets "$@" | cargo run --quiet -- transcode --input rkyv --output serde_json --suffix "\n" --type Market | jq
