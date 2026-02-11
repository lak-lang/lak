---
name: new-task
description: Create a new task spec file at .context/tasks/<slug>.md and always update .context/tasks/README.md in the same change. Use when users request adding a task document, drafting a new language feature task, or expanding the tasks index and dependency graph.
---

# New Task Skill

Create and register a new task specification under `.context/tasks/`.

## Input

- Task intent: `$ARGUMENTS`

## Workflow

1. Determine `<slug>`.
- Use concise English kebab-case.
- Avoid numeric prefixes and phase numbers.

2. Create `.context/tasks/<slug>.md`.
- Follow existing task style: `# Title`, `## Overview`, and concrete examples.
- Keep content focused on specification memo content only.
- Do not write implementation order or dependency graph inside the task file.

3. Update `.context/tasks/README.md` immediately after creating the task file.
- Add the new file under `## Task List` in the appropriate category.
- Add the new file to `## Recommended Implementation Order`.
- Add dependency lines in `## Inter-Task Dependencies` when applicable.
- Keep naming and dependency notation consistent with existing entries.

4. Validate consistency.
- Confirm the task file exists and the README references it.
- Ensure README and task file do not contradict each other.

5. Report completion.
- Return created file path and README sections updated.

## Non-Negotiable Rule

- Never finish after creating only `.context/tasks/<slug>.md`.
- Always include `.context/tasks/README.md` updates in the same task.
