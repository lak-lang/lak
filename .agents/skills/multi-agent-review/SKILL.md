---
name: multi-agent-review
description: Perform high-precision code reviews by orchestrating multiple review sub-agents with different lenses (language spec correctness, compiler invariants, safety, diagnostics quality, tests/regressions, and performance). Use when reviewing diffs, pull requests, risky refactors, parser/semantic/codegen changes, or when review quality must exceed a single-pass review.
---

# Multi-Agent Review

## Overview

Run a staged code review workflow that first understands the change, then executes multiple focused review sub-agents, validates each reported finding, and merges everything into one deduplicated severity-ranked report.

## Inputs

- Review target: staged diff (`git diff --cached`) by default, a commit range, or a pull request patch.
- Optional scope hints from user: module paths, risk focus, or skip areas.

## Workflow

1. Inspect and summarize the change before judging quality.
- Collect scope with `git status --short`, `git diff --cached --stat`, and `git diff --cached` (or a user-specified range).
- Treat staged changes as the review source of truth. Only inspect unstaged/untracked files when the user explicitly asks.
- Build a short change map: touched modules, behavior changes, and likely risk hotspots.
- Read related specs/docs when relevant (`.context/SPEC.md`, task docs, issue docs).

2. Launch sub-agents in parallel passes.
- Use every sub-agent defined in `references/sub-agents.md`.
- Give each sub-agent the same change map and diff context.
- Require output in the finding schema defined in `references/finding-schema.md`.

3. Validate every reported finding.
- Reject findings without concrete evidence (file path, line, failing condition).
- Verify correctness by reading code and checking repo rules/spec.
- Attempt lightweight reproduction where possible (targeted tests, focused commands).
- Downgrade or drop speculative findings.

4. Merge and deduplicate validated findings.
- Merge duplicates from different sub-agents into one item.
- Keep the highest justified severity and preserve supporting evidence.
- Sort by severity: critical, high, medium, low.

5. Produce final report.
- Follow output shape in `references/report-template.md`.
- Present findings first, ordered by severity, with precise file references.
- If no findings exist, state that explicitly and include residual risk/testing gaps.

## Execution Rules

- Prefer concrete, reproducible defects over stylistic comments.
- Flag behavioral regressions, unsound assumptions, and missing tests first.
- For Lak-specific changes, check parser/semantic/codegen consistency and error diagnostics quality.
- Never report a finding that was not validated in Step 3.

## References

- Sub-agent definitions: `references/sub-agents.md`
- Finding schema: `references/finding-schema.md`
- Final report format: `references/report-template.md`
