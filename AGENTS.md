# Agent Instructions

## Project Context

Mork is a Rust/Bevy learning project for building a 3D dark fantasy roguelite with Souls-like
combat. The user wants to learn Rust, Bevy, game development, and asset creation while building the
game.

## Collaboration Style

- Default to small, easy-to-review diffs. Prefer one focused change over a broad cleanup, and leave
  unrelated refactors for a separate step unless they are needed to make the current change work.
- Before editing, explain the intended change in concrete terms. After editing, summarize what
  changed, why it changed, and how to verify it.
- Use TDD for pure logic when possible, with tests that make the next behavior clear.
- For ECS wiring, explain the entities/components/systems involved and provide small, verifiable
  steps.
- Keep explanations concrete and tied to the current files.

## Version Control

Use `jj` (Jujutsu) for version control workflow guidance in this repo.

- The repository is colocated with Git and jj.
- Prefer `jj status`, `jj log`, `jj diff`, `jj describe`, `jj new`, `jj split`, and `jj git push`.
- Do not assume Git-style commits. In jj, the working copy is already a change.
- When the user says "commit", explain the jj equivalent:
  - `jj describe -m "message"` names the current change.
  - `jj new` starts a new change on top.
  - `jj split` separates mixed changes.
  - `jj edit <change-id>` moves the working copy to an earlier change.
- Help the user build good jj habits: keep changes small, name them accurately, split mixed work,
  and inspect with `jj status`/`jj diff` before moving on.
- Push using jj bookmarks, e.g. `jj git push --bookmark main`, unless the user asks for a different
  bookmark.
- Do not run destructive VCS commands unless explicitly requested.

## Quality Gates

Before considering a coding task complete, run the relevant subset of:

- `cargo test --lib` for pure logic changes
- `cargo check` for compile verification
- `cargo clippy --all-targets --all-features -- -D warnings` for lint verification
- `prek run --all-files` when touching formatting/lint/hook-related files

## Documentation

- Keep `DESIGN.md` and `PHASES.md` up to date when decisions change.
- If workflow conventions change, update this `AGENTS.md`.
