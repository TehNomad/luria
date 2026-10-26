# Contract 8i1 — core: matching a name to a CDS (M8-15)

Planner: Claude. Status: not yet delegated. Tier: full (D051). Plan rows:
8i1 and 8i2 replace **8i** in `docs/tasks/milestone-8-view.md` §7, by D056.
`luria find --cds` (8i2), go-to (8k2) and the feature list (8k3) use this.

## 1. What this contract builds

`crates/core/src/name_match.rs`: one matcher that turns a value the user typed
into one item, several candidates, or none, by D049's rules. CDSs come first:
each `ViewCds` now carries every id D049 names, with its source, and
`match_cds` matches a value against them. It returns candidates; it never
picks among several and never refuses. The cli decides what each answer
means (D056).

Rules that apply, already settled:
- D049 (M8-15, `scripts/decision D049`): the ids are the display name,
  `/gene`, `/locus_tag` and `/protein_id`, and the list form
  `name (start..end)` matched only as the whole string. Matching ignores ASCII
  case and interior whitespace. A prefix must start at a word boundary, and it
  is accepted only when it matches one CDS.
- D053: the display name, as `view_record::display_name` gives it.
- D056: the matcher is core's; the cli turns its answers into exit codes.
- D036: a span is read in D036 order, origin joins included.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `display_name(feature)` | `view_record.rs:203` (private) | the display name; do not write a second one |
| `view_cdss(record)` | `view_record.rs:260` | where `ViewCds` is built, the one construction site (`:311`) |
| `GenBankFeature::qualifiers()`, `GenBankQualifier::{name, value}` | `genbank.rs:118`, `genbank.rs:88-93` | the `/gene`, `/locus_tag` and `/protein_id` values |
| `FeatureShape::Drawn(DisplayLocation)` | `view_record.rs:110` | a CDS's span; `location.intervals()` is already in D036 order |

## 3. Files the worker may touch

1. `crates/core/src/name_match.rs`: new, with its `mod tests`.
2. `crates/core/src/view_record.rs`: `ViewCds` gains `names`, filled in
   `view_cdss`; nothing else changes.
3. `crates/core/src/lib.rs`: one line, `pub mod name_match;`, between
   `pub mod mutation;` and `pub mod overview;`.
4. `crates/core/tests/name_match_fixtures.rs`: new.

## 4. The public surface, exactly

```rust
// view_record.rs — ViewCds gains one field:
/// Every id D049 matches, in this order: the display name, then each
/// non-empty `/gene`, `/locus_tag` and `/protein_id` value, each source in
/// file order. Duplicates are kept: the first source is reported.
pub names: Vec<(NameSource, String)>,

// name_match.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameSource { DisplayName, Gene, LocusTag, ProteinId, ListForm }

#[derive(Clone, Debug, PartialEq)]
pub struct NamedItem {
    pub item: usize,          // the caller's index (for CDSs, into `record.cdss`)
    pub source: NameSource,
    pub name: String,         // as written in the record
    pub whole_only: bool,     // true for the list form: never prefix-matched
}

#[derive(Clone, Debug, PartialEq)]
pub struct NameMatch { pub item: usize, pub source: NameSource, pub name: String }

#[derive(Clone, Debug, PartialEq)]
pub enum Matched { One(NameMatch), Several(Vec<NameMatch>), Nothing }

pub fn match_name(items: &[NamedItem], value: &str) -> Matched;
/// 1-based first and last base in D036 order; `None` for a CDS not drawn.
pub fn cds_span(record: &ViewRecord, cds: usize) -> Option<(u64, u64)>;
/// Every name of every CDS, plus `"{display name} ({first}..{last})"`
/// (`ListForm`, whole only) for each drawn one; `item` indexes `record.cdss`.
pub fn cds_names(record: &ViewRecord) -> Vec<NamedItem>;
pub fn match_cds(record: &ViewRecord, value: &str) -> Matched; // match_name(&cds_names(..), ..)
```

Rules for `match_name`, each pinned by the §5 tests in brackets:

1. **Fold.** Compare after ASCII lower-casing and removing every whitespace
   character. A value that folds to empty matches `Nothing`. [2, 7]
2. **Exact first.** A name matches exactly when the folded name equals the
   folded value. If any name matches exactly, only exact matches count.
   [1, 6]
3. **Then a prefix at a word start.** A word starts at an alphanumeric
   character that is the first of the name or follows a non-alphanumeric
   one, in the name as written. A name matches when, from some word start,
   the folded rest of the name begins with the folded value. `whole_only`
   names take no part in this tier. [3, 6]
4. **Items, not names.** Matches are grouped by `item`. For each item, report
   the first of its matching names in the order given. One item is `One`,
   two or more are `Several` in ascending `item` order. [4, 5]

`cds_span`: the drawn shape's first interval's start + 1 and last interval's
end, for either strand (`pj` → `(55, 12)`, `bla` → `(3293, 4153)`).

## 5. Tests the worker writes

Units in `name_match.rs` on hand-built `NamedItem` lists (`DN` = DisplayName):

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `an_exact_name_beats_a_prefix` | items 0 `lacZ` DN, 1 `lacZalpha` DN: `LACZ` → `One` item 0; `lacza` → `One` item 1 |
| 2 | `case_and_interior_whitespace_are_ignored` | item 0 `ROP protein` DN: `rop protein`, `ROPprotein`, ` rop  PROTEIN ` → `One` item 0 |
| 3 | `a_prefix_must_start_a_word` | item 0 `beta-lactamase`: `lact`, `beta`, `BETA-LAC` → `One`; `actam` → `Nothing`. Item 0 `ROP protein`: `prot` → `One`; `rotein` → `Nothing` |
| 4 | `several_items_are_several` | items 0, 1, 2 `AAB59735.1`, `AAB59736.1`, `AAB59737.1` (ProteinId): `AAB5973` → `Several` items 0, 1, 2 in that order; `AAB59736` → `One` item 1 |
| 5 | `one_item_named_twice_is_one_match` | item 0 `tet` DN and item 0 `tet` Gene: `tet` → `One { item: 0, source: DisplayName, name: "tet" }` |
| 6 | `a_whole_only_name_is_never_a_prefix` | item 0 `tet` DN and item 0 `tet (86..1276)` ListForm, whole only: `TET (86..1276)` → `One` ListForm; `tet (86` → `Nothing` |
| 7 | `an_empty_value_matches_nothing` | `""` and `"   "` → `Nothing`, for test 4's items |

`name_match_fixtures.rs`, reading files with `open_record` (cdss in file order:
pBR322 `tet` 0, ROP 1, `bla` 2; synthetic: cdss index = feature index − 1):

| # | Test | Asserts |
| --- | --- | --- |
| 8 | `every_cds_carries_its_ids_and_their_sources` | pBR322 `tet`: `[(DisplayName, "tet"), (Gene, "tet"), (ProteinId, "AAB59735.1")]`; ROP: `[(DisplayName, "ROP protein"), (ProteinId, "AAB59736.1")]`; `bla`: `[(DisplayName, "bla"), (Gene, "bla"), (ProteinId, "AAB59737.1")]`. Synthetic cdss 1 (`m1`): `[(DisplayName, "ignored-gene"), (Gene, "ignored-gene"), (LocusTag, "m1")]`; cdss 2: `[(DisplayName, "pj")]`; cdss 12 (no id): `[(DisplayName, "CDS")]` |
| 9 | `a_span_is_read_in_d036_order` | pBR322 `tet` `(86, 1276)`, `bla` `(3293, 4153)`; synthetic `pj` (2) `(55, 12)`, `mj2` (4) `(55, 3)`, `fuzzy` (16) `(1, 12)`; `ord` (17) `None` |
| 10 | `matching_on_pbr322` | `TET` → `One` 0 DN; `aab 59737.1` → `One` 2 ProteinId; `rop` → `One` 1 DN; `tet (86..1276)` → `One` 0 ListForm; `bla(3293..4153)` → `One` 2 ListForm; `AAB5973` → `Several` 0, 1, 2; `lactamase` → `Nothing` (`/product` is not an id unless it is the display name) |
| 11 | `matching_on_synthetic` | `dup` → `Several` 14, 15; `mj` → `Several` 3, 4; `m1` → `One` 1 LocusTag; `ignored` → `One` 1 DN; `pj (55..12)` → `One` 2 ListForm |

Expected values were derived by the planner from the GenBank text.

## 6. Checks

`scripts/check`, plus `python3 -I tests/fixtures/verify.py` (still
`all 25 checks passed`).

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | No exact tier: prefixes only | 1, and 10 or 11 |
| G2 | Whitespace kept when folding | 2, 10 |
| G3 | A prefix may start anywhere (substring) | 3 |
| G4 | Matches counted per name, not per item | 5, and 10 (`TET`) |
| G5 | List forms prefix-matched like other names | 6 |
| G6 | `cds_span` from the lowest and highest base | 9 |
| G7 | `names` without the display name | 8 |

## 8. Do not

- No new dependency, no `DomainError` variant, no message text: the cli
  words the answers (D056).
- Do not change `display_name`'s rule, `import_cds`, or any existing test.

## 9. Report

Every guard experiment with its failure line.
