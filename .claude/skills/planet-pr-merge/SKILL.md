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
  "branch": "feat/<slug> (enforced by planet-spec Phase 0)",
  "worktree_path": "<actual worktree path, relative to repo-root>",
  "stage": "drafting-spec | ready-for-review | changes-requested | approved | implementing | complete | pr-changes-requested | validated",
  "updated_at": "<date>"
}
```

**HARD GATE:** read the file before doing anything else.
- Missing file: STOP. There is no feature tracked — nothing to merge.
- `stage` is anything other than `"validated"`: STOP. Tell the user the current stage and that `planet-pr-validate` must pass first (or `planet-tdd` first, if implementation itself isn't complete).
- `stage == "validated"`: proceed.

From the state file, `<feature>` is the `feature` slug. `branch` is guaranteed to be `feat/<feature>` (enforced by `planet-spec` Phase 0). Read `worktree_path` **directly from the state file** rather than deriving `.claude/worktrees/<feature>` by convention, since a native worktree tool may have chosen different actual directory naming when `planet-spec` created the worktree.

## Steps

### 1. Check CI

```bash
gh pr checks
```

If any checks are **failing or still pending**: halt and list the check names. Do not merge until all checks pass.

### 2. Squash-merge (from `<repo-root>`, not the worktree)

**Run this from `<repo-root>`, never from inside the feature worktree.** `gh pr merge` has local git side effects beyond the API call — after a successful merge it tries to switch your current checkout to the base branch (and, with `--delete-branch`, delete the local head branch too). Both of those are `git checkout`/`git branch` operations, and git only allows one worktree at a time to have a given branch checked out. `main` is already checked out in `<repo-root>`'s own working tree; running `gh pr merge` from inside the feature worktree makes gh try to check out `main` *there* too, which git refuses with something like:

```
failed to run git: fatal: 'main' is already used by worktree at '<repo-root>'
```

— and this can happen *after* the remote squash-merge already succeeded via the API, so the command exits non-zero even though the PR is actually merged. Running from `<repo-root>` sidesteps this entirely, since `main` is already the checkout there:

```bash
cd <repo-root>
gh pr merge <PR-number> --squash
```

Pass the PR number explicitly rather than relying on gh to infer it from the current branch — `<repo-root>` is on `main`, not the feature branch, so there's nothing to infer from anyway.

Deliberately **omit `--delete-branch`**. Its local-branch deletion duplicates what Step 4 already does more carefully (worktree-aware, with the required `--force`/`-D`), and its checkout/deletion side effects are exactly what caused the failure above even when *not* run from the worktree, if the local head branch happens to be checked out in another worktree at merge time (which it always is, in this workflow). Delete the remote branch yourself instead, as a plain, side-effect-free ref deletion:

```bash
git push origin --delete <branch>
```

**If `gh pr merge` still errors with a `'main' is already used by worktree'`-style message even when run from `<repo-root>`:** don't retry it. Check whether the merge went through anyway before doing anything else:

```bash
gh pr view <PR-number> --json state,mergedAt,mergeCommit
```

If `state` is `MERGED`, the remote squash-merge already succeeded (this is common — the API call and the local git cleanup are separate steps, and only the latter failed). Skip straight to deleting the remote branch (`git push origin --delete <branch>`, or `gh api -X DELETE repos/<owner>/<repo>/git/refs/heads/<branch>` if the push is also blocked) and continue to Step 4. Only if `state` is not `MERGED` should you troubleshoot the merge itself.

### 3. Delete the remote branch

```bash
git push origin --delete <branch>
```

(Skip if the recovery path above already deleted it.)

### 4. Remove the local worktree and branch

If the worktree was created with a native tool (e.g. `EnterWorktree`), use its matching removal tool (e.g. `ExitWorktree` with `action: "remove"`) targeting `worktree_path` from the state file — it handles both the worktree and its branch, and keeps the harness's own bookkeeping consistent. Prefer it over raw git commands, same as `planet-spec` Phase 0 preferred the native creation tool.

Otherwise (manual `git worktree add` was used), run from `<repo-root>` (not from inside the worktree):

```bash
git worktree remove <worktree_path> --force
git branch -D <branch>
```

`--force` / `-D` are required because after a squash merge the worktree's branch has no direct ancestry on `main` — git's ancestry check does not consider it "fully merged".

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
Branch:    <branch> deleted (local + remote)
Worktree:  <worktree_path> removed
State:     fractal-planet-workflow-state.json reset

Done. You are now on main.
```

## Constraints

- Does not re-run tests, spec review, or security checks — assumes `planet-pr-validate` already passed with `stage: "validated"`
- Never merges with failing or pending CI checks
- Always runs `gh pr merge` from `<repo-root>`, never from inside the feature worktree, and never with `--delete-branch` — its local checkout/branch-deletion side effects conflict with git's one-checkout-per-branch rule across worktrees. Remote branch deletion is always a separate, explicit `git push origin --delete <branch>`
- If `gh pr merge` errors on a local git step, always check `gh pr view --json state,mergedAt,mergeCommit` before retrying anything — the remote squash-merge frequently already succeeded via the API
- Always uses `--force` / `-D` for local worktree/branch cleanup — squash merge breaks standard ancestry checks
- Resets (deletes) the state file as its final action, not before
