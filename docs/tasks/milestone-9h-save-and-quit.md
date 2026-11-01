# Contract 9h — cli: saving the edited record, and quitting with edits unsaved

Planner: Claude. Status: implemented. Tier: light (plan M9-10).
Plan row: `docs/tasks/milestone-9-editor.md` §7, row 9h.

## 1. What this contract builds

This contract adds:
- `s` opens a "save as" prompt with a name filled in. `enter` writes the
  edited record: GenBank through 9c3's writer, FASTA through
  `render_fasta`.
- The opened file is never written. Any other existing file asks first.
- `q` and `ctrl-c` ask before throwing away edits that are not saved.

`state.rs` stays pure: it asks for a write, and `terminal.rs` does it.

Rules that apply, already settled:
- Plan §5 M9-7 (where and how a save writes), M9-9 (quitting), M9-11 and
  M9-15 (the stamp), M9-16 (the count).
- D003 — no file access in `state.rs`.
- `crates/cli/src/write.rs` header — `write_text` is the one writer, and
  its `replace` rule.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `write_text(path, text, replace)` | `crates/cli/src/write.rs:86` | Every write |
| `write_genbank`, `SaveStamp` | `crates/core/src/genbank_write.rs` (9c3) | GenBank text |
| `render_fasta` | `crates/core/src/fasta.rs:180` | FASTA text, with the buffer's sequence |
| The go-to prompt | `state.rs` `prompt`, `prompt_key` | The model for the save prompt: text typed, `enter`, `esc` |
| `opened: ViewRecord`, `edits()` | `state.rs` (9e, 9g) | What is written and the count |
| `terminal::run`, `main.rs:479-500` | — | The path reaches the view here |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs` — the save prompt, the
   questions, the dirty state, and unit tests.
2. `crates/cli/src/sequence_view/terminal.rs` — `run` takes the opened
   file's path, and performs the write the state asks for.
3. `crates/cli/src/main.rs` — pass the path to `run`; nothing else.
4. `crates/cli/src/sequence_view/save.rs` — new, if the worker wants it:
   the default name, the date and the text to write, as pure functions
   with unit tests.

## 4. Behaviour

1. **Default name.** `<stem>.edited.gb` for a GenBank record and
   `<stem>.edited.fa` for FASTA, in the opened file's directory. The stem
   is the opened file name without its last extension. The prompt shows
   the path as it will be written, relative as the user typed the opened
   path.
2. **`s`**, in `Normal` and editable, opens the prompt with the default
   name typed in. Typing edits the name, as go-to's prompt does, with
   `backspace`. `esc` closes it and writes nothing. `enter` asks for the
   write: `handle` returns a new `Action::Save`, and `terminal::run` calls
   `state.save_request()`, which gives `(PathBuf, String, bool)` — the
   path, the text, and whether to replace.
3. **Refusals before any write**, checked in the terminal, where files
   are:
   - An empty name refuses.
   - A name that is the opened file refuses: compare `fs::canonicalize`
     of both when the target exists, else the paths as given. The message
     says luria never writes over the file it opened.
   - A name that exists and is another file asks: `<name> exists: y
     replaces it, any other key keeps it`. Only `y` writes, with
     `replace: true`.
   - Otherwise it writes with `replace: false`.
4. **The text.** GenBank: `write_genbank(buffer, opened.genbank, stamp)`.
   The stamp's date is today in UTC as `DD-MMM-YYYY`, its version
   `env!("CARGO_PKG_VERSION")`, and its edits `edits()`. FASTA:
   `render_fasta` of the buffer's name and `to_sequence()`.
5. **After a write**: the message says `saved <path>` and the view is
   clean. A failed write's message is the error, and the view stays
   dirty.
6. **Dirty.** Every edit, undo and redo increments a change counter, and
   a save records it. The view is dirty when they differ. Undoing back to
   the saved state still counts as dirty; that is acceptable.
7. **`q`** in `Normal`, when dirty, asks: `edits not saved: y quits
   without saving, any other key stays`. `y` quits; any other key stays
   and clears the question. When clean, `q` quits as today.
8. **`ctrl-c`**, when dirty, asks the same question, wherever it is
   pressed. While the question is open, `y` or a second `ctrl-c` quits,
   and any other key stays. When clean, `ctrl-c` quits as today.
9. **Today's date.** A pure function from days since 1970-01-01 to
   `DD-MMM-YYYY` (civil-from-days; no new dependency), given
   `SystemTime::now()`'s days in the terminal.

## 5. Tests the worker writes

| # | Test | Asserts |
| --- | --- | --- |
| 1 | unit: `date_from_days` | 20755 → `29-OCT-2026`; 11016 → `29-FEB-2000`; 10956 → `31-DEC-1999`; 0 → `01-JAN-1970` |
| 2 | unit: `default_names` | `dir/pbr322.gb` → `dir/pbr322.edited.gb`; `x.fasta` → `x.edited.fa`; `a.b.gb` → `a.b.edited.gb` |
| 3 | unit: `s_opens_the_prompt_and_enter_asks_to_save` | `s` then `enter`: `Action::Save`, and `save_request()` gives the default path, `write_genbank`'s text and `replace` false |
| 4 | unit: `quit_asks_when_dirty` | After an `x`: `q` does not quit, `n` stays, `q` then `y` quits. When clean, `q` quits at once |
| 5 | unit: `ctrl_c_twice_quits` | Dirty: `ctrl-c` asks, a second `ctrl-c` quits |
| 6 | unit: `saved_is_clean` | After an edit and a reported successful save, `q` quits at once |
| 7 | `crates/cli/tests/` integration, new file `save_cli.rs`, only if it can run without a terminal: write through the terminal's save step (made a testable function) into a temp directory under `target/tmp`, then refuse the opened file and ask for an existing one | Writes the file; refuses the opened file; asks before replacing. If no part of this can run without a terminal, report that and leave the file out |

## 6. Checks

`scripts/check`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | The opened-file check removed | test 7 (or say why 7 could not be written) |
| 2 | `q` ignores dirty | test 4 |

## 8. Do not

- Write from `state.rs`, or anywhere but `write_text`.
- Overwrite the opened file in any path, a symlink to it included
  (`canonicalize`).
- Change `luria find`, `q5`, `review` or `new`.
