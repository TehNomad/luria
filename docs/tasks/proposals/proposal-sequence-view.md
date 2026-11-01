# Proposal: the sequence view

Status: **Draft for the owner, 2026-10-23. Not approved and not a task
contract.** It's a design sketch to react to. Once approved, it is split
into contracts (section 9). Companion to `proposal-product-direction.md`
(milestones 8 and 9), `proposal-decision-triage.md` (T1, T2, T4) and
`proposal-commands.md` (how the view is opened).

Owner answers so far (2026-10-23):

- The block layout of section 2 is approved.
- Add an overview inset for long sequences (section 2.1).
- Search is a DNA/protein toggle that starts on DNA and switches to
  protein when a letter that can't be DNA is typed. Mismatches are
  written as IUPAC codes such as `N`, not as a count (section 4).

## 1. Goal

Open a construct and get, in a terminal, what a biologist expects from a
sequence editor's sequence pane: both strands, the translation of the
chosen CDS, base and residue numbering, features, and fast go-to and
search. Milestone 8 is read-only. Milestone 9 adds placing edits.

```
luria FILE.gb|FILE.fa
```

(See `proposal-commands.md`.)

## 2. The screen

```
┌ pET28-SID1 · 8521 bp circular · CDS sid1 (+) 1..3153 ──────────── ? help ┐
│ sid1 ▸ ═══════════════════════════════════════════════════════════════════ │
│ His-tag ▸ ───────────                                                      │
│      D   L   K   P   E   V   T   Y   G   D   R   S   L   A   K   L         │
│     399 400                 405                 410                        │
│ 1195 GATCTGAAGCCGGAAGTGACCTATGGCGATCGCAGCCTGGCGAAACTG 1242                  │
│      CTAGACTTCGGCCTTCACTGGATACCGCTAGCGTCGGACCGCTTTGAC                      │
│                                                                            │
│      ...next block...                                                      │
├────────────────────────────────────────────────────────────────────────────┤
│ base 1210 · residue V404 (GTG, codon 1210–1212) · in sid1                  │
│ / search  g go to  f features  c CDS  t translation  r reverse strand       │
└────────────────────────────────────────────────────────────────────────────┘
```

The sequence and names are made up for illustration; this is not the
SID-1 fixture.

Each block, top to bottom:

1. **Feature lanes.** One line per feature overlapping the block, name
   at the left, direction arrow, a bar over its bases. At most four
   lanes; more collapse into `+3 more` (listed with `f`).
2. **Translation.** One-letter residues, centred over their codons (the
   middle base), for the selected CDS only, in its frame and on its
   strand. A minus-strand CDS reads right to left over the bottom strand.
   `t` cycles the display: off, the selected CDS, all three forward frames,
   all six frames.
3. **Residue numbers.** Every fifth residue, plus the first in each block.
4. **Top strand** with base numbers at both ends of the line (1-based,
   as GenBank writes them).
5. **Bottom strand**, complementary, toggled with `r`.

Line width follows the terminal: the largest multiple of 10 bases (and
of 3 when a CDS is shown) that fits. A thin gap every 10 bases is
optional (`space` toggles).

The **cursor** is one base, shown in reverse video, with its codon
underlined. The status line always says where it is: base number, residue
and codon if inside the selected CDS, and the features it lies in.

### 2.1 Overview inset

For a sequence longer than about two screens of blocks, a two-line
overview sits under the title bar:

```
│ 1 ──[sid1 ▸═════════════════]────[KanR ◂═══════]──[ori═══]──────── 8521 │
│              ▲▀▀         ·  ·         ·                                  │
```

- The first line is the whole sequence scaled to the screen width: each
  feature is a bar, with its name and direction where they fit. Only the
  longest features are drawn when they don't all fit; the rest stay in
  the `f` list.
- The second line shows the part on screen (`▀▀`), the cursor (`▲`),
  and search hits (`·`).
- For a circular sequence, the bar is the sequence opened at base 1. A
  feature crossing the origin is drawn at both ends.
- `o` hides or shows it. In milestone 8 it is a display only; jumping by
  moving a marker along it can come later if wanted.

## 3. Navigation

| Key | Action |
| --- | --- |
| arrows, `h j k l` | Move one base or one line |
| `w` / `b` | Next or previous codon in the selected CDS |
| `PgUp` / `PgDn`, `Home` / `End` | Page; start or end of the sequence |
| `g` | Go to: `1210` (base), `D404` or `404` (residue in the selected CDS), `sid1` (feature start) |
| `f` | Feature list, filterable; `enter` jumps to the feature, and on a CDS selects it |
| `c` | Pick the CDS for translation and numbering (GenBank CDSs, or the ORFs of triage T2) |
| `/` | Search (section 4); `n` / `N` next and previous hit |
| `o` | Overview inset on or off |
| `?` | Keys |
| `q` | Quit |

A circular sequence wraps: moving past the last base goes to base 1, and
go-to and search find matches across the origin.

## 4. Search

`/` opens one prompt with a mode badge at its left, `DNA` or `AA`:

```
/ DNA ▸ GAATTC                                   tab: switch to AA
/ AA  ▸ CxxC                                     tab: switch to DNA
```

- **It starts in DNA.** It switches to AA as soon as a letter that is not
  an IUPAC nucleotide code is typed: anything outside `A C G T U R Y S W K
  M B D H V N`, such as `E`, `F`, `I`, `L`, `P`, `Q`, `X` or `*`. The badge
  changes colour so the switch can't be missed.
- **`tab` toggles by hand**, and after that the mode no longer follows
  the text. This is needed for a peptide made only of letters that are
  also IUPAC codes, such as `DAVNG` or `GSG`, which otherwise stays DNA.
- **DNA mode.** IUPAC codes are the way to allow mismatches: `N` is any
  base at that position, `R` is A or G, and so on (`GANTC`, `RGATCY`).
  Both strands are searched, and on a circular sequence matches across the
  origin are found. Case is ignored and `U` is read as `T`.
- **AA mode.** One-letter residues; `X` is any residue and `*` a stop
  (`CXXC`, `HEXXH`). It searches the selected CDS's translation, or all
  six frames when no CDS is selected. A hit is shown on its codons.

Hits are highlighted in place and on the overview inset. The status
line shows `hit 3 of 7`, the strand or frame, and the position. With no
hits, the status line says so and the cursor stays where it was.

Not included: mismatches at unknown positions ("any two bases may
differ"), and enzyme names (`EcoRI`). Either can be added later if
wanted.

All matching is in luria-core, deterministic and tested against
independently verified fixtures (science tier), since it decides where a
user's edit will land.

## 5. Placing edits (milestone 9, sketched)

- With the cursor on a residue, `m` opens a one-line prompt prefilled
  with `D404`; the user types the target (`N`, or `N:AAC`). With a base
  range selected (`v` then move), `m` offers substitute, replace,
  delete, or insert after. The prompt is the existing shorthand, so the
  two ways in can't disagree. (`m` is free in the view: the overview
  inset uses `o`.)
- Each edit joins a **variant list** in a side pane (`tab` to focus). The
  view shows the edited bases in colour, and the mutant residue over its
  codon.
- `D` designs all variants and shows each one's primers as arrows on the
  sequence, with length and Tm. This uses the same design path as
  `luria q5 design`.
- `s` saves the request file and `x` exports, as `luria review` does
  today.

## 6. Defaults on this path (triage T4)

Design policy, numbering, codon policy, overlap and buffer come from the
defaults in the triage draft, shown in a one-line summary above the
variant list. `P` edits them.

## 7. Layers

Per D003. **luria-core** holds the new science: ORF finding, translation
of a frame on a strand, DNA (IUPAC, mismatches, circular) and protein
search, and a mapping from a base to its codon and residue in a CDS. Most
of this exists: CDS extraction, `frame::mutant_residues`, and residue to
codon. **luria-cli** lays out blocks and handles keys only.

## 8. TUI or web

The layout above is plain monospaced text, so it suits a terminal well:
sequence editors' sequence panes are themselves monospaced grids. The
TUI's weak spots are a circular map and mouse selection. Neither is
needed to place an edit. Proposal: build the TUI. The core functions in
section 7 are the same for a later web view.

## 9. Contracts, if approved

Science tier, each with independently verified fixtures:

1. core: base → (codon, residue) and residue → bases for a CDS, both
   strands, across the origin (much exists; fill gaps).
2. core: DNA search with IUPAC codes, both strands, circular.
3. core: protein search with `X` and `*`, in a CDS and in six frames.
4. core: ORF finding (triage T2).
5. core: the scaled overview: the base range each screen column stands
   for. Small, but it decides where hits are drawn, so it gets tests.

Presentation tier, one contract each, tested on ratatui's test backend
and by the owner's hand test:

6. `luria FILE`: open a file, block layout, numbering, strands.
7. Translation lanes and CDS picker.
8. Feature lanes and feature list.
9. Go-to and the search prompt with its DNA/AA toggle.
10. Overview inset.

Hand test after 10. Milestone 9 contracts follow from it.

## 10. Questions for the owner

1. Overview inset: is display-only enough for milestone 8?
2. Should enzyme sites (search by name, and a lane) be included, and if so
   when?
