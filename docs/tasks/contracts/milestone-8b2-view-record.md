---
type: Task Contract
title: "Contract 8b2 — core: ViewRecord and open_record (§4.1)"
description: "crates/core/src/view_record.rs: open_record(name, text), the one core entry point the view and luria find use to read a file."
status: deprecated
tags: [milestone-8]
---
# Contract 8b2 — core: `ViewRecord` and `open_record` (§4.1)

Planner: Claude. Status: **Implemented 2026-10-26** by worker `w-8b2` in
four rounds, reviewed and accepted. The two decisions in §4 are D053,
approved by the owner 2026-10-26. Findings from review:

* Round 2: the ceiling was checked after steps 5 and 6, so a record over the
  limit had every feature parsed and every CDS imported before it was
  refused. It now runs as soon as each reader returns. No test can see the
  order, and a comment at the check says so. The usable-CDS lookup scanned
  every imported CDS once per CDS; it is now a `BTreeMap` by feature index.
  The `unknown_record_format` message now says "first non-blank line".
* Round 3, a contract omission: no test looked inside `Usable(cds)`, so a
  wrong `Cds` passed. Test 2 now checks tet's and bla's id, strand and
  intervals, and test 3 checks the id of each usable synthetic CDS.
* Round 4: `sniff` with `contains("LOCUS")` passed every test. Test 6 now
  has `>LOCUS` (FASTA) and ` LOCUS` (refused).
* Planner re-ran seven mutations on a scratch copy: an empty name for a bare
  qualifier, no key fallback, GenBank marked `Assumed`, the id dropped,
  feature index + 1, the `LOCUS` sniff and the wrong `Cds`. All are caught.
* §4 step 6's reason for one call was only half right. `import_cds` is
  itself quadratic in the names it is given: each name scans every feature,
  and `used.contains` runs per feature (`genbank_cds.rs:575-598`). So the
  second call costs CDSs × features, about 4 × 10⁷ `is_named_cds` calls on
  an E. coli-sized record. A linear way to import every usable CDS needs
  its own contract, since `genbank_cds.rs` is outside this one, before the
  view opens genome-sized records (8k).
* Test 7 reads 10 000 000 and 10 000 001 bases in a debug build and takes
  about 20 s, which is the whole core lib suite's wall time.
* `view_cdss` has two `unreachable!` invariants: a CDS with no refusal has
  an id and is imported. They are checked against `genbank_cds.rs` and
  accepted; core already has invariant `unwrap`s.
* The 16 socket tests fail only inside the worker sandbox; they pass here. Tier: full (D051). Plan row: **8b** in
[docs/tasks/contracts/milestone-8-view.md](/tasks/contracts/milestone-8-view.md) §7, second half.

## 1. What this contract builds

`crates/core/src/view_record.rs`: `open_record(name, text)`, the one core entry
point the view and `luria find` use to read a file. It decides the format from
the content, reads the record, refuses one over 10 000 000 bases, and returns
a `ViewRecord`: the sequence, where its topology came from, every feature
(drawn, or kept with the refusal that keeps it from being drawn), and every
CDS as usable or unusable with the refusal `import_cds` gives.

Rules that apply, already settled:
- Plan §4.1 (`docs/tasks/contracts/milestone-8-view.md:147-191`) — what the record holds.
- D049 (`docs/decisions/:1729-1743`) — format by content: the first line
  `LOCUS` is GenBank, `>` is FASTA, anything else refused; a FASTA record is
  read as linear and marked as an assumption.
- D048 (`docs/decisions/:1694-1700`) — over 10 000 000 bases is refused and
  the message names the limit; tested at the boundary and one past it.
- D041 (`:1310-1316`) and D042 — CDS usability is `import_cds`'s answer,
  unchanged; a CDS's id is the first of `/locus_tag`, `/gene`, `/label`.
- D052 (`:1918-1924`) — an unusable CDS is listed with its refusal code.

## 2. What exists and must be reused

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `read_genbank(name, text)` | `genbank.rs:764` | GenBank records; its refusals are returned unchanged |
| `read_fasta(name, text, topology)` | `fasta.rs:106` | FASTA records, always with `SequenceTopology::Linear` |
| blank line = empty or only spaces, tabs, `\r` | `genbank.rs:18` | the sniff skips such lines, as both readers do |
| `GenBankRecord::features()`, `GenBankFeature::{key, location, qualifiers, line}` | `genbank.rs:100-160` | one `ViewFeature` per entry, in file order |
| `parse_display_location` | `display_location.rs:67` (8b1) | every feature's shape; never call `parse_genbank_location` here |
| `import_cds(record, names)` | `genbank_cds.rs:551` | called **twice**, see §4 step 6; never re-implement a CDS rule |
| `GenBankImport::{cds, unused}`, `UnusedFeature::{index, key, id, refusal}`, `ImportedCds::{index, cds}` | `genbank_cds.rs:121-204` | statuses and ids |
| `refusal_text` | `crates/core/tests/genbank_fixtures.rs:257` | copy it into the fixture test to compare with `synthetic-unused.tsv` |

## 3. Files the worker may touch

1. `crates/core/src/view_record.rs` — new, with its `mod tests`.
2. `crates/core/src/lib.rs` — `pub mod view_record;` after `pub mod translation;`.
3. `crates/core/src/error.rs` — two variants, one payload struct each, their
   category, code and message arms, and two rows in
   `every_variant_has_its_pinned_code` (its `cases.len()` goes up by 2).
4. The other exhaustive matches on `DomainError`, one arm per new variant and
   nothing else (8e found them; its §3 had missed them):
   `crates/cli/src/result.rs` `rule_code` (`None`) and `error_details` (the
   payload's fields, as the `InvalidSequenceCharacters` arm does), and
   `variant_name` in `crates/core/tests/core_foundation.rs`,
   `mutation_fixtures.rs` and `mutation_invariants.rs`.
5. `crates/core/tests/view_record_fixtures.rs` — new.

## 4. The public surface, exactly

```rust
pub const MAX_RECORD_BASES: u64 = 10_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordFormat { GenBank, Fasta }

/// D049: the LOCUS line stated it, or the view assumed linear for a FASTA.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopologySource { Stated, Assumed }

#[derive(Clone, Debug, PartialEq)]
pub enum FeatureShape { Drawn(DisplayLocation), NotDrawn(DomainError) }

#[derive(Clone, Debug, PartialEq)]
pub struct ViewFeature {
    /// Zero-based position in `GenBankRecord::features()`.
    pub index: usize,
    pub key: String,
    /// The first of `/label`, `/gene`, `/locus_tag`, `/product` whose value is
    /// present and not empty, in that order of preference; else the key.
    pub name: String,
    pub line: u64,
    pub shape: FeatureShape,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CdsStatus { Usable(Cds), Unusable(DomainError) }

#[derive(Clone, Debug, PartialEq)]
pub struct ViewCds {
    /// The `index` of its `ViewFeature`.
    pub feature: usize,
    /// Its D041 id (`UnusedFeature::id`), `None` when it has none.
    pub id: Option<String>,
    pub status: CdsStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ViewRecord {
    pub format: RecordFormat,
    pub sequence: Sequence,
    pub topology: TopologySource,
    pub features: Vec<ViewFeature>,
    /// Every CDS feature, in file order.
    pub cdss: Vec<ViewCds>,
}

pub fn open_record(name: Identifier, text: String) -> Result<ViewRecord, DomainError>;
```

New in `error.rs`: `UnknownRecordFormat(UnknownRecordFormatProblem { sequence:
String })`, code `unknown_record_format`, category `MalformedInput`; and
`RecordTooLong(RecordTooLongProblem { sequence: String, length: u64, limit: u64
})`, code `record_too_long`, category `Unsupported`, whose message names
`limit`. `sequence` is the identifier's value.

Steps, in order:

1. **Sniff.** Take the first line that is not blank. Starts with `LOCUS` →
   GenBank; starts with `>` → FASTA; otherwise, or no such line →
   `UnknownRecordFormat`. The whole text, unchanged, goes to the reader.
2. Read with `read_genbank`, or `read_fasta(.., SequenceTopology::Linear)`.
   A reader's refusal is returned as it is.
3. `sequence.length() > MAX_RECORD_BASES` → `RecordTooLong`. Exactly
   10 000 000 is accepted.
4. FASTA: `features` and `cdss` empty, `topology` `Assumed`. Done.
5. GenBank, `topology` `Stated`: one `ViewFeature` per feature, its shape from
   `parse_display_location(&feature.location(), length, topology)`.
6. CDSs: `import_cds(&record, &[])` lists every feature in `unused()`. A CDS
   with `refusal` `Some` is `Unusable` with it. Collect the ids of the rest
   (each has one: no id is the `no_id` refusal, and a bad id is
   `invalid_identifier_characters`, `synthetic-unused.tsv` rows 13 and 24),
   make them `Identifier`s, and call `import_cds` **once** with all of them;
   each returned `ImportedCds` is `Usable(cds)` at its `index`. Any error
   from either call or from `Identifier::create` is returned with `?`. One
   call per CDS is a defect: each call evaluates every CDS, so it is
   quadratic on a genome-sized record.

**Decision (D053) — codes.** The refusal shown for an
undrawn feature or an unusable CDS is the existing `DomainError` and its
`code()` (`fuzzy_end`, `alternative_start`, `no_id`, …), which 5a1 made a
stable contract. §4.1's `partial_location`, `non_atg_start` and `fuzzy_start`
read as illustrations, and inventing them would give one rule two codes.
pBR322's ROP is therefore unusable as `no_id`, the first rule that applies
(`genbank_fixtures.rs:645-668`), though it also has a `GTG` start.

**Decision (D053) — names.** The display name and the D041
id differ on purpose. synthetic's `m1` CDS is named `ignored-gene` in the
feature list and has id `m1`, and ROP is named `ROP protein` with no id.

## 5. Tests the worker writes

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `view_record_fixtures::pbr322_opens_as_a_stated_circular_genbank_record` | `J01749.1.gb`: `GenBank`, `Stated`, circular, 4361 bases; `features` has one entry per `read_genbank(..).features()` entry (50), each with the same `index`, `key` and `line`, in order |
| 2 | `view_record_fixtures::pbr322_has_tet_and_bla_usable_and_rop_unusable` | 3 CDSs, in file order: name `tet`, id `tet`, `Usable`; name `ROP protein`, id `None`, `Unusable` with rule `NoId`; name `bla`, id `bla`, `Usable`; the `source` features are named `source` |
| 3 | `view_record_fixtures::every_synthetic_cds_status_matches_the_not_used_table` | for each `CDS` row of `tests/fixtures/genbank/synthetic-unused.tsv` (24): the `ViewCds` whose `feature` is the row's `index` has `id` equal to the column (`-` = `None`), and is `Usable` when `refusal` is `-`, else `Unusable` with `refusal_text` equal to the column; no other `ViewCds` exists |
| 4 | `view_record_fixtures::synthetic_features_are_drawn_or_kept_with_their_refusal` | 27 features; 17 (`<1..12`) is `Drawn` with `open_before == Some(0)`; 18, 19, 20, 21 are `NotDrawn` with rules `Order`, `UnsupportedJoin`, `SinglePosition`, `RemoteReference`; names: 2 `ignored-gene`, 3 `pj`, 17 `fuzzy`, 25 `filler`, 26 `p1` |
| 5 | `view_record_fixtures::a_fasta_record_is_assumed_linear_with_nothing_else` | `tests/fixtures/view/frames-61.fa`: `Fasta`, `Assumed`, linear, 61 bases, no features, no CDSs |
| 6 | `the_format_is_decided_by_content` | `">x\nACGT\n"` and `"\n \t\n>x\nACGT\n"` → `Fasta`; `"ACGT\n"`, `""`, `"\n\n"`, `";c\n>x\nA\n"`, `"ORIGIN\n"` → `UnknownRecordFormat`; the text of `synthetic-linear.gb` with `"\n\n"` before it → `GenBank` |
| 7 | `the_ceiling_is_ten_million_bases` | `">big\n"` + 10 000 000 × `A` → `Ok`, length 10 000 000; 10 000 001 → `RecordTooLong { length: 10_000_001, limit: 10_000_000, .. }`, and its message contains `10000000` or `10 000 000` |
| 8 | `the_display_name_prefers_label_gene_locus_tag_product` | the record below → names `lab`, `tag`, `misc_feature`, `g` |

Test 8's record (the worker writes it as a literal; if `read_genbank` refuses
it, stop and report the refusal rather than changing the rule):

```text
LOCUS       names                     12 bp    DNA     linear   SYN 26-OCT-2026
FEATURES             Location/Qualifiers
     misc_feature    1..3
                     /gene="g-only"
                     /label=lab
     misc_feature    4..6
                     /product="prod"
                     /locus_tag="tag"
     misc_feature    7..9
                     /note="nothing to name it by"
     misc_feature    10..12
                     /label
                     /gene="g"
ORIGIN
        1 atggcttggt aa
//
```

Expected values come from the fixture files and the planner's reading of them
(synthetic's qualifiers at `synthetic.gb:16-20, 59, 75-78`; pBR322's at
`J01749.1.gb:232-241, 260-330`), not from running luria.

## 6. Checks

`scripts/check`, plus `cargo test -p luria-core --test view_record_fixtures`
and `python3 -I tests/fixtures/verify.py` (still `all 22 checks passed`).

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | Sniff the first line without skipping blank lines | 6 |
| G2 | `>=` instead of `>` at the ceiling | 7 |
| G3 | No ceiling | 7 |
| G4 | Prefer `/gene` over `/label` | 8 |
| G5 | Use the D041 id as the name | 2, 4 |
| G6 | Leave `NotDrawn` features out of `features` | 1, 4 |
| G7 | Shape from `parse_genbank_location` instead of 8b1's parser | 4 |
| G8 | Mark every CDS `Unusable` | 2, 3 |
| G9 | Read FASTA as circular | 5 |

## 8. Do not

- Do not touch `genbank*.rs`, `fasta.rs`, `display_location.rs` or any CDS
  rule; no byte-size limit here (that is the cli's, 8h).
- No flag or environment variable for the ceiling (plan §8, `:684-685`).
- No feature filtering by key: `source`, `gene` and every other key are kept.

## 9. Report

Every guard experiment with its failure line, and the wall time of test 7.
