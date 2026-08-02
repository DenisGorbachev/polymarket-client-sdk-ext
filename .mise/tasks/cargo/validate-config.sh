#!/usr/bin/env bash

set -euo pipefail

project_root=${MISE_PROJECT_ROOT:-$(pwd)}
config="$project_root/.cargo/config.local.toml"

[[ -f $config ]] || exit 0

taplo lint --no-auto-config --no-schema "$config"
if taplo get --file-path "$config" --output-format json patch >/dev/null 2>&1; then
  echo "$config: local [patch] overrides are not allowed during final validation or commits; commit and push the dependency, remove the override, update Cargo.lock, and retry" >&2
  exit 1
fi
