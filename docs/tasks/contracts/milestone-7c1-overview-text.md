# Milestone 7c1: the text of the overview, problem and provenance screens

Status: **Implemented 2026-10-21** by the local worker; accepted on the
first review. Notes: the `EditWarningView` and `PrimerWarningView` code
helpers are not written, since 7c1 does not use them (an unused one fails
clippy); 7c2 adds them. A member with both a design and a failure (core
forbids it) would print `designed`. Planner:
Claude. Parent: `docs/tasks/contracts/milestone-7-tui.md` §3 rules 1 and 3, §5.2
screens 2, 5 and 6, §6 (7c). The member screen's text is 7c2; drawing
these lines, with scrolling, is 7c3.

## 1. What and why

Each result screen is a list of plain text lines built from the
`ResultDocument`. Building them in a pure module, apart from drawing,
keeps them testable as strings, and keeps every value traceable to one
document field. This contract adds `crates/cli/src/review/text.rs` with
the lines of three screens: the overview, the problem screen and the
provenance screen.

## 2. Requirements this contract applies

- **Plan §3 rule 1:** every value on a line is a document field, written
  with `{}` or as specified below. No value is computed. The only
  formatting is: intervals as `[start, end)`; a float with `{}` (Rust's
  shortest round-trip form, the same digits the JSON holds); a digest
  shortened only where this contract says so.
- **Plan §3 rule 3:** an error, a clarification, a design failure and a
  warning are always listed, never summarised away.
- Lines hold ASCII only: `->` for an arrow, no `→` or `°`.

## 3. Changes

`crates/cli/src/review.rs` gains `pub mod text;`. New file
`crates/cli/src/review/text.rs`, pure, no I/O, no `ratatui` or
`crossterm`. Three public functions, each returning `Vec<String>`, one
string per line, no line holding `\n`.

Helpers you will need (private, or `pub(crate)` if 7c2 will reuse them;
your choice):
- `interval((s, e))` → `[s, e)`;
- `intervals(list)` → the intervals joined with `, `;
- the code of an `OperationWarning`, `EditWarningView`,
  `PrimerWarningView` or `ObstacleView`: its serde tag, the variant name
  in snake case (`wild_type_skipped`, `many_designs`, `frameshift`,
  `insertion_frameshift`, `replacement_frameshift`, `premature_stop`, and
  so on). Write the mapping as a `match`; do not serialize to find it.
- `mutation_summary(&MutationView) -> String`:
  - `Residue`: `<cds> <original_residue><residue><written_residue>
    <original_codon>-><written_codon>`, e.g. `synA A42G GCA->GGC`;
  - `Substitution` and `Replacement`: `<interval> <original>-><written>`;
  - `Deletion`: `<interval> del <original>`;
  - `Insertion`: `ins <written> at <point>`.
- `details_lines(&Details, indent) -> Vec<String>`: one line per pair,
  `<indent><key>: <value>`, where `Text(s)` is `s`, `Count(n)` and
  `Signed(n)` are `n`, `Absent` is `none`, and `Obstacles(list)` is the
  word `obstacles` followed by one further line per obstacle,
  `<indent>  <code>: <message>`.

### 3.1 `pub fn overview(document: &ResultDocument, selected: usize) -> Vec<String>`

In this order:
1. `template: <id>, <topology>, <length> bp` from
   `provenance.template`, or `template: none` when it is `None`.
2. `target: <id>, <strand>, <intervals>` from `target`, or `target: none`.
3. `operation: <operation>` from `provenance.operation`, or
   `operation: none`.
4. If `warnings` is not empty: `warnings:`, then one line per warning,
   `  <code>: <message>`.
5. `members: <n>`, `n` being `results.len()`.
6. One line per member, in order: a marker, `> ` for the member at
   `selected` and `  ` for every other, then `<name>  <mutation_summary>
   <outcome>  warnings: <k>`, with two spaces between parts, where
   `<outcome>` is `designed` when `design` is `Some`, and `design failed:
   <code>` when `design_failure` is `Some`, `<code>` being its
   `ErrorView.code`, and `k` the member's `warnings.len()`. A member with
   neither (reachable: `run.rs` reports it as an internal error) has the
   outcome `no design` (answered to the worker, 2026-10-21).
7. If `error` is `Some` or `clarifications` is not empty: a blank line,
   then `e problem`.
8. A blank line, then `p provenance`.

Nothing more. The state and the experimental validation are not here:
7a's banner draws them on every screen.

### 3.2 `pub fn problem(document: &ResultDocument) -> Vec<String>`

1. If `error` is `Some`: `error: <code>`, `category: <category>`,
   `message: <message>`, `rule: <rule>` only when `rule` is `Some`,
   `path: <path>` only when `path` is `Some`, then `details:` and
   `details_lines(details, "  ")` only when `details` is not empty.
2. If `clarifications` is not empty: `clarifications:`, then one line per
   item, `  <text>`.
3. If `candidates` is not empty: `candidates:`, then one line per item,
   `  <text>`.
4. If none of the three: the single line `no problem`.

A blank line separates the three groups when more than one is present.

### 3.3 `pub fn provenance(document: &ResultDocument) -> Vec<String>`

1. `package: <name> <version>`.
2. `algorithms:`, then one line per algorithm, `  <role>: <name>
   <version>`.
3. `request sha256: <hex>` or `request sha256: none`.
4. `operation: <operation>` or `operation: none`.
5. If `template` is `Some`:
   - `template: <id>`, `  sha256: <sha256>` (in full), `  length: <length>
     bp`, `  topology: <topology>`;
   - for `SourceView::Fasta`: `  source: fasta <path>`, `  header:
     <header>`;
   - for `SourceView::Genbank`: `  source: genbank <path>`, `  locus:
     <locus_name>`, `  accession: <accession or none>`, `  version:
     <version or none>`;
   - if `genbank` is `Some`: `  imported cds:`, one line per entry,
     `    <id>  <strand>  <intervals>  from /<id_qualifier>  line <line>`;
     then `  unused features:`, one line per entry, `    <key>  line
     <line>`, followed on the same line by `  <id>` when `id` is `Some`,
     and by `  refused: <code>` when `refusal` is `Some`, e.g. `    CDS
     line 40  rop  refused: alternative_start` (clarified to the worker,
     2026-10-21).

   Otherwise `template: none`.
6. If `buffer` is `Some`: `buffer: <preset>` when `preset` is `Some`,
   else `buffer: stated`; then `  monovalent: <x> mM`, `  primer: <x> nM`,
   `  dmso: <x> %`, each float written with `{}`. Otherwise `buffer:
   none`.

## 4. Tests

Unit tests may build documents by hand, as 7b's helpers do, for the
branches no fixture reaches (an operation warning, a stated buffer, a
FASTA source, `rule` and `path`). Integration tests go in a new file,
`crates/cli/tests/review_text.rs`, and run requests in directories under
`env!("CARGO_TARGET_TMPDIR")`, as `tests/review_view.rs` does.

**What a test may assert.** The documents' primer values are not
independently verified for `synthetic-linear.gb`. A test therefore
asserts that a line shows **the document's own field** (build the
expected string from the field), or a fact recorded in an independently
verified fixture table. Never paste a value copied from running the code.

Fixture facts you may use as literals:
- `tests/fixtures/genbank/synthetic-linear-record.tsv`: LOCUS `synlin`,
  393 bp, `linear`, sha256
  `a8b4ddcfa617179143d428dc103aabb7b13cf27f7b323407d11db8a01105bbd7`.
- `tests/fixtures/genbank/synthetic-linear-cds.tsv`: `synA`, `[42, 345)`,
  `+`, id from `/gene`.
- `tests/fixtures/genbank/synthetic-linear-residues.tsv`: residue 42 is
  `A`, codon `GCA`; residue 87 is `W`, codon `TGG`.
- `tests/fixtures/genbank/pbr322-cds.tsv`: `tet` and `bla`.

Documents:
- **set**: `tests/fixtures/nl/request/variants-separate.json` beside a
  copy of `synthetic-linear.gb`; `state` `verified`, two members.
- **clarification**: the request of `tests/end_to_end.rs`'s
  `the_fail_closed_example_asks_and_edits_nothing`, copied with its
  `design()`, beside a copy of `tests/fixtures/genbank/raw/J01749.1.gb`.
- **design failure**: the GenBank request of
  `a_fasta_and_a_genbank_template_give_the_same_result` (pBR322, `tet`,
  `T99C`), same directory pattern; one member with a `design_failure`.
- **failed**: a request path that does not exist.

Tests (integration unless marked):
1. `the_overview_names_the_template_and_target`: set; the first three
   lines are `template: synlin, linear, 393 bp`, `target: synA, +, [42,
   345)`, `operation: variants`.
2. `each_member_has_one_row_and_one_marker`: set, `selected` 1; exactly
   two member rows; only the second starts with `> `; the first starts
   with `  A42G  synA A42G GCA->`, then the document's own written codon,
   and ends with `designed  warnings: 0`; the second starts with
   `> W87A  synA W87A TGG->`.
3. `a_design_failure_is_on_the_overview_row`: design failure; the member
   row contains `design failed: ` followed by the document's own
   `design_failure.code`.
4. `the_problem_line_appears_only_with_a_problem`: set has no `e problem`
   line; clarification and failed have it.
5. `the_problem_screen_lists_the_clarification_and_candidates`:
   clarification; the lines include `clarifications:`, `  ` + the
   document's clarification, `candidates:`, `  tet`, `  bla`, in that
   order.
6. `the_problem_screen_shows_the_error`: failed; `error: ` + the
   document's code, `category: malformed_input`, `message: ` + its
   message.
7. `the_provenance_screen_shows_the_record`: set; includes `template:
   synlin`, `  sha256: a8b4ddcfa617179143d428dc103aabb7b13cf27f7b323407d11db8a01105bbd7`, `  locus: synlin`,
   `  accession: SYN0002`, `  version: SYN0002.1`, a line `    synA  +
   [42, 345)  from /gene  line ` followed by the document's own line
   number, and one `  <role>: <name> <version>` line per entry of the
   document's `algorithms`.
8. Unit, in `text.rs`: `no_problem_says_so` (a document with no error,
   no clarifications, no candidates gives `["no problem"]`);
   `an_operation_warning_is_listed` (one `ManyDesigns` warning gives
   `warnings:` and `  many_designs: <message>`); `a_stated_buffer_is_named_so`
   (`preset` `None` → `buffer: stated`); `details_lines_cover_every_value`
   (one of each `DetailValue`, the obstacle as `<indent>  <code>:
   <message>`).

### 4.1 Rules that must be proven

Disable each once, run, confirm the named test fails, restore, report:
1. The marker always `  `: test 2 fails.
2. The `designed` / `design failed` branches swapped: tests 2 and 3 fail.
3. `e problem` always added: test 4 fails.
4. Candidates not listed: test 5 fails.

## 5. Do not

- Change only `crates/cli/src/review.rs` (the one `pub mod` line),
  `crates/cli/src/review/text.rs` and `crates/cli/tests/review_text.rs`.
- Do not change `app.rs`, `view.rs`, `terminal.rs`, `main.rs`, any other
  test, any fixture, `docs/`, any `Cargo.toml` or `vendor/`.
- Do not draw anything; 7c3 draws these lines.
- Do not compute a value, round a float, or translate a codon.
- Do not add dependencies. Do not stage or commit.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the four proofs of §4.1, and any point where this
contract does not match the code; stop and report rather than guess.
