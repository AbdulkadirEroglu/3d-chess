#!/usr/bin/env bash
# @brief Claude Code Stop hook: if Rust code changed, run clippy and send errors back to the agent.
# @note Exit code 2 makes the agent continue and fix the errors; stop_hook_active prevents loops.

input=$(cat)
[ "$(jq -r '.stop_hook_active // false' <<<"$input")" = "true" ] && exit 0

cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0
[ -f Cargo.toml ] || exit 0

# Only check when Rust sources or manifests have uncommitted changes.
git status --porcelain -- '*.rs' '*Cargo.toml' | grep -q . || exit 0

if ! out=$(cargo clippy --workspace --all-targets --quiet -- -D warnings 2>&1); then
    echo "cargo clippy failed. Fix these before finishing (see AGENTS.md):" >&2
    echo "$out" | tail -n 80 >&2
    exit 2
fi
exit 0
