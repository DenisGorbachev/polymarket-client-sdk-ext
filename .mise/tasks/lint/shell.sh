#!/usr/bin/env bash

set -euo pipefail

fd --type f --extension sh --extension bash --extension zsh . . .mise/tasks .agents/skills .repoconf/hooks --exec-batch shellcheck --source-path "$HOME"
for file in ./*; do
  case "$file" in
    *.sh)
      if [ -f "$file" ]; then
        shellcheck --source-path "$HOME" "$file"
      fi
      ;;
  esac
done
