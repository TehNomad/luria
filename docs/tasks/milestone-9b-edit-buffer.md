# Contract 9b — core: the edit buffer

Planner: Claude. Status: draft. Tier: full.
Plan row: `docs/tasks/milestone-9-editor.md` §7, row 9b.

## 1. What this contract builds

A new core module, `edit_buffer`, that holds a record's bases and its
features while the user edits them. It applies one edit at a time:
insert, delete, replace or substitute a range of bases. After each edit it
moves every feature by fixed rules, and adjusts the qualifiers those rules
name. Nothing calls it yet: 9b2 reads location text into its types, 9c3
writes them out, and the cli contracts drive it from keys.

Rules that apply, already settled:
- `tests/fixtures/editor/README.md`, all of it — **the specification**:
  every rule in "Rules, in the table's terms", including the two
  **Reading** paragraphs. Implement exactly those rules. Its table
  `feature-edits.tsv` is the expected result of every case.
- Plan §5 M9-6 (a)–(h), M9-13, M9-14 — the owner's answers that README
  restates. M9-6 (i) (CDS checks) and M9-12 are not this contract's.
- D006 — core coordinates are zero-based half-open; the table's are
  1-based (convert in the test, §5).
- D036 — an origin-crossing feature is two ranges, the one ending at the
  record's last base first.
- D056 — records up to 10 000 000 bases; the buffer must stay fast there.

## 2. What exists and must be reused

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `DNA_ALPHABET`, `is_dna_letter` | `crates/core/src/sequence.rs:53`, `:86` | The 15 letters a buffer may hold (upper case only) |
| `normalize` | `crates/core/src/sequence.rs:323` | `to_sequence` builds the `Sequence` through it |
| `Strand` | `crates/core/src/strand.rs:7` | The location's strand |
| `Identifier` | `crates/core/src/identity.rs:119` | The buffer's name, as `Sequence` has one |
| `SequenceTopology` | `crates/core/src/sequence.rs:63` | Linear or circular; M9-14 applies to circular only |
| `DomainError` variants | `crates/core/src/error.rs:824-` | Reuse these errors and add none: `EmptySequenceContent`, `InvalidSequenceCharacters`, `CoordinateOutOfBounds`, `EmptyInterval`, `FeatureWithoutIntervals` |

## 3. Files the worker may touch

1. `crates/core/src/edit_buffer.rs` — new: the module, its unit tests.
2. `crates/core/src/lib.rs` — change: add `pub mod edit_buffer;` in
   alphabetical order, nothing else.
3. `crates/core/tests/feature_edit_fixtures.rs` — new: §5 tests 1–2.

Everything else is out of scope.

## 4. The public surface, exactly

```rust
/// A feature's location as the editor holds it (M9-17): its ranges in the
/// file's order, zero-based half-open (D006), an origin-crossing feature's
/// two ranges in D036 order. A minus-strand join lists its ranges as
/// `complement(join(…))` writes them, low to high.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditLocation {
    pub strand: Strand,
    pub ranges: Vec<(u64, u64)>,
    /// `<` on the first range's start.
    pub open_start: bool,
    /// `>` on the last range's end.
    pub open_end: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditFeature {
    pub key: String,
    pub location: EditLocation,
    /// Name and value in file order; a flag qualifier (`/pseudo`) has `None`.
    pub qualifiers: Vec<(String, Option<String>)>,
    /// The location text as read. Kept while no edit has changed
    /// `location`, and set to `None` by the first edit that does (a shift
    /// counts), so the writer can keep unmoved text exactly (plan M9-7).
    pub location_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edit {
    /// `bases` go before the zero-based base `at`; `at == length` appends.
    Insert { at: u64, bases: Vec<char> },
    /// Removes the bases `[start, end)`.
    Delete { start: u64, end: u64 },
    /// `[start, end)` replaced by `bases`, of any length (M9-6 e).
    Replace { start: u64, end: u64, bases: Vec<char> },
    /// The bases from `start` overwritten one for one by `bases` (`r`, `R`).
    Substitute { start: u64, bases: Vec<char> },
}

/// What an edit did that the status line names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditOutcome {
    /// Features left with no range, removed from the buffer, in the order
    /// they had in it, as they were before the edit (M9-6 c).
    pub removed: Vec<EditFeature>,
    /// Qualifiers M9-13 (a) removed: the feature's index in `features()`
    /// after the edit, and the qualifier's name, in feature then qualifier
    /// order.
    pub dropped: Vec<(usize, String)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EditBuffer { /* private */ }

impl EditBuffer {
    pub fn new(
        name: Identifier,
        topology: SequenceTopology,
        letters: Vec<char>,
        features: Vec<EditFeature>,
    ) -> Result<EditBuffer, DomainError>;
    pub fn name(&self) -> Identifier;
    pub fn topology(&self) -> SequenceTopology;
    pub fn length(&self) -> u64;
    pub fn letters(&self) -> &[char];
    pub fn features(&self) -> &[EditFeature];
    /// Applies one edit to the bases and every feature. On `Err` the buffer
    /// is unchanged.
    pub fn apply(&mut self, edit: &Edit) -> Result<EditOutcome, DomainError>;
    /// The bases as a `Sequence` with this name and topology, through
    /// `normalize`. Not called by `apply`: hashing is on demand.
    pub fn to_sequence(&self) -> Result<Sequence, DomainError>;
}
```

The rules, beyond the fixture README:

1. `new` refuses, in this order: no letters → `EmptySequenceContent`; a
   letter not in `DNA_ALPHABET` (lower case included) →
   `InvalidSequenceCharacters`, the first such letter's zero-based index and
   the letter; a feature with no ranges → `FeatureWithoutIntervals`
   (`feature` is its key); a range with `start >= end` → `EmptyInterval`
   (`position` is `start`); a range with `end > length` →
   `CoordinateOutOfBounds`. `sequence` in every payload is the name's text.
2. `apply` refuses, and changes nothing: `Insert` with `at > length`, or
   `Delete`, `Replace` or `Substitute` whose range passes the end →
   `CoordinateOutOfBounds`. A range with `start >= end` → `EmptyInterval`.
   An `Insert`, `Replace` or `Substitute` with no bases →
   `EmptySequenceContent`. A letter outside `DNA_ALPHABET` →
   `InvalidSequenceCharacters`, with its index in `bases`. An edit that would
   leave no bases (`Delete` of every base) → `EmptySequenceContent`.
3. `Substitute` covers `[start, start + bases.len())`. It moves nothing,
   keeps `location_text`, and counts as changing the bases of every feature
   it overlaps (`/translation`, positions).
4. `Replace` is the README's deletion then insertion, inside one `apply`:
   one `EditOutcome`, one change to the buffer.
5. A `/codon_start` whose value is not exactly `1`, `2` or `3` is left as
   written. A `/transl_except` or `/anticodon` whose value has no `pos:`
   the README's form reads is removed, and named in `dropped`, by any edit
   that changes the feature's bases or location.
6. Speed: one edit costs one move of the letters at most, plus work in
   proportion to the number of features and their ranges. No `Sequence`,
   hash or copy of the whole letters per edit.

## 5. Tests the worker writes

| # | Test (`file::name`) | Asserts |
| --- | --- | --- |
| 1 | `feature_edit_fixtures.rs::every_feature_edit_row_matches` | Reads `tests/fixtures/editor/feature-edits.tsv` at test time. For each row: a buffer of `length` `A`s with the row's topology and one feature (key, shape, qualifiers, `location_text` `Some("x")`); applies the edit; compares `length()`, the feature's shape, its qualifiers and the `named` column. Names the failing row's `id` |
| 2 | `feature_edit_fixtures.rs::location_text_follows_moves` | For every row, `location_text` is `Some("x")` after the edit exactly when the shape did not change, and the feature was not removed |
| 3 | `edit_buffer.rs` unit: `new_refuses_in_order` | Each refusal of §4 rule 1, one case each, the variant and payload |
| 4 | unit: `apply_refuses_and_changes_nothing` | Each refusal of rule 2; after each, `letters()` and `features()` equal their values before |
| 5 | unit: `letters_follow_each_edit` | `ACGTACGT`: insert `GG` at 2 → `ACGGGTACGT`; delete `[0, 2)`; replace `[1, 3)` by `T`; substitute `CC` at 0; the letters after each, exactly |
| 6 | unit: `several_features_and_removal_order` | Three features; one deletion removes the first and third and drops a `/transl_except` from the second: `removed` holds the two in order, `dropped` is `[(0, "transl_except")]`, `features()` holds the second |
| 7 | unit: `to_sequence_matches_normalize` | After two edits, `to_sequence()` equals `normalize` of the same letters, name and topology |
| 8 | unit: `edits_stay_fast_at_the_ceiling` | A 10 000 000-base buffer with 2 000 one-range features: 50 one-base inserts in the middle, then 50 one-base deletes, complete in under 5 s in a debug build. Report the time measured |

Test 1's conversion, from the README's "Columns": a 1-based `low..high`
is the range `(low - 1, high)`; `<` and `>` set `open_start` on the first
range and `open_end` on the last (fail the row if they appear elsewhere);
`ins K after N` is `Insert { at: N, bases: K × 'A' }`; `del A-B` is
`Delete { start: A - 1, end: B }`; `rep A-B K` is
`Replace { start: A - 1, end: B, bases: K × 'A' }`; `sub A-B` is
`Substitute { start: A - 1, bases: (B - A + 1) × 'C' }`. Qualifiers are
`name=value` pairs split at the first `=`; write them back the same way,
`;`-joined, `-` for none. `named` is `feature` when `removed` has the
feature, else the `dropped` names `,`-joined, else `-`.

## 6. Checks

`scripts/check`, plus:
- `cargo test -p luria-core --test feature_edit_fixtures` passes, 2 tests.

## 7. Proving the tests bite

Make each change once in `edit_buffer.rs`, run test 1, quote the failing
row's `id`, restore the file byte-identically, and report.

| # | Change | Must fail |
| --- | --- | --- |
| 1 | M9-14 at the end: no growth for `Insert` at `length` | `o1` |
| 2 | M9-14 at the start: no growth for `Insert` at 0 | `o2` |
| 3 | A mark is lost with its range instead of moving (README Reading 1) | `d4` |
| 4 | An absent `/codon_start` is never added (README Reading 2) | `s8` |
| 5 | The minus-strand 5′ end read from the low end | `s6` |
| 6 | `/translation` never dropped | `g3` |
| 7 | An insertion strictly inside a `pos:` does not touch it | `t5` |
| 8 | A deletion over a `pos:` does not touch it | `t4` |
| 9 | Insertion strictly inside a range shifts it instead of growing it | `b1` |
| 10 | `location_text` kept after a shift | test 2, row `a1` |

## 8. Do not

- No GenBank parsing or writing: no location text is read or formed here
  (9b2 and 9c2 do that). `location_text` is only kept or cleared.
- No undo, register or cursor: those are cli contracts.
- Do not change `Sequence`, `normalize`, `DomainError` or any other module.
- Do not edit `tests/fixtures/editor/`: if a row seems wrong, stop and
  report the row and why.

## 9. Report

The ten proofs with their failing row, test 8's measured time, and any
row of the table you could not make pass with the README's rules as
written.
