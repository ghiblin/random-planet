---
name: planet-pr-merge
description: Use when a validated Fractal Planet PR is ready to merge — squash-merges into main, removes the feature worktree, and resets the workflow state file
---

# Fractal Planet PR Merge

## Overview

Squash-merge a validated feature/phase PR into `main`, clean up its worktree and branch, and reset the shared workflow state file so the next feature can start clean. This is the final step of the Fractal Planet development workflow.

## Workflow state file — blocking gate

This skill coordinates with `planet-spec`, `planet-spec-review`, `planet-tdd`, and `planet-pr-validate` through a single state file at `<repo-root>/.claude/fractal-planet-workflow-state.json` — untracked (see `.gitignore`).

**Always resolve `<repo-root>` as the main repository root, never the current working directory.** Each feature worktree gets its own checked-out `.claude` directory, so a path relative to cwd would silently write a different file per worktree and break coordination. Resolve it with:

```bash
git rev-parse --git-common-dir   # e.g. /path/to/fractal-planet/.git — same for every worktree
```

`<repo-root>` is the parent directory of that path.

Schema:
```json
{
  "feature": "<slug>",
  "spec_file": "docs/specs/<NNN>-<slug>.md",
  "stage": "drafting-spec | ready-for-review | changes-requested | approved | implementing | complete | pr-changes-requested | validated",
  "updated_at": "<date>"
}
```

**HARD GATE:** read the file before doing anything else.
- Missing file: STOP. There is no feature tracked — nothing to merge.
- `stage` is anything other than `"validated"`: STOP. Tell the user the current stage and that `planet-pr-validate` must pass first (or `planet-tdd` first, if implementation itself isn't complete).
- `stage == "validated"`: proceed.

From the state file, `<feature>` is the `feature` slug. Derive (per `planet-spec` Phase 0 conventions):
- Branch: `feat/<feature>`
- Worktree path: `<repo-root>/.claude/worktrees/<feature>`

## Steps

### 1. Check CI

```bash
gh pr checks
```

If any checks are **failing or still pending**: halt and list the check names. Do not merge until all checks pass.

### 2. Squash-merge and delete remote branch

```bash
gh pr merge --squash --delete-branch
```

### 3. Remove the local worktree

Run from `<repo-root>` (not from inside the worktree):

```bash
git worktree remove <worktree-path> --force
```

`--force` is required because after a squash merge the worktree's branch has no direct ancestry on `main`.

### 4. Delete the local feature branch

```bash
git branch -D feat/<feature>
```

Use `-D` (force delete), not `-d` — after a squash merge, git's ancestry check does not consider the branch "fully merged".

### 5. Sync main

```bash
git checkout main
git pull
```

Confirm the latest commit message matches the PR title.

### 6. Reset the workflow state file

Delete `<repo-root>/.claude/fractal-planet-workflow-state.json`. Its absence is what `planet-spec`'s Phase 0 gate reads as "no feature in flight" — the next `planet-spec` run starts clean without needing to know anything about this feature.

```bash
rm <repo-root>/.claude/fractal-planet-workflow-state.json
```

### 7. Print confirmation

```
Merged:    <PR title>
Commit:    <latest commit SHA on main>
Branch:    feat/<feature> deleted (local + remote)
Worktree:  <worktree-path> removed
State:     fractal-planet-workflow-state.json reset

Done. You are now on main.
```

## Constraints

- Does not re-run tests, spec review, or security checks — assumes `planet-pr-validate` already passed with `stage: "validated"`
- Never merges with failing or pending CI checks
- Always uses `--force` / `-D` for cleanup — squash merge breaks standard ancestry checks
- Resets (deletes) the state file as its final action, not before
