# Contract 8m15 — cli: `esc` clears the hits; `ctrl-d`/`ctrl-u` half pages (M8-37, M8-38)

Planner: Claude. Status: **Implemented 2026-10-28** by worker `w-8m15` in
one round, reviewed and accepted. Findings from review:

* `half_page` follows `page`'s pattern, and also moves the top by `half`
  before `follow`. In a list, `ctrl-d`/`ctrl-u` page by `body_rows / 2`.
  At both prompts the two keys do nothing.
* With the key list open, `esc` closes it (8m3) and leaves the hits.
* `KEYS` in `sequence_view_keys.rs` changed as §3 item 2 allows (17 keys).
* G1–G4 each failed test 1 or 4, as §8 names. The `key_of` mapping has no
  test of its own, as with `ctrl-c`.

Tier:
light (D051): keys only. Owner's answers: `docs/tasks/milestone-8m-polish.md`
§1, M8-37 and M8-38.

## 1. What this contract builds

- `esc`, with no list, prompt or key list open, clears the search hits: the
  marks go, `n`/`N` have nothing to step, and the cursor stays. At a prompt
  or in a list `esc` does what it does today (closes it, leaves the hits).
- `ctrl-d` and `ctrl-u` move half a screen of blocks down and up, as in Vim.
  In an open list they move the highlight half a page.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `Key`, `handle`, the no-op arm | `crates/cli/src/sequence_view/state.rs:67`, `:450–460` | the new keys |
| `page`, `follow`, `block_of`, `visible` | `state.rs:1159` | half pages |
| the list's `PageUp`/`PageDown` arms, `List::page_row`, `body_rows` | `state.rs:1370–1381`, `crates/cli/src/sequence_view/list.rs:326` | half pages in a list |
| `help_rows` | `state.rs:850` | two new rows |
| `key_of` | `crates/cli/src/sequence_view/terminal.rs:51` | `ctrl-d`, `ctrl-u` |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs`, `terminal.rs`, `screen.rs`
   (only if an exhaustive match needs the new keys).
2. `crates/cli/tests/sequence_view_keys.rs`: the `KEYS` array only, with its
   length and comment: insert `"ctrl-d ctrl-u"` after `"PgUp PgDn"` and
   `"esc"` after `"n N"` (17 keys).
3. `crates/cli/tests/sequence_view_half_pages.rs`: new.

## 4. Rules

1. **Keys.** `Key` gains `CtrlD` and `CtrlU`. `key_of` maps `d` and `u`
   with exactly the control modifier to them, as it does `ctrl-c`. Plain `d`
   and `u` stay `Char`. At a prompt `ctrl-d`/`ctrl-u` change nothing (they
   are not text). [1, 3, 5]
2. **Half a screen.** `half = max(1, ⌊visible().len() / 2⌋)` blocks, taken
   before moving. `ctrl-d` moves the cursor `half` blocks down and `ctrl-u`
   `half` up, keeping its offset in the line and clamped to the target
   block's last base, and clamped at the first and last block (no wrap,
   even on a circular record), as `page` does. The top moves by `half` too,
   clamped to the first and last block, then `follow` keeps the cursor's
   block drawn. [1, 2]
3. **In a list.** `ctrl-d`/`ctrl-u` call `page_row(body_rows(height) / 2, ±1)`
   on the open list (at least 1). [3]
4. **`esc` clears the hits.** In the no-op arm's place for `Key::Esc`: with
   no list, prompt or key list open, `esc` sets the hits to `None`. The
   cursor, selection, lanes and scroll do not change, and nothing is said in
   the status line. With no hits it changes nothing. [4]
5. **The key list.** Two rows: after the `PgUp PgDn` row,
   `ctrl-d ctrl-u  move the cursor half a screen down or up`; after the
   `n N` row, `esc  clear the hits the screen marks`. The key line does not
   change. [5]

## 5. Hand values (the planner's)

`pbr` is `J01749.1.gb`, 4361 bp in 67 blocks of 66 (block 66 holds 4356–4360).
At 80 × 60 with `F` pressed twice (no lanes) and the lanes off, a block is 3
lines (ruler, top, bottom) and the strip is on (67 blocks is more than twice
the 13 that fit), so the body is 60 − 4 − 3 = 53 rows. `k` blocks need
`4k − 1` rows, so 13 fit: `visible()` is `0..13`, and `half` is 6. Tests
assert `visible() == 0..13` before relying on it.

From cursor 10: `ctrl-d` → 406 (block 6), top 6; `ctrl-d` → 802 (block 12),
top 12; `ctrl-u` → 406, top 6. From cursor 4168 (block 63, offset 10),
`ctrl-d` → 4360 (block 66, offset clamped to 4). From cursor 10, `ctrl-u` →
10, top 0.

## 6. Tests (`tests/sequence_view_half_pages.rs`)

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `ctrl_d_and_ctrl_u_move_half_a_screen` | §5's values for cursor and `top` (read the top as `visible().start`) |
| 2 | `half_pages_stop_at_the_ends` | §5's clamp at block 66 and at block 0 |
| 3 | `half_pages_in_a_list` | `pbr` at 80 × 24, `f` (well over 21 rows), `Home`: `ctrl-d` → highlight row 10, `ctrl-d` → 20, `ctrl-u` → 10. At the go-to prompt with `12` typed, `ctrl-d` leaves the prompt text `12` and the cursor where it was |
| 4 | `esc_clears_the_hits` | `pbr` at 80 × 24, search `AATT` (hits shown, cursor 58): `esc` → `hits()` is `None`, cursor still 58, no screen cell has `bg` `Yellow`; `n` then moves nothing. Before that: with the `/` prompt open, `esc` closes it and the hits stay (unchanged rule) |
| 5 | `the_key_list_has_the_new_keys` | `?` lists `ctrl-d ctrl-u  …` right after the `PgUp PgDn` row, and `esc  …` right after `n N` |

## 7. Checks

`scripts/check`, `python3 -I tests/fixtures/verify.py`, and every
`sequence_view*` target.

## 8. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | `half` is the whole `visible().len()` | 1 |
| G2 | the top does not move (only `follow`) | 1 |
| G3 | `esc` outside a prompt still changes nothing | 4 |
| G4 | `esc` at the `/` prompt also clears the hits | 4 |

## 9. Do not

- No new dependency. No change to `crates/core`, `layout.rs`, any fixture,
  or any existing test except §3 item 2.
- `PageUp`/`PageDown`, `j`/`k` and the key line stay as they are.
- If any other existing test fails, finish everything else, then stop and
  report it with its assertion, what it expected and what it got. Don't
  change it.
- Don't edit this contract.

## 10. Report

Every guard with its failure line and md5 lines before and after, the check
results, any existing test that failed, and any choice §4 left open.
