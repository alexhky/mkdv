# Branch and Worktree Workflow

This repository works as a standard Git checkout. Do not assume a particular
home-directory layout, a bare repository at `.bare`, or a shared `worktrees/`
directory. Feature branches are required before editing because the branch
protection hook rejects changes on `main`; additional Git worktrees are
optional.

## Before Editing

Check the active branch and preserve existing worktree changes:

```bash
git branch --show-current
git status --short
git log --oneline -5
```

If the current branch is `main`, create a scoped branch in the current checkout:

```bash
git switch -c feature/<short-name>
# or: git switch -c fix/<short-name>
# or: git switch -c docs/<short-name>
```

Branch creation keeps uncommitted files in place. Treat existing modifications
as user-owned and do not overwrite, discard, or include them accidentally.

## Optional Separate Worktree

Use a separate worktree when isolation or concurrent branch work is useful.
Run this from the repository root and choose an explicit sibling path:

```bash
git fetch origin
git worktree add ../mkdv-<short-name> -b feature/<short-name> origin/main
cd ../mkdv-<short-name>
```

If network access is unavailable but local `main` is known to be current, use
`main` as the final argument instead of `origin/main`.

List and remove optional worktrees with:

```bash
git worktree list
git worktree remove ../mkdv-<short-name>
```

Never remove a worktree that contains uncommitted changes without explicit user
approval.

## Feature Documentation

User-facing feature work and substantial fixes should add a numbered devlog;
see `devlog-workflow.md`. Start from `docs/devlog/TEMPLATE.md` and choose the
next available number. Small documentation-only corrections do not need a
devlog unless the user requests one.

## Before Writing Code

These checks are also covered by `context-awareness.md`:

1. Read every file you will modify.
2. Check recent commits relevant to the area.
3. Search `docs/LESSONS.md` for related gotchas.
4. Preserve established egui and renderer patterns.
5. Confirm the active branch is not `main`.

## Branch Protection

The hook in `.claude/hooks/` prevents edits on `main`. A branch is mandatory;
a separate worktree is a workflow choice, not a repository requirement.
