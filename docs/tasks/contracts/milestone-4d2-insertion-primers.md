---
type: Task Contract
title: "Milestone 4d-2 task contract: 5′ tails and insertion pairs in primer.rs"
description: "primer.rs can represent a primer with a non-annealing 5′ tail, and can check an insertion pair."
status: deprecated
tags: [milestone-4]
---
# Milestone 4d-2 task contract: 5′ tails and insertion pairs in `primer.rs`

Status: **Implemented 2026-10-15** by the local worker; accepted on planner review (§9).
Date: 2026-10-15
Planner: Claude. Parent: [docs/tasks/contracts/milestone-4-indels.md](/tasks/contracts/milestone-4-indels.md), pass 4. Follows
`milestone-4d1-insertion.md` (implemented). The design layer is 4d-3.

## 1. Goal

`primer.rs` can represent a primer with a non-annealing 5′ tail, and can check
an insertion pair. This is the first time the `tail` field it has always
carried is actually filled. Nothing designs an insertion yet: the design
layer keeps refusing one until 4d-3.

## 2. Requirements, inline (D032, approved 2026-10-15)

For an insertion of `k` bases at point `p`:

- The forward primer's **annealing** portion is template `[p, p + f)`; the
  reverse primer's is the reverse complement of template `[p − r, p)`. Both
  primers' annealing portions abut at `p`.
- `k ≤ 6`: the forward primer's 5′ tail is the whole insertion; the reverse
  primer has no tail.
- `k > 6`: the forward tail is the last `ceil(k/2)` inserted bases; the
  reverse tail is the reverse complement of the first `floor(k/2)`.
- A primer's text is its tail followed by its annealing portion, 5′→3′.
- **A tail does not anneal**, so it never counts toward the annealing overlap
  (H1, `minAnnealingOverlap`).

From Milestone 4d-1: `MutationResult::Insertion`, `result.is_insertion()`,
`target_coordinates()` is **empty** for an insertion, and the record carries
`point()` and `inserted()`.

## 3. Required changes, all in `crates/core/src/primer.rs`

### 3.1 A tail is a third kind of part

- `PrimerPartRole` gains `Tail`. `name()` returns `"tail"`;
  `carries_mutation()` is false for it. Update the enum's doc comment: a tail
  is no longer "refused by this layer".
- `PrimerPart.span` becomes `Option<NucleotideInterval>` and `span()` returns
  `Option<NucleotideInterval>`. It is `None` only for a `Tail` part, because a
  tail covers no template coordinate. Every existing construction site passes
  `Some(...)`.
- `annealing_of` skips a part whose span is `None`.
- `partition_fault` is unchanged: a tail part occupies primer indices like any
  other, so the partition still covers the text exactly.

### 3.2 Constructing a primer with a tail

Add a new public function, leaving `construct_primer` as it is:

```rust
pub fn construct_tailed_primer(
    template: &Sequence,
    result: MutationResult,
    role: PrimerRole,
    tail: String,
    annealing: String,
    span: NucleotideInterval,
) -> Result<Primer, DomainError>
```

It builds a primer whose `sequence` is `tail + annealing` and whose `span`
covers the **annealing portion only**. Requirements, each a refusal:

1. `result.is_insertion()` must be true; otherwise
   `InvalidPrimerCandidate`, reason containing `"a 5′ tail"`.
2. `tail` must not be empty (use `construct_primer` for a primer with no
   tail); `annealing` must not be empty. `EmptySequenceContent` for an empty
   annealing portion, `InvalidPrimerCandidate` for an empty tail.
3. Every letter of both must be a DNA letter, as `construct_primer` checks.
4. `annealing.len()` must equal the span's width, with
   `construct_primer`'s existing message.
5. The annealing text must be the **template** at the span, oriented for the
   role (reverse primers reverse-complement it), else
   `PrimerTextAtSpanMismatch`.
6. `require_sequence` for the span and the result, and `span.within(template)`,
   as `construct_primer` does.

The resulting `Primer`:

- `parts`: exactly two, in primer-index order — the `Tail` part (span `None`,
  `annealing: false`, indices `[0, tail.len())`) then one `Context` part
  (span `Some(span)`, `annealing: true`, covering the rest). An insertion
  primer's annealing portion is wholly wild-type, so it is one `Context` run.
- `tail`: `Some(the Tail part)`.
- `tail_reason`: `"a 5′ tail of <n> bases carries the insertion and does not
  anneal"`.
- `changed_positions`: empty. No template position under this primer changed.
- `annealing_sequence`: the annealing text. `annealing_spans`: `[span]`.
  `annealing_length` and `annealing_overlap`: the annealing portion's length.
  `overlap_positions`: its primer indices, which start at `tail.len()`.
- `whole_primer_match_count`: the annealing portion's length. The tail matches
  no template position.
- `edited_span`: `None` for both roles. The forward primer's text spans the
  junction between inserted and template bases, so it is not one interval of
  the edited artifact. Say so in a comment.
- `length`: the whole text's length, tail included.

### 3.2a Correction, 2026-10-15: `construct_primer` accepts an insertion

As first written, §3.2 said to leave `construct_primer` as it is, yet T6, T7,
T9 and T10 build an untailed reverse primer with it on an insertion result.
That cannot work: `construct_primer` refuses any result whose edited artifact
differs in length, and only a deletion was exempt. The worker was steered,
before writing any code, to make these changes for an insertion result.
Substitutions are unchanged.

1. Skip the equal-length and "stray" checks, as for a deletion.
2. Compare the primer text against the **template** at the span, as for a
   deletion. The changed positions are empty.
3. Refuse a span with `start < p < end` (it would read straight across the
   insertion point) with `InvalidPrimerCandidate`, reason containing
   `"insertion point"`.
4. `edited_span` is `None` for every primer of an insertion.

### 3.3 `assemble_pair` for an insertion

When `result.is_insertion()`:

- **skip** H2's coverage check. `target_coordinates()` is empty for an
  insertion, so the loop is already vacuous; make the skip explicit with a
  comment rather than relying on that.
- **require** `forward.span().start() == p` and `reverse.span().end() == p`,
  where `p` is the insertion point from the record. Otherwise
  `InvalidPrimerCandidate`, `primer: "pair"`, naming `p`, the forward start
  and the reverse end.
- **require** that the two tails rebuild the insertion: the reverse tail's
  reverse complement followed by the forward tail must equal
  `record.inserted()`. Otherwise `InvalidPrimerCandidate`, `primer: "pair"`,
  with a reason containing `"rebuild the insertion"`. Read each tail from
  `Primer::tail()`; a primer with no tail contributes an empty string.
- `deleted_width` is 0, so `abuts()` already means "the annealing portions
  touch at `p`".

To reach the point and the inserted text, add a private helper that returns
`Option<(u64, String)>` from a `MutationResult`, `None` unless it is an
insertion. Do not add a public accessor to `MutationResult`.

### 3.4 Errors

No new `DomainError` variants. Do not edit `error.rs`.

## 4. Required tests, in `primer.rs`'s test module

Add a helper `insertion(p, before, after, inserted) -> MutationResult` using
`apply_insertion` on `seq_a()` (`"ATGGCTTGGAATCAGCAGCAGCAGCAGCAG"`, 30 nt) with
`result_name()` and no scope. Values derived by plain string slicing (planner,
2026-10-15): at `p = 12` the flanks are `T` (index 11) and `C` (index 12);
template `[12, 22)` is `CAGCAGCAGC`; `rc(template[2,12))` = `rc("GGCTTGGAAT")`
= `ATTCCAAGCC`.

| # | Test | Input | Expect |
|---|---|---|---|
| T1 | a tailed forward primer reports its tail | `insertion(12, T, C, "GGG")`, forward, tail `"GGG"`, annealing `"CAGCAGCAGC"`, span `[12,22)` | ok; `sequence() == "GGGCAGCAGCAGC"`; `length() == 13`; `annealing_overlap() == 10`; `annealing_length() == 10`; `changed_positions()` empty; `edited_span()` is `None`; `tail()` is `Some`, its `role()` is `Tail`, its `span()` is `None`, its `primer_indices() == (0, 3)`; `parts().len() == 2`; `tail_reason()` contains `"does not anneal"` |
| T2 | the annealing portion must be the template | same but annealing `"CAGCAGCAGA"` | `PrimerTextAtSpanMismatch` |
| T3 | the annealing length must match the span | same but annealing `"CAGCAGCAG"` with span `[12,22)` | `InvalidPrimerCandidate` |
| T4 | a tail is refused for a substitution | `construct_tailed_primer` with the module's existing `edit()` result | `InvalidPrimerCandidate`, reason contains `5′ tail` |
| T5 | the parts of a tailed primer partition it | T1's primer | `partition_fault(parts, changed_positions, length)` is empty |
| T6 | a short insertion pairs with an untailed reverse primer | T1's forward + `construct_primer(reverse, [2,12), "ATTCCAAGCC")` on the same insertion result | ok; `abuts()`; `boundary() == Some(12)`; `deleted_width() == 0` |
| T7 | a split insertion needs both tails to rebuild it | `insertion(12, T, C, "GATTACA")` (7 bases, so forward tail `"TACA"`, reverse tail `rc("GAT") == "ATC"`): forward tailed with `"TACA"`, reverse tailed with `"ATC"` and annealing `"ATTCCAAGCC"` at `[2,12)` | pair is ok |
| T8 | tails that do not rebuild the insertion are refused | as T7 but the forward tail is `"TACG"` | `InvalidPrimerCandidate`, reason contains `rebuild the insertion` |
| T9 | an annealing portion away from the point is refused | T1's forward + a reverse primer at `[1,11)` (`rc(template[1,11))`) | `InvalidPrimerCandidate` |
| T10 | a tail does not count toward the bound | T1's pair, `require_design_constraints(pair, 10)` ok and `(pair, 11)` refused | `AnnealingOverlapBelowBound` for 11 |

For T6, T9 and T10 build the reverse primer with the existing
`construct_primer`, since a `k ≤ 6` insertion gives it no tail.

## 5. Permitted changes to existing tests

Only what §3.1's `Option` span forces, and only mechanically: a test that
reads `PrimerPart::span()` must unwrap it. Change no expectation value. If no
existing test reads it, change nothing.

## 6. Do not

- Do not add dependencies or error variants, or edit `error.rs`.
- Do not change `construct_primer`'s behaviour or messages, or
  `PrimerPlacement`'s refusal of `ConfineToTail`.
- Do not touch `mutation.rs`, `primer_design.rs`, `q5.rs` or `q5_design.rs`.
- Do not edit [docs/](/) or any fixture.

## 7. Checks

The five in `AGENTS.md`, with `export TMPDIR=$PWD/target/tmp`. Expected: 377
tests (367 + 10), none failing.

## 8. Handoff

Changed files, exact check results, added test names, anything unclear. If
something is ambiguous, stop and report rather than choosing.

## 9. Outcome, 2026-10-15

Three worker runs. The first was cut off by the harness's time limit with the
library built but 20 mechanical test compile errors: stray trailing
semicolons, and `&str` passed where `String` was taken. The second fixed them.
The third added T11 and fixed three stale doc comments.

Planner-run checks: fmt, clippy `-D warnings` and `git diff --check` pass, and
**378 tests pass (367 + 11), 0 failures**.

**Mutation check.** Disabling each new rule in turn:

| Rule disabled | Caught by |
| --- | --- |
| tails rebuild the insertion | `tails_that_do_not_rebuild_the_insertion_are_refused` |
| annealing portions meet at `p` | `an_annealing_portion_away_from_the_insertion_point_is_refused` |
| tailed annealing text is the template | `the_annealing_portion_of_a_tailed_primer_must_be_the_template` |
| a tail only for an insertion | `a_tail_is_refused_for_a_substitution` |
| an untailed primer may not straddle `p` | nothing at first; **T11 added**, now caught |

The straddle rule came from the planner's §3.2a correction, which originally
had no test.

**Accepted judgement calls:** a tailed primer reports placement
`ConfineToTail`, derived from its shape (the caller-facing refusal of
`ConfineToTail` in `construct_primer` is unchanged); DNA-letter positions in a
tailed primer are whole-primer indices; an empty tail is refused before an
empty annealing portion. The last two are untested.
