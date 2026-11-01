# Contract 9l — cli: the `:` command line, `ZZ` and `ZQ`

Planner: Claude. Status: implemented. Tier: light.
Plan row: `docs/tasks/milestone-9-editor.md` §7, row 9l.

## 1. What this contract builds

Saving and quitting work as in Vim (plan M9-19, approved by the owner,
2026-10-31):
- `:` opens a command line on the status line;
- `ZZ` and `ZQ` work;
- `:w` writes the opened file itself, after keeping the original once a
  session as `<file>~`;
- `s` and its save prompt are removed;
- `q` refuses to quit while edits are unsaved.

The write stays where 9h put it: the state names the path and the text,
and the terminal writes.

Rules that apply, already settled:
- Plan §5 M9-19. It supersedes M9-7's "the opened file is never
  overwritten" and its y/n question, M9-9 for `q`, and 9h's `s` and
  default `.edited` name.
- C-1, in `crates/cli/src/write.rs`: nothing is written through a
  symlink. `write_text` keeps that rule, and this contract uses it
  unchanged.
- 9h §4.4 (the text) and §4.5 (after a write) are unchanged, except as
  §4.6 below says.

## 2. What exists

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `Action` | `crates/cli/src/sequence_view/state.rs:128` | Gains `SaveAndQuit` |
| `handle`: the quit question, `typing`, the `q` and `ctrl-c` arm | `state.rs:535-580` | `q` changes; `ctrl-c` stays |
| `s` and the save prompt (`save_prompt`, `save_prompt_key`, 9h2's status and key arms) | `state.rs:404`, `:802-822`, `:1595-1632`, `status_body`, `keys` | Removed; the command line takes their place |
| `save_request`, `report_save`, `set_message`, `dirty` | `state.rs:1639-1670` | Kept and changed as §4 says |
| `goto_base` | `state.rs:3106` | `:N` calls it |
| `blocked_words`, the "save needs the record to have been opened from a file" refusal | `s`'s arm | Reused by `:w` |
| `help_rows` | `state.rs:2115` | The `q  quit` row is replaced (§4.9) |
| `save::default_name` and its test | `crates/cli/src/sequence_view/save.rs:33`, `:174` | Removed: no default `.edited` name is left |
| `save_step`, the `Action::Save` branch of `run` | `crates/cli/src/sequence_view/terminal.rs:127-155`, `:241-274` | New signature (§4.7); the question closure goes |
| `write_text` | `crates/cli/src/write.rs:86` | Unchanged; every write and the backup go through it |
| `save_cli.rs` | `crates/cli/tests/save_cli.rs` | Its tests move to the new rules |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs`.
2. `crates/cli/src/sequence_view/terminal.rs`.
3. `crates/cli/src/sequence_view/save.rs`: only to remove `default_name`,
   its test, and the doc lines that name it.
4. `crates/cli/tests/save_cli.rs`.

Existing tests that may change, and only so far as §4 makes them:
- 9h's unit tests that press `s`, use `save_prompt()`, or expect `q` to
  ask;
- 9h2's four save-prompt tests, which are replaced by test 1;
- `save_cli.rs`, all of it.

Keep what each of these asserts wherever the meaning is unchanged: the
text, clean after a report, the empty-name refusal, and nothing written
through a link.

## 4. Behaviour

1. **Opening.** `:` in `Normal` opens the command line with nothing
   typed. This works in every view: editable or not, and opened from a
   file or not. Everywhere else `:` does what it does today: text in a
   list, a prompt or an insert mode, and nothing in `Visual`.
2. **Typing.** Printable keys add to the line, `q` included (add the line
   to `typing`). `backspace` drops the last character, and on an empty
   line closes it, as Vim does. `esc` closes it and runs nothing. `enter`
   closes it and runs the trimmed text.
3. **Drawn.** While the line is open, the status line is `:` followed by
   the typed text, exactly as typed. A message set while it is open shows
   instead, as in the other prompts. The key line is
   `type  backspace  enter run  esc close`. Put this `keys` arm above the
   comment "The key list has no keys to offer…", not between that comment
   and the `help` arm it describes.
4. **Commands.** A command is an exact word, an optional `!` straight
   after it, and for the write commands an optional name after
   whitespace. The name is the rest of the line, trimmed.

   | Typed | Does |
   | --- | --- |
   | (empty) | nothing |
   | digits only | `goto_base` of that 1-based number |
   | `q` | quits if clean. If dirty, stays, with the message `edits not saved: :w writes them, :q! discards them` |
   | `q!` | quits |
   | `w`, `w!`, `w NAME`, `w! NAME` | writes (§4.5) and stays |
   | `wq`, `wq!`, with or without a name | writes, then quits if the write succeeded |
   | `x`, `x!`, with or without a name | when dirty, as `wq`; when clean, quits without writing |

   Anything else, `q NAME` and `q!` with a name included, gives the
   message `not a command: :<trimmed text>`.
5. **What a write asks for.** It needs an editable record opened from a
   file. Otherwise the message is the same refusal `s` gave
   (`blocked_words`, or the "opened from a file" sentence), and nothing
   is asked of the terminal.
   - The target is NAME if one was given. Otherwise it is the session's
     last written name, and with none yet, the opened path.
   - `replace` is true in three cases: `!` was typed, the target equals
     the last written name, or the target equals the opened path as
     given. A file this session opened or wrote may be written again
     without `!`.
   - `handle` returns `Action::Save` for `w`, and the new
     `Action::SaveAndQuit` for `wq` and `x`. `save_request()` returns
     `(target, text, replace)`, so the state has to hold the request
     between `handle` and `save_request`.
6. **After a write.** `report_save` behaves as in 9h §4.5. On success it
   also stores the path as the last written name.
   - In `run`, for `Save` and `SaveAndQuit`: take `(path, text, replace)
     = state.save_request()`, then call `save_step` (§4.7) with a
     session-long `backed_up: bool` that starts `false`, then
     `report_save`.
   - When `save_step` made a backup, `run` then sets the message to
     `saved <path>, the original kept as <backup>`.
   - For `SaveAndQuit` with `Ok`, leave the loop as `Quit` does. On
     `Err`, stay, with the error as the message.
7. **`save_step`.** The new signature:

   ```rust
   pub fn save_step(opened: &Path, target: &Path, text: &str, replace: bool,
                    backed_up: &mut bool) -> Result<Option<PathBuf>, String>
   ```

   The `Option` is the backup made by this call. The function takes
   these steps in order:
   1. An empty target refuses, as today.
   2. **The opened file.** The target is the opened file when the two
      canonicalize to the same path, or, when they cannot, when they are
      equal as given.
      - `opened` itself must be a regular file (`symlink_metadata`).
        Otherwise it refuses with `<opened> is a link, not a file: :w
        NAME writes elsewhere`, and writes nothing.
      - If `*backed_up` is false, the original is read with
        `fs::read_to_string(opened)` and written to `<opened>~` (the
        opened path with `~` appended to its file name) with
        `write_text(.., true)`. A failure refuses with `<backup>:
        <error>; nothing written`. Then `*backed_up` becomes true.
      - Then `write_text(opened, text, true)`. The opened path is
        written, not a link that leads to it. The result is
        `Ok(Some(backup))` if this call made the backup, else `Ok(None)`.
   3. **Any other existing file**, with `replace` false, refuses with
      `<target> exists: :w! replaces it` and writes nothing. With
      `replace` true, it is written through `write_text(target, text,
      true)`.
   4. **A new name** is written with `replace: false`, as today.

   The terminal asks no question any more, so the question closure and
   `io_problem` go.
8. **`ZZ`, `ZQ` and `q`.** In `Normal`, `Z` waits for one more key:
   - `Z` acts as `:x`;
   - `Q` acts as `:q!`;
   - any other key cancels the wait and is dropped: it does nothing,
     not even what it does alone.

   `q` acts exactly as `:q`, and the quit question no longer opens for
   `q`. `ctrl-c` is unchanged: when dirty it asks, and `y` or a second
   `ctrl-c` quits. Below `MIN_WIDTH` by `MIN_HEIGHT`, `q` and `ctrl-c` are
   still the only keys, so dirty, only `ctrl-c` twice leaves.
9. **`s`** is unbound in `Normal`: it changes nothing. The `?` list's
   last row, `q  quit`, becomes
   `:w :q :wq :x  write, quit, both (! forces)  ZZ ZQ  q quits`. The list
   keeps its row count.

## 5. Tests the worker writes

Unit tests in `state.rs` use `new_for_file(pbr(), 80, 24,
Path::new(PBR_PATH), 20_755)` unless the row says otherwise. The state
writes no file, so nothing in them touches the fixture. "Dirty" means
one `x` pressed first. `:w⏎` means the keys `:`, `w`, `enter`.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `colon_opens_the_command_line` | `:` makes `status()` exactly `:` and `keys()` the line in §4.3. Then `w`: `:w`. Then `backspace`: `:`. Another `backspace` closes it, and the status starts `base 1`. `:` `w` `esc`: closed, and every key gave `Action::Continue` |
| 2 | `colon_w_writes_the_opened_file_then_the_last` | Dirty, `:w⏎` gives `Action::Save`, and `save_request()` is `(PBR_PATH, text, true)`, with the text 9h's test checks. `:w out.gb⏎` then `report_save(Ok)`: clean, message `saved out.gb`. Dirty again, `:w⏎`: `(out.gb, _, true)` |
| 3 | `a_name_and_a_bang` | `:w other.gb⏎`: path `other.gb`, `replace` false. `:w! other.gb⏎`: `replace` true |
| 4 | `q_refuses_while_dirty` | Clean: `q` gives `Quit`, and so does `:q⏎`. Dirty: `q` gives `Continue`, and the status is §4.4's message. The same for `:q⏎`. `:q!⏎` gives `Quit` |
| 5 | `wq_and_x` | Dirty: `:wq⏎` and `:x⏎` each give `SaveAndQuit`. Clean: `:x⏎` gives `Quit`, and `:wq⏎` gives `SaveAndQuit` |
| 6 | `zz_and_zq` | Dirty: `Z` `Z` gives `SaveAndQuit`, and `Z` `Q` gives `Quit`. Clean: `Z` `Z` gives `Quit`. `Z` then `j`: `Continue`, with the cursor where it was |
| 7 | `colon_number_goes_to_base` | `:123⏎`: the cursor is zero-based 122, and the status starts `base 123` |
| 8 | `unknown_commands_are_named` | `:foo⏎`: the message is `not a command: :foo`. `:q now⏎`: `not a command: :q now` |
| 9 | `s_is_unbound` | `s`: `Continue`, not dirty, and the status is the same as before the key |
| 10 | `a_write_needs_a_file` | `ViewState::new` (no file): `:w⏎` gives `Continue`, with the "opened from a file" message. `:q⏎` gives `Quit` |

`save_cli.rs`, each in its own temporary directory as today:

| # | Test | Asserts |
| --- | --- | --- |
| 11 | `the_opened_file_is_written_after_a_backup` | Opened file `a.gb` holding `ORIGINAL`. `save_step(a, a, TEXT, false, &mut false-flag)` gives `Ok(Some(a.gb~))`: `a.gb~` holds `ORIGINAL`, `a.gb` holds `TEXT`. A second call with `TEXT2` and the same flag gives `Ok(None)`: `a.gb` holds `TEXT2`, and `a.gb~` still holds `ORIGINAL` |
| 12 | `a_link_to_the_opened_file_writes_the_file` | `link` → `a.gb`. `save_step(a, link, TEXT, false, ..)`: `a.gb` holds `TEXT`, `link` is still a symlink to it, and `a.gb~` holds the original |
| 13 | `an_opened_link_is_not_written` | The opened path is a symlink to `real.gb`. `save_step(link, link, ..)`: `Err` naming the link; `real.gb` unchanged, and no `link~` exists |
| 14 | `a_backup_that_cannot_be_written_writes_nothing` | `a.gb~` is a directory. `save_step(a, a, ..)`: `Err`, and `a.gb` still holds the original |
| 15 | `an_existing_other_file_needs_replace` | With `replace` false: `Err` containing `:w! replaces it`, and the file is unchanged. With `true`: written, and no `~` file made |
| 16 | `a_new_name_is_written` and `an_empty_name_refuses` | As today, on the new signature |

The symlink tests stay `#[cfg(unix)]`, as 9h's are.

## 6. Checks

`scripts/check`; `cargo test -p luria-cli --test save_cli`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | `:q` and `q` skip the dirty check | test 4 |
| 2 | `replace` ignores the last written name | test 2 |
| 3 | `:x` writes when clean | test 5 |
| 4 | No backup is made | test 11 |
| 5 | The backup is made on every write, ignoring `backed_up` | test 11 |
| 6 | `save_step` writes an existing other file when `replace` is false | test 15 |

Disable each rule once, run the tests, restore it, and report what failed.

## 8. Do not

- Change `write.rs`, `screen.rs`, core, or any test not named in §3.
- Write through a symlink, or around `write_text`. That includes
  `fs::copy` for the backup, which follows a link at its target.
- Add commands beyond §4.4: no `:e`, no `:wa`, no `:s`, no ranges.
- Add dependencies.

## 9. Report

Also report:
- anything in §4 you could not make work as written;
- the hand checks the owner should do in a terminal: `:w` twice on a
  copy of a file (one `~`, still the original), `:w!` onto an existing
  file, and `:wq` on a write that fails.
