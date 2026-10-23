# Milestone 7m8b: the words preview on the variant screens

Status: **Ready for the worker, 2026-10-22.** Planner: Claude. Parent:
`docs/tasks/milestone-7m-polish.md`, M7-60 and M7-61. Builds on 7m8a
(committed `aede827`): `review::preview(document, request) -> Result<Approved, String>`.

## 1. The owner's answers

- M7-60: when the model's reading can be approved, `luria new` runs the
  design in memory and shows the variant screens as a preview, before
  anything is saved. A reading with questions or an error still opens
  on the proposal text.
- M7-61: the model's reading moves behind `d`. It is at the bottom of
  each variant's Details screen, as M7-52 put the provenance.

## 2. Changes

### 2.1 `crates/cli/src/review/app.rs`

- `App` gains `reading: Option<Vec<String>>`, `None` in `new`, set by
  `pub fn with_reading(self, lines: Vec<String>) -> App` and read by
  `pub fn reading(&self) -> Option<&[String]>`. An app with a reading is
  a preview.
- `Action` gains `Approve`.
- In `handle`, after the help and `?` handling, and only in a preview:
  - `a` on `Overview`, `Member(_)` or `Details(_)` returns
    `Action::Approve`.
  - `x` on `Overview` does nothing (`Continue`). A preview has no export.

  Without a reading, every key behaves exactly as now.

### 2.2 `crates/cli/src/review/text.rs`

- `pub fn preview_details(document: &ResultDocument, index: usize, reading: &[String]) -> Vec<String>`:
  `details(document, index)`, then one empty line, then
  `"the model's reading of your words:"`, then `reading`'s lines, each
  indented by two spaces (an empty line stays empty).
- `pub fn preview_help(screen: Screen, problem: bool, expandable: bool) -> Vec<String>`:
  `help(...)`'s lines, with `key("a", "save and run")` inserted just
  before the `q` line on `Overview`, `Member(_)` and `Details(_)`.
  `help` itself is unchanged.

### 2.3 `crates/cli/src/review/view.rs`

With `app.reading()` `Some`:

- The title is `state_badge(state)` then `" · preview, nothing is saved yet "`
  in place of `validation_words`. The state stays on screen (D020).
- `Details(i)` draws `text::preview_details(document, i, reading)`.
- Key lines: on `Overview` drop `  x export`, and on `Overview`,
  `Member(_)` and `Details(_)` add `  a save` just before `  ? help`.
- The help pop-up uses `text::preview_help`.

Without a reading, the drawing is byte for byte as now. `max_scroll`
counts the lines actually drawn, including the reading on Details.

### 2.4 `crates/cli/src/new/terminal.rs`

- After `interpret(...)` returns `document` and `request_path` is known:
  if `proposal::approvable(&document)` and
  `review::preview(&document, &request)` is `Ok(found)`, build
  `App::new(found.document).with_template(found.template).with_frames(found.frames).with_reading(text::proposal(&document))`
  and keep it as `preview: Option<App>`. Also keep the `ProposalApp`,
  because the save uses its document. Otherwise (`Err`, or not
  approvable) `preview` is `None`, and the proposal screen shows as now.
- Drawing: the preview, when present, is drawn with `view::draw` and
  `draw_popup` over it. Otherwise the proposal and the form are drawn as
  now.
- Keys, while `preview` is `Some`:
  - With the pop-up open, keys go to the form, exactly as the proposal
    path does now (`SaveWords`, replace, `save_failed`).
  - Otherwise `ctrl-c` ends the session. Anything else goes to
    `app.handle`, and after it `app.clamp_scroll(view::max_scroll(&app, area))`,
    as `review/terminal.rs` does:
    - `Action::Approve` calls `form.open_words_popup()`.
    - `Action::Quit` (`q`, or `esc` on the overview) sets both `preview`
      and `proposal` to `None`, back to the form with its values kept.
    - Anything else continues.
- A paste is ignored while the preview is shown, as on the proposal.

### 2.5 Docs

- `docs/cli.md` §12.3: replace

  ```
  The model's reading opens as §11.2's proposal: questions, the request as
  it will be written, and where each value came from. Nothing is written
  until you press `a` there, which opens the save pop-up below.
  ```

  with

  ```
  When the model's reading can be saved, luria designs it in memory and
  opens on §11.1's result screens as a preview: the title says `preview,
  nothing is saved yet`, and each variant screen shows the primers, both
  strands and the mutant protein in frame, so you can check each
  mutation. The model's reading (the request as it will be written, where
  each value came from, the quotes, and what was sent and received) is at
  the bottom of each variant's Details screen (`d`). `a` opens the save
  pop-up below; `q`, or `esc` on the overview, goes back to the form and
  writes nothing. A reading with questions or an error opens as §11.2's
  proposal instead, and cannot be saved.
  ```

- `docs/decisions.md` D047's Words bullet: replace

  ```
    needs a GenBank template (M6-7), and its proposal is shown before
    anything is saved. Without a backend the form names `--backend`. A line
  ```

  with

  ```
    needs a GenBank template (M6-7). A reading that can be saved is
    designed in memory and shown on the variant screens as a preview, with
    the reading itself at the bottom of Details; nothing is written before
    the save pop-up (M7-60, M7-61). This narrows D046's "nothing runs until
    you approve" for `luria new` only. Without a backend the form names
    `--backend`. A line
  ```

  and in D047's Status line, `answers M7-45 to M7-59` becomes
  `answers M7-45 to M7-61`.

Each old text occurs exactly once. If one does not, stop and report it.

## 3. Tests: `crates/cli/tests/review_preview.rs` (new)

Build the preview as 7m8a's `a_preview_is_the_saved_run` does: corpus case
`two-separate` through `interpret` with a scripted reply, then
`review::preview`. Copy the helpers you need from `review_approve.rs`.
Draw on a `TestBackend` of 120 by 40.

1. `a_preview_says_nothing_is_saved`: the title row holds `✓ verified`
   and `preview, nothing is saved yet`, and not `not tested at the bench`.
2. `a_saves_from_the_variant_screens`: with a reading, `a` gives
   `Action::Approve` on `Overview`, on `Member(0)` and on `Details(0)`.
   Without a reading, `a` on `Overview` gives `Continue`.
3. `a_preview_has_no_export`: with a reading, `x` leaves the screen on
   `Overview`, and the overview's key line holds `a save` and no
   `x export`.
4. `details_end_with_the_models_reading`: `text::preview_details` ends
   with the heading, then `text::proposal(&interpretation)`'s lines,
   indented by two spaces. Its start equals `text::details`.
5. `the_help_names_the_save_key`: `preview_help(Screen::Member(0), ...)`
   holds `  a          save and run`, and `help(...)` does not.

Existing tests must pass unchanged.

### 3.1 Rules that must be proven

1. Drop the `a` → `Approve` arm: test 2 fails.
2. Make `Details` in a preview draw plain `text::details`: test 4 still
   passes because it is text-level. So also add to test 4 a drawn check:
   open `Details(0)` in a preview, scroll to the bottom (`End` is not a
   key, so press `PgDn` until the scroll stops), and the last body row
   holds the reading's last line. With the rule dropped, this check
   fails.

## 4. Do not

- Change only `review/app.rs`, `review/text.rs`, `review/view.rs`,
  `new/terminal.rs`, the new test file, `docs/cli.md` and
  `docs/decisions.md` as in §2.5.
- No change to `luria review`'s behaviour (no reading is ever set
  there), to the shorthand path's save, to fixtures, Cargo files or
  `vendor/`.
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result and both §3.1 proofs.

## 6. Planner answers to w7m8b

- `Action::Approve` makes `review/terminal.rs`'s match in `run` non-exhaustive.
  Approved: add one arm `Action::Approve => {}` there, with a comment that
  `luria review` never sets a reading, so the arm is never reached. This is
  the only change allowed in `review/terminal.rs`; its behaviour is unchanged.
