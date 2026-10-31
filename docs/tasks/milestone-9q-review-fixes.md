# Contract 9q — core, cli: the review's five correctness fixes

Planner: Claude. Status: implemented. Tier: full (the GenBank writer, M9-10).
Plan row: `docs/tasks/milestone-9-editor.md` §7, row 9q.

## 1. What this contract fixes

A code review of milestone 9 (2026-10-31) found five bugs. The planner
checked each one against the code, and the owner asked for them to be
fixed before the merge. Two of them share one cause, so there are four
fixes:
1. **A crash.** After `u` or `ctrl-r` in Visual mode, the selection can
   run past the record's end, and `x` or `y` then panics.
2. **A wrapped value read back wrong.** The reader joins a quoted
   value's lines with one space and trims each line
   (`crates/core/src/genbank.rs:461`, `:765`). So every break the writer
   makes inside a run with no space adds a space on re-read. That
   includes the case where a line holds only the closing `"`, and a
   break inside an escaped `""`, which makes the record refuse to open.
3. **A stale overtype entry.** In `R`, the old letter is remembered
   before the substitution, so a refused substitution leaves an entry
   behind.
4. **A refused write.** If the opened path is a symlink, `:w REAL` to the
   file it points at is refused, though REAL is a regular file.

## 2. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs`: rules 1 and 3, and tests 1, 2
   and 5.
2. `crates/core/src/genbank_write.rs`: `quoted_lines` and its doc
   comment, and tests 3 and 4. The existing test `qualifiers_wrap_at_79`
   may change only where §3.2 changes the rule (the "100-character word"
   case).
3. `crates/cli/src/sequence_view/terminal.rs`: `save_step`'s
   opened-file branch.
4. `crates/cli/tests/save_cli.rs`: test 6.

## 3. Rules

1. **Visual after undo and redo.** `undo_group` and `redo_group` end a
   Visual selection: `editing` becomes `Normal`, as `esc` makes it. Also,
   `selection()` clamps its end to the record's length and returns
   `None` when the record is empty, so no caller can read past the end.
2. **Quoted values break only at a safe space.** A line may break only
   at a space with a non-space character on each side. That space is
   dropped, and the reader puts it back. A run with no such space stays
   on one line, even when the line passes column 79. The closing `"`
   always ends the value's last line, after at least one character of
   the value, or straight after the opening `"` for an empty value. It
   is never alone on a line. Lines that can break keep the 79-column
   limit as today. Update the doc comment and `qualifiers_wrap_at_79`'s
   100-character case to this rule.
3. **Overtype memory.** In `overtype_key`, the `(base, letter)` pair is
   pushed only after `apply_edit` succeeds. Read the letter before the
   edit, push it after.
4. **Writing the opened file through another name.** In `save_step`,
   when the target is the opened file (same canonical path), the path
   written is the target if `symlink_metadata(target)` is a regular
   file, and otherwise the opened path. The link refusal and the backup
   then apply to the path written. The backup is `<written>~`, made once
   a session as today.

## 4. Tests

| # | Test | Asserts |
| --- | --- | --- |
| 1 | unit, `state.rs`: `undo_in_visual_ends_the_selection` | pBR322: `i`, `AAAAA`, `esc`, `End`, `v`, `u`. The mode is `Normal`, `selection()` is `None`, and `x` then cuts one base without a panic (length 4360) |
| 2 | unit: `selection_never_runs_past_the_end` | A view whose Visual anchor is set past the end directly (in the test): `selection()` ends at the record's length |
| 3 | unit, `genbank_write.rs`: `quoted_values_read_back_exactly` | For each value below, write one `misc_feature` with `/note="<value>"` through `write_genbank`, read the text back with `read_genbank`, and check that the note equals the value. Also check that no line is a lone `"`. Values: (a) a 120-character run of `a`; (b) a spaceless run that exactly fills the first line's room, then the same + 1, and the same − 1; (c) 70 `a`, then `""` (an escaped quote pair), then 70 `b`, with no spaces, placed so that a break at 79 would fall between the two quotes; (d) `alpha  beta` repeated to 200 characters (double spaces); (e) prose of single-spaced words, 200 characters; (f) the empty value |
| 4 | unit: `breakable_lines_stay_within_79` | Values (e) and (d): every written line is at most 79 characters |
| 5 | unit, `state.rs`: `a_refused_overtype_leaves_nothing_to_take_back` | Push nothing when `apply_edit` refuses. If no real refusal can be reached from a test, say so and test the order instead: after one successful `R` letter, `overtype_taken` has one entry, and its letter is the letter from before the edit |
| 6 | `save_cli.rs::the_target_of_an_opened_link_is_written` | `link.gb` → `real.gb`. `save_step(link, real, TEXT, false, &mut false)`: `real.gb` holds `TEXT`, `real.gb~` holds the original, and `link.gb` is still a symlink to `real.gb`. `save_step(link, link, ..)` still refuses, as test 13 of 9l says. `#[cfg(unix)]` |

Test 3's expectations are the input values themselves, not writer output.

## 5. Checks

`scripts/check`; `cargo test -p luria-core --test genbank_write_fixtures`
(the committed round-trip fixtures must pass unchanged).

## 6. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | `undo_group` keeps Visual | test 1 |
| 2 | `selection()` without the clamp | test 2 |
| 3 | `quoted_lines` breaks a spaceless run at 79 again | test 3 (a), (c) |
| 4 | `quoted_lines` may break at a space next to another space | test 3 (d) |
| 5 | `save_step` always writes the opened path | test 6 |

Disable each rule once, run the tests, restore it, check with `cmp`
against a backup in `target/tmp/`, and report what failed.

## 7. Do not

- Change the reader, the fixtures, or any test not named in §2.
- Change anything else the review listed (performance is 9r).
- Add dependencies.
