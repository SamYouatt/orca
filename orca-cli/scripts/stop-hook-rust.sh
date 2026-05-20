#!/usr/bin/env bash
set -euo pipefail

crate_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
repo_dir="$(cd "$crate_dir/.." && pwd)"

cd "$repo_dir"

changed_rs_file="$(mktemp)"
trap 'rm -f "$changed_rs_file"' EXIT

git diff --name-only --diff-filter=ACMR HEAD -- 'orca-cli/*.rs' 'orca-cli/src/**/*.rs' 'orca-cli/tests/**/*.rs' |
  sed 's#^orca-cli/##' > "$changed_rs_file"

cd "$crate_dir"

if [[ -s "$changed_rs_file" ]]; then
  xargs rustfmt --edition 2024 < "$changed_rs_file"
else
  cargo fmt --quiet
fi

cargo clippy --all-targets --all-features
