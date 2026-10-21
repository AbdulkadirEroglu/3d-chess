---
name: feature
description: Implement one feature or task in this chess game following the project workflow - read the docs, search for existing code, plan, implement, test, log, document, self-check. Use whenever asked to build, add, implement or change behavior in this repo.
argument-hint: <what to build>
---

# Feature workflow

Implement: **$ARGUMENTS**

Follow these steps in order. Don't skip any.

## 1. Load context
- Read `AGENTS.md`, `ARCHITECTURE.md`, and the current milestone in `ROADMAP.md`.
- If the task isn't in the current milestone, stop and say so. Don't build it.

## 2. Search before you write
- Find existing code for every concept the task touches: search for the glossary terms and likely function names with `rg`.
- List what you'll **reuse**, what you'll **extend**, and what is genuinely **new**.
- If something similar exists, extend it. Never write a second version.

## 3. Plan
- Write a short plan: the files to touch, the new public items (name + one line each), and the tests.
- If the plan touches more than 3 files or adds a new module, show it to the user and wait for approval.
- Prefer the simplest design. One function over several helpers, no abstractions "for later".

## 4. Implement
- Follow `AGENTS.md`: architecture rules, Doxygen comments (`@file`/`@brief` on every file and function), logging levels.
- Use glossary names only.
- Touch only what the task needs. No drive-by refactors or renames.

## 5. Test and check
- Add or adjust tests (a regression test for bugs).
- Run, and make sure all of these pass:
  ```sh
  cargo fmt --all
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  ```
- Never silence a lint to get green. Fix the code, or use `#[expect(lint, reason = "...")]` with a real reason.

## 6. Document
- Update `ARCHITECTURE.md` (code map, key items, status) for every public item added, moved or removed.
- Update `ROADMAP.md` checkboxes if a roadmap item is now complete.
- Update `TECH_STACK.md` if a dependency changed.

## 7. Report (don't commit)
End with:
- What changed, file by file (one line each).
- **New public items** (name + location) so duplicates are easy to spot.
- Check results (fmt / clippy / test) with the actual outcome.
- A suggested commit message. **Never run `git commit`.** The maintainer commits.
