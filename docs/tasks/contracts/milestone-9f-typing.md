# Contract 9f — cli: insert, overtype, replace and the terminal's paste

Planner: Claude. Status: implemented. Tier: light (plan M9-10).
Plan row: [docs/tasks/contracts/milestone-9-editor.md](/tasks/contracts/milestone-9-editor.md) §7, row 9f.

## 1. What this contract builds

Typing bases. `i` and `a` insert, `R` overtypes, and `r` replaces one
base, or every base of a selection. In insert mode the terminal's own
paste arrives as one piece of text, which is checked as M9-4 says and
inserted in one go. Every change goes through 9e's `apply_edit`.

Rules that apply, already settled:
- Plan §5 M9-2 (keys, and in insert and overtype every printable key is
  input), M9-3 (the top strand), M9-4 (letters and what a paste drops),
  M9-5 (the terminal's paste works in insert mode).
- 9d's `EditMode` and its blocked-record rule; 9e's `apply_edit`.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `EditMode`, `selection()`, the blocked message | `state.rs` (9d) | Gains the variants below |
| `apply_edit`, `Edit::{Insert, Substitute}` | `state.rs` (9e), `crates/core/src/edit_buffer.rs` | Every change |
| `DNA_ALPHABET`, `is_dna_letter` | `crates/core/src/sequence.rs:53`, `:86` | The 15 letters, after folding to upper case |
| `key_of`, `run` | `crates/cli/src/sequence_view/terminal.rs:46`, `:94` | Bracketed paste and the paste event |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs` — the modes, keys and `paste`
   below, and unit tests.
2. `crates/cli/src/sequence_view/terminal.rs` — enable bracketed paste
   when the session opens, and disable it on every way out, as the
   terminal is restored today. Hand `Event::Paste(text)` to
   `ViewState::paste`.

## 4. Behaviour

`EditMode` gains `Insert { point: u64, typed: u64 }`, `Overtype` and
`ReplaceOne`. While any of them is active, `q` is a refused letter, not
quit, and `ctrl-c` behaves as today (9h changes it).

1. **`i`** enters `Insert` with the point at the cursor (before its base).
   **`a`** enters it with the point at the cursor + 1 (after its base).
   `typed` starts at 0.
2. **In `Insert`**:
   - A letter in `DNA_ALPHABET`, either case, is `apply_edit(Insert { at:
     point, bases: [upper] })`. Then `point` and `typed` go up by 1, and
     the cursor goes to `point`, clamped to the last base.
   - Any other printable key (digits, space, `q`, the view's keys) changes
     nothing. Its message names the character and says only bases can be
     typed.
   - **`backspace`**, with `typed > 0`, deletes the base before `point`
     (`Delete [point-1, point)`). `point` and `typed` go down by 1. With
     `typed == 0` it does nothing: a backspace never deletes a base that
     was there before this insert. *This key is not in M9-2; it is the
     planner's addition, flagged to the owner.*
   - **`esc`** returns to `Normal`, with the cursor on the last base
     typed, or where it was if nothing was typed.
3. **`R`** enters `Overtype`:
   - A letter substitutes the base at the cursor (`Substitute { start:
     cursor, bases: [upper] }`), and the cursor moves one right.
   - At the last base, a letter is still substituted, but the cursor
     stays. A further letter is refused, with a message that overtype
     stops at the end of the record. Overtype never makes the record
     longer.
   - Any other printable key is refused as in insert. `esc` returns to
     `Normal`.
4. **`r`** enters `ReplaceOne`, from `Normal` or from `Visual`:
   - The next key, if a letter, substitutes the cursor's base. From
     `Visual`, it substitutes every selected base with that letter in one
     `Substitute`, and the mode returns to `Normal`.
   - `esc` cancels. Any other key cancels with a refusal message.
5. **`paste(&mut self, text: &str)`**, called for the terminal's paste:
   - In `Insert`, M9-4 applies, in this order:
     - a line whose first non-space character is `>` is dropped whole;
     - spaces, tabs, `\r`, `\n` and ASCII digits are dropped;
     - any other character not in `DNA_ALPHABET` after folding refuses the
       whole paste, changing nothing. The message names the character and
       its line and column in the pasted text, both 1-based.
   - What is left, if not empty, is one `apply_edit(Insert …)` at the
     point. `point` and `typed` go up by its length.
   - Outside `Insert`, a paste changes nothing. Its message says to press
     `i` or `a` first.
6. **Status line**: `-- INSERT --`, `-- OVERTYPE --` or `-- REPLACE --`
   leads it, as `-- VISUAL --` does.
7. On a blocked record, `i a r R` refuse as 9d says.

## 5. Tests the worker writes

| # | Test (`state.rs` unit) | Asserts |
| --- | --- | --- |
| 1 | `insert_before_and_after` | pBR322, cursor 9: `i`, `G`, `C`, `esc`: bases 10–11 (1-based) are `GC` and the length is 4363. `a` at cursor 9, then `t`: the new base is the 11th, upper case |
| 2 | `insert_refuses_non_bases` | In `Insert`, `q`, `1`, space and `x` each leave the record unchanged and set a message. The session goes on: `q` does not quit |
| 3 | `backspace_only_takes_what_was_typed` | `i`, `A`, `A`, backspace × 3: the length is back to 4361 and the letters equal pBR322's |
| 4 | `overtype_stops_at_the_end` | `End`, `R`, `G`, `G`: the last base is `G`, the length is unchanged, and the second `G` is refused |
| 5 | `r_replaces_a_selection` | `v`, `l` × 2, `r`, `n`: three bases are `N`, the mode is `Normal`, and the length is unchanged |
| 6 | `paste_drops_what_m9_4_drops` | In `Insert`, paste `">x\nACGT 12\nacgt\n"`: 8 bases inserted, `ACGTACGT` |
| 7 | `paste_refuses_a_bad_character_whole` | Paste `"ACGT\nAC!T"`: nothing inserted; the message names `!`, line 2, column 3 |
| 8 | `paste_outside_insert_does_nothing` | In `Normal`, a paste changes nothing and sets a message |
| 9 | `insert_moves_features` | pBR322, `i` at zero-based 50, 3 letters: the tet CDS now starts at 1-based 89 (through 9e) |

## 6. Checks

`scripts/check`. Bracketed paste cannot be unit-tested through a terminal:
report that you checked the enable and disable calls by reading
`run` and its restore path, and that `cargo build` succeeds.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | `backspace` ignores `typed` | test 3 |
| 2 | The paste's `>`-line rule removed | test 6 |

## 8. Do not

- Cut, yank, paste from the register, undo (9g). Save (9h).
- Type on the bottom strand, or reverse-complement anything (M9-3).
- Leave bracketed paste on after the session ends on any path, a panic's
  restore included if `run` restores there today.
