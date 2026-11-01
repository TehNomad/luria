# Milestone 4b-1 task contract: deletion primers and pairs in `primer.rs`

Status: **Implemented 2026-10-15** by the local worker; accepted on planner review (§8).
Date: 2026-10-15
Planner: Claude. Parent: `docs/tasks/contracts/milestone-4-indels.md`, pass 3, first
half. The second half, the design layer, is `milestone-4b2-deletion-design.md`.

## 1. Goal

`construct_primer` and `assemble_pair` accept a deletion result and check a
deletion pair against D031's invariant. Nothing designs a deletion yet: the
design layer keeps refusing one until 4b-2.

## 2. Requirements, inline

From **D031** (`docs/decisions/`, approved 2026-10-15). For a deletion of
`[s, e)` with width `w = e − s`:

- The forward primer's 5′ end is at `e` and it covers `[e, e + f)`. The
  reverse primer's 5′ end is at `s` and it covers `[s − r, s)`. Neither primer
  carries a non-template base.
- **Pair invariant:** reverse end + `w` = forward start. Template abutment,
  the substitution rule, is the case `w = 0`.

From **Milestone 4a**: `MutationResult::Deletion(NucleotideDeletion)` exists.
`result.is_deletion()` is true only for it. `result.target_coordinates()` is
the removed source coordinates, ascending and contiguous, so
`s = first` and `e = last + 1`. `result.edited()` is the shorter artifact.

## 3. Required behaviour, all in `crates/core/src/primer.rs`

### 3.1 `construct_primer` with a deletion result

When `result.is_deletion()`:

1. **Skip** two checks: that the source and edited artifact are equally long,
   and the "stray" containment check. Both compare position by position, which
   is meaningless across a length change. The deletion was already verified
   independently in `mutation::verify`.
2. **New check:** if the span overlaps `[s, e)` (`span.start() < e &&
   span.end() > s`), return `DomainError::InvalidPrimerCandidate` with
   `primer: label` and a reason that contains `"deleted"`.
3. The primer text is compared against the **template** (`before`) at the
   span, oriented as now, instead of against `after`. On mismatch, return
   `PrimerTextAtSpanMismatch` as now.
4. The changed positions inside the primer are empty: pass an empty vector
   where `claimed` feeds `changed_within`. Every later step (parts,
   annealing, overlap, tail reason) then runs unchanged and reports an
   all-annealing primer.
5. `edited_span`: for a **forward** primer, an interval created against
   `result.edited().reference()`. It is `[span.start() − w, span.end() − w)`
   when `span.start() >= e`, and `[span.start(), span.end())` when
   `span.end() <= s`. For a **reverse** primer it stays `None`, as now. Put
   this in a new private helper. Leave `edited_span_of` and the substitution
   path unchanged.

When `!result.is_deletion()`, behaviour is byte-for-byte unchanged.

Update the doc comment on `Primer::edited_span`. Its last sentence says
insertions and deletions are not supported; say instead that for a deletion
the forward primer's edited interval is shifted left by the deleted width and
names the edited artifact.

### 3.2 `PrimerPair` and `assemble_pair`

- `PrimerPair` gains a private field `deleted_width: u64` and
  `pub fn deleted_width(self) -> u64`.
- In `assemble_pair`, when `result.is_deletion()`:
  - **skip** H2, the coverage check. A deletion's removed bases lie between
    the primers by design.
  - **require** `reverse.span().end() == s` and `forward.span().start() == e`.
    Otherwise return `InvalidPrimerCandidate` with `primer: "pair"` and a
    reason that names `s`, `e`, the reverse end and the forward start.
  - set `deleted_width = w`.
- Otherwise `deleted_width = 0`, and every existing check runs as now.
- `abuts()` becomes `self.separation() == self.deleted_width as i64`. Its doc
  comment states D031's generalisation: `w = 0` gives template abutment.
- `boundary()`: when `deleted_width > 0` and
  `reverse end + deleted_width == forward start`, return
  `Some(reverse end)`. That is `s`, where the 5′ ends meet in the product.
  Otherwise keep the current behaviour. Add one sentence of doc for this case.
- `relationship()`: when `deleted_width > 0` and
  `separation() == deleted_width`, the phrase is
  `"abutting across a deletion of <w> template positions"`. Otherwise keep the
  current phrases.

### 3.3 Errors

No new `DomainError` variants. Do not edit `error.rs`.

## 4. Required tests, in `primer.rs`'s test module

Add a helper `deletion() -> MutationResult`: `apply_deletion` on `seq_a()`
(`"ATGGCTTGGAATCAGCAGCAGCAGCAGCAG"`) with no scope, `result_name()`, and a
`NucleotideDeletionIntent` for `[9, 12)` asserting `"AAT"`. The edited
artifact is `ATGGCTTGGCAGCAGCAGCAGCAGCAG`, 27 nt. All values below were derived
by plain string slicing (planner, 2026-10-15).

| # | Test | Input | Expect |
|---|---|---|---|
| D1 | a forward deletion primer is all template | forward, `[12, 22)`, `"CAGCAGCAGC"` | ok; `changed_positions()` empty; `annealing_overlap() == 10`; `tail()` is `None`; `edited_span()` is `Some` with start 9, end 19, and `reference() == deletion().reference()` |
| D2 | a reverse deletion primer | reverse, `[0, 9)`, `"CCAAGCCAT"` | ok; `edited_span()` is `None`; `annealing_overlap() == 9` |
| D3 | a primer may not cover a deleted base | forward, `[11, 21)`, `"TCAGCAGCAG"` | `InvalidPrimerCandidate`, reason contains `deleted` |
| D4 | the text must be the template | forward, `[12, 22)`, `"CAGCAGCAGA"` | `PrimerTextAtSpanMismatch` |
| D5 | a deletion pair abuts across the deletion | D1 + D2 | ok; `deleted_width() == 3`; `separation() == 3`; `abuts()`; `boundary() == Some(9)`; `relationship()` contains `deletion of 3` |
| D6 | a forward primer away from the deletion edge is refused | forward `[13, 23)` `"AGCAGCAGCA"` + D2 | `InvalidPrimerCandidate` |
| D7 | a reverse primer away from the deletion edge is refused | D1 + reverse `[0, 8)` `"CAAGCCAT"` | `InvalidPrimerCandidate` |
| D8 | a substitution pair has no deleted width | the pair built in `an_abutting_pair_reports_where_its_five_prime_ends_meet` | `deleted_width() == 0`; `abuts()` |

Build D1–D7 with `construct_primer` and `assemble_pair` directly, mirroring
the existing `forward_primer`, `reverse_primer` and `pair_of` helpers but
passing `deletion()`. Where a step is expected to fail, assert on the
`Err`, not on a panic.

## 5. Do not

- Do not add dependencies or error variants, or edit `error.rs`.
- Do not touch `primer_design.rs`, `mutation.rs`, `q5.rs` or `q5_design.rs`.
  The design layer's deletion refusal stays until 4b-2.
- Do not change any existing test or fixture. All 325 existing tests must
  pass unchanged.
- Do not change the substitution path's behaviour or messages.
- Do not edit `docs/`.

## 6. Checks

```bash
export TMPDIR=$PWD/target/tmp
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Expected: 333 tests passing (325 + 8), none failing.

## 7. Handoff

Report changed files, each check's exact result, the added test names, and
anything unclear. If something is ambiguous, stop and report rather than
choosing.

## 8. Outcome, 2026-10-15

The worker's run was aborted before it wrote a handoff, but the work was
complete. The planner checked the result directly:

```
cargo fmt --all -- --check                              pass
cargo test --workspace                                  pass, 333 tests (325 + 8), 0 failures
cargo clippy --workspace --all-targets -- -D warnings    pass
git diff --check                                         pass
```

Only `crates/core/src/primer.rs` changed. The 50 removed lines are the
substitution checks moved under `if !deletion` (or into an `else` branch), not
deleted. The substitution path's checks and messages are textually unchanged,
and all 325 earlier tests pass. Tests D1–D8 assert the §4 values exactly.

The worker added one helper the contract did not name: `deleted_region`,
returning `(s, e)` from the claimed coordinates. It returns `(0, 0)` for an
empty claim rather than panicking, and is only consulted for a deletion.
Accepted.
