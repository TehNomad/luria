# Milestone 7d2: the reverse tail's context, tested

Status: **Implemented 2026-10-21** by the local worker; accepted after one
question (the template is 100 bases, not 99). The proof failed as it
should: with the reverse tail placed left, the window became `[27, 70)`. Planner: Claude. Parent:
[docs/tasks/contracts/milestone-7d-context.md](/tasks/contracts/milestone-7d-context.md) (implemented), whose proof 2 could
not fail because no fixture had a reverse-primer tail.

## 1. What and why

The planner added `tests/fixtures/review/context-ins-04.txt`, written by
`tests/fixtures/review/reference/context.py` from row `ins-04` of
`tests/fixtures/q5-primer/native-insertion.tsv`: an insertion of `GATTACA`
whose forward primer has the tail `TACA` and whose reverse primer has the
tail `ATC`. This contract adds the test that compares 7d's output with it,
and proves 7d's reverse-tail rule.

## 2. Changes

Add one test to `crates/cli/tests/review_context.rs`, and nothing else:

`the_ins_04_context_is_the_fixture`: a workspace under
`env!("CARGO_TARGET_TMPDIR")` holding `syn.fa`, a FASTA file whose header
is `>syn` and whose sequence is the `ins-04` row's `template` column
without its `seq:` prefix (read the row from the TSV at test time, by the
`case_id` column; do not paste the sequence), and this request:

```json
{
  "schema": "luria.q5-request/1",
  "coordinates": "zero-based-half-open",
  "template": {
    "id": "syn",
    "fasta": "syn.fa",
    "topology": "linear",
    "cds": [{"id": "syn", "intervals": [[0, 99]], "strand": "+",
             "completeness": "complete", "geneticCode": "standard",
             "numbering": "from-cds-start"}]
  },
  "edit": {"name": "ins-04", "insertion": {"point": 50,
           "before": "<base 49 of the template>",
           "after": "<base 50 of the template>", "insert": "GATTACA"}},
  "design": <end_to_end.rs's design(), bound 15>
}
```

with `before` and `after` taken from the template read from the row. First
assert the template's length is 100 (corrected 2026-10-21: the row is 100 bases) and the document's `state` is
`verified`. Then, as the file's other tests do, compare
`text::context(&document, 0, Some(&letters))`, each line followed by `\n`,
with the whole of `tests/fixtures/review/context-ins-04.txt`.

(The CDS is declared only because a FASTA request requires one; it plays
no part in an insertion's design.)

## 3. Proof

Disable 7d's reverse-tail rule once (place the reverse tail left of the
primer's first spanned column, as the forward tail is placed), run the
test, confirm it fails, restore, and report what you saw.

## 4. Do not

- Change only `crates/cli/tests/review_context.rs`, adding the one test
  and any helper it needs.
- Do not change `text.rs` (except to disable and restore for §3), any
  fixture, other tests, [docs/](/), any `Cargo.toml` or `vendor/`.
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

If the test fails against the unchanged code, stop and report the first
differing line: either 7d or the fixture is wrong, and the planner
decides which.
