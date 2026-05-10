---
name: jj-split-changes
description:
  Split the jj working-copy change into many tiny named revisions; each code slice should compile
  and pass tests when possible. Use only jj for VCS; never git commit/amend/rebase/reset.
disable-model-invocation: true
---

# JJ split changes

**Bias:** more revisions, smaller diffs. Prefer peeling until each change is one intent and—as far as
the project allows—**builds and tests clean on its own**. Docs-only or asset-only slices skip
compile gates.

**Inspect:** `jj status`, `jj diff --summary`, `jj log -r 'ancestors(@, 8)'`.

**Plan before mutating:** bottom-of-stack → top; one line per slice (intent + paths/hunks). Call out
any slice that needs **interactive** hunk selection (`jj commit -i` or `jj split`)—do not fake that
in a non-interactive shell; ask the user to pick hunks, then continue.

**Peel one slice at a time** (smallest self-contained unit first when order allows):

- Whole files / paths stay in `@` and push the rest upward:

  ```bash
  jj commit -m "imperative intent, one story" <filesets>
  ```

  Selected paths stay in the current change; remainder becomes the new `@`.

- Partial files: `jj commit -i -m "..."` or `jj split` (interactive).

**After each peel:** `jj status`, `jj diff --summary`. **Code slices:** run the project’s usual check
(e.g. `cargo check`, `cargo test --lib`, `cargo clippy …`) before the next peel when it changed
Rust. If a slice cannot compile or test alone without its neighbor, either **fold it into the next
slice** or **stop** and say why—do not leave a broken middle revision without calling it out.

**Name:** `jj describe -m "..."` on the final `@` if it still has no message. Messages describe
**intent**, not “part 2” or “split”.

**Stack repair (only when needed):** `jj absorb` / `jj absorb <paths>` for obvious fixups into
earlier open changes; `jj squash --from … --into …` or `jj rebase -r … --insert-before|after …`
to move or reorder. Prefer explaining the move first. No broad abandon or history rewrite without
explicit approval.

**Safety:** no destructive jj unless the user asked. Unrelated local edits: separate slice or leave
alone—ask if unclear.
