---
type: Task Contract
title: "Contract 9m — cli: the ? key list in sections, scrolled"
description: "The ? list names every key the view has, edit keys and : commands included."
status: deprecated
tags: [milestone-9]
---
# Contract 9m — cli: the `?` key list in sections, scrolled

Planner: Claude. Status: implemented. Tier: light.
Plan row: [docs/tasks/contracts/milestone-9-editor.md](/tasks/contracts/milestone-9-editor.md) §7, row 9m.

## 1. What this contract builds

The `?` list names every key the view has, edit keys and `:` commands
included. The rows are grouped under section headings. The list scrolls
when it is longer than the body (owner, 2026-10-31). This supersedes
8m3 §4 rule 2's "any key closes the list": in the list, `j` and `k`
scroll.

## 2. What exists

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `help`, `help_rows`, the `?` arm, the "any key closes" branch of `handle` | `crates/cli/src/sequence_view/state.rs:362`, `:2283`, `:753`, `:632` | Changed as §4 says |
| `keys`'s `help` arm (`"any key closes"`) | `state.rs:2239` | New text |
| `status_body` | `state.rs` | Gains the list's position (§4.4) |
| The feature list's window and its scroll keys | `list_rows`, `list_key` in `state.rs` | Copy how they find the body's height and clamp |
| `draw_help` | `crates/cli/src/sequence_view/screen.rs:376` | Unchanged if `help_rows` returns the window (§4.2) |
| `question_mark_lists_every_key`, `KEYS` | `crates/cli/tests/sequence_view_keys.rs:52`, `:249` | Rewritten (§5) |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs`.
2. `crates/cli/tests/sequence_view_keys.rs`: `KEYS` and
   `question_mark_lists_every_key` only.
3. `crates/cli/src/sequence_view/screen.rs`, only if `draw_help` must
   change to draw the window.

## 4. Behaviour

1. **The rows.** These are the rows of the full list, in this order. A
   heading row is the heading word alone. Every other row is the key
   text, two spaces, and the description. Rows marked "as today" keep
   their text from today's `help_rows`, unchanged. `T` is shown only
   when the topology was assumed, as today.

   | Section | Rows |
   | --- | --- |
   | `Moving` | `hjkl`, `PgUp PgDn`, `ctrl-d ctrl-u`, `Home End`, `w b`, `g`: as today |
   | `Viewing` | `t`, `F`, `C`, `T`, `f`, `c`, `o`, `O`: as today |
   | `Searching` | `/`, `n N`, `esc`: as today |
   | `Editing` | the ten rows below |
   | `Saving and quitting` | the eight rows below |

   Editing:
   ```
   v  select from the cursor; esc ends the selection
   i a  insert before the cursor, after it; esc ends
   R  overtype from the cursor; esc ends
   r  replace the base, or every selected base, with the next letter typed
   backspace  in i, a and R: take back the last base typed
   x  cut the base under the cursor, or the selection
   d  cut the selection
   y  copy the selection into luria's register
   p P  paste the register after the cursor, before it
   u ctrl-r  undo one edit, redo it
   ```
   Saving and quitting:
   ```
   :w  write the opened file; the first write keeps the original as file~
   :w NAME  write to NAME; :w! NAME replaces a file that exists
   :q :q!  quit; :q refuses while edits are unsaved, :q! drops them
   :wq :x ZZ  write and quit; :x and ZZ write only when there are edits
   ZQ  quit without writing
   :N  go to base N
   q  quit, as :q
   ?  this list of every key the screen has
   ```
2. **The window.** `help_rows()` returns only the rows that fit the
   body, starting at the list's scroll position. The body's height is
   found as the feature list finds it. `?` opens the list at the top.
3. **Keys while the list is open:**
   - `j` and down scroll one row; `k` and up scroll back one;
   - `PgDn` and `PgUp` scroll one window;
   - `Home` and `End` go to the top and to the last full window;
   - `esc`, `q` and `?` close the list;
   - every other key does nothing; `ctrl-c` is unchanged.

   The position is clamped: it never goes above the top, nor past the
   point where the last row is the body's last row. When the whole list
   fits, no key scrolls it. A resize keeps the position clamped.
4. **Status and key lines.** While the list is open, the status line is
   `keys F-L of N`: the 1-based first and last rows shown, and the row
   count, headings included. The key line is
   `jk PgUp PgDn Home End  scroll  esc ? q  close`.
5. Nothing else changes. Outside the list no key changes meaning.

## 5. Tests

`KEYS` becomes every row's leading text in §4.1's order, headings
included, for pBR322 (no `T`). Headings are matched as the whole row.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `sequence_view_keys.rs::question_mark_lists_every_key` (rewritten) | At 80 x 24: `?` shows exactly the first 20 rows of `KEYS`, and the status line is `keys 1-20 of 39`. Then `End`: the last 20, and the status is `keys 20-39 of 39`. Drawn rows are read from the screen, as today. Every key row is its key, two spaces and a description. `j` at the end changes nothing; `Home` returns to the top. The key line is §4.4's. `q` closes the list and does not quit; after it, the key line is the Normal one, as today |
| 2 | unit, `state.rs`: `the_key_list_scrolls_and_clamps` | `?`, then `j` three times: the status is `keys 4-23 of 39`. `k` five times: `keys 1-20 of 39`. `PgDn`: `keys 20-39 of 39`, clamped. `PgUp`: `keys 1-20 of 39` |
| 3 | unit: `other_keys_leave_the_list_open` | `?`, then `x`, `:` and `i`: the list is still open, nothing is dirty, no command line is open, and the mode is `Normal`. `esc` closes the list |
| 4 | unit: `a_list_that_fits_does_not_scroll` | A view tall enough for all 39 rows: `j` and `PgDn` leave the status `keys 1-39 of 39` |

The `39` assumes pBR322 gives exactly the rows of §4.1 without `T`: 5
headings and 34 key rows. If your count differs, stop and report.

The tests in `sequence_view_levels.rs` and `sequence_view_half_pages.rs`
read rows in the first window. They must pass unchanged.

## 6. Checks

`scripts/check`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | `j` scrolls past the last full window (no clamp) | test 2 |
| 2 | Any key closes the list again | test 3 |
| 3 | `help_rows` returns all rows, not the window | test 1 |

Disable each rule once, run the tests, restore it, check with `cmp`
against a backup in `target/tmp/`, and report what failed.

## 8. Do not

- Change what any key does outside the open list.
- Colour or style the headings.
- Add dependencies, or change tests other than §3 item 2's.
