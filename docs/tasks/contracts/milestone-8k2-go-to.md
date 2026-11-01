---
type: Task Contract
title: "Contract 8k2 — cli: go-to (§5.3)"
description: "g opens a one-line prompt on the status line."
status: deprecated
tags: [milestone-8]
---
# Contract 8k2 — cli: go-to (§5.3)

Planner: Claude. Status: **Implemented 2026-10-26** by worker `w-8k2` in one
round, reviewed and accepted. Tier: light (D051): tests assert behaviour, not
wording; the hand test (8m) may change keys and sentences. Plan row: **8k2**
in [docs/tasks/contracts/milestone-8-view.md](/tasks/contracts/milestone-8-view.md) §7. Findings from review:

* Tests 1–10 and G1–G7 behaved as §7 says.
* The planner ran ten mutations; nine were caught. The tenth (letting
  residue `residues() + 1`, the stop codon, through the range check) is
  equivalent: the letter lookup then finds no residue and gives the same
  refusal.
* Choices left open and accepted: a letter with no digits is a name;
  digits too large for `u64` saturate for a residue number as for a base;
  the prompt's status line is `go to {text}`; `g` inside the prompt is
  text; `resize` leaves the prompt open; several matches pass the trimmed
  text as the list's filter.

## 1. What this contract builds

`g` opens a one-line prompt on the status line. `enter` goes to a base, to
a residue's codon in the selected CDS, or to a feature by name; several
matching features open 8k3's feature list filtered to them. No search
(8l1), no overview (8l2).

Rules that apply, already settled:
- Plan §5.3 (`docs/tasks/contracts/milestone-8-view.md:392-406`), as D057 amends it:
  digits alone are a base; a residue is a letter and a number (`D404`),
  refused with the actual residue when the letter is wrong.
- D056: names are matched by core's `match_name`, over the same names the
  feature list uses (8k3's `feature_names`).
- D055: a residue stands over its codon's middle base, so go-to puts the
  cursor there, as `w` and `b` do (8k1 rule 5).

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `ViewState`, `handle`, `follow`, `enter_feature`, `open_list` | `crates/cli/src/sequence_view/state.rs` (8k1, 8k3) | the prompt lives in the state; a single named feature is `enter_feature`, several are `open_list(ListKind::Features, text)` |
| `feature_names`, `ListKind` | `sequence_view/list.rs` (8k3) | the names a feature is matched by |
| `match_name`, `Matched` | `crates/core/src/name_match.rs:135` | exact first, then word-start prefix |
| `CdsMap::residues`, `residue_bases` | `crates/core/src/cds_map.rs:104`, `:125` | the residue count; a residue's three bases in reading order, the middle one at `[1]` |
| `Chosen` (`map`, `letters`) | `state.rs` (8k1) | the selected CDS's map and residue letters |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view.rs`: add `pub mod goto;`.
2. `crates/cli/src/sequence_view/goto.rs`: new, the parser and its
   `mod tests`.
3. `crates/cli/src/sequence_view/state.rs`: `g`, the prompt, its routing,
   `prompt()`, the status and key lines while it is open.
4. `crates/cli/tests/sequence_view_goto.rs`: new.

## 4. The public surface, exactly

```rust
// goto.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Nothing,                                  // blank
    Base(u64),                                // 1-based, as typed
    Residue { letter: char, number: u64 },    // `D404`
    Name(String),                             // anything else, trimmed
}
pub fn parse_target(text: &str) -> Target;
// state.rs
impl ViewState {
    pub fn prompt(&self) -> Option<&str>;     // the text typed; None when closed
}
```

Rules (the §5 tests in brackets):

1. **Parse** (`parse_target`, on the text trimmed of whitespace). Empty →
   `Nothing`. Only ASCII digits → `Base`. One ASCII *uppercase* letter then
   only ASCII digits → `Residue`. Anything else → `Name`, so `d404` and
   `m1` are names. Digits too large for `u64` give `u64::MAX`, which is out
   of range. [9]
2. **The prompt.** `g` opens it with no text (nothing below 80 × 24, and not
   while a list is open: there `g` is filter text). While it is open:
   every `Char` is text, `q` included; `Backspace` drops the last
   character; `Esc` closes it and changes nothing else; `Enter` closes it
   and acts (rules 3–5); `CtrlC` quits; other keys do nothing. Below
   80 × 24, `q` quits, as 8k3 round 2 made it for an open list. The status
   line shows the text typed; the key line shows the prompt's keys. [8, 10]
3. **A base.** `Base(n)` with `1 ≤ n ≤ length`: the cursor goes to `n − 1`
   and the scroll follows. Otherwise the cursor stays and `message` says
   the range. `Nothing` does nothing. [1]
4. **A residue.** No CDS selected, or `number` outside `1..=residues()`:
   the cursor stays and `message` says why. The residue's letter is not
   `letter`: the cursor stays and `message` names the residue, its actual
   letter and the letter typed (`residue 404 is V, not D`). Otherwise the
   cursor goes to the codon's middle base, `residue_bases(number)[1]`, and
   the scroll follows. [2, 3, 4]
5. **A name.** `match_name(feature_names(record), name)`: `One` →
   `enter_feature` on it, exactly as the feature list's `enter` (8k3 rule
   4: the jump, the CDS selection, the refusals); `Several` →
   `open_list(ListKind::Features, name)`, the cursor stays; `Nothing` →
   the cursor stays and `message` says no feature is named that. [5, 6, 7]

## 5. Tests the worker writes (`tests/sequence_view_goto.rs`)

`pbr` is `tests/fixtures/genbank/raw/J01749.1.gb` (4361 bp; `bla` = cdss 2,
minus, 286 residues; M1's codon is zero-based 4152, 4151, 4150 and S2's
4149, 4148, 4147). `lin` is `tests/fixtures/genbank/synthetic-linear.gb`
(`synA` selected on open; from `genbank/synthetic-linear-residues.tsv`: M1
at `[42, 45)`, A42 at `[165, 168)`, W87 at `[300, 303)`). `syn` is
`tests/fixtures/genbank/synthetic.gb` (feature 2 = `ignored-gene`,
`/locus_tag="m1"`, `complement(13..24)`, cdss 1, usable). 80 × 24. "`g X`"
means `g`, type `X`, `Enter`. Values were derived by the planner.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `a_number_goes_to_that_base` | `pbr`: `g 1210` → cursor 1209, `prompt()` is `None`, the cursor's block in `visible()`; `g 4361` → 4360; `g 0` → 4360, `message()` is `Some`; `g 4362` → 4360, `Some`; `g` then `Enter` → 4360, `message()` is `None`. `lin`: `g 100` → cursor 99 (a base, though a CDS is selected) |
| 2 | `a_residue_goes_to_its_codon_middle` | `lin`: `g A42` → 166; `g W87` → 301; `g M1` → 43. `pbr`, `select_cds(Some(2))`: `g M1` → 4151; `g S2` → 4148 |
| 3 | `a_wrong_letter_is_refused_with_the_actual_residue` | `lin`, cursor 0: `g D42` → cursor 0, `message()` contains `42`, `A` and `D` |
| 4 | `a_residue_needs_a_cds_and_a_number_in_range` | `pbr` (none selected): `g M1` → cursor 0, `message()` is `Some`. After `select_cds(Some(2))`: `g X287` → cursor 0, `Some` (287 is the stop codon, no residue); `g X0` → cursor 0 |
| 5 | `a_name_goes_to_its_feature` | `pbr`: `g mobile` → cursor 3147 (`mobile_element` 3148..4361); `g rop` → 1914, `selected() == None`, `message()` contains `no_id`. `syn`: `g m1` → cursor 12, `selected() == Some(1)` |
| 6 | `several_names_open_the_list` | `pbr`: `g bla` → `list()` open, kind `Features`, filter `bla`, rows `[37, 38, 39, 40]`, cursor 0, `prompt()` is `None` |
| 7 | `an_unknown_or_undrawn_name_does_not_move` | `pbr`: `g zzz` → cursor 0, `message()` is `Some`, no list. `syn`: `g ord` → cursor 0, `message()` contains `order`; `g fuzzy` → cursor 0, `message()` is `Some` |
| 8 | `keys_go_to_the_prompt` | `pbr`: `g`, `j`, `q` → each `Continue`, `prompt() == Some("jq")`, cursor 0; `Backspace` → `Some("j")`; `Esc` → `None`, cursor 0; `g`, `CtrlC` → `Quit`. `resize(79, 24)` with the prompt open, `q` → `Quit` |
| 9 | `parse_target_by_shape` | unit, in `goto.rs`: `""`, `"  "` → `Nothing`; `"1210"`, `" 42 "` → `Base(1210)`, `Base(42)`; `"D404"` → `Residue { letter: 'D', number: 404 }`; `"d404"`, `"m1"`, `"404D"`, `"D4x"` → `Name` of the trimmed text; `"99999999999999999999999"` → `Base(u64::MAX)` |
| 10 | `the_status_line_shows_the_prompt` | `pbr`: `g`, `12` → the status row (row 21) contains `12` |

## 6. Checks

`scripts/check`, plus `cargo test -p luria-cli --test sequence_view_goto`,
`--test sequence_view_list` and `--test sequence_view` (8k1 and 8k3 still
pass).

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | Digits alone are a residue when a CDS is selected | 1 |
| G2 | A residue goes to its codon's first base in reading order | 2 |
| G3 | The residue letter is not checked | 3 |
| G4 | A lowercase letter starts a residue too | 5, 9 |
| G5 | `Several` jumps to the first match | 6 |
| G6 | `q` quits while the prompt is open | 8 |
| G7 | `Base(n)` goes to `n` instead of `n − 1` | 1 |

## 8. Do not

- No new dependency; no change to `crates/core`, `layout.rs`, `list.rs`,
  `review`, or any existing test or fixture.
- No search, no history of past go-to text, no completion.
- No file is ever written.

## 9. Report

Every guard experiment with its failure line and md5 lines, and every
choice §4 left open.
