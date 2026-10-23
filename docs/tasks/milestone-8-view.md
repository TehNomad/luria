# Milestone 8 plan: the sequence view (`luria FILE`) and `luria find`

Status: **Draft for the owner, 2026-10-23. Not approved.** Planner:
Claude. Branch: `feat/milestone-8`. Questions M8-1 to M8-10 are open
(§9). Their answers become D048 in contract 8n.

Builds on the owner-approved parts of the 2026-10-23 proposals:
`proposal-sequence-view.md` (the block layout, the overview inset, the
DNA/AA search toggle with IUPAC codes for mismatches) and
`proposal-commands.md` (a bare file opens the view, subcommand names
win, generic `find` beside workflow `q5`). `proposal-product-direction.md`
and `proposal-decision-triage.md` are not approved yet. This plan
depends on them only where §9 asks.

## 1. What this milestone is

```
luria FILE                       open a GenBank or FASTA file in the view
luria find FILE PATTERN [--aa] [--cds ID]
```

A read-only view of a construct: both strands, the translation of a
chosen CDS or of all six frames, base and residue numbering, features,
an overview of the whole sequence, go-to, and search. You can't place an
edit or design anything yet (milestone 9). `luria find` prints the same
search hits the view shows, without a terminal.

The milestone succeeds when the owner can open their own plasmid files,
find a residue by number or a site by sequence in a few keystrokes, and
trust that every number and residue on screen is right. The second part
is tested; the first is the owner's hand test.

## 2. What exists today, established 2026-10-23

Facts, not design choices:

1. `luria_core::genbank::read_genbank` reads one record: LOCUS name,
   topology (refused unless `linear` or `circular`), sequence and every
   feature with its location kept as **text**.
2. `luria_core::genbank_location::parse_genbank_location` turns a location
   text into intervals and a strand. It refuses fuzzy ends (`<1..12`),
   `order(`, `^`, remote references and every join except an origin join
   (D009, D036). Many real records have fuzzy-ended features, and some
   have spliced joins.
3. `luria_core::genbank_cds::import_cds` imports named CDSs and refuses
   each unusable one with a reason, including any non-`ATG` start (D042).
   In pBR322 (`tests/fixtures/genbank/raw/J01749.1.gb`), `tet` and `bla`
   import, and the ROP CDS (no `/gene`, `GTG` start) does not.
4. `luria_core::fasta::read_fasta` reads one record. FASTA has no topology
   and no features.
5. The base-to-codon arithmetic for a CDS exists only privately:
   `mutation.rs::codon_positions` (residue → three reference positions,
   either strand, through an origin join). Nothing maps a base to its
   residue.
6. `frame::mutant_residues` lays residues on codon middles for an edited
   construct, for a single-interval CDS only.
7. `sequence::bases_of` gives the base set of an IUPAC letter;
   `complement_letter` complements IUPAC letters.
8. `args.rs::parse` knows `help`, `version`, `q5`, `review` and `new`, and
   rejects anything else as an unknown command. `luria` with no argument
   is a usage error.
9. `luria-cli` already depends on `ratatui` 0.30 and `crossterm` 0.29
   (vendored, D046). `luria review` and `luria new` have terminal set-up,
   `TestBackend` tests and a file picker that can be reused.
10. Fixture tooling: Biopython 1.88 is pinned
    (`tests/fixtures/requirements.txt`); `tests/fixtures/verify.py` runs
    every generator's `--check` (D039).
11. pBR322 is 4361 bp and circular. Its one EcoRI site, `GAATTC`, starts at
    base 4359 and runs across the origin (bases 4359–4361 and 1–3,
    1-based). It's a real cross-origin search case.

## 3. Scope

### In

- **Opening a file** (`luria FILE`). GenBank (`.gb`, `.gbk`, `.genbank`)
  and FASTA (`.fa`, `.fasta`, `.fna`), one record, by content rather than
  extension where they disagree (M8-6).
- **The view**, as `proposal-sequence-view.md` §2 lays it out:
  - feature lanes;
  - the translation lane, with residues on codon middles;
  - residue numbers every fifth residue;
  - the top strand with 1-based base numbers at both ends of the line;
  - the bottom strand.
  - Plus the cursor and the status line, the overview inset (§2.1), and
    `?` help.
- **Translation**: off, the selected CDS, three forward frames, or six
  frames. Forward frames sit above the top strand and reverse frames below
  the bottom strand.
- **Navigation**: moving by base, by line or by codon; paging; go-to (base,
  residue in the selected CDS, feature name); the feature list; the CDS
  picker.
- **Search**: the `/` prompt with its DNA/AA badge, as
  `proposal-sequence-view.md` §4 describes it; `n`/`N`.
- **`luria find`**: the same matching, printed as a table.
- **Dispatch**: subcommand names win; any other first argument is a file
  to open; a word that is neither a subcommand nor a file suggests the
  closest subcommand.

### Out (later milestones)

- Placing edits, designing, saving, `luria DESIGN.json`, the start screen
  for a bare `luria` (milestone 9).
- ORF detection (triage T2, milestone 9), unless the owner moves it here
  (M8-4).
- Enzyme names and enzyme lanes; mismatches at unknown positions.
- SnapGene `.dna`; multi-record files; editing features.
- Any change to `luria q5`, `luria review` or `luria new`.

## 4. Core rules (science tier)

These decide which base, residue or position the user sees, so each
gets fixtures verified independently (D018, D039). All functions are
pure, live in `luria-core`, add no dependency, and take zero-based
half-open coordinates (D006). The view converts to 1-based at display.

### 4.1 The view's record

A `ViewRecord` built from a parsed file:

- the `Sequence` (with its topology);
- **every feature**: key, display name (first of `/label`, `/gene`,
  `/locus_tag`, `/product`, else the key), strand, and intervals. A
  feature whose location `parse_genbank_location` refuses is kept with
  its refusal, listed in the feature list as "not drawn" with the reason,
  and never drawn. Fuzzy ends: M8-1.
- **every CDS** as `usable` (imported by `import_cds`, unchanged) or
  `unusable` with its reason. Only usable CDSs can be selected for
  translation and residue numbering (the view must not number residues
  luria would refuse to edit).
- for FASTA: no features, no CDSs, and the topology from M8-3.

### 4.2 The CDS map

For a usable CDS, both directions:

- `residue_bases(cds, number) -> [u64; 3]`: the codon's three reference
  positions, in reading order. This is `codon_positions` made public, with
  its existing tests.
- `base_residue(cds, position) -> Option<(number, place)>`: the residue
  whose codon holds the base, and the base's place (1, 2 or 3) in reading
  order. `None` outside the CDS. Must round-trip with `residue_bases` for
  every base of every fixture CDS.

Covers plus and minus strands and origin-joined CDSs (D036), and counts
the terminal stop as a codon with no residue number (D010). For the stop
codon, `base_residue` gives `None` for the number and the status line says
"stop codon".

### 4.3 Frame translation

`frame_residues(sequence, strand, frame, window) -> Vec<FrameResidue>`:
the residues whose codon middles fall in the window, for frame 1, 2 or 3
on either strand. On the plus strand, frame *f* starts at base *f*
(1-based). On the minus strand, frame *f* starts at the last base minus
(*f* − 1) and reads leftwards. Frames are counted on the linear
coordinates and **do not run across the origin**: a codon that would
cross it isn't shown in frame mode (M8-2). Symbols follow
`FrameResidue`: one letter, `*` for a stop, `?` for an unreadable codon.

The selected CDS's lane uses the CDS map instead, so a CDS crossing the
origin is translated across it.

### 4.4 DNA search

`find_dna(sequence, pattern) -> Vec<DnaHit>`; a hit has a zero-based
start, a length, a strand, and whether it wraps the origin.

- Case is ignored; `U` is read as `T`. A pattern letter outside the IUPAC
  set is an error naming the letter. An empty pattern is an error.
- **Matching rule.** A template letter matches a pattern letter when the
  template letter's base set is a subset of the pattern letter's
  (`bases_of`). So pattern `N` matches anything, pattern `R` matches `A`,
  `G` and `R`, and a template `N` matches only pattern `N` (M8-7).
- **Both strands.** The minus-strand hits are the hits of the pattern's
  reverse complement on the top strand, reported with strand `-` and the
  same top-strand coordinates.
- **Palindromes.** A hit found on both strands at the same span is
  reported once, as strand `both` (M8-8).
- **Circular.** On a circular sequence, a hit may start in the last
  `len(pattern) − 1` bases and wrap. A pattern longer than the sequence
  finds nothing. On a linear sequence nothing wraps.
- **Order.** Hits are sorted by start, then plus before minus.

### 4.5 Protein search

`find_protein(target, pattern) -> Vec<ProteinHit>`:

- Pattern letters: the 20 one-letter residues, `X` (any residue) and `*`
  (stop); case ignored; anything else is an error naming the letter.
- **In a CDS:** matches the CDS's translation (with the terminal stop as
  `*`). A hit gives its first and last residue numbers and the bases it
  covers, so the view can highlight codons and the overview can mark them.
  An unreadable codon (`?`) matches only `X`.
- **In six frames** (no CDS selected, or `p:*`, see §5.4): each frame's
  residues as §4.3 defines them, no wrapping, each hit carrying its frame
  and strand.

### 4.6 The overview scale

`overview_columns(length, width) -> Vec<(u64, u64)>`: the base range each
of `width` columns stands for. The ranges tile the sequence exactly,
differ in size by at most one base, and are in order. Also the inverse,
`overview_column(length, width, position)`. A feature, the visible part,
the cursor and every hit are drawn in the columns their bases fall in. A
feature crossing the origin is drawn in both end ranges.

## 5. Presentation

The block layout and keys are as `proposal-sequence-view.md` §2–§4 shows
them. Details fixed here:

### 5.1 Layout

- **Line width**: the largest multiple of 10 bases that fits beside the
  base numbers, and of 30 when a translation lane is on, so codons never
  split across lines **in frame 1 or the selected CDS when it starts in
  phase**. Otherwise a codon split across lines shows its residue on the
  line holding its middle base. Minimum screen 80 × 24, as for
  `luria review`.
- **Feature lanes**: up to four per block, longest first; the rest as
  `+N more`. A bar shows direction (`▸`/`◂`) and is clipped at the block
  edges.
- **Minus-strand CDS**: its lane sits below the bottom strand, as the
  reverse frames do, with residues on codon middles and numbers
  decreasing left to right. A plus-strand CDS's lane sits above the top
  strand. The residue-number line goes on the same side as its lane.
- **Cursor**: one base, reverse video; its codon, in the selected CDS,
  underlined.
- **Status line**: base (1-based), the residue and codon when inside the
  selected CDS (`V404 (GTG, 1210–1212)`; for a minus-strand codon, the
  codon as read), the features at the cursor, and the current hit
  (`hit 3 of 7`).
- **Title bar**: name, length, topology (with `(assumed)` for FASTA,
  M8-3), and the selected CDS with its strand and 1-based span.

### 5.2 Keys

As `proposal-sequence-view.md` §3, plus `o` for the overview inset.
`q` quits; `ctrl-c` quits from anywhere. Nothing is ever written.

### 5.3 Go-to

`g` opens a prompt. A number goes to that base; `D404` or `404` goes to
the codon of residue 404 in the selected CDS, and a residue letter that
doesn't match the translation is refused with the actual residue
(`residue 404 is V, not D`); any other text is a feature name, matched
exactly first, then by unique prefix; several matches open the feature
list filtered to them.

### 5.4 Search

As `proposal-sequence-view.md` §4: it starts on DNA, switches to AA on
the first letter that isn't an IUPAC nucleotide code, and `tab` toggles
by hand after which it no longer follows the text. AA searches the
selected CDS, or six frames when none is selected. Hits are highlighted
in the blocks and on the overview. The first hit at or after the cursor
becomes current.

### 5.5 `luria find`

```
luria find FILE PATTERN [--aa] [--cds ID]
```

- Same mode rule as the prompt (DNA unless a letter can't be DNA);
  `--aa` forces protein.
- `--cds ID` searches that CDS's translation, and the ID must be a
  usable CDS. Without it, AA mode searches six frames.
- Output is tab-separated with a header, one row per hit:
  `start end strand` (DNA) or `start end strand frame first_residue
  last_residue` (AA). Positions are 1-based inclusive; a wrapping hit has
  `end < start`. The last line is `N hits`. Exit codes: M8-9.

### 5.6 Dispatch

- First argument `help`, `version`, `q5`, `review`, `new` or `find`: that
  subcommand, as today.
- Anything else that names an existing file: the view, with no further
  arguments allowed.
- A `.json` file: refused with "opening a design comes in milestone 9;
  use `luria review FILE.json`".
- Neither a subcommand nor a file: an error, and when the word is within
  edit distance 2 of a subcommand, "did you mean `luria find`?".
- `luria` alone is unchanged (usage text) until milestone 9's start
  screen.

## 6. Testing

- **Core (§4)**: unit tests plus fixtures in a new `tests/fixtures/view/`
  with a README. Expected values are written by Python scripts under
  `tests/fixtures/view/reference/`. The scripts use Biopython's
  `Seq.translate` and `reverse_complement` and a plain Python scan for
  IUPAC matching, share no code with `crates/`, and have `--check` modes
  wired into `verify.py`. Cases:
  - **pBR322**: `tet` (+), `bla` (−), the ROP CDS unusable; the EcoRI site
    across the origin; a palindromic site; an IUPAC pattern; `p:` searches
    for a motif in `tet` and in six frames; base ↔ residue for the first,
    last and stop codons of `tet` and `bla`.
  - **The synthetic records** (`tests/fixtures/genbank/synthetic*.gb`): a
    CDS joined across the origin on each strand; codons split by the
    origin; frames at both ends of a sequence whose length isn't a
    multiple of 3.
  - **Edge cases**: a pattern longer than the sequence; a one-base
    sequence; a template with `N`; an empty pattern; an invalid letter.
- **Disabling a rule**: for each rule in §4, the contract names a test
  that must fail when the rule is removed, and the worker proves it once
  (AGENTS.md).
- **Layout (§5.1)**: pure functions from (record, width, options, block)
  to styled lines, tested against fixture blocks for `tet` and `bla`
  written by the same Python scripts, so a residue drawn one column off
  fails.
- **Keys, prompts and screens**: on ratatui's `TestBackend`, as for
  `luria review`.
- **`luria find`**: through the binary, against the core fixtures.
- **The terminal session**: by the owner by hand, after 8l (§7).

## 7. Contracts (tentative)

Science tier, one function each:

| # | Contract |
| --- | --- |
| 8a | `tests/fixtures/view/`: the Python scripts, the expected TSVs and the README (planner-written; verified with `verify.py`) |
| 8b | core: `ViewRecord` (§4.1) |
| 8c | core: the CDS map (§4.2) |
| 8d | core: frame translation (§4.3) |
| 8e | core: DNA search (§4.4) |
| 8f | core: protein search (§4.5) |
| 8g | core: the overview scale (§4.6) |

Presentation tier (process per M8-10):

| # | Contract |
| --- | --- |
| 8h | cli: dispatch of a bare file and `find` (§5.6) |
| 8i | cli: `luria find` (§5.5) |
| 8j | cli: the block layout (§5.1), pure, fixture-tested |
| 8k | cli: the view: state, keys, cursor, status line, go-to, feature list, CDS picker |
| 8l | cli: the search prompt and the overview inset |
| 8m | the owner's hand test, then a polish contract as 7m was |
| 8n | docs: `docs/cli.md` §13, D048, the product brief's milestone list |

## 8. Not in this milestone

See §3 "Out". Also: no new dependency, no change to the request or
result schemas, no change to existing tests or fixtures.

## 9. Questions for the owner

| # | Question | Planner's proposal |
| --- | --- | --- |
| M8-1 | Features with fuzzy ends (`<1..12`, `1..>900`) are common in real records, and the location parser refuses them. Draw them in the view? | Yes: the view reads `<` and `>` as the stated base and marks the feature as partial in the feature list. Display only; CDS import (D041) still refuses them. Needs a small, separately tested parser option. |
| M8-2 | In frame mode on a circular sequence, should frames run across the origin? | No. Frames are counted from base 1 and stop at the end. A CDS crossing the origin is still translated across it in CDS mode. |
| M8-3 | FASTA has no topology. What should the view assume? | Linear, shown as `linear (assumed)` in the title, with a key (`T`) to switch to circular for this session. Circular changes wrapping search and the overview. Needs D005's triage rewording, or an explicit exception. |
| M8-4 | ORF detection (triage T2): in milestone 8 so a FASTA file has something to number, or milestone 9 as planned? | Milestone 9. Six-frame translation covers looking at a FASTA in milestone 8. |
| M8-5 | Should the CDS picker show unusable CDSs (such as pBR322's ROP)? | Yes, greyed, with the reason, not selectable. Triage T3 (accepting `GTG` starts) would make ROP usable. |
| M8-6 | A file whose extension and content disagree (a `.txt` that is GenBank)? | Decide by content: `LOCUS` first means GenBank, `>` first means FASTA, anything else is refused. |
| M8-7 | A template `N` (or other ambiguity code): should it match pattern `A`? | No: a template letter matches only a pattern letter whose base set contains all of its bases (§4.4). |
| M8-8 | A palindromic site matches on both strands at the same place. One hit or two? | One, strand `both`, in the view and in `luria find`. |
| M8-9 | `luria find` exit code when nothing matches? | 0 with `0 hits`, like a report rather than `grep` (exit 1 is luria's internal error). |
| M8-10 | Use the two-tier process (triage P1, P2) in this milestone: full contracts and fixtures for 8a–8g and 8j, lighter contracts for 8h, 8i, 8k, 8l, and no decision entries for keys and wording? | Yes. Approving triage P1–P2 now would settle it; the rest of the triage can wait. |
