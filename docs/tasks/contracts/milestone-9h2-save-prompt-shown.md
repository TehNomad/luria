---
type: Task Contract
title: "Contract 9h2 — cli: the save prompt is drawn"
description: "Contract 9h's s opens the save prompt (ViewState::save_prompt), but nothing draws it."
status: deprecated
tags: [milestone-9]
---
# Contract 9h2 — cli: the save prompt is drawn

Planner: Claude. Status: implemented. Tier: light.
Plan row: [docs/tasks/contracts/milestone-9-editor.md](/tasks/contracts/milestone-9-editor.md) §7, row 9h (a fix to 9h).

## 1. What this contract fixes

Contract 9h's `s` opens the save prompt (`ViewState::save_prompt`), but
nothing draws it. The status line and the key line still show what they
showed before, so the view looks frozen: every key goes into the hidden
name, and only `esc` or `enter` gets out. The owner found this in the
hand test (9i).

The fix: while the save prompt is open, the status line shows the name
being typed, and the key line shows the prompt's keys, just as go-to's
prompt does (8k2 §4 rule 2).

## 2. What exists

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `status_body`, its go-to and search arms | `crates/cli/src/sequence_view/state.rs:1947-1980` | Add a save-prompt arm in the same shape |
| `keys`, its go-to and search arms | `state.rs:2032-2060` | Add a save-prompt arm |
| `save_prompt`, `save_prompt_key`, `set_message` | `state.rs:404`, `:1595-1632` | Unchanged |
| The overwrite question | `crates/cli/src/sequence_view/terminal.rs:244-264` | Sets the message while the prompt is still open; it must be what the line shows |
| `s_opens_the_prompt_and_enter_asks_to_save` | `state.rs:4384` | Test 1 sits next to it |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs`, and nothing else: the two new
   arms, their doc comments, and the unit tests below.

## 4. Behaviour

1. **Status line.** In `status_body`, after the search arm and before the
   cursor text, while `save_prompt` is `Some(name)`:
   - if a message is set, the line is the message (this is how the
     overwrite question `<name> exists: y replaces it, any other key keeps
     it` reaches the screen);
   - otherwise the line is `save as <name>`, with exactly one space
     between them. With nothing typed, it is `save as` and nothing more.
2. **Key line.** In `keys`, while `save_prompt` is `Some`, the line is
   `type  backspace  enter save  esc close`. Put the arm after the search
   arm and before `help`'s.
3. Nothing else changes: no key does anything new, and the edit-mode
   markers (`-- INSERT --` …) stay as they are. `s` opens only in
   `Normal`.

## 5. Tests the worker writes (unit tests in `state.rs`)

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `the_save_prompt_is_on_the_status_line` | `new_for_file(pbr(), 80, 24, Path::new(PBR_PATH), 20_755)`, then `s`: `status()` is `save as ../../tests/fixtures/genbank/raw/J01749.1.edited.gb`, and `keys()` is `type  backspace  enter save  esc close`. Then `x`: status ends in `.edited.gbx`. Then `backspace`: the same as after `s` again |
| 2 | `the_overwrite_question_shows_over_the_prompt` | The same view, `s`, then `set_message("…exists: y replaces it, any other key keeps it".into())`: `status()` is exactly that message |
| 3 | `esc_takes_the_save_prompt_off_the_line` | The same view, `s`, `esc`: `status()` starts with `base 1`, and `keys()` is the Normal key line again |
| 4 | `an_emptied_name_says_save_as` | The same view, `s`, then `backspace` once for each character of the default name: `status()` is `save as` |

## 6. Checks

`scripts/check`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | Remove the new `status_body` arm | tests 1, 2 and 4 |
| 2 | Remove the new `keys` arm | test 1 |

Disable each rule once, run the tests, restore it, and report what failed.

## 8. Do not

- Change `screen.rs`, `terminal.rs`, `save.rs` or any existing test.
- Change what any key does.
- Add dependencies.
