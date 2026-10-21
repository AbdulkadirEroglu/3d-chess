#!/usr/bin/env bash
# @file ci.sh
# @brief Every CI check in one place. Used by GitHub Actions, the pre-commit hook and locally.
# @note Locally this reuses your dev build cache. CI sets TEST_PROFILE=ci for a faster unoptimized build.
set -euo pipefail
cd "$(dirname "$0")/.."

TEST_PROFILE="${TEST_PROFILE:-dev}"

# @brief Runs one named check; groups its output when running on GitHub Actions.
step() {
    local name="$1"
    shift
    if [ -n "${GITHUB_ACTIONS:-}" ]; then echo "::group::$name"; else echo "==> $name"; fi
    "$@"
    if [ -n "${GITHUB_ACTIONS:-}" ]; then echo "::endgroup::"; fi
}

step "Format" cargo fmt --all --check
step "Clippy" cargo clippy --workspace --all-targets --profile "$TEST_PROFILE" -- -D warnings
step "Tests" cargo test --workspace --profile "$TEST_PROFILE"
step "Licenses and advisories" cargo deny --log-level error check
step "Unused dependencies" cargo machete
# The asset validator must accept the generated test theme (it checks the generator and itself).
validate_test_theme() {
    local dir
    dir="$(mktemp -d)"
    python3 scripts/make_test_theme.py "$dir" >/dev/null
    python3 scripts/check_asset.py "$dir" >/dev/null
    rm -rf "$dir"
}
step "Asset validator" validate_test_theme
step "Copy-paste detection" npx --yes jscpd@4 crates --format rust --min-lines 8 --threshold 3 --gitignore --silent

echo "All checks passed."
