# Milestone 7k10: Ctrl-C quits; an overlap too large to read is a blocker

Status: **Implemented, 2026-10-22.** Planner: Claude. Owner
answers M7-43 and M7-44 (2026-10-22), from the planner's code review.

## 1. What and why

1. **M7-43: Ctrl-C quits, writing nothing,** on every screen of `luria
   new` and `luria review`. Today the terminal loops ignore modifiers, so
   Ctrl-C reaches the screens as `c`, and in a text field it types one.
   Quitting writes nothing, as `q` and `Esc` already do: before `y` no
   file exists, and after an approval the files were written by that
   approval, not by quitting.
2. **M7-44: an overlap too large to read is a blocker.** `Overlap`
   accepts digits only but any number of them. With words, 7k9's
   `words_profile` panics on a value over `i64::MAX`. With lines, the form
   sends `parse::<u64>().ok()`, so a value over `u64::MAX` is refused as
   "empty", and a value between `i64::MAX` and `u64::MAX` becomes a
   negative overlap through `nt as i64` in `new::build`.

## 2. Changes

### 2.1 `crates/cli/src/review/app.rs`

`Key` gains `CtrlC`. `App::handle` returns `Action::Quit` for it first,
on every screen, help shown or not.

### 2.2 `crates/cli/src/review/proposal.rs`, `crates/cli/src/new/confirm.rs`, `crates/cli/src/new/form.rs`

Each `handle` returns its quit action for `Key::CtrlC` first, on every
screen: `ProposalAction::Quit`, `ConfirmAction::Quit` and
`FormAction::Quit`. On the confirm screens this differs from `Esc`, which
goes back: Ctrl-C quits.

In `crates/cli/src/new/terminal.rs`, Ctrl-C from the proposal screen
quits the session (`break`), as Ctrl-C from the confirmation does. It
does not go back to the form the way `q` does. So the
`ProposalAction::Quit` arm must know which key it was. Check
`pressed == Key::CtrlC` there, and `break` when it is; `q` and `Esc`
still go back to the form.

### 2.3 `crates/cli/src/review/terminal.rs` and `crates/cli/src/new/terminal.rs`

`key_of` takes the whole `KeyEvent`: `KeyCode::Char('c')` with
`KeyModifiers::CONTROL` is `Key::CtrlC`; everything else is mapped as
now, from `key.code`.

### 2.4 `crates/cli/src/new/form.rs`, `form_blocker`

After the empty-overlap blocker, an overlap that does not parse as
`i64` is a blocker on `Overlap` with the message `the minimum annealing
overlap is too large; write a length in nucleotides`. Because the blocker
comes first, `words_profile`'s `expect` and `build_form`'s overlap both
read a value that fits.

### 2.5 Docs (authorized here)

- `docs/cli.md` §12.1, after `` and `esc` quits. ``: add `` `ctrl-c` quits
  from any screen, writing nothing. ``
- `docs/cli.md` §11: replace `` `q` quits from anywhere; `` with
  `` `q` or `ctrl-c` quits from anywhere; ``.

Each anchor occurs once; if one does not, stop and ask.

## 3. Tests

In `crates/cli/tests/new_form.rs`:

1. `ctrl_c_quits_from_a_text_field`: focus on `Mutations`, then
   `Key::CtrlC` gives `FormAction::Quit`, and the field is unchanged.
2. `an_overlap_too_large_is_a_blocker`: the sid-1 form with Overlap
   `99999999999999999999` (20 digits). `[ Review ]` gives `Continue`, and
   the blocker is on `Overlap` with the §2.4 message. Then the same with
   words, using `new_words.rs`'s GenBank form: the blocker is the same,
   with no panic.

In the unit tests of `review/app.rs`, `review/proposal.rs` and
`new/confirm.rs`, one test each:

3. Ctrl-C quits from each screen the app has, including the confirm
   screen, where `Esc` does not quit.

### 3.1 Rules that must be proven

1. The too-large blocker (remove it): test 2 fails. For the lines form it
   fails as the wrong message; for the words form it panics.
2. Ctrl-C on the form (remove the form's check): test 1 fails, because
   `c` would be typed.

## 4. Do not

- Change only the files named in §2 and §3 and the §2.5 doc lines.
- No change to `new::build`, `interpret`, luria-core, luria-nl, fixtures,
  Cargo files or `vendor/`. Change no existing assertion.
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the §3.1 proofs, and anything that did not match.
