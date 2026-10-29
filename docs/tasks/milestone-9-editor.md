# Milestone 9 plan: a sequence editor in the view

Status: **Draft for the owner, 2026-10-28, revised 2026-10-29 (M9-12 to
M9-16, the contract order, the 9c split). Approved: M8-27, the scope
line it starts from, and M9-1 to M9-16 as proposed (owner,
2026-10-29).** Planner: Claude. Branch:
the owner's `feat/milestone-9`. Decision: D059 (recorded by
contract 8n).

## 1. What this milestone is

M8-27 (owner, 2026-10-27): **a general sequence editor in the view**:
Vim-style `v` (select), `i` (insert) and `r` (replace), with cut, yank and
paste of any length, features following the edits, and the edited
construct saved. It reverses `proposal-product-direction.md` §5's "not a
general sequence editor" and takes in the export that proposal put in
milestone 10.

```
luria FILE        the view, now with edit modes, and a save
```

No new subcommand is planned. `luria find`, `q5`, `review` and `new` are
unchanged unless §5 says otherwise.

## 2. What exists today (planner's reading, 2026-10-28)

| Piece | Where | What it gives an editor |
| --- | --- | --- |
| `Sequence` | `crates/core/src/sequence.rs:78` | Immutable: letters (`Vec<char>`), content hash, topology. The IUPAC alphabet, ASCII case folded to upper. Each edit would build a new one and hash it again |
| GenBank reader | `crates/core/src/genbank.rs:82-160` | Keeps LOCUS name and topology, accession, version, the sequence and every feature with its location text and qualifiers. **Drops every other header line** (DEFINITION, SOURCE, REFERENCE, COMMENT) **and the LOCUS molecule type, division and date** (`Locus`, `genbank.rs:174`) |
| GenBank writer | — | **None.** `--write-edited` writes FASTA only (`crates/cli/src/write.rs`) |
| Location writer | — | **None.** Nothing in core turns intervals back into GenBank location text (`complement`, `join`, `<`/`>`) |
| Feature shapes | `crates/core/src/display_location.rs`, `view_record.rs:129` | The view's display parser: partial ends and origin-crossing joins (D036, D049, D053). **Every other `join(…)` is refused** `UnsupportedJoin`, as `order(…)`, `^` and remote references are (`genbank_location.rs:1-60`, rule 9; D009); see M9-17 |
| Edits with verification | `crates/core/src/mutation.rs:1072-1375` | `apply_nucleotide`, `apply_deletion`, `apply_insertion`, `apply_replacement`: CDS-relative edits for Q5, independently verified. Base-range edits in the view are simpler; whether to reuse these is a planner choice in 9c |
| Feature remapping | — | **None.** No code moves a feature when the sequence under it changes |
| The view | `crates/cli/src/sequence_view/` | Read-only state, layout, lanes, lists, go-to, search, ORFs. Bound today: `h l j k w b t T o g F O C f c / n N ? q`, `ctrl-c/d/u` (`state.rs:386-492`). Free for an editor: `v V i a r R x d y p P u s J K` (M8-38 kept `J`/`K` free). `ctrl-r` is not mapped from the terminal yet (`terminal.rs:46-64`). `q` quits whenever no list or prompt is open, and `ctrl-c` always (`state.rs:386`) |

## 3. Scope

### In (if §5 is answered as proposed)

- Modes: normal (today's view), visual (`v`), insert (`i`, `a`), replace
  (`r` one base, `R` overtype).
- Cut (`d`, `x`), yank (`y`) and paste (`p`, `P`) of any length, in one
  register.
- Undo and redo (M9-8).
- Features following every edit by fixed rules (M9-6), pinned by
  planner-written tables (full tier).
- CDSs, frames, ORFs, the GC line, search hits and the overview recomputed
  after each edit.
- Saving: a GenBank record from a GenBank file, FASTA from FASTA (M9-7), and
  a GenBank writer in core, pinned by round-trip fixtures checked with
  Biopython (full tier).
- A quit that asks when there are unsaved edits (M9-9).
- `docs/cli.md` §13 and the decisions this milestone settles.

### Out (unless the owner moves it in)

- Placing Q5 mutations from the view and designing primers there
  (`proposal-sequence-view.md` §5): see M9-1.
- Typing residues that become codons; editing features or qualifiers by
  hand; annotation of new features.
- The system clipboard (M9-5); SnapGene `.dna` files; a web interface
  (D059).

## 4. Process

D051 expires when milestone 8 merges, so this milestone returns to
`AGENTS.md`'s full process unless the owner approves otherwise (M9-10).

## 5. Questions for the owner

Each row has the planner's proposal. "As proposed" is a full answer.
**M9-1 to M9-16: all approved by the owner as proposed, 2026-10-29**
(M9-12 to M9-16 first, then M9-1 to M9-11); they are recorded as
decisions in 9j. With every row answered as proposed, M9-13 to M9-16 need
no change for M9-6 or M9-11. The owner also approved 9a's two readings
(2026-10-29, `tests/fixtures/editor/README.md`): a partial mark on a
removed end interval moves to the feature's new end, and an absent
`/codon_start` counts as 1 and is added when a 5′ trim makes it not 1.

| # | Question | Planner's proposal |
| --- | --- | --- |
| M9-1 | Does milestone 9 also place Q5 mutations from the view and design their primers (the proposal's `m` prompt, variant list and `D`), or only edit and save? | **Edit and save only.** Designing from the view is the next milestone. Meanwhile, the messages that promise "milestone 9" today (opening a `.json`, a `--cds` range in `find`) are reworded to "not supported yet", so the program does not promise what this milestone will not build (`find.rs:8,81,122`, `args.rs:178,608`, and the tests that pin them, `open_cli.rs:144-157` and `find_cli.rs:335`, which that contract is allowed to change). `luria new` and `review` stay |
| M9-2 | Which Vim keys, exactly? | `v` select from the cursor (`esc` ends it); `i` insert before the cursor, `a` after; `r` + a letter replaces the base, or every base of a selection; `R` overtypes until `esc`; `x` cuts the base or the selection, `d` the selection; `y` yanks the selection; `p` pastes after the cursor, `P` before. No counts (`3x`), no named registers, no `.` repeat. The view's own keys keep their meanings (`c`, `f`, `o`, `O`, `t`, `T`, `F`, `C` are not Vim's here). In insert and overtype modes every printable key is input: `q` and the view's keys are letters M9-4 refuses, not commands, and `esc` leaves the mode. `ctrl-c` follows M9-9 in every mode |
| M9-3 | Which strand do you type on? With a minus-strand CDS selected, the bottom strand is drawn bright (M8-36). | **The top strand, always**, 5′→3′ left to right, in every mode. The bottom strand follows as its complement. Typing the reverse complement of a pasted or typed sequence is later |
| M9-4 | Which letters may be typed or pasted? | The 15 IUPAC letters luria already accepts, either case, stored upper case. In a paste, spaces, line breaks and digits are dropped, so a GenBank `ORIGIN` block or a FASTA body pastes as bases; a FASTA `>` line is dropped. Any other character refuses the whole paste, with the character and its place |
| M9-5 | Where do cut and yank go? | luria's own register only, kept for the session. The terminal's own paste works in insert mode. Copying out to the system clipboard (OSC 52) is later |
| M9-6 | How do features follow an edit? This is the milestone's science, and every rule gets a table row. | (a) An edit before a feature shifts it by the change in length; after it, nothing changes. (b) An insertion strictly between two bases of an interval grows the interval; one at either edge does not. (c) A deletion inside shrinks the interval; one that covers an interval removes it, and a feature with no interval left is removed and named on the status line. (d) A deletion over one end trims that end; the end's `<`/`>` mark stays as the file wrote it. (e) A replacement is a deletion then an insertion at the same place; a substitution (`r`) never moves a feature. (f) Join features: each interval on its own. (g) A selection may not cross the origin of a circular record in this milestone; features that cross it follow (a)–(f) per interval. (h) Qualifiers stay as written, except that a CDS whose bases changed loses `/translation`, which would be stale, and M9-13's. (i) After each edit the CDSs are checked again as the view checks them on open; a selected CDS that becomes unusable (a frameshift, a lost stop) is deselected and the status line gives core's reason |
| M9-7 | How and where does a save write? | `s` opens a "save as" prompt prefilled with `<file stem>.edited.gb` (or `.fa`) beside the opened file. GenBank in, GenBank out; FASTA in, FASTA out. **The opened file is never overwritten in this milestone**; any other existing name asks, and `y` replaces it, as M7-49 does. The GenBank LOCUS line gets the new length and today's date (M9-15 for the rest of it); the header lines between LOCUS and FEATURES (DEFINITION, SOURCE, REFERENCE, COMMENT …) are written back as the file had them, which means the reader starts keeping them as text (contract 9c1). A feature no edit moved keeps its location text exactly as read; only a moved one is written fresh (9c2), so an unedited save differs from the file only on the LOCUS line and M9-11's COMMENT |
| M9-8 | Undo? Not in M8-27, but an editor without it loses work. | `u` undoes and `ctrl-r` redoes, back to the opened file, for the session |
| M9-9 | Quitting with unsaved edits? | `q` asks; `y` quits without saving. `ctrl-c` asks too, and a second `ctrl-c` quits |
| M9-10 | Two-tier process again (D051's P1 and P2) for this milestone? | **Yes, for milestone 9 only**: full tier for the feature rules, the edit buffer and the GenBank writer; light tier for keys, modes and wording |
| M9-11 | Should the saved file record that luria edited it? | A GenBank `COMMENT` line: `Edited with luria <version>: N edits.` (N as M9-16 counts). The LOCUS date is the only date, so tests can fix it and compare whole files. Nothing in FASTA |
| M9-12 | What happens to a feature whose location the view cannot read (a `parse_display_location` refusal, D053: a remote reference, a between-bases `^` site, …)? The view shows it but no rule can move it, and writing its old text back after an edit before it would give wrong coordinates without a word. | **Edit modes are refused on that record**: `v`, `i`, `a`, `r`, `R`, `x`, `p`, `P` leave the view as it is, and the status line names the first such feature and its line in the file. The view itself is unchanged. (Alternatives: allow edits and drop those features with a notice on the first edit; or allow edits but refuse the save) |
| M9-13 | Which qualifiers carry coordinates or a frame that an edit can make wrong? | (a) `/transl_except=(pos:…)` and `/anticodon=(pos:…)`: the position follows M9-6 (a)–(e) as an interval would; an edit that touches it removes the qualifier and the status line names it. (b) `/codon_start` on a CDS whose 5′ end is trimmed by k bases (the left end, or the right end of a minus-strand CDS) becomes `((codon_start − 1 − k) mod 3) + 1`; no other edit changes it. Every other qualifier stays as written (M9-6 h) |
| M9-14 | On a circular record, does an insertion at the origin grow a feature that crosses it (`join(4000..4361,1..10)` on pBR322)? Under M9-6 (b) the origin is the edge of two intervals, so it would not, though the bases are contiguous. | **It grows it.** On a circular record the last base and the first are "two bases of an interval" for a feature that crosses the origin. The bases go where they were typed: `a` on the last base appends them (`join(4000..4361+k,1..10)`), `i` on base 1 puts them first and every feature shifts (`join(4000+k..4361+k,1..10+k)`). On a linear record both are edges, as (b) says |
| M9-15 | What else does the saved LOCUS line and header carry? | LOCUS keeps the name, molecule type (`DNA`, `ds-DNA` …), topology and division as read, which the reader starts keeping (9c1). ACCESSION is kept; **VERSION is dropped**, since it names one exact sequence and the edited one is not it. M9-11's line is added as the last line of the file's COMMENT, or as a new COMMENT just before FEATURES if it has none. Saving twice in a session writes the same one line with the count at that save |
| M9-16 | What is one edit, for undo and for M9-11's count? | **One command**, as in Vim: everything typed from `i`, `a` or `R` until `esc` is one edit; each `r`, `x`, `d` and paste is one. `u` undoes one edit and `ctrl-r` redoes it; the count is the edits on the undo stack at the save, so undone edits do not count |
| M9-17 | The shared parser refuses every spliced join (`join(10..20,30..40)`, D009), so under M9-12 one such feature makes a record uneditable. Should the editor read them? (Raised by the planner, 2026-10-29, writing 9b) | **Approved by the owner, 2026-10-29: an edit-only reading.** The editor reads `join(…)` and `complement(join(…))` of ranges, with `<` on the first range's start and `>` on the last range's end, into its own location type (contract 9b2), so those records can be edited and saved. Q5, CDS import and the view's CDS checks still refuse them as D009 says. `order(…)`, `^`, remote references and every other refusal stay under M9-12 |

## 6. Testing (if §5 is answered as proposed)

- **Feature rules**: `tests/fixtures/editor/feature-edits.tsv`, one row per
  case of M9-6, M9-13 and M9-14 (feature shape and qualifiers, edit,
  expected shape and qualifiers), written by the
  planner from the rules and checked by a separate Python script that
  applies them on lists of intervals; neither runs luria (D039). pBR322 and
  `frames-62.gb` give the real shapes (joins, origin crossing, partials).
- **GenBank writer**: read, write unedited, and the output read back by
  Biopython 1.88 gives the same sequence, the same features (key,
  location, qualifiers) and the same LOCUS name; then the same after a
  scripted list of edits, against the M9-6 table's expectations. An
  unedited save differs from the input only on the LOCUS line and M9-11's
  COMMENT line (M9-7), compared as whole files.
- **Location writer**: a table of intervals and partial marks to GenBank
  text (`complement`, `join`, `<`/`>`), each row checked by Biopython
  parsing the text back to the same intervals.
- **Speed**: one edit and the recompute after it (9e) within a time limit
  the contract states, measured on pBR322 and on a generated record at
  D056's ceiling; the measurement decides whether 9e recomputes in full
  or only what the edit touched.
- **Editor keys**: light tier: behaviour, not wording.

## 7. Contracts (tentative, in order)

| # | Contract | Tier |
| --- | --- | --- |
| 9a | fixtures: the M9-6, M9-13 and M9-14 table, its checker, the README section | full (planner) |
| 9b | core: an edit buffer of bases and features, with insert, delete, replace and substitute by range, and M9-6's, M9-13's and M9-14's rules on its own location type; fast on D056's largest record | full |
| 9b2 | core: the editor's reading of a feature's location text into 9b's type (M9-17), and M9-12's refusal for every other form | full |
| 9c1 | core: the reader keeping the LOCUS molecule type and division and the header lines as text | full |
| 9c2 | core: the location writer, with its Biopython-checked table | full |
| 9c3 | core: the GenBank writer (M9-7, M9-11, M9-15) and the round-trip fixtures | full |
| 9d | cli: modes and the selection (`v`, `esc`), drawn, and M9-2's key precedence (`q` and the view's keys as input in insert and overtype) | light |
| 9e | cli: recomputing CDSs, frames, ORFs, hits and the strip after an edit, driven by 9b's buffer in tests before any key edits, within §6's time limit | light, with M9-6 (i) tested in full |
| 9f | cli: insert, replace and paste input, with M9-4's letters | light |
| 9g | cli: cut, yank, paste, undo, redo (M9-16), and `ctrl-r` from the terminal | light |
| 9h | cli: save, the overwrite prompt, quitting with unsaved edits | light |
| 9i | the owner's hand test, then a polish set | — |
| 9j | docs: `docs/cli.md` §13, the decisions, the brief | — |

Carried over from milestone 8's hand test, no change asked yet (8m §1):
the cap sentence cut short at 80 columns, the list window scrolling
early, no "no CDS" row in the picker, go-to reading `T7` as a residue, a
block taller than the body not drawn, and pBR322 taking about a second to
open. The last two matter more once the screen redraws on every key.
