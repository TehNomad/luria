# Contract 9b2 — core: the editor's reading of a location, and the buffer from a record

Planner: Claude. Status: implemented. Tier: full.
Plan row: [docs/tasks/contracts/milestone-9-editor.md](/tasks/contracts/milestone-9-editor.md) §7, row 9b2.

## 1. What this contract builds

The editor's own reading of a GenBank location text into 9b's
`EditLocation`. It accepts more than the shared science parser does:
spliced joins (M9-17) and bare one-base positions (M9-18). It also adds the
function that builds an `EditBuffer` from an opened record, or names the
first feature whose location keeps the record from being edited (M9-12).
The view, Q5 and CDS import keep refusing these forms as they do today:
nothing here changes `parse_genbank_location` or `parse_display_location`.

Rules that apply, already settled:
- Plan §5 M9-12, M9-17, M9-18 — what is read, and that anything else
  blocks editing the whole record.
- D006, D036 — zero-based half-open ranges; an origin-crossing feature is
  the range ending at the record's end, then the one starting at 0.
- D009 — strand as written, never inferred.
- `crates/core/src/genbank_location.rs:1-60` — the shared parser's rules.
  This reader keeps its step 1 (whitespace), its refusal rule names and
  their order where they overlap.

## 2. What exists and must be reused

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `EditLocation`, `EditFeature`, `EditBuffer::new` | `crates/core/src/edit_buffer.rs` | The types read into, and the buffer built |
| `GenBankLocationRule`, `UnusableGenBankLocationProblem` | `crates/core/src/error.rs:600`, `:756` | Every refusal is `DomainError::UnusableGenBankLocation` with one of these rules; no new rule or variant |
| `parse_display_location`, `DisplayLocation` | `crates/core/src/display_location.rs:54`, `:67` | Test 3 compares against it |
| `ViewRecord`, `open_record`, `display_name` | `crates/core/src/view_record.rs:169`, `:462` | `buffer_for` reads the record; the blocked feature is named as the feature list names it |
| `GenBankRecord::features`, `GenBankFeature` | `crates/core/src/genbank.rs:100`, `:161` | Each feature's key, location text, qualifiers, line |

## 3. Files the worker may touch

1. `crates/core/src/edit_location.rs` — new: `read_edit_location`, its unit
   tests.
2. `crates/core/src/lib.rs` — add `pub mod edit_location;`, alphabetical.
3. `crates/core/src/edit_buffer.rs` — change: `EditFeature` gains
   `pub line: u64` (the feature's line in the opened file; buffers built in
   tests use any number); add `buffer_for` and `EditBlocked`; update the
   unit tests' struct literals for the new field, and nothing else in them.
4. `crates/core/tests/feature_edit_fixtures.rs` — change: the struct
   literal gains `line: 1`, nothing else.
5. `crates/core/src/view_record.rs` — change: `ViewRecord` gains
   `pub genbank: Option<GenBankRecord>`: `Some` with the record as read for a
   GenBank file, `None` for FASTA, set by `open_record`. Update every place
   in the workspace that builds a `ViewRecord` literally, adding the field
   and nothing else, and list them in the report.
6. `crates/core/tests/edit_location_fixtures.rs` — new: tests 3 and 4.

## 4. The public surface, exactly

```rust
// edit_location.rs
/// Read one location text for editing (M9-17, M9-18). `length` and
/// `topology` are the record's.
pub fn read_edit_location(
    text: &str,
    length: u64,
    topology: SequenceTopology,
) -> Result<EditLocation, DomainError>;

// edit_buffer.rs
/// Why a record cannot be edited (M9-12): the first feature, in file
/// order, whose location `read_edit_location` refuses.
#[derive(Clone, Debug, PartialEq)]
pub struct EditBlocked {
    /// Its index in `GenBankRecord::features()`.
    pub feature: usize,
    pub key: String,
    /// The name the feature list shows (`ViewFeature::name`).
    pub name: String,
    pub line: u64,
    pub error: DomainError,
}

/// The buffer for an opened record: its sequence's name, topology and
/// letters, and for a GenBank record every feature in file order, with
/// `location_text` set to `GenBankFeature::location()` and `qualifiers`
/// from `GenBankQualifier::name()`/`value()`. A FASTA record has none.
pub fn buffer_for(record: &ViewRecord) -> Result<EditBuffer, EditBlocked>;
```

The grammar, after the shared parser's step 1 (space, tab, `\r`, `\n`
removed):

```text
location := span
          | "complement(" span ")"
          | "join(" item ("," item)* ")"
          | "complement(join(" span ("," span)* "))"
item     := span | "complement(" span ")"
span     := ["<"] INT [".." [">"] INT]
INT      := one or more ASCII digits, fitting in u64
```

The rules, the first that applies deciding, every refusal carrying the
text as given, the rule, `length`, `cds: None`, `line: None`:

1. Nothing left after step 1 → `Syntax`. A `:` → `RemoteReference`. A `^`
   → `BetweenBases`. `order(` → `Order`. These four are checked before the
   grammar, in this order, as the shared parser checks them.
2. Not matching the grammar exactly, or a number not fitting `u64` →
   `Syntax`. A `<` or `>` on a bare `INT` (no `..`) → `FuzzyEnd`.
3. Each span, left to right: a position of 0 → `PositionZero`; an end
   before its start → `RangeEndsBeforeStart`; a position past `length` →
   `PastSequenceEnd`. A bare `N` is the range `(N - 1, N)`; `a..b` is
   `(a - 1, b)`.
4. Strand: `complement(…)` around the whole location, or around every item
   of a `join` → `Minus`. Some items complemented and some not →
   `UnsupportedJoin`. When every item of a `join` is complemented, the
   items are in 5′→3′ order, so the ranges are reversed into low-to-high
   order before rule 5.
5. Order: each range must start after the start of the range before it
   (touching or overlapping ranges are kept, as ribosomal slippage writes
   them), except at most one wrap: a range ending at `length` followed by
   one starting at 1. A wrap on a linear record → `OriginJoinOnLinear`. A
   second wrap, or any other step back → `UnsupportedJoin`.
6. Marks: `<` only on the first range's start and `>` only on the last
   range's end, after rule 4's reversal; anywhere else → `FuzzyEnd`. They
   set `open_start` and `open_end`.

`buffer_for`: for GenBank, read every feature with `read_edit_location`.
The first refusal returns `EditBlocked`, with the name taken from the
matching `ViewFeature`, at the same index. Then `EditBuffer::new`; an error
from it is impossible for a record `open_record` accepted, so `unreachable!`
with the feature's index is acceptable there.

## 5. Tests the worker writes

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `edit_location.rs` unit: `reads_every_table_row` | Every row of the table below: the `EditLocation`, or the refusal's rule |
| 2 | unit: `refusal_payload_is_the_text_as_given` | `" 30..\n101"` on length 100: `location` is the text unchanged, `sequence_length` 100, rule `PastSequenceEnd` |
| 3 | `edit_location_fixtures.rs::agrees_with_the_display_parser` | For every feature of `tests/fixtures/genbank/raw/J01749.1.gb`, `genbank/synthetic.gb`, `genbank/synthetic-linear.gb` and `view/frames-62.gb` (read with `read_genbank`) that `parse_display_location` accepts: `read_edit_location` gives the same strand and ranges, `open_start` exactly when `open_before == Some(first range's start)`, and `open_end` exactly when `open_after == Some(last range's end - 1)`. Report the count compared |
| 4 | `edit_location_fixtures.rs::pbr322_opens_for_editing` | `buffer_for(open_record(pBR322))` is `Ok`. It has every feature, in file order, with `location_text` equal to the file's text. `misc_difference 426` has the range `(425, 426)` |
| 5 | `edit_buffer.rs` unit: `a_blocked_record_names_its_first_feature` | `buffer_for(open_record(genbank/synthetic.gb))` is `Err(EditBlocked { feature: 18, key: "CDS", name: "ord", line: 61, .. })` with rule `Order`: `order(1..6,7..12)`, the first feature this reader refuses (every one before it is a range, an origin join, or `<1..12`). Read the file with `include_str!` or at test time |
| 6 | unit: `fasta_opens_with_no_features` | A FASTA `open_record` gives a buffer with the sequence's letters and no features |

Table for test 1 (length 100, linear, unless the row says; ranges
zero-based):

| Text | Result |
| --- | --- |
| `30..40` | `+ [(29,40)]` |
| `complement(30..40)` | `- [(29,40)]` |
| `426` (length 4361) | `+ [(425,426)]` |
| `complement(27)` | `- [(26,27)]` |
| `<30..40` / `30..>40` / `complement(<30..>40)` | marks: start / end / both |
| `join(10..20,30..40)` | `+ [(9,20),(29,40)]` |
| `complement(join(10..20,30..40))` | `- [(9,20),(29,40)]` |
| `join(complement(30..40),complement(10..20))` | `- [(9,20),(29,40)]` |
| `join(<10..20,30..>40)` | `+ [(9,20),(29,40)]`, both marks |
| `complement(join(<10..20,30..40))` | `- [(9,20),(29,40)]`, start mark |
| `join(10..20,20..30)` | `+ [(9,20),(19,30)]` |
| `join(10..20)` | `+ [(9,20)]` |
| `join(56..62,1..11)` (62, circular) | `+ [(55,62),(0,11)]` |
| `join(56..62,1..11)` (62, linear) | `OriginJoinOnLinear` |
| `join(complement(1..8),complement(53..62))` (62, circular) | `- [(52,62),(0,8)]` |
| `join(30..40,10..20)` | `UnsupportedJoin` |
| `join(10..20,complement(30..40))` | `UnsupportedJoin` |
| `join(10..>20,30..40)` | `FuzzyEnd` |
| `<426` (length 4361) | `FuzzyEnd` |
| `order(10..20,30..40)` | `Order` |
| `10^11` | `BetweenBases` |
| `J00194.1:100..202` | `RemoteReference` |
| `0..10` | `PositionZero` |
| `40..30` | `RangeEndsBeforeStart` |
| `30..101` | `PastSequenceEnd` |
| `join(10..20,30..40` | `Syntax` |

## 6. Checks

`scripts/check`, plus `cargo test -p luria-core --test edit_location_fixtures`
(2 tests) and `--test feature_edit_fixtures` (2 tests, unchanged results).

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | Rule 4's reversal skipped | test 1, `join(complement(30..40),complement(10..20))` |
| 2 | Rule 5 accepts a step back | test 1, `join(30..40,10..20)` |
| 3 | A bare `N` read as `(N, N + 1)` | tests 1 and 4 |
| 4 | Rule 6 accepts `>` on a first range | test 1, `join(10..>20,30..40)` |
| 5 | `buffer_for` skips refused features instead of blocking | test 5 |

## 8. Do not

- Change `genbank_location.rs`, `display_location.rs`, the view, Q5 or CDS
  import behaviour. Test 3 must not need any change to them.
- Change 9b's rules or tests beyond the `line` field.
- Write location text (9c2 does).

## 9. Report

The `ViewRecord` literal sites you updated, and test 3's count. If test 5's
expected feature is not the one this reader refuses first, stop and report
which one is and why.
