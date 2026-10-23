# Milestone 7m9a: `[ Save and run ]` writes and runs at once

Status: **Implemented, 2026-10-23.** Planner: Claude. Parent:
`docs/tasks/milestone-7m-polish.md`, M7-62 to M7-65. Follows 7m8b
(committed `c1d650f`). 7m9b later removes 7m8a's in-memory run.

## 1. The owner's answers

- M7-62: a words reading with no questions is written and run at once,
  under the form's file name. The result screens are where the mutation
  is checked. If the file exists, a prompt names it and only `y` replaces
  it. A reading with questions or an error is shown and writes nothing.
- M7-63: the same for mutation lines. `[ Save and run ]` writes and runs
  at once. The name pop-up goes, and `Save as` is a form field again,
  after Overlap. The only pop-up left asks before replacing a file.
- M7-64: the model's reading is at the bottom of each variant's Details
  screen on the result screens.
- M7-65: the preview screens go: the preview title, the `a` key, and the
  preview help.

For reference, `Save as` as a field (before 7m4) is in
`git show 81c1ead:crates/cli/src/new/form.rs` and `.../new/text.rs`.
Read it there. Do not check it out.

## 2. Changes

### 2.1 The form, `crates/cli/src/new/form.rs`

- `Field::SaveAs` returns, between `Overlap` and `Review` in `Field` and
  `ORDER`. It is edited like the other one-line fields (characters,
  `Backspace`, `Delete`, `Left`, `Right`), and editing it sets
  `save_touched`. `sync_save_as` is unchanged.
- `Popup` becomes `{ Closed, Replace(Vec<String>) }`. Remove `Naming`,
  `popup_words`, `open_words_popup`, `FormAction::Back` and
  `save_as_cursor` (the cursor comes from the field, as for other
  fields). `popup_failure` goes too: failures are blockers now.
- `[ Save and run ]` (`Enter` on `Review`):
  1. The existing blockers run, in order, unchanged.
  2. Then `Save as`: empty gives the blocker
     `(Field::SaveAs, None, "write the request's file name")`. A `/` or
     `\` in it gives `(Field::SaveAs, None, "write a file name, not a path")`.
     This runs before words are sent, so a bad name never calls the model.
  3. Then, as now: words give `Interpret {..}`, and lines give
     `Save { built, replace: false }`, returned at once. Store the
     action to repeat in `pending`: the `Save` itself, or
     `SaveWords { replace: false }` for `Interpret`.
- `ask_replace(names)` opens `Replace(names)`. In it, `y` returns
  `pending`'s action with `replace: true` (`Save` or `SaveWords`). Any
  other key closes the pop-up and sets the blocker
  `(Field::SaveAs, None, "nothing was written")`. `ctrl-c` still quits
  first.
- `save_failed(message)` closes any pop-up and sets the blocker
  `(Field::SaveAs, None, message)`.

### 2.2 Text and drawing, `crates/cli/src/new/text.rs` and `crates/cli/src/new/view.rs`

- The `Save as` row (`row_line("Save as", ...)`) is drawn after the
  Overlap row and before the button, with the cursor at
  `LABEL + form.cursor()` as before 7m4.
- `popup_lines` has only the `Replace` case. Its words are unchanged.
- Remove the `Naming` drawing and its cursor.

### 2.3 The loop, `crates/cli/src/new/terminal.rs`

`run_new` gains a parameter `reading: &mut Option<Vec<String>>`, after
`written`.

- `FormAction::Save { built, replace }`: as now (plan, taken, ask or
  `approve_new`), with no pop-up before it.
- `FormAction::Interpret {..}`: draw the waiting line and call
  `interpret` as now. Then:
  - Not `proposal::approvable(&document)`: show the `ProposalApp` as
    now. `q`/`esc` go back to the form, and `ctrl-c` ends the session.
    There is no `a`: the loop no longer opens a save from the proposal.
  - Approvable: keep `(document, template)` as `words`, and save:
    `request = request_path(&form, cwd)`. If
    `fs::symlink_metadata(&request).is_ok()`, call
    `form.ask_replace(vec![name_of(&request)])`. Otherwise call
    `approve_replacing(&document, &template, &request, false)`.
- `FormAction::SaveWords { replace: true }`: `approve_replacing` with
  `words`'s document and template.
- An approval's `Ok(found)` sets `*written = Some(request)`,
  `*reading = Some(text::proposal(&document))` (words only), and ends
  the session with `found`. Its `Err(message)` calls
  `form.save_failed(message)`.
- Remove the preview: the `preview` variable, `review::preview` and the
  drawing of `view::draw` in this loop.

### 2.4 The result screens, `crates/cli/src/review/app.rs`, `view.rs`, `text.rs`, `terminal.rs`

`reading` stays, meaning "the model's reading, shown at the bottom of
Details". Everything else about previews goes:

- `app.rs`: remove `Action::Approve`, and the `a` and `x` preview arms
  in `handle`. With a reading, every key behaves as without one.
- `view.rs`: the title is always `state_badge` + `validation_words`. The
  key lines and help are always the non-preview ones. `Details(i)` still
  draws `text::preview_details` when a reading is set. Rename it
  `details_with_reading`.
- `text.rs`: remove `preview_help`.
- `review/terminal.rs`: remove the `Action::Approve => {}` arm and its
  comment. That file is then as before 7m8b.

### 2.5 `crates/cli/src/main.rs`

`emit_new` passes `&mut reading` to `run_new`. If `reading` is `Some`,
the result `App` gets `.with_reading(lines)`.

### 2.6 Docs

Each old text below occurs exactly once. If one does not, stop and
report it.

1. `docs/cli.md` §12.1, after the Overlap row, add the row

   ```
   | Save as | The request file's name: `<name>.request.json`, or `<LOCUS>.request.json` for GenBank, until you change it. |
   ```

2. `docs/cli.md` §12.3: replace the whole paragraph that begins
   `When the model's reading can be saved, luria designs it in memory and`
   and ends `proposal instead, and cannot be saved.` with

   ```
   When the model's reading has no questions, luria writes the request and
   runs it at once, as §12.4 says, and the result opens on §11.1's
   screens: each variant shows the primers, both strands and the mutant
   protein in frame, so check there that the mutation is the one you
   meant. The model's reading (the request as written, where each value
   came from, the quotes, and what was sent and received) is at the
   bottom of each variant's Details screen (`d`). A reading with questions
   or an error opens as §11.2's proposal instead and writes nothing; `q`
   or `esc` goes back to the form.
   ```

3. `docs/cli.md` §12.4: replace the paragraph from
   `` `[ Save and run ]` opens a pop-up with the request's file name, which you ``
   to `pop-up and writes nothing.` with

   ```
   `[ Save and run ]` writes the request under the `Save as` name: beside
   a template file, as `--write-request` requires (§10), or, for a paste,
   in the current directory, with the pasted FASTA. It then runs the
   request as `luria q5 design` does, and the result opens as §11.1 does,
   with export when `--export DIR` was given (§11.3). With words, this
   happens once the model has answered.
   ```

4. `docs/decisions.md` D047: replace the `luria new` bullet, from
   `` * **`luria new`** (M7-31, M7-48). `` to
   `writes nothing and runs nothing.`, with

   ```
   * **`luria new`** (M7-31, M7-62, M7-63). A form builds a
     `luria.q5-request/1` request from a template, mutation lines and the
     overlap. `[ Save and run ]` writes it under the form's `Save as` name
     and runs it, then shows it on `luria review`'s screens, where the
     user checks the mutations. A file that exists is replaced only on
     `y`. Quitting before saving writes nothing and runs nothing.
   ```

5. `docs/decisions.md` D047's Words bullet: replace

   ```
     needs a GenBank template (M6-7). A reading that can be saved is
     designed in memory and shown on the variant screens as a preview, with
     the reading itself at the bottom of Details; nothing is written before
     the save pop-up (M7-60, M7-61). This narrows D046's "nothing runs until
     you approve" for `luria new` only. Without a backend the form names
   ```

   with

   ```
     needs a GenBank template (M6-7). A reading with no questions is
     written and run at once, and the reading itself is at the bottom of
     each variant's Details (M7-62, M7-64); for `luria new` only, this
     replaces D046's rule that nothing is written until the user approves,
     so the result screens are where the reading is checked. A reading
     with questions writes nothing. Without a backend the form names
   ```

6. D047's Status: `answers M7-45 to M7-61` becomes
   `answers M7-45 to M7-65`.

## 3. Tests

New tests:

1. `new_form.rs`, `save_and_run_saves_at_once`: the sid-1 FASTA form,
   complete. `Enter` on `[ Save and run ]` returns `Save { replace: false, .. }`
   on the first press, and `popup()` is `Closed`.
2. `new_form.rs`, `save_as_is_a_field_after_overlap`: `Tab` from
   `Overlap` lands on `SaveAs`, then `Review`. The field starts as
   `<name>.request.json`. Typing in it changes `save_as()`, and a later
   change to Name no longer changes it.
3. `new_form.rs`, `a_bad_save_as_is_a_blocker`: empty, and `a/b.json`,
   each give their blocker on `SaveAs` and no `Save`.
4. `new_form.rs`, `a_taken_name_asks_and_only_y_replaces` (rewrite the
   existing test to the new rule): after `Save`, `ask_replace(names)`
   opens `Replace`. `y` gives `Save { replace: true, .. }`. In a fresh
   form, `n` closes it, gives the blocker `nothing was written`, and
   returns no `Save`.
5. `new_words.rs`, `words_then_y_replace`: a words form. `Enter` gives
   `Interpret`. Then `ask_replace(vec!["x.json".into()])` and `y` give
   `SaveWords { replace: true }`.
6. `new_words.rs`, `a_bad_save_as_never_calls_the_model`: a words form
   with `Save as` emptied. `Enter` gives the blocker and no `Interpret`.
7. `review_preview.rs`: delete tests 1, 2, 3 and 5 (their rules are
   gone). Keep `details_end_with_the_models_reading`, using
   `details_with_reading`, and add `a_reading_changes_only_details`:
   with a reading, the title holds `not tested at the bench`, `a` gives
   `Continue` on `Overview`, `x` opens `Export`, and the overview's key
   line holds `x export` and no `a save`.

Approved changes to existing tests: tests whose assertions are about
the `Naming` pop-up are rewritten to the rules above or deleted when
they have no counterpart (`save_opens_the_popup_with_the_suggested_name`,
`the_popups_name_is_edited_and_checked`, `esc_closes_the_popup_and_keeps_the_form`,
`the_popup_says_what_it_writes`, `a_proposal_replaces_only_when_asked`).
Tests that pressed `Enter` on the pop-up to reach `Save` now reach it
on the first `Enter`; only those key presses change. The expected form
lines in `new_text.rs` gain the `Save as` row. Report each test you
changed or deleted, and why. If a change would alter any other
assertion, stop and report it.

### 3.1 Rules that must be proven

1. Make the first `Enter` open nothing and return `Continue`: test 1
   fails.
2. Let any key on `Replace` replace: test 4 fails.

## 4. Do not

- Change only the files named in §2 and the tests named in §3.
- No change to `luria review`'s behaviour, `interpret`, fixtures, Cargo
  files or `vendor/`. Leave 7m8a's `preview`, `prepare_bytes` and
  `run_bytes_with_constructs` in place; 7m9b removes them.
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, both proofs, and every test changed or deleted.

## 6. Planner answers to w7m9a (2026-10-22)

w7m9a stopped mid-§2.1 (context); `form.rs` is partly changed and does not
compile. Its four questions:

1. `new/view.rs` `LABELS`: add `"Save as"` after `"Overlap"` (it was there
   at `81c1ead`), so the row's label is styled as the others are.
2. `the_popup_says_what_it_writes`: do not delete it whole. Rewrite it to
   the `Replace` pop-up only: after `ask_replace(vec!["sid-1.request.json"])`,
   `popup_lines` equals the lines `Replace` draws today (read them from
   `popup_lines`'s `Replace` arm, unchanged). Keep any part that already
   tests `Replace`. Delete only the `Naming` parts.
3. `new_words.rs::a_proposal_replaces_only_when_asked`: **keep it.** Its
   assertions are about `approve_replacing`, which is unchanged; change only
   a key press that reached the removed `Naming` pop-up, if any.
4. The paste's FASTA name: keep it on the form, as at `81c1ead`. Under the
   `Save as` row, for a paste, a continued row
   `template saved as <paste_file>`. Only the row itself, no blank row above
   the button (the 7m3 layout has none). The sid-1 expected lines in
   `new_text.rs` gain only `Save as`; a paste form's expected lines, if a
   test pins them, gain both rows.

## 7. Implementation notes (planner, 2026-10-23)

- Three workers: w7m9a and w7m9b ran out of context; w7m9c finished.
- 1150 tests passed, 0 failed, 1 ignored; fmt, clippy, diff check clean;
  fixtures 19 of 19. Both §3.1 proofs failed as required.
- Approved beyond §3: `new_form.rs::focus_order_is_screen_order` expects
  `[Overlap, SaveAs, Review]` (§3 test 2's rule).
- `review/terminal.rs` is byte-identical to its state before 7m8b.
- Untested: the terminal loop's save at once (the owner's hand test).
