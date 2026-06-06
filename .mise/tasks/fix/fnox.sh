#!/usr/bin/env bash

set -euo pipefail

project_root=${MISE_PROJECT_ROOT:-$(pwd)}
fnox_toml="$project_root/fnox.toml"
cargo_toml="$project_root/Cargo.toml"
package_name=$(taplo get --file-path "$cargo_toml" --strip-newline "package.name") || {
  echo "failed to read package.name from $cargo_toml" >&2
  exit 1
}
expected_keychain_service="$package_name"
expected_pass_prefix="$package_name/"

read_toml_value() {
  taplo get --file-path "$fnox_toml" --strip-newline "$@"
}

keychain_service=$(read_toml_value "providers.keychain.service") || {
  echo "failed to read providers.keychain.service from $fnox_toml" >&2
  exit 1
}
pass_prefix=$(read_toml_value "providers.pass.prefix") || {
  echo "failed to read providers.pass.prefix from $fnox_toml" >&2
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
