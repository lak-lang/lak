# Review Lenses

Run all lenses against the same target change set. Keep each pass independent, then merge only after validation.

## A. Spec-Consistency Reviewer

Mission: Detect mismatches between implementation and language/spec behavior.

Focus:
- Syntax/grammar behavior vs `.context/SPEC.md`
- Type rules, mutability, visibility, module rules
- Inconsistent handling between parser and semantic analyzer

Deliver:
- Only concrete mismatches with spec references.

## B. Compiler-Invariant Reviewer

Mission: Detect cross-stage inconsistencies in compiler pipeline.

Focus:
- AST changes reflected in lexer/parser/semantic/codegen/runtime/tests/docs where required
- Type/operator checks remain sound
- Module resolution, symbol binding, and codegen type mapping consistency

Deliver:
- Broken invariants and missing update locations.

## C. Diagnostics-and-Errors Reviewer

Mission: Detect degraded error quality and broken error-test assumptions.

Focus:
- Error message clarity, location labeling, and help text quality
- ANSI-colored diagnostics expectations in e2e/error tests
- Regression in exact-match test philosophy for error tests

Deliver:
- User-visible diagnostic regressions with evidence.

## D. Safety-and-Robustness Reviewer

Mission: Detect panic paths, unsafe assumptions, and abuse-prone logic.

Focus:
- Unhandled `None`/`Err` paths, unwrap-like risk, crash-only branches
- Boundary checks, malformed input handling, import/path edge cases
- Hidden trust assumptions on source/module content

Deliver:
- Conditions that can crash compiler or produce unsound behavior.

## E. Test-Regression Reviewer

Mission: Detect insufficient coverage for changed behavior.

Focus:
- Missing happy-path and error-path tests for changed code
- Missing end-to-end coverage when user-visible behavior changed
- Brittle/weak assertions that may hide regressions

Deliver:
- Specific missing tests with exact target files/modules.

## F. Performance-Complexity Reviewer

Mission: Detect likely compile-time regressions and unnecessary complexity.

Focus:
- Hot-path allocations/clones, repeated traversals, N^2 loops on AST/token streams
- Expensive operations added to frequently executed paths
- Simpler alternatives with same behavior

Deliver:
- Material performance risks with concrete hotspots.

## Required Output For Every Lens

Use the schema in `references/finding-schema.md` for each finding.
Return an empty list if no validated finding is found.
