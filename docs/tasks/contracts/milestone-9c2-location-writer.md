# Contract 9c2 — core: writing a location back as GenBank text

Planner: Claude. Status: implemented. Tier: full.
Plan row: [docs/tasks/contracts/milestone-9-editor.md](/tasks/contracts/milestone-9-editor.md) §7, row 9c2.

## 1. What this contract builds

A function that writes an `EditLocation` (9b) as GenBank location text.
The text it writes reads back to the same location through 9b2's
`read_edit_location`, and Biopython reads it as the same intervals,
strand and partial ends. The planner gives every expected text in §5. The
worker puts them in a fixture table, with a Biopython check that does
not run luria (D039).

Rules that apply, already settled:
- Plan §5 M9-17, M9-18 — joins are written as `join(…)` /
  `complement(join(…))`; a one-base range is written as a bare `N`.
- D006, D036 — zero-based half-open ranges; an origin-crossing feature
  lists the range ending at the record's end first.
- D039 — the reference script lives under the fixture's `reference/`, has
  a `--check` mode, and is listed in `tests/fixtures/verify.py`.

## 2. What exists and must be reused

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `EditLocation` | `crates/core/src/edit_buffer.rs` | What is written |
| `read_edit_location` | `crates/core/src/edit_location.rs` (9b2) | Test 2 reads every text back |
| `tests/fixtures/editor/README.md`, "Columns" | — | The shape notation the new table uses (`+ <10..20,30..>40`) |
| `tests/fixtures/editor/reference/feature_edits.py` | — | The style for the new script: docstring, `--check`, `OK`/`DIFFERS` lines |
| `tests/fixtures/requirements.txt` | — | Biopython 1.88, already pinned; add nothing |

## 3. Files the worker may touch

1. `crates/core/src/edit_location.rs` — change: add `write_edit_location`
   and its unit tests.
2. `tests/fixtures/editor/locations.tsv` — new: §5's table, exactly.
3. `tests/fixtures/editor/reference/locations.py` — new: the Biopython check.
4. `tests/fixtures/editor/README.md` — change: add a section "Location
   text (9c2)" after "Shapes": the columns, the rules of §4, and the
   command that checks it. Change nothing else in the file.
5. `tests/fixtures/verify.py` — add `"editor/reference/locations.py"` after
   the feature-edits line.
6. `tests/fixtures/README.md` — in the `editor/` row and the verify
   paragraph, name `locations.tsv` and `locations.py`. Nothing else.
7. `crates/core/tests/location_text_fixtures.rs` — new: tests 1 and 2.

## 4. The public surface, exactly

```rust
/// The GenBank text of a location, on one line (9c3 wraps it).
pub fn write_edit_location(location: &EditLocation) -> String;
```

Rules:
1. A range `(s, e)` is written `s+1..e`. A one-base range (`e == s + 1`)
   is written `e`, as a bare number, unless it carries a mark: then
   `<s+1..e` or `s+1..>e`.
2. `<` goes before the first range's start when `open_start`, and `>`
   before the last range's end when `open_end`.
3. One range: the range. Several: `join(` the ranges `,`-joined `)`, in the
   location's order.
4. `Minus`: the whole text above inside `complement(…)`, so a minus join
   is `complement(join(…))`. `join(complement(…),…)` is never written.

## 5. Tests and the table

`locations.tsv` has the columns `id`, `length`, `topology`, `shape` (9a's
notation), and `text`. Write these rows exactly:

| id | length | topology | shape | text |
| --- | --- | --- | --- | --- |
| l1 | 100 | linear | `+ 30..40` | `30..40` |
| l2 | 100 | linear | `- 30..40` | `complement(30..40)` |
| l3 | 4361 | circular | `+ 426..426` | `426` |
| l4 | 100 | linear | `- 27..27` | `complement(27)` |
| l5 | 100 | linear | `+ <30..40` | `<30..40` |
| l6 | 100 | linear | `+ 30..>40` | `30..>40` |
| l7 | 100 | linear | `- <30..>40` | `complement(<30..>40)` |
| l8 | 100 | linear | `+ <30..30` | `<30..30` |
| l9 | 100 | linear | `+ 10..20,30..40` | `join(10..20,30..40)` |
| l10 | 100 | linear | `- 10..20,30..40` | `complement(join(10..20,30..40))` |
| l11 | 100 | linear | `+ <10..20,30..>40` | `join(<10..20,30..>40)` |
| l12 | 100 | linear | `- <10..20,30..40` | `complement(join(<10..20,30..40))` |
| l13 | 100 | linear | `+ 10..20,20..30` | `join(10..20,20..30)` |
| l14 | 100 | linear | `+ 10..20,25..25,30..40` | `join(10..20,25,30..40)` |
| l15 | 62 | circular | `+ 56..62,1..11` | `join(56..62,1..11)` |
| l16 | 62 | circular | `- 53..62,1..8` | `complement(join(53..62,1..8))` |
| l17 | 4366 | circular | `+ 4000..4366,1..10` | `join(4000..4366,1..10)` |

`locations.py` reads each row, builds a minimal GenBank record of
`length` bases (`n`s are fine) with one `misc_feature` at `text`, and reads
it with `Bio.SeqIO.read(…, "genbank")`. It then asserts that the
feature's parts, read low to high, are the shape's ranges, with the same
strand on every part, and with Biopython's `BeforePosition` /
`AfterPosition` exactly where the shape has `<` / `>`. Biopython lists a
minus join's parts 5′→3′, so reverse them before comparing. It writes
nothing, prints `OK locations.tsv (17 rows)` or `DIFFERS locations.tsv`
with the row, and exits 1 on a difference.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `location_text_fixtures.rs::every_location_writes_its_text` | For each row, `write_edit_location` of the shape is `text` |
| 2 | `location_text_fixtures.rs::every_text_reads_back` | For each row, `read_edit_location(text, length, topology)` is the shape |
| 3 | `edit_location.rs` unit: `write_then_read_is_identity` | 9b2's accepted rows from its §5 table: read, write, read again gives the same location |

## 6. Checks

`scripts/check`; `cargo test -p luria-core --test location_text_fixtures`
(2 tests); `.venv/bin/python -I tests/fixtures/verify.py` (all pass,
`locations.py` among them). If the sandbox has no `.venv`, report that and
run `python3 -I tests/fixtures/editor/reference/locations.py --check` if
Biopython imports.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | One-base ranges written `a..a` | test 1, l3, l4, l14 |
| 2 | Minus joins written `join(complement(…),…)` | test 1, l10, l12, l16 |
| 3 | In `locations.py`, skip the minus-strand reversal | its check, on l10 |

## 8. Do not

- Wrap lines (9c3 does).
- Change `read_edit_location`'s rules, 9a's files, or any other fixture.
- Make any Rust test call Python (D039).
