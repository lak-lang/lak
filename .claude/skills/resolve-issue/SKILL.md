---
name: resolve-issue
description: Resolve an issue described in ./issues/<slug>.md. Read the issue file, investigate, plan, implement, and test to fix the problem.
argument-hint: <slug or issue file path>
---

# Resolve Issue Skill

Resolves problems described in issue files under `./issues/`.

## Input

- Issue specification: $ARGUMENTS
  - Accepts a slug only (e.g., `unhelpful-toplevel-statement-error`) or a file path (e.g., `issues/unhelpful-toplevel-statement-error.md`)

## Execution Steps

### 1. Read the Issue File

Resolve the file path from the arguments and read the issue file:

- If only a slug is given: read as `./issues/<slug>.md`
- If a file path is given: read it directly

Issue files contain the following sections:

- **Summary**: A concise description of the problem
- **Details**: Specific code examples, error messages, and relevant code locations
- **Expected Behavior**: How it should work

### 2. Reproduce the Problem

Actually reproduce the problem described in the issue:

- If code examples are provided, compile and run them to confirm the problem
- Verify that the error messages match those described in the issue
- If the problem cannot be reproduced, report this to the user and ask for guidance

### 3. Investigation

Starting from the code locations mentioned in the issue, identify the root cause:

- Read the relevant code locations described in the "Details" section of the issue
- Investigate related code
- Refer to the language specification `.context/SPEC.md` to confirm the expected behavior per the spec

### 4. Planning

Based on the investigation results, enter Plan mode and formulate a fix strategy.

### 5. Implementation

Implement the fix according to the approved plan.

### 6. Testing

Verify that the fix is correct:

- Confirm that existing tests pass (`cargo test`)
- Confirm that the issue is resolved (re-verify using the same steps as Step 2)
- Add new tests as needed

### 7. Completion Report

Report the following to the user:

- Summary of the fix
- List of changed files
- Test results

## Notes

- Issue files do not contain proposed fixes (they only describe facts). You must investigate and formulate the fix strategy yourself
- Carefully consider the scope of impact and avoid breaking existing behavior
- If the fix becomes large-scale, confirm with the user during the planning phase
