# Milestone 7m4: one pop-up to save and run, asking before it replaces

Status: **Implemented, 2026-10-22.** Planner: Claude. Parent:
`docs/tasks/contracts/milestone-7m-polish.md`, M7-48 and the `luria new` half of
M7-49. Builds on 7m3 (committed `81c1ead`).

## 1. The owner's answers this contract applies

- M7-48: a pop-up on the form asks for the request's file name and shows
  the pasted FASTA's name. `enter` writes and runs, and `esc` closes it.
  The confirmation screens go. With words, the proposal screen stays, so
  the model's reading and questions are seen. Its approval opens the same
  pop-up.
- M7-49 (`luria new`): when a file it would write exists, a prompt names
  it, and `y` replaces it; any other key writes nothing.
  `luria q5 interpret --write-request` and `luria review --template` still
  never replace a file.

## 2. Changes

### 2.1 The form (`crates/cli/src/new/form.rs`)

- `Field::SaveAs` goes from `Field`, `ORDER` and focus. The form keeps
  `save_as` (still filled from the name or LOCUS, and never replaced once
  the user edits it) as the pop-up's name field.
- `pub enum Popup { Closed, Naming, Replace(Vec<String>) }`, with
  `popup()`, `popup_failure() -> Option<String>`, and a private
  `popup_words: bool` (true when the pop-up is saving a proposal).
- `[ Review ]` is now labelled `[ Save and run ]` (the field keeps its
  name `Review`). When the form builds, it opens `Naming` and returns
  `Continue`. Words still return `Interpret` (unchanged); no pop-up
  opens before the model has been asked.
- `pub fn open_words_popup(&mut self)` opens `Naming` with
  `popup_words = true`. The loop calls it when the proposal is approved
  (§2.3).
- While the pop-up is open, `handle` sends every key to it (`ctrl-c`
  still quits first):
  - `Naming`: characters, `Backspace`, `Delete`, `Left`, `Right` edit
    `save_as`, which sets `save_touched`. `Esc` closes the pop-up, and
    with `popup_words` it returns `FormAction::Back`, so the loop shows
    the proposal again. `Enter`: an empty name is the failure `write the
    request's file name`, and a name with `/` or `\` is `write a file
    name, not a path`. Otherwise it returns `FormAction::Save { built,
    replace: false }`, building again exactly as `[ Save and run ]` did,
    or with `popup_words`, `FormAction::SaveWords { replace: false }`.
  - `Replace(names)`: `y` returns the same action with `replace: true`.
    Any other key goes back to `Naming` with the failure `nothing was
    written`.
- `pub fn ask_replace(&mut self, names: Vec<String>)` opens
  `Replace(names)`. `pub fn save_failed(&mut self, message: String)` goes
  back to `Naming` with that failure.
- `FormAction::Review(Box<Built>)` becomes `Save { built: Box<Built>,
  replace: bool }`. Add `SaveWords { replace: bool }` and `Back`.

### 2.2 Writing (`crates/cli/src/new/confirm.rs`, `crates/cli/src/review.rs`)

- Delete `ConfirmApp`, `ConfirmScreen`, `ConfirmAction`, `confirm_lines`
  and `draw_confirm`. Keep `Planned` and `plan`.
- `pub fn taken(planned: &Planned) -> Vec<String>`: the file names of the
  paste and the request that exist (by `symlink_metadata`), paste first.
- `approve_new(planned, replace: bool)`. The location check is
  `request_location_problem` as now, except that with `replace` its
  `output_exists` refusal is ignored. The paste and the request are
  written with `crate::write::write_text(path, text, replace)` (7m2's
  writer: with `replace` it refuses a non-regular file and replaces by
  rename; without it, it creates a new file only). Messages are as now.
- `review.rs`: `pub fn approve_replacing(document, template, request,
  replace: bool) -> Result<Approved, String>`, which is `approve` with the
  same rule for `output_exists` and `write_text(…, replace)`. `approve`
  becomes `approve_replacing(…, false)`, so its behaviour is unchanged.
  Nothing else in `review.rs` changes.

### 2.3 The loop (`crates/cli/src/new/terminal.rs`)

- `FormAction::Save { built, replace }`: `planned = plan(*built, …,
  &form.save_as())`. Without `replace`, when `taken(&planned)` is not
  empty, call `form.ask_replace(taken)` and continue. Otherwise
  `approve_new(&planned, replace)`: `Ok` ends the loop as now, and `Err`
  goes to `form.save_failed`.
- Words: on the proposal's reading screen, when `proposal::approvable`
  holds, `a` calls `form.open_words_popup()` instead of reaching the
  proposal app. The proposal's own `a`/`y` confirmation is not used in
  `luria new`. While the pop-up is open, keys go to the form, and the
  proposal is drawn with the pop-up over it. `SaveWords { replace }`: the
  request path is `request_path(&form, cwd)`. Without `replace`, when it
  exists, call `ask_replace([its name])`. Otherwise call
  `review::approve_replacing(document, template, request, replace)`.
  `Back` closes the pop-up over the proposal.

### 2.4 Text and drawing (`crates/cli/src/new/text.rs`, `crates/cli/src/new/view.rs`)

- `form_lines`: no `Save as` row, and the button reads `[ Save and run ]`.
  The paste's `template saved as <name>.fa` row moves into the pop-up.
  `cursor` is `None` while the pop-up is open.
- `pub fn popup_lines(form: &NewForm) -> Vec<String>`:
  - `Naming`: `Request    <save_as>`, then `           beside <template
    file name>` for a file or `           in the current directory` for a
    paste; for a paste, `Template   <name>.fa, from the paste`; then the
    failure, when there is one, as `✗ <failure>`; then an empty line, and
    `enter save and run  esc back`.
  - `Replace(names)`: `These files exist:`, one line per name (two spaces,
    the name), an empty line, and `y replace them  any other key: back`.
- `pub fn draw_popup(frame: &mut Frame, form: &NewForm)`: when the pop-up
  is open, `Clear` and a rounded block titled ` Save `, centred, as wide
  as the longest line plus 4 and at most the screen, with the cursor in
  the name on `Request`'s row while `Naming`. The `✗` line is red bold.
  `draw_form` calls it last, and the loop calls it after `draw_proposal`
  in the words case.
- The form's key line: `enter save` replaces `enter review`.

## 3. Tests

Approved changes to existing assertions, each mechanical, from §2.1:

- every test asserting `FormAction::Review(built)` presses `Enter` once
  more (the pop-up's) and matches `FormAction::Save { built, replace:
  false }`;
- tests that tab to `Field::SaveAs` set the name through the pop-up
  instead;
- `new_text.rs`'s sid-1 form loses its `Save as` row and the empty line
  before the button, and the button text changes;
- `new_confirm.rs` tests of `ConfirmApp` and its screens
  (`the_confirmation_shows_the_request_as_written`, `quitting_writes_nothing`,
  `esc_returns_to_the_form`) are deleted; tests of `approve_new` gain
  `false`.

Report each changed test by name. Fixtures do not change.

New tests:

1. `new_form.rs`, `save_opens_the_popup_with_the_suggested_name`: the
   sid-1 form, `[ Save and run ]`: `Naming`, and `save_as` is
   `sid-1.request.json`.
2. `new_form.rs`, `the_popups_name_is_edited_and_checked`: `Backspace`
   until empty, then `Enter` gives the empty-name failure; `a/b.json`
   gives the path failure; `x.json` gives `Save` and `save_as` `x.json`.
3. `new_form.rs`, `esc_closes_the_popup_and_keeps_the_form`.
4. `new_form.rs`, `a_taken_name_asks_and_only_y_replaces`.
5. `new_confirm.rs`, `approve_new_replaces_only_when_asked`: the request
   file holds `old\n`. With `false`: `Err` naming it, and the file is
   unchanged. With `true`: `Ok`, and the file is the request text.
6. `new_confirm.rs`, `replace_never_writes_through_a_link` (unix): the
   request path is a symlink to a file outside the directory. With
   `true`: `Err`, and the target is unchanged.
7. `new_text.rs`, `the_popup_says_what_it_writes`: `popup_lines` for the
   sid-1 file form and for the syn paste form, then for `Replace`.
8. `new_words.rs`, `a_proposal_replaces_only_when_asked`: case
   `two-separate`. `review::approve_replacing` over an existing request
   file refuses with `false`, and with `true` writes
   `request/variants-separate.json`'s request.

### 3.1 Rules that must be proven

1. Replace only on `y` (make `approve_new` replace always): test 5 fails.
2. No write through a link (write the request with `fs::write` when
   `replace`): test 6 fails.

## 4. Do not

- Change only `new/form.rs`, `new/text.rs`, `new/view.rs`,
  `new/terminal.rs`, `new/confirm.rs`, `review.rs` (§2.2 only),
  `main.rs` (only if a removed name forces it), and the tests named in
  §3.
- No change to `interpret`, the proposal app, `new.rs`, `write.rs`,
  fixtures, docs, Cargo files or `vendor/`.
- Do not stage or commit. This is a large contract: if your context runs
  low, stop at a step boundary and report what is done and what is left.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the §3.1 proofs, and every existing assertion
changed.

## 6. Planner answers to w7m4 (2026-10-22)

The first worker, w7m4, wrote §2.1, §2.2 and §2.4 and ran out of context
before §2.3. Its choices are accepted:

- `form_blocker`'s empty-`SaveAs` blocker goes with `Field::SaveAs`; the
  pop-up's `write the request's file name` failure replaces it.
- `NewForm::save_as_cursor()` is added for §2.4's pop-up cursor.
- `Replace` shows no failure line; `ask_replace` and `open_words_popup`
  clear the failure.
- `new_view.rs` tests that read the key line change only if they assert
  `enter review`, which becomes `enter save` (approved).

## 7. Implementation notes (planner, 2026-10-22)

- 1133 tests passed, 0 failed, 1 ignored; fmt, clippy, diff check clean;
  fixtures 19 of 19. Both §3.1 proofs failed as required. w7m4 wrote §2.1,
  §2.2 and §2.4; w7m4b wrote §2.3 and the tests.
- On the proposal, `a` opens the pop-up whenever the proposal is
  approvable, not only on its reading screen; the proposal's own confirm
  screen is never reached in `luria new`, so the two are the same.
- The planner fixed two comments naming the removed `Save as` row
  (`main.rs`, `new/terminal.rs`).
- Untested: the terminal loop (pop-up over the form and over the
  proposal, `y` to replace, `esc` back).
