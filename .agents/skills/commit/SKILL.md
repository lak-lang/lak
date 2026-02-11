---
name: commit
description: Create a Git commit safely and consistently in this repository. Use when the user asks to commit changes, prepare a commit message, or finalize edited files into version control.
---

# Commit Skill

Create a clean commit from repository changes.

## Input

- Commit intent and optional scope: `$ARGUMENTS`

## Workflow

1. Check working tree state.
- Run `git status --short`.
- Detect staged files, unstaged files, and untracked files.

2. Confirm commit scope.
- Include only files related to the current task.
- Exclude unrelated local changes.

3. Review changes before staging.
- Run `git diff` for unstaged changes.
- Run `git diff --staged` for already staged changes.

4. Stage files explicitly.
- Use `git add <specific-file-path>` for each file.
- Never use `git add .`, `git add --all`, or `git add -A`.

5. Build commit message from repository convention.
- Use `fix:` or `feat:` when behavior changes.
- Use `ci:`, `chore:`, or `docs:` when behavior does not change.
- Keep the subject concise and specific.

6. Create commit.
- Run `git commit -m "<type>: <summary>"`.

7. Report result.
- Show commit hash, subject, and committed file list.
- Mention skipped files if unrelated changes remain.

## Guardrails

- Avoid committing unrelated modifications.
- Avoid destructive git operations.
- Keep commit history atomic and reviewable.
