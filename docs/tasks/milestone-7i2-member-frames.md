# Milestone 7i2: each member's mutant protein, for the review

Status: **Ready for the worker, 2026-10-21.** Planner: Claude. Parent:
`docs/tasks/milestone-7-tui.md` §5.3a (M7-18, M7-21). Builds on 7i1
(`luria_core::frame::mutant_residues`, committed). Next: 7i3 (the
member-screen text) uses what this contract returns.

## 1. What and why

The redesigned member screen shows the mutant protein. `luria review`
must take it from luria-core's `frame::mutant_residues`, run on the
edited construct core itself produced for that member, never translated
in the CLI (plan §3 rule 1, M7-21). This contract builds that list for
every member once, when the review opens, and carries it in `App` and
`Approved` beside the template.

## 2. Changes

### 2.1 `crates/cli/src/run.rs`

Add, beside `run_with_output`, a public function that returns the run's
`Outcome` **and** the edited constructs it made, without writing them:

```rust
/// The run `run` makes, with each member's edited construct as core made
/// it, in `document.results` order, keyed by member name (plan §5.3a).
pub fn run_with_constructs(request_path: &Path) -> (Outcome, Vec<(String, Sequence)>);
```

It must use the same internal path as `run_with_output(path, None)`
(`prepare`, then `single_edit` or `variant_set` exactly as now) so the
document is identical; `run_with_output` itself and every existing
caller stay unchanged in behaviour. Refactor only as far as needed
(for example a private helper returning `(Outcome, Artifacts)` that
both call). The names are the `Identifier` values `Artifacts` already
holds, as `value().to_string()`.

### 2.2 `crates/cli/src/review.rs`

```rust
/// One member's mutant protein: each residue core read, from
/// `frame::mutant_residues` over the whole edited construct.
pub type MemberFrame = Result<Vec<FrameResidue>, String>;

/// Every member's mutant protein, in `document.results` order.
pub fn member_frames(
    document: &ResultDocument,
    template: &Sequence,
    annotations: &[Cds],
    constructs: &[(String, Sequence)],
) -> Vec<MemberFrame>;
```

For member `i` (`document.results[i]`), in this order; the first that
applies gives `Err` with exactly the text in quotes:

1. `document.target` is `None`: `"no target CDS, so no protein"`.
2. No CDS in `annotations` whose `id().value()` equals the target's
   `id`: `"the target CDS is not among the template's annotations"`.
3. No construct named `member.name`: `"no edited construct"`.
4. The construct's `content_hash().hex_digest()` differs from
   `member.mutation`'s `edited.sha256` (every `MutationView` variant has
   `edited`): `"the edited construct differs from the one this result
   reports"` (D005, fail closed).
5. The template's digest differs from
   `document.provenance.template.sha256` (or that is `None`): `"the
   template differs from the one this result was computed on"`.
6. The edit span, from `member.mutation`:
   - `Substitution { interval, written, .. }`, `Replacement { interval,
     written, .. }`: `EditSpan { start: interval.0, end: interval.1,
     written_length: written.len() }`;
   - `Deletion { interval, .. }`: `written_length: 0`;
   - `Insertion { point, written, .. }`: `start = end = point`,
     `written_length: written.len()`;
   - `Residue { coordinates, written_codon, .. }`: `start` the least and
     `end` one past the greatest of `coordinates`, `written_length` that
     same width (a residue substitution keeps the length; `coordinates`
     are template positions, D040). Empty `coordinates`: `Err("no edit
     coordinates")`.
7. `frame::mutant_residues(construct, cds, span, 0, construct.length())`;
   its `DomainError` becomes `Err(error.message())`.

Nothing else is computed: no codon, no translation, no column.

### 2.3 `App` and `Approved`

- `crates/cli/src/review/app.rs`: `App` gains a field `frames:
  Vec<MemberFrame>`, empty from `App::new`, a builder
  `pub fn with_frames(self, frames: Vec<MemberFrame>) -> App` and an
  accessor `pub fn frame(&self, index: usize) -> Option<&MemberFrame>`.
  No key handling changes.
- `crates/cli/src/review.rs`: `Approved` gains `pub frames:
  Vec<MemberFrame>`; `approve` builds it with `run_with_constructs` (in
  place of `run_with_output(request, None)`) and `member_frames`, from
  the same `prepare(request)` it already calls (`prepared.template`,
  `prepared.annotations`). A prepare failure gives an empty list, as it
  gives `template: None` now.
- `crates/cli/src/main.rs` `emit_review`: use `run_with_constructs` in
  place of `run_with_output(&request, None)`, build the frames from the
  `prepare` it already calls, and pass them with `.with_frames(...)`;
  the interpret path passes `approved.frames`. Nothing drawn changes in
  this contract.

## 3. Tests

### 3.1 `crates/cli/tests/review_frames.rs` (new)

Build the requests the way `crates/cli/tests/review_context.rs` does
(copy its `workspace`, `design`, `sid1_template`, `request` and
`synthetic` helpers; do not change that file). Expected residues are
the rows of `tests/fixtures/review/frame.tsv`, read at test time and
parsed as 7i1's test does (`number:symbol:middle`), never pasted.

1. `native_01_gives_the_fixture_frame`: SID-1, replacement `[1209,
   1212)` GAT→AAC, bound 15 (the request `review_context.rs`'s
   `native_edit` makes). The residues of `member_frames(...)[0]` whose
   `middle` lies in the row's window equal the `native-01` row.
2. `del_11_gives_the_fixture_frame`: deletion `[1209, 1212)` GAT;
   row `del-11`.
3. `ins_14_gives_the_fixture_frame`: insertion at 1209, `GAT`; row
   `ins-14`.
4. `a_residue_request_uses_its_codon`: SID-1, a residue edit `{"cds":
   "sid-1", "number": 404, "from": "D", "to": "N", "codon": "AAC"}`
   (the `SID1_REQUEST` in `crates/cli/src/result.rs` tests). Its
   residues equal row `native-01`'s within that row's window (the same
   construct).
5. `a_frameshift_reads_on`: the synthetic template of
   `native-deletion.tsv` row `del-02` (it is the `ins-04` template; read
   it with `synthetic`-style code from `native-deletion.tsv`), CDS
   `[[0, 99]]` `+`, deletion `[45, 46)` `T`; equal to row
   `syn-del-frameshift` within its window.
6. `a_changed_construct_gives_no_frame`: call `member_frames` with
   native-01's document but a construct whose bases differ (one base
   changed): `Err` with rule 4's text.
7. `no_target_gives_no_frame`: native-01's document with `target` set
   to `None`: `Err` with rule 1's text.

### 3.2 Rules that must be proven

Disable each once, run, restore; report the failing test.

1. Rule 4 (digest of the construct): test 6 fails.
2. Rule 6's insertion `start = end = point`: test 3 fails.
3. Using `run_with_constructs`' construct rather than the template
   (pass the template as the construct): tests 1 to 3 fail.

## 4. Do not

- Change only `crates/cli/src/run.rs`, `crates/cli/src/review.rs`,
  `crates/cli/src/review/app.rs`, `crates/cli/src/main.rs` and the new
  `crates/cli/tests/review_frames.rs`.
- No change to any existing test, to luria-core, fixtures, docs,
  Cargo files or `vendor/`. No new dependency.
- Do not change what any screen draws (7i3 does that).
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the proofs of §3.2, and anything in this contract
that did not match the code; stop and report rather than guess.
