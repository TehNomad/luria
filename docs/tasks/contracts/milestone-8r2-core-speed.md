---
type: Task Contract
title: "Contract 8r2 — core: frame windows and DNA search without whole-record work"
description: "Two functions get faster, and neither may change a single output: frames::frame_residues reads only the codons whose middle base is in its window."
status: deprecated
tags: [milestone-8]
---
# Contract 8r2 — core: frame windows and DNA search without whole-record work

Planner: Claude. Status: **Implemented 2026-10-28** by worker `w-8r2` in
one round, reviewed and accepted. In `--release`, six frame windows of a
10 Mb record went from 0.39 s to about 45 µs, and a `GAATTC` search from
about 0.5 s to 21 ms. G1 failed test 1 but not `frames_fixtures`, whose
windows are the whole record, so the numbering cannot be seen there: the
planner's error in §7, and test 1 covers it. Tier: full (the frame and search rules
are D050's and D052's, and their fixtures guard this change).
Plan row: none. The owner asked, on 2026-10-28, for the speed findings of
the milestone 8 code review to be fixed before milestone 9.

## 1. What this contract builds

Two functions get faster. **Neither may change a single output.**

1. `frames::frame_residues` reads only the codons whose middle base is in
   the window it is given. Today it copies the whole record
   (`sequence.letters()`) and walks every codon from the frame's start to
   the end, keeping only those in the window. The view calls it per block,
   so a 10 Mb record costs whole-record passes on every key press.
2. `search::find_dna` and `search::covers` compare letters without heap
   allocation, and the scan reads the record's letters by slice.

Rules that apply, already settled:
- D050, D052, `milestone-8-view.md` §4.3 — what a frame residue is, and
  its number counted from the frame's first codon. `frame-tables.tsv` and
  `frame-tails.tsv` pin them.
- D050, D052, §4.4 — containment, not intersection; overlapping hits; one
  `Both` hit for a palindrome; wrapping by topology.
  `search-letters.tsv` (225 rows), `search-dna-patterns.tsv` and
  `search-dna-hits.tsv` pin them.
- D056 — records up to 64 MiB, `MAX_RECORD_BASES` 10 000 000.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `frame_residues` | `crates/core/src/frames.rs:119` | Item 1 |
| `codon_at`, `codon_symbol` | `crates/core/src/frame.rs:112`, `:79` | Keep using them for each codon in the window |
| `Sequence::letter_slice` | `crates/core/src/sequence.rs:401` | Borrow the letters; never `letters()` |
| `covers`, `span_matches`, `top_strand_starts`, `find_dna` | `crates/core/src/search.rs:102-260` | Item 2 |
| `bases_of` | `crates/core/src/sequence.rs:145` | The base sets the masks must agree with |
| Guards | `crates/core/tests/frames_fixtures.rs`, `search_fixtures.rs`, and the unit tests in both files | Must pass unchanged |

## 3. Files the worker may touch

1. `crates/core/src/frames.rs` — `frame_residues`'s body and comments, and
   new tests in its test module.
2. `crates/core/src/search.rs` — the bodies of `covers`, `span_matches` and
   `top_strand_starts` (and a private helper or two), and new tests.

## 4. The changes, exactly

Public signatures stay as they are.

**Item 1, `frame_residues`.**
- Borrow `sequence.letter_slice()`.
- Work out arithmetically the first codon of the frame whose middle base
  is in `[window_start, window_end)`, and that codon's number. On the
  plus strand codon `k` (from 0) starts at `offset + 3k`; on the minus
  strand it starts at `length - 1 - offset - 3k` and reads down. The
  number is `k + 1`, the same number today's walk gives it.
- Read codons from there, in the same order as today, and stop at the
  first codon whose middle base is outside the window, or where
  `codon_at` gives `None`.
- The work must be proportional to the window's width, not the record's
  length. A window past the end, an empty window, and a record shorter
  than three bases or than the frame's offset still give an empty vector.
  `window_end < window_start` is still refused first.

**Item 2, search.**
- `covers(template, pattern)` keeps its meaning and its public signature,
  but works from a 4-bit base mask per letter (A=1, C=2, G=4, T=8, the
  IUPAC codes as unions of those, from the same sets `bases_of` gives).
  Containment is `t & !p == 0` with `t != 0`. A letter outside the
  alphabet has no mask and matches nothing, as today.
- `find_dna` turns the pattern and its reverse complement into masks once.
  `span_matches` reads the template from `letter_slice()` by index,
  wrapping with `% size` only where `start + offset >= size`. No `Vec` or
  `String` is built per start.
- The order of hits, the `Both` merge and `wraps_origin` are untouched.

## 5. Tests the worker writes

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `frames.rs::a_window_gives_what_the_whole_frame_gives_inside_it` | For a 61-base and a 62-base linear sequence, both strands, all three frames, and every window `[s, e)` with `0 <= s <= e <= length + 3`: the result equals today's full walk filtered to the window. Write that full walk as a private test helper (the loop as it is today), so the comparison does not depend on the new code |
| 2 | `search.rs::the_masks_agree_with_the_base_sets` | For all 15 × 15 alphabet pairs, plus `X` and `U` on either side: `covers` equals containment over `bases_of`'s sets, computed in the test |
| 3 | `frames.rs::a_window_of_a_long_record_is_quick` | `#[ignore]`. A 10 000 000-base linear sequence (`ACGT` repeated), six-frame, a 60-base window at 9 000 000: all six calls together take under 5 ms in `--release` |
| 4 | `search.rs::a_search_of_a_long_record_is_quick` | `#[ignore]`. The same sequence, `GAATTC`: `find_dna` takes under 500 ms in `--release` |

## 6. Checks

`scripts/check`, plus:
- `cargo test -p luria-core --test frames_fixtures --test search_fixtures`
  passing, unchanged.
- `cargo test -p luria-core --release -- --ignored a_window_of_a_long_record_is_quick a_search_of_a_long_record_is_quick`.
  Also run tests 3 and 4 once against today's code (put the new bodies
  aside, then restore them) and report both timings, before and after.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | In `frame_residues`, number the first codon in the window as 1 | test 1 and `frames_fixtures` |
| G2 | Start the window one codon late | test 1 |
| G3 | In `covers`, test intersection (`t & p != 0`) | test 2 and `search_fixtures` |

## 8. Do not

- Change a public signature, an output, an error, or a fixture.
- Add a cache, a dependency, `unsafe`, or threads.
- Touch `protein_search.rs`, `frame.rs`, the cli, or `letters()`'s
  callers elsewhere.

## 9. Report

The before and after timings of tests 3 and 4, and G1 to G3 with their
failure lines.
