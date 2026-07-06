---
name: planet-spec
description: Use when writing a spec for a Fractal Planet feature/phase — guides BDD + DDD discovery through five phases before writing the spec file
---

# Fractal Planet Spec Writing

## Overview

Writing a spec before touching code prevents rework. This skill guides you through five discovery phases. Do not write the spec file until all five phases have complete answers.

## Workflow state file

`planet-spec`, `planet-spec-review`, and `planet-tdd` coordinate through a single state file at `<repo-root>/.claude/fractal-planet-workflow-state.json` — untracked (see `.gitignore`). It records which feature is in flight and blocks a skill from running out of order.

**Always resolve `<repo-root>` as the main repository root, never the current working directory.** Each feature worktree gets its own checked-out `.claude` directory, so a path relative to cwd would silently write a different file per worktree and break coordination. Resolve it with:

```bash
git rev-parse --git-common-dir   # e.g. /path/to/fractal-planet/.git — same for every worktree
```

`<repo-root>` is the parent directory of that path. Read and write `<repo-root>/.claude/fractal-planet-workflow-state.json` there regardless of which worktree the skill is currently running in.

Schema:
```json
{
  "feature": "<slug>",
  "spec_file": "docs/specs/<NNN>-<slug>.md",
  "branch": "<actual git branch name backing this feature's worktree>",
  "worktree_path": "<actual worktree path, relative to repo-root>",
  "stage": "drafting-spec | ready-for-review | changes-requested | approved | implementing | complete | pr-changes-requested | validated",
  "updated_at": "<date>"
}
```

`branch` and `worktree_path` record whatever a worktree-creation tool **actually** produced — never assume `feat/<slug>` / `.claude/worktrees/<slug>` literally. A native worktree tool, if available, may choose its own naming (e.g. sanitizing slashes, prefixing branch names); recording the real values keeps `planet-pr-merge` correct regardless of which mechanism created the worktree.

- If the file doesn't exist, this is the first skill run for this worktree — create it.
- If it exists for a **different** `feature` and `stage` is not `complete`: stop and tell the user another feature is mid-flight (name it and its stage). Ask whether to resume that feature instead, or confirm overwriting the state to start this one.
- If it exists for the **same** `feature`: resume — don't redo phases already answered in the existing spec file.

## Phase 0 — Create the worktree

**HARD GATE: no commit goes on `main`. If the active branch is `main`, stop.**

Before touching any file:
- Choose a descriptive slug for the feature/phase (e.g. `cube-render`, `domain-data-model`, `icosahedron-subdivision`) — check `docs/roadmap.md` for the next phase's name if this is a roadmap phase rather than an ad-hoc feature
- Invoke `superpowers:using-git-worktrees` (use the Skill tool directly), declaring a preferred worktree directory `.claude/worktrees/<slug>` and branch `feat/<slug>` — but a native worktree tool the skill defers to may choose different actual naming (e.g. sanitizing slashes). After creation, note the **actual** branch name and worktree path produced, whatever they are
- Write `<repo-root>/.claude/fractal-planet-workflow-state.json` per "Workflow state file" above, with `stage: "drafting-spec"`, and `branch`/`worktree_path` set to the actual values from the previous step (not the declared preference, if they differ)

All work — spec, implementation, tests — happens in this worktree.

## Phase 1 — Domain type(s) involved

Answer before moving on:
- Which type(s) in `planet-core` or `planet-renderer` are at the centre of this feature?
- Do they already exist, or are they new? Check `docs/specs/000-architecture.md` for the planned domain model
- Which fields/behaviour are relevant to this feature?

## Phase 2 — Operations

Answer before moving on:
- What does this feature compute or render? Is it pure generation logic (`planet-core`), rendering/input logic (`planet-renderer`), or both?
- Does it introduce a new public function/method, or change an existing one's contract?

## Phase 3 — Pre and post conditions

Answer before moving on:
- What must be true before the operation starts (e.g. a valid `Mesh`, a `SubdivisionDepth` within range)?
- What does the system guarantee after the operation completes (e.g. face count, determinism, no cracks)?

## Phase 4 — BDD scenarios

Write at least:
- One happy path scenario (Given/When/Then)
- One boundary/edge-case scenario (e.g. max depth reached, zero-length edge, empty preset list)

Follow the BDD scenario style in `rules.md` — reference fixtures explicitly (`Given an icosahedron mesh`, `Given a Planet generated with seed <n> and the <Preset> preset`, never a bare `Given a mesh`), and give sibling feature files the same core scenario set in the same order.

## Phase 5 — Acceptance criteria

Write a list of testable conditions. Each criterion must be answerable with pass/fail by a unit, integration, or BDD test. Vague criteria ("the mesh looks right") are not acceptable.

## Hard gate — final

Do not write the spec file until all five phases have complete answers. If any phase has no answer, the feature is not understood well enough to be specced.

Write the spec to: `docs/specs/<NNN>-<slug>.md` inside the worktree.

Required sections in the spec file:
1. Requirements
2. Domain model involved
3. Function/API contracts
4. BDD scenarios
5. Acceptance criteria

Once the spec file is written, update `<repo-root>/.claude/fractal-planet-workflow-state.json`: set `spec_file` to its path and `stage: "ready-for-review"`. This is what unblocks `planet-spec-review`.
