# Contract 8n — docs: `luria FILE` and `luria find` in the docs, D058, D059

Planner: Claude. Status: **Implemented 2026-10-28** by worker `w-8n` in
one round, reviewed and accepted: every old text matched once (the
`--backend` line only within §1, as §4.1 says), G1 and G2 each failed
their test, and the planner's `scripts/check` passed. Tier: neither (plan §7: 8n is a
documentation contract). Plan row: `docs/tasks/contracts/milestone-8-view.md` §7, 8n.

## 1. What this contract builds

Milestone 8's user documentation: `docs/specs/cli.md` gains §13 and the two
commands in §1, the README and the product brief say what milestone 8
built and what milestone 9 is, `luria help` names the two commands, and
`docs/decisions/` records D058 and D059. One new test pins §13's command
lines and its two `luria find` examples to the program.

The texts below are the planner's, checked against the program on
2026-10-28. Paste them verbatim; do not reword. Every old text must occur
exactly once; if one does not, stop and report it.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| Page helpers | `crates/cli/tests/documented_new.rs:13-40` | Model for `section_13` and `fenced_after` |
| Usage-line test | `crates/cli/tests/documented_new.rs:119` | Model for test 1 |
| Running the binary | `crates/cli/tests/cli.rs:12` | `Command::new(env!("CARGO_BIN_EXE_luria"))` |

## 3. Files the worker may touch

1. `docs/specs/cli.md` — change, §4.1 below.
2. `docs/decisions/` — change, §4.2.
3. `README.md` — change, §4.3.
4. `docs/product/brief.md` — change, §4.4.
5. `crates/cli/src/args.rs` — change: the usage text's last paragraph only, §4.5.
6. `crates/cli/tests/documented_view.rs` — new, §5.

## 4. The edits

### 4.1 `docs/specs/cli.md`

1. In the opening paragraph, after the sentence ending `is §12 (D047).`,
   add: `` `luria FILE`, a view of a GenBank or FASTA file, and `luria
   find`, a search of one, are §13 (D048 to D058). `` Re-wrap that
   paragraph to the page's width (72 columns or fewer).
2. In §1's command block (the line also occurs in §12; change only §1's),
   after the line
   `    [--backend openai-compatible --endpoint URL --model NAME [--allow-remote]]`
   add the two lines `luria FILE` and
   `luria find FILE PATTERN [--aa] [--cds NAME] [--circular]`, exactly as
   `luria help` prints them but without its two-space indent.
3. In §1's table, after the `new ...` row, add:

   ```
   | `FILE` | Opens a GenBank or FASTA file in a read-only view of its sequence (§13). |
   | `find FILE PATTERN ...` | Searches a file for bases or a protein and prints the hits as a table (§13.6). |
   ```

4. In §5's table, the row for code 0 ends
   `for `luria review`, you quit |`. Make that
   ``for `luria review`, you quit; for `luria find`, any number of hits, `0 hits` included |``.
5. Append the whole of `docs/tasks/contracts/milestone-8n-cli-section.txt` at the
   end of the file, after one blank line. Keep its tables and fences as
   they are.

### 4.2 `docs/decisions/`

1. In D034, replace `* **Milestone 8** is the web interface.` with
   `* **Milestone 8** is the web interface. D059 replaces this line.`
2. In D052, the bullet `* **ORF finding stays in milestone 9** (M8-4).`
   ends `is not approved and is not implemented here.` After that line,
   add the line `  D058 replaces this bullet (M8-40).`
3. In D055, the bullet `* **No display toggles in milestone 8.**` ends
   `polish contract after the hand test (8m).` After that line, add
   `  D058 replaces this bullet (M8-23, M8-29, M8-30, M8-40).`
4. In D057, the first bullet ends
   `letter catches a number that names the wrong residue.` After that
   line, add `  D058 says where on the codon the cursor lands (M8-35).`
5. Append the whole of `docs/tasks/contracts/milestone-8n-decisions.txt` at the end
   of the file, after one blank line.

### 4.3 `README.md`

1. In the command block, after the `[--backend ...]` line of `luria new`,
   add the same two lines as §4.1 item 2.
2. Replace `approve; see §12 (D047). The web interface is next (Milestone 8, D034).`
   with

   ```
   approve; see §12 (D047). Milestone 8 adds `luria FILE`, a read-only
   view of a GenBank or FASTA file's sequence, translation and features,
   with go-to and search, and `luria find`, the same search as a table;
   see §13 (D048 to D058). Milestone 9 is an editor in that view (D059).
   ```

### 4.4 `docs/product/brief.md`

Replace the block from the line `Milestone 8: Web interface` to the line
`The order from Milestone 4 on was set by D034 (owner, 2026-10-16).`,
both included, with:

```
Milestone 8: Sequence view

Open a GenBank or FASTA file in the terminal and read it: both strands,
translation, numbering, features, go-to and search.

Implemented: luria FILE, with frame, feature and ORF lanes, an overview
strip, go-to, and DNA and protein search; luria find, the same search as
a table; docs/specs/cli.md section 13 (D048 to D058).

Milestone 9: Sequence editor

Edit the sequence in the view: select, insert, replace, cut, yank and
paste, with features following the edits, and save the edited construct
(D059).

Later: Web interface

Expose the same capabilities through a web interface without duplicating
scientific logic. Not scheduled (D059).

The order from Milestone 4 on was set by D034 (owner, 2026-10-16), and
from Milestone 8 on by D059 (owner, 2026-10-27).
```

### 4.5 `crates/cli/src/args.rs`

In `USAGE`'s last paragraph, replace `form and runs them once you approve. See docs/specs/cli.md.`
with `form and runs them once you approve. A FILE alone opens it in a`,
then `read-only sequence view, and 'find' searches it. See docs/specs/cli.md.`
on the next line. Change nothing else in the file.

## 5. Tests the worker writes

`crates/cli/tests/documented_view.rs`, with `section_13()` returning
everything after the heading `## 13. Looking at a sequence: \`luria FILE\` and \`luria find\``.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `the_documented_command_lines_are_the_usage_text` | Each line of §13's first fenced block, trimmed, is a line of `luria_cli::args::USAGE`, trimmed |
| 2 | `the_documented_find_examples_are_what_find_prints` | Every fenced block in §13 whose first line starts `$ luria find ` (there are two): run the binary with the rest of that line split on spaces as its arguments, current directory the repository root (`../..`). It exits 0, and its stdout has the block's other lines, line for line, each compared as its whitespace-separated words (the page aligns with spaces, `find` prints tabs). Assert there are exactly two such blocks |

## 6. Checks

`scripts/check`, plus `cargo test -p luria-cli --test documented_view`,
`documented_new`, `documented_review` and `cli`, all passing.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | In `docs/specs/cli.md` §13's first example, change `4359` to `4358` | test 2 |
| G2 | In §13's command block, change `--circular` to `--circ` | test 1 |

## 8. Do not

- Reword any text given here, or re-wrap anything but §4.1 item 1.
- Touch the view's code, `luria find`'s output, or any other test.
- Edit D048 to D057 beyond §4.2's four added lines, or any other doc.

## 9. Report

Each old text that did not occur exactly once, and G1 and G2's failure
lines.
