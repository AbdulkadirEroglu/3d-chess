---
name: audit
description: Audit uncommitted or recent changes in this chess game against AGENTS.md - duplicated logic, needless helpers or abstractions, missing comments or logging, glossary violations, scope creep, undocumented public items. Use after an agent finishes a task and before the maintainer commits.
argument-hint: "[git range or path, default: uncommitted changes]"
---

# Audit

Audit **$ARGUMENTS** (if empty: all uncommitted changes, `git diff HEAD` plus untracked files).

This is a read-only review. Report findings; don't fix anything unless the user asks.

## Checks

For each changed file, check:

1. **Duplication.** For every new function, type or constant, search the whole codebase (`rg`) for existing code doing the same or similar thing. Report any near-duplicate with both locations.
2. **Needless complexity.**
   - Helpers used only once that could live inline in one function.
   - Traits, generics, wrapper types or config options with only one use.
   - Code written "for later".
3. **Architecture rules** (`AGENTS.md`):
   - Input goes only through `MoveCommand`.
   - `chess_core` and `chess_ai` have no Bevy.
   - No `cozy-chess` types outside `chess_core`.
   - Asset paths only in `theme`.
   - The AI runs off the main thread.
4. **Comments.**
   - Every file has `//! @file` and `//! @brief`.
   - Every function has `/// @brief`.
   - Comments are short and explain *why*.
5. **Logging.**
   - Events are logged at the right level, with simple repeated events at `debug!`.
   - No per-frame or hot-loop logging.
   - No `println!`.
6. **Glossary.** Names match `ARCHITECTURE.md`; no synonyms.
7. **Scope.** Everything belongs to the current milestone in `ROADMAP.md`, and there are no unrelated edits.
8. **Docs.** `ARCHITECTURE.md` lists every new public item. `TECH_STACK.md` covers any dependency change.
9. **Tests.** New logic has tests; bug fixes have a regression test.
10. **Silenced lints.** Any `#[allow]`/`#[expect]` has a real reason.

Also run the duplicate detector:
```sh
npx --yes jscpd@4 crates --format rust --min-lines 8 --gitignore
```

## Output

A table: severity (must fix / should fix / nit), file:line, finding, suggested fix. Then one line: "ready to commit" or "needs changes". If nothing is wrong, say so plainly.
