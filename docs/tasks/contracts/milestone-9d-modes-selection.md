# Contract 9d — cli: edit modes, the selection, and the blocked record

Planner: Claude. Status: implemented. Tier: light (plan M9-10), behaviour tested,
wording not pinned.
Plan row: `docs/tasks/contracts/milestone-9-editor.md` §7, row 9d.

## 1. What this contract builds

The view learns two things in this contract:
- **Whether its record can be edited.** At open it calls 9b2's
  `buffer_for` and keeps the buffer, or keeps the reason the record is
  blocked (M9-12).
- **Visual mode.** `v` starts a selection at the cursor and the movement
  keys extend it. `esc` ends it. The selected bases are drawn marked on
  both strands' rows.

Nothing is edited yet. 9e applies edits, 9f types them, 9g cuts and pastes.

Rules that apply, already settled:
- Plan §5 M9-2 (the keys), M9-3 (the top strand), M9-12 (a blocked record),
  M9-6 (g) (a selection never crosses the origin).
- D003 — `state.rs` stays pure: no file or terminal access.

## 2. What exists and must be reused

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `ViewState`, `new`, `handle`, `status` | `crates/cli/src/sequence_view/state.rs:244`, `:313`, `:380`, `:786` | Gain the fields and key arms below |
| The key match | `state.rs:416-506` | `v` and the blocked keys go here; `Key::Char(_)` falls through today |
| Hit and cursor marking | `crates/cli/src/sequence_view/screen.rs:420-` | The selection is marked the same way, as one more layer |
| `buffer_for`, `EditBuffer`, `EditBlocked` | `crates/core/src/edit_buffer.rs` (9b2) | Called once in `new` |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs` — the fields, keys and status
   words below, and their unit tests.
2. `crates/cli/src/sequence_view/screen.rs` — draw the selection, and its
   unit tests.
3. `crates/cli/src/sequence_view/mod.rs` — only if a new item must be
   exported.

## 4. Behaviour

1. **State.** `ViewState` gains:
   - `edit: Result<EditBuffer, EditBlocked>`, from `buffer_for(&record)`
     in `new`;
   - `editing: EditMode`, a private enum that starts `Normal`. It has one
     other variant here, `Visual { anchor: u64 }`; 9f adds more.
   - `pub(super) fn selection(&self) -> Option<(u64, u64)>`. In `Visual` it
     is the zero-based half-open `[min(anchor, cursor), max(anchor,
     cursor) + 1)`, otherwise `None`. It is always a plain range, so it
     never crosses the origin of a circular record. When the cursor wraps
     past the end, the selection is the range between the two bases.
2. **`v`** in `Normal`, with an editable record, enters `Visual` with the
   anchor at the cursor. `v` or `esc` in `Visual` returns to `Normal` and
   changes nothing else: an `esc` that ends the selection does not also
   clear the hits.
3. **In `Visual`**, every movement key and every key that moves the cursor
   moves it as in `Normal`, and the selection follows. The movement keys
   are `h j k l w b`, the arrows, page up and down, `ctrl-d`, `ctrl-u`,
   home and end. The keys that move the cursor are go-to's `enter`, `n`,
   `N`, and `enter` in a list. Opening a list, go-to, search or the key
   list keeps the selection. `T` and `t` work as in `Normal`.
4. **Blocked record.** With `edit` an `Err`, the editing keys change
   nothing and set a message naming the feature: its name, key and line,
   and the error's `message()`. The editing keys are `v i a r R x d y p
   P u s` and `ctrl-r` once 9g maps it. The view's own keys work as today.
5. **Status line.** In `Visual` it begins with `-- VISUAL --` and the
   number of bases selected and their 1-based range (`12 bases,
   101..112`), then what it says today. A message replaces it as today.
6. **Drawing.** Every selected base's cell, on the base row and the
   complement row of every block on screen, gets a marking distinct from
   a hit's yellow and from the cursor's reversal: a blue background
   (`Color::Blue`) under the letter's own colour. The cursor and the codon
   underline stay drawn over it. A hit's yellow is covered where the two
   overlap.
7. `q` and `ctrl-c` behave as today (9h changes them).

## 5. Tests the worker writes

Light tier: assert behaviour, not wording, except where a row names text.

| # | Test (`state.rs` or `screen.rs` unit) | Asserts |
| --- | --- | --- |
| 1 | `v_selects_from_the_cursor` | pBR322: cursor at 9, `v`, `l` × 3: `selection()` is `(9, 13)`; `h` × 5 gives `(5, 10)`; `esc` gives `None` and leaves the cursor at 5 |
| 2 | `esc_ending_a_selection_keeps_hits` | A search with hits, `v`, `esc`: the hits are still there |
| 3 | `selection_survives_goto_and_lists` | `v`, then go-to `200` `enter`: the selection runs from the anchor to base 199; opening and closing `f` keeps it |
| 4 | `blocked_record_refuses_edit_keys` | `genbank/synthetic.gb` (blocked, 9b2 test 5): each of `v i a r R x d y p P u s` leaves `selection()` `None` and sets a message holding `ord` and `61` |
| 5 | `status_names_visual` | In `Visual`, `status()` starts with `-- VISUAL --` and holds the count and range |
| 6 | `screen.rs`: `selection_cells_are_marked` | A drawn frame: every selected base's cell on both rows has the blue background; the cursor cell is still reversed |
| 7 | `circular_end_selects_the_plain_range` | pBR322 (circular), cursor at 0, `v`, `End`: `selection()` is `(0, 4361)`, never a two-part range |

## 6. Checks

`scripts/check`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | `esc` in `Visual` also clears the hits | test 2 |
| 2 | The blocked check removed from `v` | test 4 |

## 8. Do not

- Edit anything: no `apply`, no register, no typing (9e–9g).
- Change any existing key's meaning in `Normal`, or any existing test.
- Read or write files from `state.rs`.
