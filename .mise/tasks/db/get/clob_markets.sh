#!/usr/bin/env bash
#MISE quiet=true

set -euo pipefail

fjall get --value-prefix len-u64-le clob_markets "$@" |
  cargo run --quiet -- transcode --input rkyv --output serde_json --suffix $'\n' --type Market |
  jq
