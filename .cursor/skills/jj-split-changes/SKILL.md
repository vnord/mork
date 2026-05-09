---
name: jj-split-changes
description:
  Split the current Jujutsu working-copy changeset into many smaller, well-described, logically
  self-contained named diffs. Use when explicitly asked to split jj changes, changesets, diffs, or
  work into smaller jj changes.
disable-model-invocation: true
---

# JJ Split Changes

## Goal

Turn the current jj working-copy change into a stack of small named changes. Each change should be
logically self-contained, reviewable, and as minimal as practical.

Use jj commands only for version-control operations. Do not use git commands to split, commit,
amend, reset, or rewrite history.

## Workflow

1. Inspect the current work:
   - `jj status`
   - `jj diff --summary`
   - `jj diff`
   - `jj log -r 'ancestors(@, 5)'`

2. Identify candidate slices:
   - Group by behavior or intent, not just by file.
   - Prefer the smallest slice that still compiles or makes conceptual sense.
   - Keep mechanical formatting, generated files, docs, tests, and behavior changes separate when
     they can stand alone.
   - Avoid preserving compatibility between in-progress slices unless a later slice needs to be
     independently runnable.

3. Present the split plan before mutating:
   - Proposed order from bottom to top.
   - Description for each change.
   - Files or hunks expected in each change.
   - Any slice that requires interactive hunk selection.

4. Create one named slice at a time:
   - For whole-file or fileset slices, prefer:
     ```bash
     jj commit -m "describe the first slice" <filesets>
     ```
     The selected filesets remain in the current change; the remaining changes move into a new
     working-copy change on top.
   - For partial-file slices, use jj's interactive split/commit flow:
     ```bash
     jj commit -i -m "describe the selected slice"
     ```
     or:
     ```bash
     jj split
     ```
     Do not pretend to drive an interactive diff editor through a non-interactive shell. Ask the
     user to make the hunk selection, then continue from the resulting state.

5. Repeat from the new working-copy change:
   - Re-run `jj status` and `jj diff --summary`.
   - Peel off the next smallest coherent slice.
   - Continue until the remaining working-copy change is itself one coherent named diff.

6. Use `jj absorb` when refining an existing stack:
   - Consider it when the working-copy change contains small fixes that clearly belong in earlier
     mutable changes.
   - Preview the relevant stack first:
     ```bash
     jj log -r 'ancestors(@, 10)'
     ```
   - Absorb all unambiguous fixes, or limit by path:
     ```bash
     jj absorb
     jj absorb <filesets>
     ```
   - Use explicit destinations when the intended stack range matters:
     ```bash
     jj absorb --into <revsets>
     ```
   - Review the operation before continuing:
     ```bash
     jj op show -p
     ```
   - Treat remaining changes as intentional; `jj absorb` leaves ambiguous changes in the source
     revision.

7. Repair and polish the stack when needed:
   - Split an earlier slice instead of restarting the whole workflow:
     ```bash
     jj split -r <revision>
     ```
   - Move a misplaced whole change into the right destination:
     ```bash
     jj squash --from <source-revision> --into <target-revision>
     ```
   - Reorder a completed slice when the stack reads better in a different order:
     ```bash
     jj rebase -r <revision> --insert-before <target-revision>
     jj rebase -r <revision> --insert-after <target-revision>
     ```
   - Re-check descendants after moving changes:
     ```bash
     jj status
     jj log -r 'ancestors(@, 10)'
     ```

8. Name the final remaining change:

   ```bash
   jj describe -m "describe the final slice"
   ```

9. Verify the result:
   - `jj log -r 'ancestors(@, 10)'`
   - `jj status`
   - `jj diff -r <each-created-change>` when needed
   - Run project-specific checks when the split changes behavior or tests.

## Naming Guidelines

Use concise imperative messages that explain the intent:

```text
Separate combat hit detection from damage application
Add stamina cost checks for player attacks
Move combat component registration into plugin setup
```

Avoid messages that describe the splitting operation itself:

```text
Split combat changes
WIP
Part 1
```

## Safety Rules

- Do not run destructive jj operations unless the user explicitly approves them.
- Do not squash, abandon, rebase broad ranges, or edit unrelated changes while splitting.
- Do not use `jj absorb` as a substitute for planning independent changes; use it for fixups into an
  existing stack.
- Do not use `jj squash` or `jj rebase` to paper over a bad split plan; explain the intended move
  first.
- If unrelated user changes are present, leave them untouched or make them a separate named slice
  only with user approval.
- If a clean split is impossible without editing code, stop and explain the blocker before making
  code changes.
- If tests fail after splitting but the original mixed change also failed, report that clearly
  instead of silently changing behavior.
