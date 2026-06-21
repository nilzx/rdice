#!/usr/bin/env bash
set -euo pipefail

mode="${1:---dry-run}"
dependent_packages=(rdice-cli rdice-tui)

if [[ "$mode" != "--dry-run" && "$mode" != "--publish" ]]; then
  echo "Usage: bash scripts/release.sh [--dry-run|--publish]" >&2
  exit 2
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Working tree must be clean before release automation runs." >&2
  exit 1
fi

cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

cargo publish -p rdice-core --dry-run

for package in "${dependent_packages[@]}"; do
  if ! cargo publish -p "$package" --dry-run; then
    echo "Full dry-run for $package is blocked until rdice-core is indexed at this version."
    echo "Inspecting $package package contents without dependency verification."
    cargo package -p "$package" --no-verify --list >/dev/null
  fi
done

if [[ "$mode" == "--dry-run" ]]; then
  echo "Release dry-run completed."
  exit 0
fi

git push origin HEAD

cargo publish -p rdice-core

for package in "${dependent_packages[@]}"; do
  for attempt in {1..30}; do
    if cargo publish -p "$package" --dry-run; then
      cargo publish -p "$package"
      break
    fi

    if [[ "$attempt" == "30" ]]; then
      echo "Timed out waiting for rdice-core to be available for $package." >&2
      exit 1
    fi

    echo "Waiting for rdice-core to be indexed before publishing $package."
    sleep 20
  done
done

echo "Release publish completed."
