# Contract 9n — core: a save replaces luria's own earlier stamp

Planner: Claude. Status: implemented. Tier: full (the GenBank writer, M9-10).
Plan row: [docs/tasks/contracts/milestone-9-editor.md](/tasks/contracts/milestone-9-editor.md) §7, row 9n.

## 1. What this contract fixes

Each GenBank save adds the line `Edited with luria <version>: N edits.`
(M9-11). When a file luria saved is opened and saved again, the old line
stays and a second one is added. The owner's hand test left two such
lines in one file. The owner's answer (M9-20, 2026-10-31): **one stamp
per file**. If the last line of the last COMMENT block is already a luria
stamp, the save replaces that line instead of adding one.

## 2. What exists

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `comment`, `header_lines`, `last_of_comment_block` | `crates/core/src/genbank_write.rs:85`, `:148`, `:176` | The change is in `header_lines` |
| `comment_placement` (unit test) | `genbank_write.rs:515` | Tests 1–3 sit beside it; its `record(extra)` helper builds headers |
| `an_unedited_save_is_the_file`, `saved`, `opened` | `crates/core/tests/genbank_write_fixtures.rs:144`, `:64`, `:51` | Test 4 uses the same helpers |
| `tests/fixtures/editor/pbr322-saved.gb` | made by `saved.py` from `J01749.1.gb` without luria (9c3 §5) | Test 4's input and its expectation |

## 3. Files the worker may touch

1. `crates/core/src/genbank_write.rs`: `header_lines`, a new private
   function that recognises a stamp, its doc comments, and unit tests
   1–3.
2. `crates/core/tests/genbank_write_fixtures.rs`: test 4 only.

## 4. Behaviour

1. **A stamp line** is one of these:
   - `COMMENT` followed by 5 spaces, then the stamp text;
   - 12 spaces, then the stamp text.

   The stamp text is exactly `Edited with luria `, then a version (one
   or more characters, none of them a space or `:`), then `: `, then one
   or more ASCII digits, then ` edit.` or ` edits.`. The line ends
   there, with nothing after it.
2. **In `header_lines`**, after VERSION is dropped:
   - if the last COMMENT block's last line is a stamp line, it is
     replaced by the new stamp. A `COMMENT     ` lead stays
     `COMMENT     `, and a 12-space lead stays 12 spaces;
   - otherwise the save does as today: it adds the line after the
     block, or adds a new COMMENT section when there is none.
3. Only that one line is ever replaced. A stamp line elsewhere (not the
   last line of the last block) is left as written.

## 5. Tests

| # | Test | Asserts |
| --- | --- | --- |
| 1 | unit: `an_earlier_stamp_is_replaced` | A header whose COMMENT block is `COMMENT     first` then `            Edited with luria 0.1.0: 3 edits.`. A save with version `9.9.9`, 1 edit: the block is `COMMENT     first` then `            Edited with luria 9.9.9: 1 edit.`, and the header has the same number of lines as the input less VERSION |
| 2 | unit: `a_stamp_section_is_replaced` | A header whose only COMMENT is `COMMENT     Edited with luria 0.1.0: 0 edits.`: it becomes `COMMENT     Edited with luria 9.9.9: 1 edit.`, and no second COMMENT is added |
| 3 | unit: `near_misses_are_kept` | Three headers, each saved once. (a) The stamp is the block's first line and `            more text` follows it. (b) The last line is `            Edited with luria: 3 edits.` (no version). (c) The last line is `            Edited with luria 0.1.0: 3 edits. and more`. In each case the line is kept and the new stamp is added after the block, as today |
| 4 | `genbank_write_fixtures.rs::a_saved_file_saves_to_itself` | Open `tests/fixtures/editor/pbr322-saved.gb` and save it unedited with the stamp `an_unedited_save_is_the_file` uses (date `29-OCT-2026`, version `9.9.9`, 0 edits): the text equals the fixture byte for byte. Report the first difference if it does not |

Test 4's expectation is the committed fixture, made by `saved.py`
without luria. If it fails for a reason other than the stamp, stop and
report it. Do not change the fixture.

## 6. Checks

`scripts/check`; `cargo test -p luria-core --test genbank_write_fixtures`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | `header_lines` never replaces (today's code) | tests 1, 2 and 4 |
| 2 | The stamp match ignores what follows `edits.` | test 3 (c) |
| 3 | Any stamp line in the block is replaced, not only the last | test 3 (a) |

Disable each rule once, run the tests, restore it, check with `cmp`
against a backup in `target/tmp/`, and report what failed.

## 8. Do not

- Change `comment`, the stamp's text, or any other part of the writer.
- Change any fixture or existing test.
- Add dependencies (no regex crate).
