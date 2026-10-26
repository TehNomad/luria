# Contract 8b1 — core: the display-only location parser (§4.1, M8-1)

Planner: Claude. Status: ready, delegated 2026-10-26. The two decisions in
§4 are D053, approved by the owner 2026-10-26. Tier: full (D051). Plan row:
**8b** in `docs/tasks/milestone-8-view.md` §7, first half; 8b2
(`ViewRecord`, `open_record`) uses this.

## 1. What this contract builds

`crates/core/src/display_location.rs`: `parse_display_location`, which reads a
GenBank feature location for **drawing only**. A location with a `<` or `>` on
an outer end (a partial feature) is read by taking the stated base as the bound
and recording which end is open. Every other location gets exactly the answer
`parse_genbank_location` gives. The science-tier parser is untouched: CDS
import keeps refusing partial locations, so a partial CDS is never selectable
for numbering.

Rules that apply, already settled:
- D049 (`docs/decisions.md:1719-1727`) — partial features are drawn, by their
  own parser; no flag on `parse_genbank_location`; its tests stay as they are.
- D041, D042 — CDS import is unchanged by this contract.
- D006 — positions in the result are zero-based; the text is 1-based.
- D036 — a join is only an origin join, `[a - 1, n)` then `[0, b)` for either
  strand (`genbank_location.rs:505-535`); this module never reorders it.

## 2. What exists and must be reused

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `parse_genbank_location(text, sequence_length, topology)` | `genbank_location.rs:417` | does all the reading after the markers are removed; never re-implement its grammar |
| `GenBankLocation` (`intervals()`, `strand()`) | `genbank_location.rs:69` | the drawn shape, returned inside `DisplayLocation` as is |
| step 1's whitespace rule (drop `' '`, `\t`, `\r`, `\n`) | `genbank_location.rs:426-430` | apply the same rule before looking for markers |
| `DomainError::UnusableGenBankLocation(UnusableGenBankLocationProblem { location, rule, sequence_length, cds, line })` | `error.rs:755` | every refusal; `location` carries the text **as given**, markers included; `cds` and `line` are `None` here |
| `GenBankLocationRule::FuzzyEnd` | `error.rs:609` (code `fuzzy_end`) | the refusal for a marker this parser does not draw |

## 3. Files the worker may touch

1. `crates/core/src/display_location.rs` — new, with its `mod tests`.
2. `crates/core/src/lib.rs` — one line, `pub mod display_location;`, between
   `pub mod codon_usage;` and `pub mod error;`.
3. `crates/core/tests/display_location_fixtures.rs` — new, one test (§5 #10).

Everything else is out of scope.

## 4. The public surface, exactly

```rust
/// A location as the view draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayLocation {
    /// The shape, exactly as `parse_genbank_location` reads the unmarked text.
    pub location: GenBankLocation,
    /// `<`: the zero-based base (D006) the feature may begin before.
    pub open_before: Option<u64>,
    /// `>`: the zero-based base the feature may continue past.
    pub open_after: Option<u64>,
}

pub fn parse_display_location(
    text: &str,
    sequence_length: u64,
    topology: SequenceTopology,
) -> Result<DisplayLocation, DomainError>;
```

Behaviour, in order; the first step that answers decides:

1. Compact the text with the shared parser's whitespace rule. If it has no
   `<` and no `>`, return `parse_genbank_location(text, ..)`'s answer
   unchanged: `Ok` with both `open_*` fields `None`, `Err` as it is.
2. If the compacted text has more than one `<`, more than one `>`, or a marker
   not followed at once by a digit, return `parse_genbank_location(text, ..)`'s
   answer for the text as given (today's refusal, unchanged).
3. Remove the markers and call `parse_genbank_location` on the result. On
   `Err(UnusableGenBankLocation(problem))`, set `problem.location` to the text
   as given and return it. Any other error is returned as it is.
4. **Outer ends only.** Read the digit run after each marker (checked parse).
   A `<` run's value must equal the first interval's start + 1; a `>` run's
   value must equal the last interval's end (both 1-based, both in D036 order,
   which is the same for either strand). If either check fails, or a run does
   not parse, refuse with `FuzzyEnd` for the text as given. Otherwise
   `open_before = Some(value - 1)` and `open_after = Some(value - 1)` for their
   markers, with `checked_sub` and the same refusal on `None`. These are
   reference positions whatever the strand: `complement(<1..12)` gives
   `open_before == Some(0)`.

**Decision (D053) — outer ends only.** A marker on an inner
end (`1..<12`, or `join(55..>60,1..5)`, whose `>` sits on the origin boundary)
is not drawn and keeps its `fuzzy_end` refusal. It cannot be drawn honestly as
one bar with an open end, and no fixture holds one. Checking against the parsed
intervals instead of the text's first and last numbers matters for the spelling
`join(complement(1..b),complement(a..n))`, whose first and last numbers are the
origin boundary. Rejected alternative: accept every marker and record them all,
which needs a drawing rule for an open inner boundary that §4.1 does not give.

**Decision (D053) — the truer refusal.** Once the markers
are removed, another rule may apply first: `order(<1..10,20..30)` is refused as
`order` here and `join(1..>5,8..10)` as `unsupported_join`, while
`parse_genbank_location` says `fuzzy_end` for both (`genbank_location.rs:744-746`).
The feature list should give the reason that stands without the marker. The
shared parser's own answers are unchanged.

## 5. Tests the worker writes

Units in `display_location.rs`. "60 lin" / "60 circ" give the length and
topology passed in.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `a_location_without_markers_is_the_shared_parsers_answer` | `1..12`, `complement(1..12)`, `order(1..5,8..10)`, `J00001.1:1..5` (60 lin) and `join(55..60,1..5)` (60 circ): `Ok` equals `parse_genbank_location`'s location with both `open_*` `None`; `Err` equals its error exactly |
| 2 | `a_left_partial_takes_the_stated_base` | `<1..12`, 60 lin → intervals `[(0, 12)]`, `Plus`, `open_before == Some(0)`, `open_after == None` |
| 3 | `a_right_partial_takes_the_stated_base` | `1..>12`, 60 lin → `[(0, 12)]`, `None`, `Some(11)` |
| 4 | `both_ends_open` | `<3..>12`, 60 lin → `[(2, 12)]`, `Some(2)`, `Some(11)` |
| 5 | `a_complement_partial_keeps_reference_coordinates` | `complement(<1..12)` → `Minus`, `[(0, 12)]`, `Some(0)`, `None`; `complement(1..>12)` → `None`, `Some(11)` |
| 6 | `a_partial_origin_join_in_each_spelling` | 60 circ, each → `[(54, 60), (0, 5)]`, `open_before == Some(54)`, `open_after == Some(4)`: `join(<55..60,1..>5)` (`Plus`); `complement(join(<55..60,1..>5))` (`Minus`); `join(complement(1..>5),complement(<55..60))` (`Minus`) |
| 7 | `a_marker_on_an_inner_end_keeps_fuzzy_end` | rule `FuzzyEnd`, `location` equal to the text given, for: `1..<12`, `>1..12`, `<<1..12`, `<1..>>12`, `<x1..12` (60 lin); `join(55..>60,1..5)`, `join(55..60,<1..5)`, `join(complement(<1..5),complement(55..>60))` (60 circ) |
| 8 | `the_shared_parsers_other_refusals_still_apply` | `location` equal to the text given in each: `order(<1..10,20..30)` → `Order`; `join(1..>5,8..10)` → `UnsupportedJoin`; `<1..61` → `PastSequenceEnd`; `J00001.1:<1..5` → `RemoteReference` (60 lin); `join(<55..60,1..>5)` on 60 lin → `OriginJoinOnLinear` |
| 9 | `whitespace_inside_a_partial_location_is_ignored` | `complement( <1..12 )` equals test 5's first result |
| 10 | `display_location_fixtures::the_synthetic_fuzzy_cds_is_drawn_and_still_refused_for_import` | read `tests/fixtures/genbank/synthetic.gb` with `read_genbank`; the feature whose `/gene` is `fuzzy` has location `<1..12` (file line 59); on the record's length (60) and topology (circular) `parse_display_location` gives `[(0, 12)]`, `Plus`, `Some(0)`, `None`; `parse_genbank_location` on the same text still gives `FuzzyEnd` |

Expected values were derived by the planner by hand from the GenBank text
(1-based inclusive `a..b` is zero-based `[a - 1, b)`), not by running code.

## 6. Checks

`scripts/check`, plus `cargo test -p luria-core --test display_location_fixtures`
and `python3 -I tests/fixtures/verify.py` (still `all 22 checks passed`).

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| E1 | Skip step 3: always return the shared parser's answer | 2–6, 9, 10 |
| E2 | `open_after = Some(value)` instead of `value - 1` | 3, 4, 5, 6 |
| E3 | Skip step 4's comparison (accept any marker that parses) | 7, at the origin-join rows at least |
| E4 | Do not restore the given text into `problem.location` | 8 |
| E5 | Compare `<` with the **last** interval's start and `>` with the **first** interval's end | 6 |
| E6 | Drop step 2's count check | 7 (`<<1..12`, `<1..>>12`) — if it still passes, report how your step 4 caught it |

## 8. Do not

- Do not touch `genbank_location.rs`, `genbank_cds.rs` or `error.rs`: no flag,
  no new rule, no new variant. Every refusal here is an existing rule.
- Do not build `Feature`s or `NucleotideInterval`s; the plain pairs in
  `GenBankLocation` are the shape (`feature.rs:29` refuses some shapes).
- Do not consult the strand anywhere in this module.

## 9. Report

Every guard experiment with its failure line, and any GenBank location form
you met that §4's steps do not answer.
