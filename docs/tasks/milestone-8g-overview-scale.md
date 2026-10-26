# Contract 8g — core: the overview scale (§4.6)

Planner: Claude. Status: ready, not yet delegated. Its two decisions are
D054, approved by the owner 2026-10-26. Tier: full (D051), but there is no
fixture table: plan §7 gives 8a1 tables for §4.2–§4.5 only. The tests here
are hand-derived values plus a property sweep that checks the rules without
using the formula. Plan row: **8g** in `docs/tasks/milestone-8-view.md` §7.

## 1. What this contract builds

`crates/core/src/overview.rs` has two functions. `overview_columns(length,
width)` gives the base range each overview column stands for.
`overview_column(length, width, position)` is its inverse: the column a base
is drawn in. The overview inset (8l2) draws features, the visible part, the
cursor and hits with these, and does no arithmetic of its own.

Rules that apply, already settled:
- Plan §4.6 (`docs/tasks/milestone-8-view.md:338-346`): the ranges tile the
  sequence exactly, differ in size by at most one base, and are in order.
- D054: the floor formula, and a record shorter than `width` drawn one
  column per base.
- D006: zero-based, half-open ranges.
- D048: a record has at most 10 000 000 bases, but these functions take any
  `u64` and must not overflow.
- D036: a feature that crosses the origin is two intervals. The caller maps
  each one, so it is drawn in both end ranges. This module knows nothing
  about features or topology.

## 2. What exists and must be reused

Nothing; this is plain integer arithmetic. Use `u128` for the
intermediate products, so no input can overflow, and no floating point.

## 3. Files the worker may touch

1. `crates/core/src/overview.rs`: new, with its `mod tests`.
2. `crates/core/src/lib.rs`: one line, `pub mod overview;`, between
   `pub mod mutation;` and `pub mod primer;`.

Everything else is out of scope.

## 4. The public surface, exactly

```rust
/// The base range `[start, end)` (D006) each overview column stands for, in
/// column order. There are `w = min(width, length)` columns, and column `i`
/// is `[⌊i·length/w⌋, ⌊(i+1)·length/w⌋)` (D054).
pub fn overview_columns(length: u64, width: u64) -> Vec<(u64, u64)>;

/// The column base `position` is drawn in: the one `i` whose range holds it,
/// which is `⌊((position + 1)·w − 1) / length⌋` with `w = min(width, length)`.
/// `None` when `position >= length`, or when `length` or `width` is 0.
pub fn overview_column(length: u64, width: u64, position: u64) -> Option<u64>;
```

Rules:

1. **Tiling.** The ranges are in order, each one's `start` is the previous
   one's `end`, the first starts at 0, and the last ends at `length`.
2. **Even.** Range sizes differ by at most one, and no range is empty. The
   larger ranges are spread through the row by the floor formula, not
   bunched at either end (D054).
3. **More columns than bases.** When `width > length`, the result has
   `length` entries, one base each: `[(0, 1), (1, 2), …]`, and
   `overview_column` gives `Some(position)`. The strip is drawn narrow
   (D054); where 8l2 places the shorter strip is not this module's concern.
4. **Zero.** `width == 0` or `length == 0` gives an empty `Vec`, and
   `overview_column` gives `None`. Neither panics.
5. **Inverse.** For every `position < length`, `overview_column` gives the
   unique `i` with `columns[i].0 <= position < columns[i].1`.
6. **No overflow.** `overview_columns(u64::MAX, 3)` and
   `overview_column(u64::MAX, u64::MAX, u64::MAX - 1)` return without
   panicking.

## 5. Tests the worker writes

Units in `overview.rs`.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `ten_bases_on_three_and_four_columns` | `(10, 3)` → `[(0,3), (3,6), (6,10)]`; `(10, 4)` → `[(0,2), (2,5), (5,7), (7,10)]` |
| 2 | `one_column_holds_everything` | `(4361, 1)` → `[(0, 4361)]`; `(1, 1)` → `[(0, 1)]` |
| 3 | `a_short_record_is_drawn_one_column_per_base` | `(3, 5)` → `[(0,1), (1,2), (2,3)]`; `overview_column(3, 5, p)` for `p` = 0, 1, 2 → `Some(0)`, `Some(1)`, `Some(2)`; `(1, 80)` → `[(0, 1)]`; `(60, 80)` → 60 entries, entry `p` equal to `(p, p + 1)` |
| 4 | `pbr322_on_eighty_columns` | `(4361, 80)`: 80 ranges; first `(0, 54)`, second `(54, 109)`, last `(4306, 4361)`; 41 ranges of 55 and 39 of 54; `overview_column(4361, 80, 0)` → `Some(0)`, `(.., 53)` → `Some(0)`, `(.., 54)` → `Some(1)`, `(.., 4360)` → `Some(79)` |
| 5 | `zero_length_or_width_gives_nothing` | `(0, 80)` and `(4361, 0)` → empty; `overview_column` → `None` for `(0, 80, 0)`, `(4361, 0, 0)` and `(4361, 80, 4361)` |
| 6 | `every_small_case_tiles_evenly_and_inverts` | for every `length` in 1..=64 and `width` in 1..=96: the result has `min(width, length)` entries, none empty, and rules 1, 2 and 5 hold, checked by walking the ranges (not by re-evaluating the formula) |
| 7 | `no_input_overflows` | rule 6's two calls; the first returns 3 ranges ending at `u64::MAX`, the second `Some(u64::MAX - 1)`. Do not call `overview_columns(u64::MAX, u64::MAX)`: it would allocate `u64::MAX` entries |

Hand derivation for test 4, so you can check it rather than trust it:
- Boundary `i` is ⌊4361·i/80⌋. 4361/80 = 54.5125, so ⌊54.5125⌋ = 54,
  ⌊109.025⌋ = 109, and ⌊54.5125·79⌋ = ⌊4306.4875⌋ = 4306.
- The sizes add to 4361 = 80·54 + 41, so 41 ranges have 55 bases and 39
  have 54.
- For position 54: (55·80 − 1)/4361 = 4399/4361, which floors to 1.
- For position 4360: (4361·80 − 1)/4361 floors to 79.

And for test 7's second call, `w = length = u64::MAX`, so the column is
⌊(length² − 1)/length⌋ = `length − 1`.

## 6. Checks

`scripts/check`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | Larger ranges first (`size = length / width`, plus one for the first `length % width`) | 1 (`(10, 4)`), 4 |
| G2 | No clamp: always `width` entries, empty ones where the formula leaves them | 3, 6 |
| G3 | Inverse as `position * w / length` | 4, 6 |
| G4 | `u64` products instead of `u128` | 7 |
| G5 | Last range ends at `width * (length / width)` | 1, 4, 6 |
| G6 | `overview_column` uses `width` where it should use `w` | 3, 6 |

## 8. Do not

- No floating point, and no dependency.
- No topology, feature or hit types in this module; it takes integers only.

## 9. Report

Every guard experiment with its failure line.
