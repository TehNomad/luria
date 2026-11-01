# Review M5 C-3, C-7, C-8, C-13: CLI test gaps

Status: **Implemented 2026-10-18** by the local worker; accepted after one round (the `pair_summary` amendment in §2.3). Planner: Claude. Parent:
[docs/tasks/reviews/review-post-m5.md](/tasks/reviews/review-post-m5.md) C-3, C-7, C-8 and C-13. Requirements:
[docs/tasks/contracts/milestone-5-cli.md](/tasks/contracts/milestone-5-cli.md) §6 (end-to-end cases against
independently verified fixtures; each case run twice, byte-identical, D012),
D030 (annealing temperature: the lower Tm rounded half up, plus 1 °C, capped
at 72 °C), D040 (codon coordinates in feature interval order).

Test-only. Files:
- `crates/cli/tests/end_to_end.rs`
- `crates/cli/tests/run.rs` (only the Ta assertions, §2.3)
- `crates/cli/tests/write_edited.rs`
- `crates/core/src/primer_design.rs` (test module only)

## 1. Gaps

- **C-3.** §6 requires residue edits "including minus strand and
  origin-crossing, against `mutation/cases.tsv`" through the executable. No
  CLI test reads `cases.tsv`. The reviewer ran both by hand and they match.
- **C-7.** Only three cases run twice, and each pair runs from the same
  directory with the same absolute path, so a path leaking into the output
  couldn't fail the test.
- **C-8.** Ta is compared with `format!("{:.2}")`, so 63.996 would pass as
  `"64.00"`. Ta is a whole number by definition (D030). No unit test covers
  a lower Tm at exactly x.5.
- **C-13.** The deletion and insertion files that `--write-edited` writes are
  checked by length only, not against the document's
  `mutation.edited.sha256`.

## 2. Required change

### 2.1 C-3: two residue cases from `tests/fixtures/mutation/cases.tsv`

Add one test per row to `end_to_end.rs`. In each, write the fixture sequence
as a FASTA file into the test's workspace: read
`../../tests/fixtures/mutation/<file>`, drop lines starting with `;`, and
write `">" + reference + "\n" + <the remaining text, trimmed> + "\n"`. Do not
paste the sequence into the test. Run the request with
`--write-edited <workspace>/out`. If the existing `luria` helper can't pass
extra arguments, add a sibling helper that can; don't change `luria`.

**`res-a-minus-7`:** file `mut-a.txt`, reference `mut-a`.
```json
{
  "schema": "luria.q5-request/1",
  "coordinates": "zero-based-half-open",
  "template": {
    "id": "mut-a", "fasta": "mut-a.fa", "topology": "linear",
    "cds": [{ "id": "cds-2", "intervals": [[0, 30]], "strand": "-",
              "completeness": "complete", "geneticCode": "standard",
              "numbering": "from-cds-start" }]
  },
  "scope": "cds-2",
  "edit": { "name": "mut-a-res-minus",
            "residue": { "cds": "cds-2", "number": 7, "from": "I", "to": "L",
                         "policy": "default" } },
  "design": <design()>
}
```
Assert:
- exit 7 (the 30-base template can't be designed, G7) and `state`
  `verified`;
- `results[0].mutation`: `originalResidue` `"I"`, `writtenResidue` `"L"`,
  `writtenCodon` `"CTG"`, `coordinates` `[9, 10, 11]`;
- the written file `out/mut-a-res-minus.fa`, with its header line dropped
  and its other lines joined, equals
  `ATGGCTTGGCAGCAGCAGCAGCAGCAGCAG`.

**`res-c-origin`:** file `mut-c.txt`, reference `mut-c`, topology
`"circular"`, CDS `cds-1` with intervals `[[22, 24], [0, 7]]`, strand `"+"`,
scope `cds-1`, edit name `mut-c-res-origin`, residue number 1, from `"W"`,
to `"L"`, policy default. Assert:
- exit 7 and `state` `verified`;
- `writtenCodon` `"CTG"`, `coordinates` `[22, 23, 0]` (D040);
- the written file's sequence equals `GCTTGGAACGAACGAACGAACACT`.

The expected values are the `expected` column of those two rows
(`edited:`, `codon:`, `coords:`). If the exit status isn't 7, stop and
report the document rather than changing the expectation.

### 2.2 C-7: determinism for every case

- Change `document_at` so it runs the executable twice in the same
  workspace and asserts the two stdout texts are byte-identical (message:
  `"{test} is not deterministic"`), then returns the first. Every test that
  uses `document_at` or `document` is then covered.
- Add a test `the_output_does_not_depend_on_where_luria_runs`: set up two
  workspaces with different names. In the first, run with the request path
  absolute (as `luria` does). In the second, run with
  `Command::current_dir(<second workspace>)` and the request path given as
  the relative `request.json`. Use the D404N request from
  `every_case_is_byte_identical_when_run_twice`. Assert both exit 0 and both
  stdout texts are byte-identical.

### 2.3 C-8: exact Ta

- In `end_to_end.rs` and `run.rs`, wherever `annealingTemperature` is
  compared through `two(...)` or a formatted string, compare instead
  `value.as_f64()` with `Some(<the same number as f64>)`, e.g.
  `Some(60.0)`. Keep the expected numbers the tests already state. The Tm
  comparisons stay at two decimals.
- In `primer_design.rs`'s test module, add a test of `pair_summary`. If
  `MeltingTemperature` can't be built in a test from a chosen Celsius
  value, stop and report rather than adding a constructor. Expectations
  (planner-derived from D030; `f64::round` rounds half away from zero):

  | forward Tm | reverse Tm | expected Ta |
  | --- | --- | --- |
  | 65.0 | 62.5 | 64.0 |
  | 65.0 | 62.4999 | 63.0 |
  | 74.0 | 71.5 | 72.0 (73 capped) |
  | 70.6 | 73.0 | 72.0 |

  Compare the second element of the returned pair with `==`.

  **Amendment, 2026-10-18 (planner):** the worker reported that
  `MeltingTemperature` has private fields and no constructor from a chosen
  value. So this contract now allows one behavior-preserving production
  change in `primer_design.rs`: `pair_summary` takes
  `(forward_celsius: f64, reverse_celsius: f64)` instead of two
  `&MeltingTemperature`, with the same body on those values. Its two call
  sites pass `forward_tm.celsius()` and `reverse_tm.celsius()`. Its doc
  comment is unchanged apart from naming the parameters. No other production
  change is allowed.

### 2.4 C-13: written file against the document

In `write_edited.rs`, in the test(s) that write a deletion and an insertion
construct, add for each written file: the file's digest (the existing
`digest_of` helper) equals that member's `mutation.edited.sha256` in the
document from the same run. If no existing test writes a deletion or an
insertion, say so and add the assertion to the set test for every member.

## 3. How to proceed

These tests describe current behavior. They should pass without production
changes. If any fails, stop and report the failure output. Do not change
production code or fixture files.

## 4. Do not

- Change production code, fixtures, or any test not named above.
- Add dependencies.
- Paste fixture sequences into the tests.

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, the new and changed assertions, the check results,
and any deviation.
