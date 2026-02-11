---
name: new-issue
description: Create an issue in markdown format as ./issues/<slug>.md. Document only the problem itself, without including any proposed fixes.
argument-hint: <problem description>
---

# Issue Creation Skill

Record issues reported by users as issue files in `./issues/<slug>.md`.

## Important Constraints

- Document **only the observed issue** in the file
- Do **not include any** fixes, solutions, or recommendations
- Do not include speculation about causes or implementation proposals

## Input

- Problem description: $ARGUMENTS

## Execution Steps

### 1. Understand the Issue

Based on the user's report, accurately understand the issue:

- Compile and execute as needed to verify the behavior
- Investigate the codebase as needed to identify specific locations
- Refer to `.context/SPEC.md` to confirm expected behavior according to the specification

### 2. Determine the slug

Decide on a concise English slug that represents the issue:

- Use kebab-case (e.g., `integer-overflow`, `import-shadowing`, `redundant-clone-calls`)
- Make it brief yet descriptive

### 3. Create the Issue File

Create `./issues/<slug>.md` using the following format.
Include **only relevant sections** based on the type of issue, and omit sections that do not apply.

```markdown
# [Issue Title]

## Summary

[Brief description of the issue (1-2 sentences)]

## Details

[Detailed explanation of the issue. Include specific code examples, error messages, output results, relevant code locations, etc.]

## Expected Behavior

[Explanation of how it should behave]
```

### 4. After File Creation

Report the created issue file path and a summary of its contents to the user.

## Notes

- Do not write fix proposals such as "should be", "is recommended", "can be fixed with"
- Do not create sections for root cause analysis or proposed fixes
- Focus on describing observable facts only
- Include error messages and output as-is without omission
