# Contract 9e — core and cli: the view after an edit

Planner: Claude. Status: implemented. Tier: light, with M9-6 (i) full (plan
M9-10).
Plan row: `docs/tasks/contracts/milestone-9-editor.md` §7, row 9e.

## 1. What this contract builds

After an edit, the view shows the edited record: its features, CDSs,
frames, ORFs, search hits and the overview strip. Core gains `rebuild`,
which makes a `ViewRecord` from an edit buffer the way `open_record`
makes one from a file. The view gains `apply_edit`, the one path every
later edit key takes. If an edit leaves the selected CDS unusable, the
view deselects it and says why (M9-6 i). Tests drive `apply_edit`
directly; no key edits yet.

Rules that apply, already settled:
- Plan §5 M9-6 (i) — CDSs are checked again exactly as on open, so
  `rebuild` must use `open_record`'s own feature and CDS code, not a copy.
- D041, D042, D052, D053 — what makes a CDS usable, and how a refusal is
  named; unchanged.
- Plan §6 "Speed" — this contract measures it.

## 2. What exists and must be reused

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `open_record`, `view_features`, `view_cdss`, `check_ceiling` | `crates/core/src/view_record.rs:316-462` | `rebuild` runs the same functions on a built `GenBankRecord` |
| `GenBankRecord::from_parts`, `GenBankFeature::new`, `GenBankQualifier::new` | `crates/core/src/genbank.rs` (9c1) | Build the record from the buffer |
| `write_edit_location` | `crates/core/src/edit_location.rs` (9c2) | A moved feature's location text |
| `EditBuffer::apply`, `to_sequence`, `EditOutcome` | `crates/core/src/edit_buffer.rs` | The edit itself |
| `ViewState::choose`, `relayout`, `gc_line`, `orfs`, `hits`, `search_record` | `crates/cli/src/sequence_view/state.rs:1110`, `:1487`; `overview.rs`; `search.rs` | What is recomputed |

## 3. Files the worker may touch

1. `crates/core/src/view_record.rs` — add `rebuild`. Move the GenBank arm
   of `open_record` into a private function both call, keeping
   `open_record`'s behaviour exactly.
2. `crates/core/tests/view_rebuild.rs` — new: tests 1–3.
3. `crates/cli/src/sequence_view/state.rs` — `apply_edit`, the last-search
   memory, and unit tests 4–8.
4. `crates/cli/src/sequence_view/search.rs` — only if the last search's
   mode and pattern need an accessor.

## 4. The public surface, exactly

```rust
// view_record.rs
/// The record as the view reads it after edits: `opened`'s format and
/// topology source; for GenBank, `GenBankRecord::from_parts` of `opened`'s
/// LOCUS name, accession and version, `buffer.to_sequence()`, and one
/// `GenBankFeature::new` per buffer feature (key; `location_text` or else
/// `write_edit_location`; its qualifiers; its `line`); then the same
/// features, CDSs and ceiling `open_record` gives. For FASTA, the sequence
/// and no features. `genbank` is the built record.
pub fn rebuild(buffer: &EditBuffer, opened: &ViewRecord) -> Result<ViewRecord, DomainError>;

// state.rs
impl ViewState {
    /// One edit, and the view after it: `EditBuffer::apply`, then
    /// `rebuild`, then everything §4 below lists. On `Err` nothing changes
    /// and the message holds the error's `message()`.
    pub(super) fn apply_edit(&mut self, edit: Edit) -> Result<EditOutcome, DomainError>;
}
```

After a successful edit, `apply_edit`:
1. **Record.** The view's `ViewRecord` becomes `rebuild`'s. The record as
   opened is kept unchanged in a new field `opened: ViewRecord`, for 9h's
   save.
2. **Selected CDS.** It is found again by its feature's `line`.
   - If it is still there and `Usable`, it stays selected, and its `Chosen`
     map is rebuilt.
   - If it is `Unusable`, or gone, it is deselected and the lanes go
     `Off`. The message names the CDS and gives the refusal's `message()`
     (M9-6 i).
   - With no CDS selected before the edit, none is selected after.
3. **Message.** Removed features (`EditOutcome::removed`) and dropped
   qualifiers are named in the message, after any CDS sentence: `removed:
   <name>, …` and `/transl_except dropped from <name>`.
4. **Overview.** `gc` is recomputed for the current width.
5. **ORFs.** If they were computed (`orfs` is `Some`), they are computed
   again for the topology the view reads. Otherwise they stay `None`.
6. **Hits.** The last search, which `ViewState` now remembers (mode and
   pattern), runs again on the new record. The current hit becomes the
   first at or after the cursor, or else the first. With no last search,
   there are none.
7. **Cursor and layout.** The cursor is clamped to the last base, and the
   layout is redone, anchored on the cursor's block.

## 5. Tests the worker writes

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `view_rebuild.rs::an_unedited_rebuild_is_the_opened_record` | pBR322 and `view/frames-62.gb`: `rebuild(buffer_for(r), r)` equals `r` in `sequence`, `features` (key, name, line, shape) and `cdss` (feature, id, status) |
| 2 | `…::a_frameshift_matches_open` (full tier) | Two GenBank records written by hand in the test, 30 bases: one with a usable `CDS 1..12` (`/gene`, `ATG…TAA`), and the same after one base inserted at base 5, its CDS `1..13`. `rebuild` of the first's buffer after `Insert { at: 4, .. }` of that base gives the CDS the same `CdsStatus` that `open_record` gives the second: `Unusable`, with the same error |
| 3 | `…::rebuild_keeps_the_ceiling` | A buffer one base over `MAX_RECORD_BASES` (built by inserting into a buffer at the ceiling) rebuilds to the same `RecordTooLong` `open_record` gives |
| 4 | `state.rs` unit: `selected_cds_stays_through_a_neutral_edit` | pBR322 with tet selected: an `Insert` of 3 bases at zero-based 50 (before tet): tet still selected, its first codon now at base 89 (1-based) |
| 5 | unit: `a_frameshift_deselects_with_the_reason` (full tier) | pBR322 with tet selected: `Insert` of 1 base at 200: no CDS selected, lanes `Off`, and the message holds the `message()` of the rebuilt tet's refusal |
| 6 | unit: `hits_follow_the_edit` | Search `CTGCAG` (PstI), then `Insert` of 3 bases at 0: every hit's start moved by 3. (Not EcoRI: its site crosses pBR322's origin, so an insertion at 0 splits it) |
| 7 | unit: `removal_is_named` | `Delete` of `[23, 27)` removes `misc_binding 24..27`. The message names it |
| 8 | unit: `edits_are_fast_enough` | pBR322, 20 one-base inserts through `apply_edit` with ORFs shown, in a debug build: each under 1 s. Report the slowest, and separately the time of one `apply_edit` on a 10 000 000-base FASTA |

## 6. Checks

`scripts/check`; `cargo test -p luria-core --test view_rebuild` (3 tests).

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | `apply_edit` keeps the old selection without checking its status | test 5 |
| 2 | `rebuild` writes every location with `write_edit_location`, ignoring `location_text` | test 1 (frames-62's origin joins, if their text differs; otherwise say so and skip) |
| 3 | Hits not recomputed | test 6 |

## 8. Do not

- Bind any key to an edit (9f, 9g).
- Copy `open_record`'s feature or CDS code: share it.
- Change what `open_record` returns for any file.

## 9. Report

Test 8's times. If test 8's limit fails, stop and report the profile of
one edit (which step takes the time) instead of optimising on your own.

## 10. Result (planner)

Test 8's times, measured by the planner. pBR322, slowest of 20 edits with
ORFs shown: 47 ms in a debug build, 7 ms in release. One edit on a
9 999 999-base FASTA: 10.1 s in debug, 1.65 s in release. The recompute stays
in full for now. Whether the largest records need a partial recompute
before 9f's typing is an open question for the owner. §7 proof 2 was skipped:
no fixture's location text differs from what `write_edit_location` writes.
