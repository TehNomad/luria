# Contract 9o — cli: `backspace` in `R` brings the base back

Planner: Claude. Status: implemented. Tier: light.
Plan row: `docs/tasks/contracts/milestone-9-editor.md` §7, row 9o.

## 1. What this contract builds

In `R` (overtype), `backspace` takes back the last base typed in this
`R`, as Vim does. The base that was there before comes back and the
cursor moves onto it. Today `backspace` does nothing in `R` (owner,
2026-10-31). The docs and the `?` list then say `in i, a and R` again.

## 2. What exists

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `overtype_key`, `open_overtype`, `overtype_stopped` | `crates/cli/src/sequence_view/state.rs:1026-1067`, `:908`, `:381` | The new arm and the new memory |
| `insert_key`'s `Key::Backspace` arm | `state.rs:973` | The same rule: only bases typed in this command |
| `apply_edit`, `close_group`, `undo_group` | `state.rs` (9e, 9g) | The restore is one more `Substitute` in the `R` group |
| `Sequence::letter_at` | used at `state.rs:2468` | The base before it is overtyped |
| `help_list`'s `backspace` row | `state.rs:~2335` | `in i and a` becomes `in i, a and R` |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs`.
2. `docs/specs/cli.md`: the §13.7 `backspace` row only, which becomes `In
   `i`, `a` and `R`: take back the last base you typed. It never removes
   a base that was there before; in `R` the base that was there comes
   back`.
3. `docs/decisions/`: in D060, the words `In `i` and `a`, `backspace`
   takes back only bases typed in the current command` become
   ``backspace` takes back only bases typed in the current command; in
   `R` the overtyped base comes back (contract 9o)`.

## 4. Behaviour

1. **Memory.** `R` keeps, for this `R` only, a list of `(base, letter
   before)` for each base it overtyped, in order. It is cleared when `R`
   opens and when `esc` leaves it.
2. **Typing.** Before each substitution in `R`, push the cursor's base
   and its current letter (`letter_at`). This includes the substitution
   at the last base, which sets `overtype_stopped`. A letter refused at
   the end ("overtype stops…") pushes nothing.
3. **`backspace`.** If the list is empty, nothing changes: no edit, no
   cursor move, no message. Otherwise:
   - pop the last pair;
   - `apply_edit(Substitute { start: base, bases: vec![letter] })`;
   - move the cursor to `base` and call `follow()`;
   - set `overtype_stopped` to false.

   The restore is part of the same undo group as the typing, so `esc`
   then `u` undoes the whole `R` in one step.
4. **`?` list.** The `backspace` row becomes
   `backspace  in i, a and R: take back the last base typed`.

## 5. Tests (unit, `state.rs`, on `ViewState::new(pbr(), 80, 24)`)

Read the original letters from the opened record in the test. Never
copy them from what the code writes.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `backspace_in_overtype_restores` | Cursor on zero-based 10. `R`, then `A` `A` `A`, then `backspace` twice: base 10 is `A`, bases 11 and 12 are the original letters, and the cursor is 11 |
| 2 | `backspace_with_nothing_overtyped_does_nothing` | `R` then `backspace`: the record is unchanged, the cursor stays, and the view is not dirty |
| 3 | `backspace_after_the_end_stop` | Cursor on the last base. `R`, `A` (substitutes and stops), `backspace`: the last base is its original letter. Then `C`: the last base is `C`, with no "stops" message |
| 4 | `one_overtype_is_one_undo` | `R`, `A` `C` `G`, `backspace`, `esc`, `u`: the sequence equals the opened one, and a second `u` finds nothing more to undo |
| 5 | `the_key_list_says_r` | The `backspace` row of the key list (`help_list`) is §4.4's text |

## 6. Checks

`scripts/check`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | `backspace` moves the cursor back but does not restore | test 1 |
| 2 | `backspace` does not clear `overtype_stopped` | test 3 |
| 3 | The restore closes the group first (its own undo step) | test 4 |

Disable each rule once, run the tests, restore it, check with `cmp`
against a backup in `target/tmp/`, and report what failed.

## 8. Do not

- Change what `backspace` does in `i` and `a`, or what `R` does
  otherwise.
- Change any existing test or fixture, or any other part of the docs.
