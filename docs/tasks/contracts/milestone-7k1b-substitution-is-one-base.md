---
type: Task Contract
title: "Milestone 7k1b: a range is a replacement"
description: "7k1 read a same-length range such as 1210-1212 GAT>AAC as a Substitution; a request's substitution is one base only, as q5 design refuses it."
status: deprecated
tags: [milestone-7]
---
# Milestone 7k1b: a range is a replacement

Status: **Implemented 2026-10-21.** The old rule failed `every_shorthand_row_matches` on `1210-1212 GAT>AAC`, as required. Planner: Claude. Corrects
7k1 (committed as bbe71f8).

## 1. What and why

7k1 read a same-length range such as `1210-1212 GAT>AAC` as a
`Substitution`. A request's `substitution` is one base only: `q5 design`
refuses `GAT>AAC` as a substitution ("expected original must be exactly
one letter"), and `q5-primer` row `native-01` writes that same edit as a
`replacement`. The planner's rule was wrong and is corrected in
`tests/fixtures/new/reference/shorthand.py`, whose output
`tests/fixtures/new/shorthand.tsv` now reads it as a replacement.
`crates/cli/tests/shorthand.rs`'s `every_shorthand_row_matches` fails on
`1210-1212 GAT>AAC` until this is done.

## 2. Change, `crates/cli/src/shorthand.rs` only

- In the range branch (about line 214), choose `Reading::Substitution`
  only when `x` and `y` are one base each; otherwise
  `Reading::Replacement`.
- Doc comments: `Substitution` is `1210 G>A` (one base to one base);
  `Replacement` is any other base change, `1210-1212 GAT>AAC` and
  `1210-1212 GAT>TT`.

## 3. Proof

Restore the old rule (same length chooses substitution) once:
`every_shorthand_row_matches` fails on `1210-1212 GAT>AAC`. Restore.

## 4. Do not

Change anything but `crates/cli/src/shorthand.rs`. No change to tests or
fixtures. Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```
