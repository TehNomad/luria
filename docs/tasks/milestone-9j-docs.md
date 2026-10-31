# Contract 9j — docs: the editor in `docs/cli.md`, the decisions, the brief

Planner: Claude. Status: implemented. Tier: light (docs only).
Plan row: `docs/tasks/milestone-9-editor.md` §7, row 9j.

## 1. What this contract does

It records milestone 9 in three documents. The planner wrote every word
that goes in, in §4 to §6. The worker places the text where this
contract says, keeps the documents' existing style around it, and checks
that every key, message and number in the text matches the code.

## 2. Files the worker may touch

1. `docs/cli.md`: §13 only.
2. `docs/decisions.md`: append D060, D061 and D062 after D059. This
   contract allows the edit (AGENTS.md).
3. `docs/product-brief.md`: the "Milestone 9" paragraph only.
4. `crates/cli/tests/documented_view.rs`: only the literal §13 heading it
   splits `docs/cli.md` on, changed to §4.1's new title.
5. `crates/cli/src/sequence_view/state.rs`: only the `?` list's
   `backspace` row, from `in i, a and R` to `in i and a`.
## 3. Checking the text against the code

Before writing, check each of these in the code. Report every mismatch
with its `file:line`, and do not change the text to hide one:
- every key in §4.3 to §4.5, against `crates/cli/src/sequence_view/state.rs`
  (`handle`, `run_command`, `help_rows`);
- the messages quoted: `edits not saved: :w writes them, :q! discards
  them`, `<target> exists: :w! replaces it`, and the COMMENT line
  `Edited with luria <version>: N edits.`, against `state.rs`,
  `terminal.rs` and `crates/core/src/genbank_write.rs`;
- 60 bases a line for FASTA (`crates/core/src/fasta.rs`) and for ORIGIN
  (`genbank_write.rs`).

## 4. `docs/cli.md` §13

1. **Title** becomes `## 13. Looking at and editing a sequence: `luria
   FILE` and `luria find``.
2. **Intro paragraph.** Replace "It reads the file and writes nothing."
   with "You can edit the sequence and save it (§13.7, §13.8)." Replace
   "The decisions behind them are D048 to D058,
   `docs/tasks/milestone-8-view.md` and `docs/tasks/milestone-8m-polish.md`."
   with "The decisions behind them are D048 to D058 and D060 to D062,
   `docs/tasks/milestone-8-view.md`, `docs/tasks/milestone-8m-polish.md`
   and `docs/tasks/milestone-9-editor.md`."
3. **§13.2 table, "Status line" row.** Append: "; in an edit mode, the
   mode first (`-- INSERT --`, `-- OVERTYPE --`, `-- REPLACE --`,
   `-- VISUAL --` with the bases selected); while a prompt or the `:`
   line is open, what you have typed".
4. **§13.3 table.** Change three rows and add two before the `?` row:

   | Key | What it does |
   | --- | --- |
   | `esc` | Close the open list or prompt; in an edit mode, leave it; otherwise, clear the hits |
   | `v` `i` `a` `R` `r` `x` `d` `y` `p` `P` `u` `ctrl-r` | Edit (§13.7) |
   | `:` `ZZ` `ZQ` | Save and quit (§13.8) |
   | `?` | List every key, in sections; `j` and `k` scroll the list, `esc` closes it |
   | `q` | Quit, as `:q` does (§13.8); `ctrl-c` quits from anywhere, asking first when edits are unsaved |

5. **New §13.7 and §13.8** after §13.6, with the text of §5 below.

## 5. The new sections, word for word

````markdown
### 13.7 Editing

You type on the top strand, 5′ to 3′, in every mode. The bottom strand,
the translation, the features, the ORFs, the hits and the overview
follow each edit (D060).

| Key | What it does |
| --- | --- |
| `v` | Select from the cursor; the movement keys grow the selection, and `esc` ends it. A selection does not cross the origin |
| `i` `a` | Insert before the cursor, or after it, until `esc` |
| `R` | Overtype from the cursor until `esc`. It stops at the last base |
| `r` | Replace the base, or every selected base, with the next letter you type |
| `backspace` | In `i` and `a`: take back the last base you typed. It never removes a base that was there before |
| `x` | Cut the base under the cursor, or the selection |
| `d` | Cut the selection |
| `y` | Copy the selection |
| `p` `P` | Paste after the cursor, or before it |
| `u` `ctrl-r` | Undo one edit, or redo it |

The letters are the 15 IUPAC nucleotide codes, in either case, stored
upper case. In `i` and `a`, the terminal's paste types its text:
spaces, line breaks and digits are dropped and so is a FASTA `>` line,
so a GenBank `ORIGIN` block or a FASTA body pastes as bases. Any other
character refuses the whole paste and the status line names it. Cut and
copy go to luria's own register, kept for the session; the system
clipboard is not used.

One edit is one command: everything typed from `i`, `a` or `R` until
`esc`, or one `r`, `x`, `d`, `p` or `P`. `u` goes back one edit at a
time, as far as the file as it was opened. There are no counts (`3x`),
named registers or `.`.

Features follow the edits (D061):
- An edit before a feature moves it. An edit after it changes nothing.
- An insertion strictly inside an interval grows it; one at either edge
  does not. On a circular record, an insertion at the origin grows a
  feature that crosses it.
- A deletion inside an interval shrinks it, and one that covers it
  removes it. A feature with no interval left is removed, and the status
  line names it.
- A deletion over one end trims that end. A `<` or `>` mark stays.
- A replacement is a deletion and then an insertion. `r` never moves a
  feature. Each interval of a `join` follows on its own.
- A CDS whose bases changed loses `/translation`. A `/transl_except` or
  `/anticodon` position follows as an interval would; an edit that
  touches it removes the qualifier, and the status line names it. A CDS
  whose 5′ end is trimmed gets the `/codon_start` that keeps its frame.

After each edit, the CDSs are checked as they are when the file opens.
A selected CDS that the edit makes unusable, by a frameshift or a lost
stop, is deselected, and the status line gives the reason.

A record with a feature whose location the editor cannot read
(`order(…)`, a `^` site, a reference to another record) cannot be
edited. The edit keys do nothing, and the status line names the feature
and its line in the file. The view works as before.

Every edit redraws the whole record. On pBR322 an edit takes a few
milliseconds; on a record of 10 000 000 bases it takes more than a
second.

### 13.8 Saving and quitting

| Command | What it does |
| --- | --- |
| `:w` | Write the opened file. The first write in a session keeps the original as `FILE~` |
| `:w NAME` | Write to `NAME`. After it, `:w` writes `NAME` |
| `:w! NAME` | Write to `NAME`, replacing the file there |
| `:q` | Quit. With unsaved edits it refuses: `edits not saved: :w writes them, :q! discards them` |
| `:q!` `ZQ` | Quit, dropping unsaved edits |
| `:wq` | Write, then quit |
| `:x` `ZZ` | Write if there are unsaved edits, then quit |
| `:N` | Go to base `N` |
| `q` | As `:q` |
| `ctrl-c` | Quit. With unsaved edits it asks first, and `y` or a second `ctrl-c` quits |

`!` after `w`, `wq` or `x` replaces an existing file. Without it, luria
writes over only the opened file and a file this session has already
written. A write that fails leaves the view open with the error on the
status line. If the opened path is a symbolic link, `:w` refuses, and
`:w NAME` writes elsewhere; luria never writes through a link.

GenBank is saved as GenBank and FASTA as FASTA (D062). A GenBank save
keeps the LOCUS name, molecule type, topology and division, and writes
the new length and today's date (UTC). The header lines from DEFINITION
to the last before FEATURES are written as the file had them, except
that VERSION is dropped, since it names the sequence before the edits.
The last line of COMMENT, or a new COMMENT before FEATURES, says
`Edited with luria VERSION: N edits.`, where N counts the edits not
undone. A file luria saved before keeps one such line: the new one
replaces the old. A feature no edit moved keeps its location text as the file had
it. The bases follow `ORIGIN`, 60 to a line. A FASTA save is the line
`>NAME` and the bases, 60 to a line. An unedited GenBank save differs
from the file only in the LOCUS date, the VERSION line, the COMMENT line
and trailing spaces.
````

## 6. `docs/decisions.md`: D060 to D062

Append these, word for word, after D059, in D059's own format.

````markdown
## D060: Editing in the view (Milestone 9)

Status: recorded by the planner, 2026-10-31, from the owner's answers
M9-1 to M9-5, M9-8, M9-10, M9-16 and M9-19 (2026-10-29 to 2026-10-31,
`docs/tasks/milestone-9-editor.md` §5), implemented in contracts 9d to
9m. Refines D059's milestone 9.

Decision:

* **Edit and save only** (M9-1). Placing Q5 mutations from the view and
  designing their primers is a later milestone. The messages that
  promised "milestone 9" say "not supported yet".
* **Vim's keys, as `docs/cli.md` §13.7 lists them** (M9-2, M9-19): `v`,
  `i`, `a`, `R`, `r`, `x`, `d`, `y`, `p`, `P`, `u`, `ctrl-r`, and the
  `:` commands, `ZZ` and `ZQ`. No counts, named registers or `.`. The
  view's own keys keep their meanings: `c`, `f`, `o`, `O`, `t`, `T`, `F`
  and `C` are not Vim's here. In `i`, `a` and `R` every printable key is
  input. In `i` and `a`, `backspace` takes back only bases typed in the
  current command,
  and `R` stops at the last base (planner, approved 2026-10-29).
* **The top strand, 5′ to 3′, in every mode** (M9-3).
* **The 15 IUPAC codes, either case, stored upper case** (M9-4). A
  paste drops spaces, line breaks, digits and a FASTA `>` line, and
  refuses any other character, naming it and its place.
* **luria's own register** (M9-5), for the session. The terminal's paste
  works in `i` and `a`. The system clipboard (OSC 52) is later.
* **One command is one edit** (M9-8, M9-16), for `u` and `ctrl-r` and
  for the count a save records. Undo reaches the file as opened. A view
  is dirty when its edits differ in number from those at the last save,
  so an undo after a save makes it dirty (planner, approved 2026-10-29).
* **Two tiers for milestone 9 only** (M9-10): full tier for the feature
  rules, the edit buffer and the GenBank writer; light tier for keys,
  modes and wording, as D051 had it for milestone 8.

Reason:

The owner works in Vim, and the view already used Vim's movement keys.
One command per edit is what Vim users expect from `u`.

Reversibility:

Keys can change without changing what an edit does. Needs approval.

## D061: How features follow an edit (Milestone 9)

Status: recorded by the planner, 2026-10-31, from the owner's answers
M9-6, M9-12 to M9-14, M9-17 and M9-18 and the readings of 9a
(2026-10-29, `docs/tasks/milestone-9-editor.md` §5,
`tests/fixtures/editor/README.md`), implemented in contracts 9a, 9b,
9b2 and 9e.

Decision:

* **The rules of M9-6** (a) to (i), as `docs/cli.md` §13.7 states them
  and `tests/fixtures/editor/feature-edits.tsv` pins them, one row per
  rule. A selection may not cross the origin of a circular record.
* **At the origin of a circular record** (M9-14), the last base and the
  first are two bases of one interval for a feature that crosses it, so
  an insertion there grows the feature. On a linear record both ends
  are edges.
* **Qualifiers** (M9-13). `/transl_except=(pos:…)` and
  `/anticodon=(pos:…)` follow as an interval would, and are removed,
  with a notice, when an edit touches their position. `/codon_start` on
  a CDS whose 5′ end is trimmed by k bases becomes
  `((codon_start − 1 − k) mod 3) + 1`; an absent one counts as 1 and is
  added when the result is not 1 (9a). A CDS whose bases changed loses
  `/translation`. Every other qualifier stays as written.
* **A partial mark on a removed end interval** moves to the feature's
  new end (9a).
* **The editor's own reading of locations** (M9-17, M9-18): `join(…)`
  and `complement(join(…))` of ranges, with `<` on the first start and
  `>` on the last end, and a bare `N` or `complement(N)` as one base.
  Q5, CDS import and the view's CDS checks still refuse these forms as
  D009 says. The location writer writes a one-base range as `N`.
* **Any other form refuses editing** (M9-12): `order(…)`, `^`, remote
  references and the parser's other refusals. The edit keys do nothing,
  and the status line names the first such feature and its line.
* **CDSs are checked again after each edit** with the code that checks
  them on open (M9-6 i); a selected CDS made unusable is deselected
  with core's reason.

Reason:

A feature's coordinates are biology: a rule that moved them wrongly
without a word would give wrong constructs. Each rule has a fixture row
from an independent reference (D039). A location the editor cannot move
correctly makes the record read-only, not silently wrong.

Reversibility:

Changing a rule changes saved files. Needs approval and a fixture row.

## D062: Saving an edited record (Milestone 9)

Status: recorded by the planner, 2026-10-31, from the owner's answers
M9-7, M9-9, M9-11, M9-15, M9-19 and M9-20 (2026-10-29 to 2026-10-31,
`docs/tasks/milestone-9-editor.md` §5), implemented in contracts 9c1,
9c2, 9c3, 9h, 9l and 9n. M9-19 replaces M9-7's "the opened file is never
overwritten" and its `.edited` default name, and M9-9 for `q`.

Decision:

* **GenBank in, GenBank out; FASTA in, FASTA out** (M9-7).
* **`:w` writes the opened file**, as Vim does (M9-19). Before the first
  write over it in a session, the original is kept as `FILE~`. `:w NAME`
  writes elsewhere, and later `:w` writes `NAME`. Any other existing
  file is replaced only with `!`. An opened symbolic link is not
  written, and nothing is written through a link.
* **Quitting** (M9-9, M9-19). `q` and `:q` refuse while edits are
  unsaved; `:q!` and `ZQ` drop them. `ctrl-c` asks, and `y` or a second
  `ctrl-c` quits.
* **The GenBank header** (M9-15). LOCUS keeps the name, molecule type,
  topology and division, with the new length and today's date in UTC.
  The header lines are written back as read. ACCESSION is kept; VERSION
  is dropped, since it names one exact sequence.
* **The record says luria edited it** (M9-11): the last line of COMMENT,
  or a new COMMENT just before FEATURES, is `Edited with luria
  <version>: N edits.` (`1 edit.` for one), N being the edits not undone
  at the save. If that last line is already such a line, from an
  earlier save, it is replaced, so a file has one (M9-20). FASTA gets
  no such line; its header is `>NAME`.
* **A feature no edit moved keeps its location text** exactly as read;
  only a moved one is written fresh. `ORIGIN` has no trailing space.

Reason:

The owner found that never writing the opened file stacked names
(`x.edited.edited.gb`). Vim's rule with one backup per session keeps the
original once without that. Dropping VERSION keeps a saved file from
claiming to be the sequence it no longer is.

Reversibility:

The backup and the overwrite rule can change without changing what is
written. The file format needs approval.
````

## 7. `docs/product-brief.md`

Replace the paragraph under "Milestone 9: Sequence editor" with:

```
Edit the sequence in the view: select, insert, replace, cut, yank and
paste, with features following the edits, and save the edited construct
(D059).

Implemented: editing in luria FILE with Vim's keys, features and CDSs
following each edit, undo and redo, and :w, :q, :wq, :x, ZZ and ZQ;
GenBank and FASTA saves; docs/cli.md sections 13.7 and 13.8 (D060 to
D062).
```

## 8. Checks

`scripts/check`. `scripts/decision D062` prints the new entry.

## 9. Do not

- Change any file under `crates/` or `tests/` beyond §2 items 4 and 5.
- Reword the text of §4 to §7. Report a mismatch instead (§3).
- Change any other section of the three documents.
