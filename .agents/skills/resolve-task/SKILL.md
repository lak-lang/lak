---
name: resolve-task
description: Execute a task specification created under .context/tasks/<slug>.md. Use when users ask to implement or complete a task from the tasks directory, including investigation, coding, testing, and status updates.
---

# Resolve Task Skill

Implement language/compiler work described in task files under `.context/tasks/`.

## Input

- Task specification: `$ARGUMENTS`
  - Accept a slug (example: `if-expression`) or a file path (example: `.context/tasks/if-expression.md`).

## Workflow

1. Read the task file.
- If only a slug is given, read `.context/tasks/<slug>.md`.
- If a file path is given, read it directly.
- Extract required behavior, examples, and out-of-scope notes from the task.

2. Check task ordering and dependencies.
- Read `.context/tasks/README.md`.
- Confirm dependency conditions for the target task in `## Inter-Task Dependencies`.
- If prerequisites appear missing, report the risk and ask whether to proceed.

3. Investigate current implementation.
- Trace affected compiler/runtime layers using `AGENTS.md` and `.context/SPEC.md`.
- Identify exact code locations that must change (lexer/parser/semantic/codegen/runtime/tests/docs).
- Reproduce current behavior with a minimal sample when possible.

4. Plan implementation.
- Enter Plan mode.
- Define concrete edits and tests before coding.
- Keep scope aligned to the target task file.

5. Implement the task.
- Update source code to satisfy the task specification.
- Preserve existing behavior outside task scope.
- Add or adjust tests (unit tests, error tests, and e2e tests as needed).

6. Validate.
- Run focused tests first.
- Run broader tests when impact is cross-cutting.
- Re-run the task examples (or equivalent) to confirm expected behavior.

7. Update project status docs.
- Update `.context/IMPLEMENTATION_STATUS.md` for features completed by this task.
- Do not rewrite task intent in the task file unless the user explicitly requests edits.

8. Report completion.
- Summarize implemented behavior.
- List changed files and executed tests with results.
- Mention remaining follow-up work, if any.

## Guardrails

- Keep implementation strictly tied to the selected task.
- Prefer exact assertions in tests.
- Avoid destructive git operations.
