#!/usr/bin/env bash

set -euo pipefail

project_root=${MISE_PROJECT_ROOT:-$(pwd)}
project_name=$(basename -- "$project_root")
config_path="$project_root/fnox.toml"
expected_keychain_service="$project_name"
expected_pass_prefix="$project_name/"

read_toml_value() {
  taplo get --file-path "$config_path" --strip-newline "$@"
}

keychain_service=$(read_toml_value "providers.keychain.service") || {
  echo "failed to read providers.keychain.service from $config_path" >&2
  exit 1
}
pass_prefix=$(read_toml_value "providers.pass.prefix") || {
  echo "failed to read providers.pass.prefix from $config_path" >&2
  exit 1
}

if [[ "$keychain_service" != "$expected_keychain_service" ]]; then
  echo "expected providers.keychain.service to be \"$expected_keychain_service\", got \"$keychain_service\"" >&2
  exit 1
fi

if [[ "$pass_prefix" != "$expected_pass_prefix" ]]; then
  echo "expected providers.pass.prefix to be \"$expected_pass_prefix\", got \"$pass_prefix\"" >&2
  exit 1
fi
