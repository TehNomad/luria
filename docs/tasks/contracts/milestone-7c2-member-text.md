# Milestone 7c2: the text of the member screen

Status: **Implemented 2026-10-21** by the local worker; accepted on the
first review. Notes: a part line is `<start>-<end>  <role> <annealing>
<sequence>`, one space before the annealing word (the contract's wrapped
text, pinned by test 3); a whole-number float prints without `.0` (`66`
where the JSON holds `66.0`), as 7c1's `{}` rule gives; a tail's `reason`
is core's own text and may hold `′`, which is a document field, not a
literal. Planner:
Claude. Parent: `docs/tasks/contracts/milestone-7-tui.md` §3 rules 1 and 3, §5.2
screen 3, §6 (7c). Builds on 7c1 (`crates/cli/src/review/text.rs`).

## 1. What and why

The member screen is where a primer pair is reviewed: the edit, its
warnings, and each primer with its parts, Tm and warnings, or the reason
no pair was designed. This contract adds its lines to `text.rs`, built the
same way as 7c1's: plain strings from document fields, tested as strings.

## 2. Requirements this contract applies

The same as 7c1 §2: every value is a document field; intervals as
`[start, end)`; floats with `{}`; ASCII only; nothing computed, rounded or
translated; warnings and failures always listed.

## 3. Changes

`crates/cli/src/review/text.rs` gains:

```rust
pub fn member(document: &ResultDocument, index: usize) -> Vec<String>;
```

`index` is in range (the app guarantees it); out of range, return the
single line `no such member`. Reuse 7c1's helpers (`interval`,
`intervals`, `mutation_summary`, `details_lines`); do not write second
copies. 7c1 did not write the codes of `EditWarningView` and
`PrimerWarningView`: add them here, as `match` mappings to the serde tag
(`frameshift`, `insertion_frameshift`, `replacement_frameshift`,
`premature_stop`; `gc_content_outside_band`, `single_base_run`,
`dinucleotide_repeat`, `three_prime_gc_run`, `length_outside_guideline`,
`above_ordering_limit`), each beside a function returning the warning's
`message`.

Lines, in order, for `let m = &document.results[index]`:

1. `member: <name>`.
2. `mutation: <mutation_summary>`.
3. The mutation's own fields, each `  <label>: <value>`:
   - every kind: `state`, then `coordinates` (the numbers joined with
     `, `), then `edited: <id>, <length> bp, sha256 <sha256>`;
   - `Residue` also: `cds`, `residue`, `codon policy: <codon_policy>`, and
     `codon source: <codon_policy_source>` only when it is `Some`;
   - `Insertion` also: `before: <before>`, `after: <after>`.
4. If `m.warnings` is not empty: `warnings:`, then `  <code>: <message>`
   per warning.
5. If `m.design_failure` is `Some(e)`: `design failed: <e.code>`,
   `  <e.message>`, and `details_lines(&e.details, "  ")` when not empty.
6. If `m.design` is `Some(d)`:
   - `design: <policy>, <style>, <algorithm.name> <algorithm.version>`;
   - `  placement: <placement>`, `  min annealing overlap:
     <min_annealing_overlap>`, `  edit span: <interval>`;
   - `  annealing temperature: <annealing_temperature> C`,
     `  tm balance met: <yes or no>`;
   - `  pair: <relationship>, separation <separation>, deleted width
     <deleted_width>`, followed by `, boundary <b>` when `boundary` is
     `Some`, and `, abuts` when `abuts` is true;
   - for `forward`, then `reverse`, the primer lines below;
   - if `unresolved` is not empty: `unresolved:`, then `  <input>:
     <effect>` per entry.
7. A blank line, then `c context` when `design` is `Some`, and in every
   case `esc back`.

Primer lines, for a `PrimerView` `p`:
- `<role> 5'-<sequence>-3'  <length> nt  span <interval>`, followed by
  `  edited span <interval>` when `edited_span` is `Some`;
- if `tail` is `Some`: `  tail: <sequence> (<reason>)`;
- `  parts:`, then per part: `    <primer_start>-<primer_end>  <role>
  <annealing or not annealing>  <sequence>`, followed by `  <interval>`
  when `span` is `Some`;
- `  annealing: <sequence>, <length> nt, overlap <overlap>, spans
  <intervals of spans>`;
- `  tm: <celsius> C, <basis>`, then `  tm substituted because:
  <substituted_because>` when `Some`;
- `  whole primer tm: <celsius> C, <basis>` when `whole_primer_tm` is
  `Some`;
- if `warnings` is not empty: `  warnings:`, then `    <code>: <message>`.

The enthalpy and entropy are not shown on this screen; they stay in the
JSON export.

## 4. Tests

Integration tests in `crates/cli/tests/review_text.rs` (added by 7c1;
add to it), with the same documents as 7c1 §4 (`set`, `design failure`),
and the same rule: an expected value is either the document's own field
or a fact from an independently verified fixture table.

Fixture facts you may use as literals: those of 7c1 §4, and
`tests/fixtures/genbank/synthetic-linear-residues.tsv`'s residue 42 = `A`,
codon `GCA`, interval `[165, 168)`.

1. `the_member_screen_names_the_edit`: set, member 0; the lines include
   `member: A42G`, `  cds: synA`, `  residue: 42`, and `  coordinates:
   165, 166, 167`.
2. `each_primer_is_written_five_to_three`: set, member 0; one line equals
   `forward 5'-` + the document's forward `sequence` + `-3'  ` + its
   length + ` nt  span ` + its span as `[s, e)`, followed by the edited
   span when the document has one; likewise one line for `reverse`.
3. `each_part_is_listed_in_primer_order`: set, member 0; the forward
   primer's part lines appear in the document's order, one per part, each
   holding the part's own `sequence`.
4. `the_tm_and_its_basis_are_shown`: set, member 0; a line equals `  tm: `
   + the document's forward `tm.celsius` written with `{}` + ` C, ` + its
   `basis`; when `substituted_because` is `Some`, the next line shows it.
5. `a_design_failure_replaces_the_pair`: design failure; the lines include
   `design failed: ` + the document's code and `  ` + its message, no line
   starts with `forward 5'-`, and there is no `c context` line.
6. `back_is_always_offered`: both documents end with `esc back`.
7. Unit, in `text.rs`: `an_insertion_shows_before_and_after`,
   `a_tail_is_shown_with_its_reason`, `a_primer_warning_is_listed`
   (built by hand, as 7b's helpers build documents);
   `out_of_range_is_no_such_member`.

### 4.1 Rules that must be proven

1. The primer line written 3' to 5' (the sequence reversed): test 2
   fails.
2. `substituted_because` never shown: test 4 fails when the document's
   forward Tm has one (it does for the set's member 0, a tandem mismatch;
   check the document, do not assume), or else the unit test you add for
   it fails.
3. The design-failure lines left out: test 5 fails.

## 5. Do not

- Change only `crates/cli/src/review/text.rs` and
  `crates/cli/tests/review_text.rs`.
- Do not change other files, any fixture, `docs/`, any `Cargo.toml` or
  `vendor/`.
- Do not draw anything, compute a value, round a float, reverse or
  complement a sequence.
- Do not add dependencies. Do not stage or commit.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the proofs of §4.1, and any point where this contract
does not match the code; stop and report rather than guess.
