---
type: Task Contract
title: "Contract 9c3 — core: the GenBank writer"
description: "write_genbank: an edit buffer written back as one GenBank record, next to the record it was opened from."
status: deprecated
tags: [milestone-9]
---
# Contract 9c3 — core: the GenBank writer

Planner: Claude. Status: implemented. Tier: full.
Plan row: [docs/tasks/contracts/milestone-9-editor.md](/tasks/contracts/milestone-9-editor.md) §7, row 9c3.

## 1. What this contract builds

`write_genbank`: an edit buffer written back as one GenBank record, next
to the record it was opened from. A feature that no edit changed is
written as the file had it, line for line. A changed feature is written
fresh. The header comes back as read, except for three changes: the LOCUS
line, no VERSION line, and one COMMENT line saying luria edited it. Two
pBR322 files built by a script from NCBI's file, without running luria,
pin the whole output byte for byte.

Rules that apply, already settled:
- Plan §5 M9-7, M9-11, M9-15 — what is kept, dropped and added.
- 9c1's accessors (`locus_line`, `molecule_type`, `division`, `date`,
  `header_lines`, `GenBankFeature::lines`), 9c2's `write_edit_location`.
- D039 — the script that builds the expected files is tracked, has
  `--check`, and is listed in `verify.py`.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `EditBuffer`, `EditFeature` | `crates/core/src/edit_buffer.rs` | What is written |
| `GenBankRecord` and 9c1's accessors | `crates/core/src/genbank.rs` | The record as opened |
| `write_edit_location` | `crates/core/src/edit_location.rs` | A changed feature's location |
| `read_genbank`, `open_record`, `buffer_for` | core | Tests read the output back |
| `tests/fixtures/genbank/raw/J01749.1.gb` | — | pBR322: LOCUS line 1, `VERSION` line 5, COMMENT from line 145, FEATURES 231, tet CDS `86..1276` from line 260, ORIGIN 399 |

## 3. Files the worker may touch

1. `crates/core/src/genbank_write.rs` — new: `write_genbank`, `SaveStamp`,
   unit tests.
2. `crates/core/src/lib.rs` — add `pub mod genbank_write;`.
3. `tests/fixtures/editor/reference/saved.py` — new: writes the two files below.
4. `tests/fixtures/editor/pbr322-saved.gb`, `pbr322-substituted.gb` — new,
   written by `saved.py` and never by luria.
5. `tests/fixtures/editor/README.md` — add a section "Saved records (9c3)":
   what each file is and the command that rebuilds or checks them.
6. `tests/fixtures/verify.py`, `tests/fixtures/README.md` — list `saved.py`
   and the two files, as 9c2 did for `locations.py`.
7. `crates/core/tests/genbank_write_fixtures.rs` — new: tests 1–4.

## 4. The public surface, exactly

```rust
/// What the save stamps on the record (M9-11, M9-15): the LOCUS date as
/// `DD-MMM-YYYY`, luria's version, and the edits counted as M9-16 counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveStamp {
    pub date: String,
    pub version: String,
    pub edits: usize,
}

/// The record as a save writes it: `\n` line ends, ending `//\n`.
pub fn write_genbank(buffer: &EditBuffer, opened: &GenBankRecord, stamp: &SaveStamp) -> String;
```

The record, in order:

1. **LOCUS**:
   - The line is `LOCUS`, then 7 spaces, then the name (`opened.locus_name()`)
     left-aligned in 16 columns, then a space.
   - Then the length (`buffer.length()`) right-aligned in 11 columns, then
     ` bp `.
   - Then the molecule. A molecule with a `-` (`ds-DNA`) is split after the
     `-` into 3 columns (`ds-`) and the rest. One without (`DNA`, `mRNA`)
     gets 3 spaces, then itself. Either way the rest is left-aligned in 6
     columns, then 2 spaces.
   - Then the topology (`linear` or `circular`, from the buffer)
     left-aligned in 8 columns, a space, the division, a space, and
     `stamp.date`.
   - A missing molecule is `DNA` and a missing division is `UNK`. A name
     longer than 16 characters pushes the rest right, keeping one space.
   - pBR322's line comes back as the file has it, except for the date.
2. **Header**: `opened.header_lines()` in order, except:
   - the `VERSION` line and its continuation lines (lines starting with a
     space, up to the next line that does not) are left out;
   - after the last line of the last COMMENT block (the `COMMENT` line and
     its continuation lines), the line
     `            Edited with luria {version}: {n} {edit|edits}.` is
     added, with 12 spaces in front. The word is `edit` for 1 and `edits`
     otherwise;
   - with no COMMENT block, `COMMENT     Edited with luria …` becomes the
     last header line.
3. **FEATURES**: the line `FEATURES             Location/Qualifiers`, then
   each buffer feature in order:
   - **Unchanged**: some feature of `opened` is not yet used and has the
     same key, `location()` equal to its `location_text`, and the same
     `(name, value)` qualifiers in the same order. Take the first such in
     file order and write its `lines()` verbatim.
   - **Changed**, written fresh:
     - The key line is 5 spaces, then the key left-aligned in 16 columns
       (a longer key gets one space), then the location. The location is
       `location_text` if `Some`, else `write_edit_location`.
     - A location line over 79 characters breaks after the last `,` that
       fits. Continuation lines start with 21 spaces.
     - Then each qualifier on a line starting with 21 spaces:
       - `/name` for no value;
       - `/name=value` unquoted for `anticodon citation codon_start compare
         direction estimated_length mod_base number rpt_type rpt_unit_range
         tag_peptide transl_except transl_table` (Biopython's
         `FTQUAL_NO_QUOTE`);
       - otherwise `/name="value"`, with each `"` doubled.
     - A quoted line over 79 characters breaks at the last space that
       fits. The space is dropped, as the reader joins lines with one
       space. A run with no space breaks at 79 characters. Unquoted
       values are not wrapped.
4. **ORIGIN**: the line `ORIGIN`, with no trailing space. Then the bases,
   lower case, 60 to a line: the 1-based number of the line's first base,
   right-aligned in 9 columns, then a space and the line's groups of 10
   joined by single spaces. Then `//`.

## 5. Tests and the fixture

`saved.py` reads `J01749.1.gb` as text and writes the two files with plain
text operations. It never imports luria and never runs it.
- **`pbr322-saved.gb`**: the original, with five changes:
  - the LOCUS date set to `29-OCT-2026`;
  - line 5 (`VERSION`) removed;
  - `            Edited with luria 9.9.9: 0 edits.` inserted before the
    FEATURES line;
  - the ORIGIN line's trailing spaces removed;
  - line ends made `\n`.
- **`pbr322-substituted.gb`**: the same, except:
  - the comment says `1 edit.`;
  - base 100 is changed in ORIGIN to `a` if it is not `a`, else `c`, in
    the same case and place;
  - the tet CDS's `/translation` lines (from `/translation=` up to the
    line where the quoted value closes) are removed.
- The script's `--check` regenerates both in memory and compares them byte
  for byte. It also reads all three files with Biopython and asserts:
  - each file's features have the same keys and locations as the
    original's;
  - the qualifiers are the same, apart from the one removed `/translation`;
  - the sequence is the original's, with only base 100 changed in the
    substituted file.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `genbank_write_fixtures.rs::an_unedited_save_is_the_file` | pBR322 opened, `buffer_for`, no edit, stamp (`29-OCT-2026`, `9.9.9`, 0): `write_genbank` equals `pbr322-saved.gb` byte for byte |
| 2 | `…::a_substitution_rewrites_only_its_cds` | The same with one `Substitute` at zero-based 99 of `pbr322-substituted.gb`'s new base, stamp edits 1: equals that file |
| 3 | `…::an_edited_save_reads_back` | pBR322 with a 3-base `Insert` at 99 and a `Delete` of `[2000, 2010)`: the output, read by `read_genbank` and `buffer_for`, gives the same letters, and the same features (key, location, qualifiers) as the edited buffer |
| 4 | `…::locus_line_forms` | The LOCUS line for: pBR322; `ds-DNA` with division; no molecule and no division (`DNA`, `UNK`); a 20-character name — each exact |
| 5 | unit: `qualifiers_wrap_at_79` | A changed feature with a 200-character `/note` of words, a `/note` of one 100-character word, `/pseudo`, `/codon_start=2`, and a value holding `"`: exact lines, none over 79 characters |
| 6 | unit: `comment_placement` | With no COMMENT; with one COMMENT block followed by another section; with two COMMENT blocks (the line goes after the second) |

## 6. Checks

`scripts/check`; `cargo test -p luria-core --test genbank_write_fixtures`
(4 tests); `.venv/bin/python -I tests/fixtures/verify.py` (all pass), or as
9c2 §6 says without a `.venv`.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | Unchanged features re-rendered instead of written verbatim | test 1 |
| 2 | The VERSION line kept | test 1 |
| 3 | "Unchanged" matched on key and location only, ignoring qualifiers | test 2 |
| 4 | The molecule not split at `-` | test 4 |

## 8. Do not

- Write a file (9h does). No clock: the date comes in the stamp.
- Change `genbank.rs` (9c1 is done), `edit_buffer.rs` or 9a's files.
- Make a Rust test call Python (D039).

## 9. Report

If test 1 or 2 differs, the first differing line, both versions. Stop
there: whether the writer or the file is wrong is the planner's decision.
