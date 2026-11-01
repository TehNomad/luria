---
type: Task Contract
title: "Contract 9g — core and cli: cut, yank, paste, undo and redo"
description: "x and d cut and y yanks, into one register kept for the session."
status: deprecated
tags: [milestone-9]
---
# Contract 9g — core and cli: cut, yank, paste, undo and redo

Planner: Claude. Status: implemented. Tier: light, with the core undo full
(plan M9-10).
Plan row: [docs/tasks/contracts/milestone-9-editor.md](/tasks/contracts/milestone-9-editor.md) §7, row 9g.

## 1. What this contract builds

This contract adds:
- `x` and `d` cut and `y` yanks, into one register kept for the session.
- `p` and `P` paste it.
- `u` undoes one edit and `ctrl-r` redoes it, back to the opened file.

One edit is one command (M9-16): an insert or overtype from entering the
mode to `esc` is one; each `r`, `x`, `d` and paste is one. Core gains the
undo step itself. Undoing an edit must give back exactly the buffer
before it: its letters, and its features with their qualifiers and
location text.

Rules that apply, already settled:
- Plan §5 M9-2 (keys), M9-5 (luria's own register only), M9-8 (undo
  and redo for the session), M9-16 (what one edit is).
- 9e's `apply_edit` and its recomputing; 9f's modes.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `EditBuffer::apply`, `check`, `Plan` | `crates/core/src/edit_buffer.rs:618`, `:729` | The undo record is made inside the same `apply` |
| `apply_edit`, the recompute steps | `state.rs` (9e) | Undo and redo recompute the same way |
| `EditMode`, `selection()` | `state.rs` (9d, 9f) | What `x`, `d`, `y` act on |
| `key_of` | `crates/cli/src/sequence_view/terminal.rs:46` | `ctrl-r` → a new `Key::CtrlR` |

## 3. Files the worker may touch

1. `crates/core/src/edit_buffer.rs` — `Undo`, `apply_undoable`, `undo`,
   and unit tests. `apply` keeps its signature and behaviour.
2. `crates/cli/src/sequence_view/state.rs` — the register, the keys, the
   undo and redo stacks, and unit tests. Split the recompute steps of
   `apply_edit` into a private function that undo and redo call too.
3. `crates/cli/src/sequence_view/terminal.rs` — `ctrl-r`.

## 4. The public surface, exactly

```rust
// edit_buffer.rs
/// What it takes to put the buffer back as it was before one edit.
#[derive(Clone, Debug, PartialEq)]
pub struct Undo { /* private: the span the edit wrote, the letters it
                     removed, and the features as they were */ }

impl EditBuffer {
    /// `apply`, and the step that undoes it.
    pub fn apply_undoable(&mut self, edit: &Edit)
        -> Result<(EditOutcome, Undo), DomainError>;
    /// Puts back the letters and features `undo` holds. It must be the
    /// undo of the last edit applied and not yet undone; anything else is
    /// a programming error (`debug_assert!` on the length).
    pub fn undo(&mut self, undo: Undo);
}
```

Behaviour in the view:
1. **Register**: one `Vec<char>`, empty at open, replaced by each cut or
   yank, kept for the session.
2. **`x`**: in `Normal`, cuts the cursor's base (`Delete [c, c+1)`). In
   `Visual`, cuts the selection and returns to `Normal`. The register
   gets what was cut, and the cursor goes to the cut's start, clamped.
3. **`d`**: in `Visual`, the same as `x`. In `Normal`, it does nothing
   and its message says to select with `v` first.
4. **`y`**: in `Visual`, copies the selection to the register, returns to
   `Normal`, and says how many bases. In `Normal`, it says to select
   first.
5. **`p`** pastes after the cursor (`Insert` at `c + 1`); **`P`** pastes
   before it (`Insert` at `c`). The cursor goes to the last pasted base.
   With an empty register, the message says there is nothing to paste.
6. **Edits as one command.** Every edit `apply_edit` makes now records its
   `(Edit, Undo)`. The edits of one command go together as one group on
   the undo stack:
   - an insert or overtype session's edits form one group, closed by
     `esc`; a backspace's delete belongs to its session;
   - every other command is a group of one;
   - a new group clears the redo stack.
7. **`u`** undoes the last group: its undos run in reverse order, then
   the view is recomputed once. The group moves to the redo stack, and
   the cursor goes to the start of the group's first edit, clamped. With
   nothing to undo, the message says so. `u` in `Insert` or `Overtype`
   is a refused letter (9f), not undo.
8. **`ctrl-r`** applies the last undone group's edits again, in order,
   with `apply_undoable`. The fresh undos replace the old ones, and the
   group returns to the undo stack.
9. `pub(super) fn edits(&self) -> usize` is the undo stack's length, the
   count 9h stamps (M9-16).

## 5. Tests the worker writes

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `edit_buffer.rs` unit: `undo_restores_exactly` (full tier) | For each of 9a's table rows (build them as `feature_edit_fixtures.rs` does, or a representative 15 covering every rule group, listed in the test): `apply_undoable` then `undo` gives a buffer equal to the one before, features, qualifiers and `location_text` included |
| 2 | unit: `undo_in_reverse_order` | Three edits on pBR322's buffer, undone last first: equal to the opened buffer |
| 3 | `state.rs` unit: `cut_then_paste_moves_bases` | `v`, `l` × 2, `x`, then `p`: the three bases are back after the cursor's base; the length is unchanged overall |
| 4 | unit: `yank_keeps_the_record` | `v`, `l`, `y`: the length is unchanged, `P` adds 2 bases before the cursor |
| 5 | unit: `an_insert_session_is_one_undo` | `i`, `A`, `C`, `G`, `esc`, `u`: the record equals pBR322's, and `edits()` is 0 |
| 6 | unit: `redo_after_undo` | `x`, `u`, `ctrl-r`: the base is cut again and `edits()` is 1; a new `x` after `u` clears redo, so `ctrl-r` then says there is nothing to redo |
| 7 | unit: `undo_brings_back_a_removed_feature` | `Delete` covering `misc_binding 24..27`, then `u`: the feature is back with its `location_text`, and the view's features equal the opened ones |
| 8 | unit: `d_and_y_need_a_selection` | `d` and `y` in `Normal` change nothing and set a message |

## 6. Checks

`scripts/check`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | `undo` restores the letters but not the features | test 1 or 7 |
| 2 | Each typed letter its own group | test 5 |
| 3 | A new edit does not clear redo | test 6 |

## 8. Do not

- Write to the system clipboard (M9-5: later).
- Change 9b's `apply` rules or 9a's files.
- Save (9h).
