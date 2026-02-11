# Finding Schema

Use this exact structure per finding.

- `id`: Stable short id (`SPEC-001`, `INV-002`)
- `severity`: `critical|high|medium|low`
- `confidence`: `high|medium|low`
- `title`: One-line defect summary
- `evidence`:
  - `file`: repo-relative path
  - `line`: 1-based line number
  - `snippet`: short paraphrase of relevant code behavior
- `impact`: What breaks and for whom
- `repro_or_reasoning`: Minimal repro steps or strict reasoning chain
- `fix_direction`: Concrete fix direction (not full patch)
- `test_gap`: Missing test that should fail before fix

Reject candidate findings that cannot fill `evidence` and `impact` concretely.
