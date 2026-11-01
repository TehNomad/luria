# Review fix C2: make the Watson–Crick parameter check rerunnable

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review. Worst deviation 0.0147 kcal/mol (GA/CT).
Planner: Claude. Parent: [docs/tasks/reviews/review-post-m4.md](/tasks/reviews/review-post-m4.md) finding C2.

## 1. The gap

`tests/fixtures/tm/README.md` says the Watson–Crick table was verified
against published ΔG°37 values. `nn-matched.tsv` has no ΔG°37 column, and
`crates/core/tests/tm_tables.rs` reruns that check for the mismatch table
only (`every_mismatch_parameter_still_reproduces_its_published_free_energy`).
The Watson–Crick check can't be rerun from tracked files.

## 2. The published values (verified by the planner, 2026-10-16)

The 2004 review (SantaLucia & Hicks) returned HTTP 403 and could not be read.
The values below come from an open-access source with the same Watson–Crick
ΔG°37 set:

SantaLucia, J. Jr (1998), "A unified view of polymer, dumbbell, and
oligonucleotide DNA nearest-neighbor thermodynamics", *PNAS* **95**:1460–1465,
PMC19045. The doublet values are Table 1, column "Unified (ref. 22)". The
symmetry value is from the text: "ΔG°(sym) equals +0.43 kcal/mol … if the
duplex is self-complementary".

| motif | dG37 |
| --- | --- |
| AA/TT | -1.00 |
| AT/TA | -0.88 |
| TA/AT | -0.58 |
| CA/GT | -1.45 |
| GT/CA | -1.44 |
| CT/GA | -1.28 |
| GA/CT | -1.30 |
| CG/GC | -2.17 |
| GC/CG | -2.24 |
| GG/CC | -1.84 |
| symmetry | 0.43 |

The 1998 paper states initiation per duplex end: 0.98 for a terminal G·C and
1.03 for a terminal A·T. The fixture's 2004 formulation splits this into one
`initiation` term plus a `terminal_AT` term per A·T end, so the equivalent
values are:

- `initiation` = 2 × 0.98 = **1.96**;
- `terminal_AT` = 1.03 − 0.98 = **0.05**.

The planner checked by hand that every row's ΔH°/ΔS° reproduces these values
within 0.016 kcal/mol, using `ΔG°37 = ΔH° − 310.15 × ΔS° / 1000`.

## 3. Required changes

**3.1 `tests/fixtures/tm/nn-matched.tsv`**
- Add a fourth column `dG37_published` to the header line and to all 13 data
  rows, with exactly the values in §2.
- Do not change any existing ΔH° or ΔS° value, motif name or row order.
- Extend the comment block above the table to name the §2 source and the two
  derived values. Match the style of the comment block in `nn-mismatch.tsv`.

**3.2 `crates/core/tests/tm_tables.rs`**
- Add `every_watson_crick_parameter_still_reproduces_its_published_free_energy`,
  modelled on the existing mismatch test. Use the same identity, the same
  `0.13` tolerance with the same rounding comment, and fail when a row
  carries no `dG37_published`.
- Cover all 13 rows of `nn-matched.tsv`.
- `rows()` already reads an optional fourth column, so no parser change
  should be needed. If one is, change only what is required.
- `the_embedded_watson_crick_table_equals_the_fixture` must keep passing
  unchanged.

**3.3 `tests/fixtures/tm/README.md`.** Replace the paragraph saying the
Watson–Crick check "cannot be rerun from tracked files"
([docs/tasks/reviews/review-post-m4.md](/tasks/reviews/review-post-m4.md) C2) with a short statement of what §2 says:
- the 1998 source, and that the 2004 review could not be retrieved on
  2026-10-16;
- the two derived values;
- the test name that now reruns the check.

Keep the mismatch paragraph as it is.

## 4. Do not

- Edit files other than the three named above.
- Change any ΔH° or ΔS° value, any other fixture, or the Rust Tm code.
- Look values up anywhere else. §2 is the source; if a value seems wrong,
  stop and report.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 6. Handoff

Report the changed files, the new test, its worst observed deviation, the
exact check results, and any deviation from this contract.
