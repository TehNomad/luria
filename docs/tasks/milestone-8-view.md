# Milestone 8 plan: the sequence view (`luria FILE`) and `luria find`

Status: **Draft for the owner, 2026-10-23; amended 2026-10-24 after the
planner re-checked §2 against the code and the fixtures. Every question is
now answered (§9), so §4–§6 are settled by D048–D052; what has not been
approved is the plan's contract split, which is exercised one contract at a
time.** Planner: Claude. Branch: `feat/milestone-8`.

Owner answers (2026-10-24): M8-11 to M8-14 as proposed; M8-16 to M8-19
as proposed; M8-1, M8-3, M8-6 and M8-20 as proposed (partials drawn by
their own display parser, format by content, FASTA linear with `T` and
`--circular`); M8-15 matched against every id source, and an ambiguous
`--cds` in `luria find` **refuses with the candidate list** rather than
prompting; M8-10, the two-tier process **for milestone 8 only**.

Recorded now rather than in 8n, because the §6 fixtures encode them:
**D048** (M8-11 to M8-14), **D049** (reading the record: M8-1, M8-3,
M8-6, M8-15, M8-20), **D050** (the search and layout rules the search
fixtures encode: M8-16 to M8-19), **D051** (the two-tier process for
this milestone).

Still open: none. **M8-2, M8-4, M8-5, M8-7, M8-8 and M8-9** were accepted
as proposed on 2026-10-24 and are recorded as **D052**, alongside D049
(reading the record) and D050 (the search and frame rules). §9 is closed;
the contracts can be written against §4–§6 as they stand.

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
    1-based). `GAATTC` occurs **nowhere else in the record**, so a linear
    search finds none of it: it is a real cross-origin search case and the
    only way to see it is to search circularly. Internal palindromic sites
    are there too — `GGATCC` at 375, `AAGCTT` at 29, `CATATG` at 2295.
12. The existing synthetic records are 60 bp circular (`synthetic.gb`) and
    393 bp linear (`synthetic-linear.gb`). Both lengths are multiples of 3,
    and `synthetic.gb` already holds a CDS joined across the origin on each
    strand and every refused location form.

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
- **Guard rails**: a terminal check for the view, a byte and a base
  ceiling for the view and `find`, and a cap on the hits shown (M8-12 to
  M8-14).

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
  and never drawn. Partial features are drawn, by their own parser.
- **every CDS** as `usable` (imported by `import_cds`, unchanged) or
  `unusable` with its reason. Only usable CDSs can be selected for
  translation and residue numbering (the view must not number residues
  luria would refuse to edit).
- for FASTA: no features, no CDSs, and the topology from M8-3.

One core entry point builds it, not two: `open_record(name, text) ->
ViewRecord` sniffs the content (M8-6: `LOCUS` first is GenBank, `>` first
is FASTA, anything else refused), parses, and constructs the record, so
the view and `luria find` can never disagree about what a file is. This
is a core function in 8b, not part of dispatch in 8h. Format by content,
not by extension, is D049 (M8-6).

**Partial features are drawn** (M8-1, D049). A location with `<` or `>` is
read by a separate display-only parser, `parse_display_location`, which
takes the stated base as the bound and records which end is uncertain.
`parse_genbank_location` keeps its rules and its tests exactly as they
are — no flag on it changes what the science tier accepts (D009, D036).
The feature list names the uncertain end; go-to does not jump to it
(§5.3); CDS import still refuses such a location, so it is never
selectable for numbering and D041 and D042 are unchanged.

The refusal for every undrawn feature and unusable CDS is a **code**
naming the rule (`partial_location`, `non_atg_start`, `fuzzy_start`, …),
never a sentence; the cli renders the wording, as the 5a1 error codes do.
Otherwise milestone 8 bakes user-visible sentences into the science tier
and triage P2's "wording changes freely" stops being true.

The ceiling (M8-14, owner). `open_record` refuses a record over
`MAX_RECORD_BASES` = 10 000 000 bases and names the limit; the cli (8h)
refuses a file whose byte length is over `MAX_RECORD_BYTES` before it is
read, so a gigabase FASTA never reaches the parser. `luria-core` sets no
sequence length limit today, so this ceiling is new and gets a test of
its own at the boundary and one past it.

### 4.2 The CDS map

For a usable CDS, both directions. `Cds` alone cannot answer either
question, so the map is built from the CDS **and its translation**:
core does not assume the final codon is a stop — `translate()` records
`terminal_stop()` as a fact about the sequence, and
`a_complete_cds_needs_no_start_or_stop_codon` in
`crates/core/src/translation.rs` proves a complete CDS may have none — and
the residue count is the translation's, not `total_length() / 3`. Import
already guarantees the two agree: step 10 of `import_cds` translates every
usable CDS and passes its errors through.

```rust
// crates/core/src/cds_map.rs
pub struct CdsMap { /* intervals, strand, total_length, residues, terminal_stop */ }

impl CdsMap {
    pub fn new(cds: &Cds, translation: &Translation) -> CdsMap;
    pub fn residues(&self) -> u64;          // Translation::residues().len()
    pub fn codons(&self) -> u64;            // total_length() / 3
    pub fn terminal_stop(&self) -> bool;
    /// The three reference positions of residue `number`'s codon, in the
    /// order the codon is read. `CodonOutsideSequence` for a number with no
    /// residue, including `residues + 1`, which addresses the stop codon.
    pub fn residue_bases(&self, number: u64) -> Result<[u64; 3], DomainError>;
    /// `None` for a base outside the CDS.
    pub fn base_residue(&self, position: u64) -> Option<CodonPlace>;
}

pub struct CodonPlace {
    pub codon: u64,        // 1-based, counted in reading order
    pub place: u64,        // 1..=3, in reading order
    pub residue: Option<u64>,   // None only for the terminal stop's codon
}
```

`residue_bases` is `mutation.rs::codon_positions` reused (`pub(crate)`), so
the `CodonOutsideSequence` rule and its arithmetic are already tested
there; `base_residue` is the inverse, computed from the intervals. Covers
plus and minus strands and origin-joined CDSs (D036), and counts the
terminal stop as a codon with no residue number (D010). For the stop codon
`base_residue` gives `residue: None` and the status line says "stop codon";
when `terminal_stop()` is false every codon of that CDS has a number
(D010 excludes a *stop*, and a CDS whose last codon is not one has nothing
to exclude).

The arc reading a wrapped CDS uses is not open: D036 plus the hand-derived
`mj1`/`mj2` entry in `tests/fixtures/genbank/README.md` fix it — the CDS's
text is the joined arc along the molecule, and a minus CDS takes its
reverse complement, so a minus CDS crossing the origin starts at the
complement of the arc's lowest-numbered base. `tests/fixtures/view/`
pins the resulting codons for `pj`, `fP` and `fM`, and its reference script
compares that walk with Biopython's `location.extract()` for each.

**Not D040.** `residue_bases` returns the three bases in the order the
codon is read — descending on a minus strand — because that is what
`mutation.rs::codon_positions` already returns and what the view draws.
D040 orders the *intervals a result records* in reference order and says
outright that this "is not the strand's reading order". Both are right;
a fixture that mixes them up is not. 8c names the order in the function
doc and in every fixture column header.

### 4.3 Frame translation

`frame_residues(sequence, strand, frame, window) -> Vec<FrameResidue>`:
the residues whose codon middles fall in the window, for frame 1, 2 or 3
on either strand. On the plus strand, frame *f* starts at base *f*
(1-based). On the minus strand, frame *f* starts at the last base minus
(*f* − 1) and reads leftwards. Frames are counted on the linear
coordinates and **do not run across the origin**: a codon that would
cross it isn't shown in frame mode (M8-2, owner 2026-10-24, D052). The
numbering is the same on a circular record — a codon that would wrap is
excluded whether or not the record is circular. Symbols follow
`FrameResidue`: one letter, `*` for a stop, `?` for an unreadable codon.

A frame's last codon is complete or absent. When the length isn't a
multiple of 3, the 1 or 2 bases left at the far end of a frame are **not
drawn at all** (M8-18, owner 2026-10-24, D050); `?` is reserved for a
*complete* codon that cannot be read (an ambiguity code such as `N`, or a codon D010 refuses to
guess at). The two are different cases and the reference script must not
let Biopython pad or trim the tail for it.

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
  `G` and `R`, and a template `N` matches only pattern `N` (M8-7, owner
  2026-10-24, D052).
- **Both strands.** The minus-strand hits are the hits of the pattern's
  reverse complement on the top strand, reported with strand `-` and the
  same top-strand coordinates.
- **Palindromes.** A hit found on both strands at the same span is
  reported once, as strand `both` (M8-8, owner 2026-10-24, D052).
- **Circular.** On a circular sequence, a hit may start in the last
  `len(pattern) − 1` bases and wrap. A pattern longer than the sequence
  finds nothing. On a linear sequence nothing wraps.
- **Order.** Hits are sorted by start, then plus before minus.
- **Overlaps.** Every match position is reported, overlapping included:
  `AA` on `AAA` is three plus-strand hits (M8-16, owner 2026-10-24,
  D050). A search that kept
  only non-overlapping hits would hide the site the user came to look
  for. There is **no cap in core**; the cap is presentation, in §5.4 and
  §5.5 (M8-12).
- **Equal length.** A pattern as long as the sequence can match at every
  one of its rotations on a circular record, and at exactly one place on
  a linear one. Both are fixture cases, not afterthoughts.

### 4.5 Protein search

`find_protein(target, pattern) -> Vec<ProteinHit>`:

- Pattern letters: the 20 one-letter residues, `X` (any residue) and `*`
  (stop); case ignored; anything else is an error naming the letter.
- **In a CDS:** matches the CDS's translation (with the terminal stop as
  `*`). A hit gives its first and last residue numbers — the CDS's own
  numbering, 1 at the first translated codon (D010) — and the bases it
  covers, so the view can highlight codons and the overview can mark
  them. An unreadable codon (`?`) matches only `X`, and never `*`.
- **In six frames** (no CDS selected, or `p:*`, see §5.4): each frame's
  residues as §4.3 defines them, no wrapping, each hit carrying its frame
  and strand.
- **Whose residue numbers** (M8-17, owner 2026-10-24, D050; it decides what
  `luria find`'s AA columns mean). In CDS mode the numbers are the CDS's,
  and a hit running onto the terminal stop carries no last residue —
  D010 says the stop is not a residue — printed as `stop` by `find` and
  as "…to the stop codon" in the view. In frame mode there is no CDS, so
  the numbers count codons **within that frame** from its first codon,
  and a frame hit always carries strand and frame so its numbers can be
  located; the view never shows a frame-relative number as if it were a
  residue of a CDS.
- `target` is a named type fixed in 8f's contract **before** 8a writes
  the AA fixtures: `ProteinTarget::Cds(&Cds)` or
  `ProteinTarget::SixFrames(&Sequence)`, and a `ProteinHit` with the base
  span, strand, frame, and the two optional residue numbers. Fixtures for
  a signature that changes afterwards get rewritten twice.

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
  base numbers, and of 30 when a translation lane is on. Minimum screen
  80 × 24, as for `luria review`.
- **Phase, not a fallback** (M8-19, owner 2026-10-24, D050). When a
  translation lane is on, every block starts at a base in phase with the
  displayed translation — for a minus-strand CDS, in phase counted from
  its far end — so a codon never splits across lines. Single-base
  movement moves the cursor inside the block; the block scrolls a whole
  line at a time. Six-frame mode snaps each side to its own frames:
  forward frames above the top strand, reverse frames below the bottom.
  The rejected alternative — a split codon's residue drawn over the line
  holding its middle base — means every residue changes which bases it
  sits on whenever the user single-bases past a block edge, and it gives
  §6's column fixtures two layouts per width instead of one.
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

As `proposal-sequence-view.md` §3, plus `o` for the overview inset and,
if M8-3, `T` to switch a FASTA's assumed topology for the
session (capital: `t` already cycles the translation lanes, §5.5, D049). `q` quits; `ctrl-c` quits from anywhere. Nothing is ever
written.

### 5.3 Go-to

`g` opens a prompt. A number goes to that base; `D404` or `404` goes to
the codon of residue 404 in the selected CDS, and a residue letter that
doesn't match the translation is refused with the actual residue
(`residue 404 is V, not D`); any other text is a feature name, matched
exactly first, then by unique prefix; several matches open the feature
list filtered to them.

Go-to never jumps to a feature whose location was not drawn (M8-1): its
start is not stated, and parking the cursor at the bound of a `<1..12`
feature claims a precision the record does not have. Selecting it in the
list shows why — `not drawn: start not stated` — and `enter` does not
move the cursor.

### 5.4 Search

As `proposal-sequence-view.md` §4: it starts on DNA, switches to AA on
the first letter that isn't an IUPAC nucleotide code, and `tab` toggles
by hand after which it no longer follows the text. AA searches the
selected CDS, or six frames when none is selected (M8-11: a file with one
usable CDS opens with it selected; a file with several opens with none
and the title bar says `no CDS selected`, because picking one silently is
what D005 is for). Hits are highlighted in the blocks and on the
overview. The first hit at or after the cursor becomes current.

Two prompt details 8l1 states: deleting the letter that forced the switch
returns the mode to DNA, as long as `tab` has not been pressed; and an
empty pattern is DNA mode, no search, no error.

**Cap** (M8-12). The view holds at most 200 hits, and when there are more
the status line says `showing the first 200 of 8521 hits — refine your
pattern`; `n` cycles inside what is shown. An all-ambiguity pattern such
as `N` or `NNNN` matches at every position of the record, each hit
reported once as strand `both` (M8-8, D052): 8521 hits on an 8521 bp construct.
An uncapped hit list is not a UI.

### 5.5 `luria find`

```
luria find FILE PATTERN [--aa] [--cds ID]
```

- Same mode rule as the prompt (DNA unless a letter can't be DNA);
  `--aa` forces protein.
- `--cds VALUE` (M8-15, owner 2026-10-24, D049) searches one CDS's
  translation, and the CDS must
  be a usable one. The value is matched against every id the record
  offers — display name, then `/gene`, `/locus_tag` and `/protein_id` —
  exact first, then by unique prefix, the same matcher as go-to and the
  feature list. Several CDSs matching is an **error listing the
  candidates with the source each id came from and their spans**: never a
  guess (D005), and never a prompt, because `luria find` has no terminal
  and a command a script runs must fail rather than wait. Choosing by
  eye is the view's `c` picker. `luria q5 design --cds` (milestone 9)
  shares this matcher and adds the range form (`--cds 43..345`,
  `--cds c345..43`); in milestone 8 a range is refused with "searching a
  range of bases comes in milestone 9", so the flag's grammar never
  changes under a running script.
- `--circular` (M8-20, owner 2026-10-24, D049) treats the sequence as
  circular. Only FASTA needs
  it, since a GenBank record states its topology, and it is off by
  default so `find` and the view agree unless the user asked otherwise.
  Without it a site across the end is simply not found, and the last line
  says `linear: hits do not cross the end`.
- Without `--cds`, AA mode searches six frames.
- **The output names what it searched, and claims nothing else (D052, owner
  2026-10-24).** `--cds` mode puts the CDS's name and span in the header; six-
  frame mode already carries strand and frame on every row; `--circular` forces
  the topology line. That is not decoration: without it a `0 hits` row is read
  as "this construct has no EcoRI site", which the run cannot know — it looked
  at one molecule, in one mode. A non-zero count is no safer: "no hits" proves
  absence in that sample only, never about the organism or the wider construct.
  Contract 8i carries the exact wording and the test that it appears.
- Output is tab-separated with a header, one row per hit:
  `start end strand` (DNA) or `start end strand frame first_residue
  last_residue` (AA). Positions are 1-based inclusive; a wrapping hit has
  `end < start`. Residue numbers follow §4.5, so `last_residue` can be
  the word `stop`, and in frame mode they count within the frame. At most
  200 rows print (M8-12), and past that the count line reads `showing the
  first 200 of N hits`. The last line is `N hits`. **A search that finds
  nothing is a successful run**: exit 0 with `0 hits` (M8-9, owner
  2026-10-24, D052). luria reserves exit 1 for an internal error, and the
  `grep` habit of exiting 1 on no match would read here as a broken
  program; the usage, malformed-input and design-failure codes stay as
  `docs/cli.md` §5–§6 has them (2, 4, 7).

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
- A bare file needs a terminal on stdin and stdout, checked by the same
  `review::terminal_problem` the other two interactive commands use
  (M8-13): `luria FILE > out` is a usage error with the exit code
  `docs/cli.md` §5 already gives, not a half-drawn screen. The terminal
  check comes before the file is opened, so a bad path and a missing
  terminal can't both claim the error.
- Both file paths are refused before parsing when they are over
  `MAX_RECORD_BYTES` (§4.1, M8-14); `find` shares `open_record` with the
  view, so the two read a file exactly the same way.

## 6. Testing

- **Core (§4)**: unit tests plus fixtures in a new `tests/fixtures/view/`
  with a README. Expected values are written by Python scripts under
  `tests/fixtures/view/reference/`. The scripts use Biopython's
  `Seq.translate` and `reverse_complement` and a plain Python scan for
  IUPAC matching, share no code with `crates/`, and have `--check` modes
  wired into `verify.py`. Two traps the README names, because Biopython's
  defaults are not luria's rules and the scripts must not follow them:
  Biopython's own IUPAC matching is **intersection**-based, so a template
  `N` against pattern `A` matches there and must not match here (M8-7) —
  the script hand-codes §4.4's subset rule; and Biopython writes an
  unreadable codon as `X` where §4.3 writes `?`, and resolves a trailing
  partial codon its own way (§4.3, M8-18). `translate` and
  `reverse_complement` are used only where the rules already agree. Cases:
  - **pBR322** (`tests/fixtures/genbank/raw/J01749.1.gb`, 4361 bp
    circular): `tet` (+) and `bla` (−) usable, the ROP CDS (no `/gene`,
    `GTG` start) unusable. `GAATTC` occurs **nowhere inside** this record
    — it exists only as the wrap at 4359 (`GAA` + `TTC`), so pBR322 proves
    circular search or nothing. Internal palindromic sites are there for
    M8-8: `GGATCC` at 375, `AAGCTT` at 29, `CATATG` at 2295 — **each of
    those four sites occurs exactly once in the whole record**, which
    Biopython confirms and which matches pBR322's well-known single EcoRI,
    BamHI, HindIII and NdeI sites; a palindromic search for `GAATTC` here
    is therefore **one** hit at 4359 with strand `both`, never two. An
    IUPAC
    pattern; `p:` searches for a motif in `tet` and in six frames; base ↔
    residue for the first, last and stop codons of `tet` and `bla`.
  - **The existing synthetic records**, read and never modified (§8):
    `tests/fixtures/genbank/synthetic.gb` (60 bp, circular) already gives
    a CDS joined across the origin on each strand, codons split by the
    origin, and one record holding every refused location (`<1..12`,
    `order(`, a non-origin `join(`, a remote reference) — so §4.1's "not
    drawn" list and M8-1 need no new fixture. `synthetic-linear.gb` is 393
    bp linear.
  - **One new record for the frame edges.** Both existing lengths, 60 and
    393, are multiples of 3, so the "length isn't a multiple of 3" case
    has no fixture yet: `tests/fixtures/view/frames-62.gb`, 62 bp
    circular, a plus-strand CDS and a minus-strand CDS whose codons
    straddle the origin, and 2 bases left over at the far end of every
    frame. New records and TSVs live under `tests/fixtures/view/` and get
    their `--check` wired into `verify.py`; nothing under
    `tests/fixtures/genbank/` is touched.
  - **No existing fixture has an ambiguity code.** Checked across every
    `.gb`, `.fa` and `.fasta` under `tests/fixtures/`: every base is `A`,
    `C`, `G` or `T`, including J01749.1 at 4361 bases. Three now-settled
    rules therefore have no source data anywhere: the degenerate codon
    drawn as `?` (§4.3), a template `N` that must not match pattern `A`
    (M8-7, D052), and the all-ambiguity pattern that hits at every base
    and makes the 200 cap matter (M8-12, D048). 8a1 adds
    `tests/fixtures/view/ambiguity.gb`, a short circular GenBank record
    with IUPAC letters both inside and outside its CDS, and the README
    records which script produced which column. A new file, not an edit of
    an existing one (§8).
  - **Edge cases**: a pattern longer than the sequence; a pattern exactly
    as long as it, on a circular and on a linear record (§4.4); a one-base
    sequence; a template with `N`; overlapping hits; an empty pattern; an
    invalid letter.
- **Disabling a rule**: for each rule in §4, the contract names a test
  that must fail when the rule is removed, and the worker proves it once
  (AGENTS.md).
- **Layout (§5.1)**: pure functions from (record, width, options, block)
  to styled lines. The Python reference emits **column tables**, not
  drawn screens: for a record, width and block, the column of every base,
  base number, residue, residue number and feature-bar cell. Rust then
  asserts, through `TestBackend`, that the line it renders holds the right
  glyph at each of those columns — a residue one column off fails. Drawing
  the expected screen in Python instead would set a second implementation
  of the layout against the first, and would pin box-drawing characters
  and styling into scientific fixtures, which is not what D018 verifies
  and costs review rounds rather than catching defects.
- **Keys, prompts and screens**: on ratatui's `TestBackend`, as for
  `luria review`.
- **`luria find`**: through the binary, against the core fixtures.
- **The terminal session**: by the owner by hand, after 8l2 (§7).

## 7. Contracts (tentative)

Order matters. 8a is the critical path: seven contracts wait on it, its
contract is written here, and §9 is now closed (D048–D052), so all of it is
writable. The code still goes to the worker: `AGENTS.md` gives the planner
the task contracts, the fixtures and the review, and every line under
`crates/` to the worker — including this module. The one exception in this
plan is 8a1, whose fixtures are planner-written by design (they are
expectations, not implementation, and D039 requires them to be derived
without running luria). It still lands in two pieces, and the **CDS-map**
tables land first: §4.2 rests on decided rules alone (D006, D010, D036,
D040, M8-11 and D050's order note), so contract 8c gets its fixtures first
and the worker starts while the frame and search tables of 8a1 are being
verified. Splitting it is also the schedule decision: one contract that
blocks seven is the risk this plan should not carry.

**Full tier** (D051: 8a1, 8a2, 8b–8g, 8j) — one function or module each,
with the process the milestones so far used: named signatures, exact tests
and commands, a "do not" list, planner-written fixtures with
independently verified expectations, and a decision entry for what it
settles.

| # | Contract |
| --- | --- |
| 8a1 | `tests/fixtures/view/`: the Python reference for §4.2–§4.5, the expected TSVs, the new `frames-62.gb`, and the README (planner-written; verified with `verify.py`) |
| 8a2 | `tests/fixtures/view/`: the §5.1 column tables for the layout, same scripts and README |
| 8b | core: `ViewRecord` and `open_record` (§4.1), reason codes, the record ceiling |
| 8c | core: the CDS map (§4.2) |
| 8d | core: frame translation (§4.3) |
| 8e | core: DNA search (§4.4) |
| 8f | core: protein search (§4.5) |
| 8g | core: the overview scale (§4.6) |
| 8j | cli: the block layout (§5.1), pure, tested against the 8a2 column tables — the D050 phase rule lives here, so it is full tier |

**Light tier** (D051: 8h, 8i, 8k1–8k3, 8l1–8l2 — the screen). Contract
size per triage P3; process per D051: a key binding, a colour or an
on-screen sentence is not an M8-n question and gets no decision entry, and
the owner and planner settle those between contracts. What a light-tier
test asserts is behaviour — that `j` moves the selection to the next base,
that the cursor never leaves the view, that opening an unreadable file
refuses with a reason — not that the status line reads `base 4153 of 4361
(1-based, inclusive)`. That difference is the tier's whole point: the
wording is what the hand test will change.

| # | Contract |
| --- | --- |
| 8h | cli: dispatch of a bare file and `find`, the terminal check, the byte ceiling (§5.6) |
| 8i | cli: `luria find`, including `--cds` matching and `--circular` (§5.5) — light on wording, full on behaviour: its tests assert D050's numbering, the `stop` column, the cap, D052's disclosure that the count is about the file searched, and D049's refusal-with-candidates |
| 8k1 | cli: the view's state, cursor movement, blocks and status line |
| 8k2 | cli: go-to (§5.3) |
| 8k3 | cli: the feature list and the CDS picker (§5.2, M8-5, M8-11) |
| 8l1 | cli: the search prompt with its mode rule and cap (§5.4) |
| 8l2 | cli: the overview inset (§4.6, §5.1) |
| 8m | the owner's hand test, then a polish contract as 7m was |
| 8n | docs: `docs/cli.md` §13, the decisions this milestone settled, the product brief's milestone list |

8k as a single contract bundles five user-visible features — state,
keys, cursor, status line, go-to, feature list, CDS picker — and will end
on the 120-turn worker cap (`worker.md`) rather than on a defect. Split
as above, per D051; the same for 8l. 8m and 8n are neither tier: the hand
test is the owner's, and 8n is a documentation contract that lists what was
settled.

## 8. Not in this milestone

See §3 "Out". Also: no new dependency, no change to the request or
result schemas, and no change to existing tests or fixtures — the new
fixtures of §6 are **new files**, and nothing already under
`tests/fixtures/` is edited, moved or re-run into new expectations.
`tests/fixtures/genbank/raw/J01749.1.gb` is a download, pinned by
accession **and version** (`.1`) and byte size, with the fetch command in
the README; a future `J01749.2` is a different fixture, not this one. A
search may not be sold as a safeguard: the hit count is a count of what is
in the file the run was given, and luria says nothing about the organism,
the construct, or a site outside that file (D052). The disclosure wording
lives in contract 8i; §5.5 has the rule.
The refusal reason codes and their matching tests are part of 8b, not a
later contract. The `--cds` flag's grammar is settled — matching included
(M8-15, D049) — and only its implementation waits for 8g. The cap and the
file-size limits are not tunables: a contract may not grow a flag or an
environment variable for them. 8m, 8n and any polish contract are not
permission to fix anything §4–§6 left undecided.

## 9. Questions and answers — closed 2026-10-24

Every row below is answered. The rows keep the planner's reasoning because
the contracts cite it; the bold text is the owner's decision and the
D-number that records it.

| # | Question | Answer |
| --- | --- | --- |
| M8-1 | Features with fuzzy ends (`<1..12`, `1..>900`) are common in real records, and the location parser refuses them. Draw them in the view? | **Owner, 2026-10-24: accepted as proposed.** The view reads `<` and `>` as the stated base and marks the feature as partial in the feature list. Display only; CDS import (D041) still refuses them. **Its own display-only parser**, not a flag on `parse_genbank_location`: a boolean that changes what the shared parser accepts is exactly the shape D009/D036 do not want, and the science parser then stays untouched. Go-to does not jump to a feature whose start is not stated (§5.3). D049. |
| M8-2 | In frame mode on a circular sequence, should frames run across the origin? | **Owner, 2026-10-24: accepted as proposed.** No. Frames are counted from base 1 and stop at the end. A CDS crossing the origin is still translated across it in CDS mode. D052. |
| M8-3 | FASTA has no topology. What should the view assume? | **Owner, 2026-10-24: accepted as proposed.** Linear, shown as `linear (assumed)` in the title, with `T` switching it to circular for the session. Circular changes wrapping search and the overview. D049 records the explicit exception to D005: one visible, changeable, labelled assumption, not a silent inference. It does **not** reword D005 as a whole and does not touch D035 — what an assumed-circular FASTA may be designed on is milestone 9's question. |
| M8-4 | ORF detection (triage T2): in milestone 8 so a FASTA file has something to number, or milestone 9 as planned? | **Owner, 2026-10-24: accepted as proposed.** Milestone 9. Six-frame translation covers looking at a FASTA in milestone 8. Triage T2 stays unapproved. D052. |
| M8-5 | Should the CDS picker show unusable CDSs (such as pBR322's ROP)? | **Owner, 2026-10-24: accepted as proposed.** Yes, greyed, with the reason, not selectable. The reason is 8b's refusal code rendered by the cli (D042), not a second judgement in the ui. Triage T3 (accepting `GTG` starts) would make ROP usable and stays unapproved. D052. |
| M8-6 | A file whose extension and content disagree (a `.txt` that is GenBank)? | **Owner, 2026-10-24: accepted as proposed.** Decide by content: `LOCUS` first means GenBank, `>` first means FASTA, anything else is refused. One core `open_record` does it for the view and for `find` (§4.1). D049. |
| M8-7 | A template `N` (or other ambiguity code): should it match pattern `A`? | **Owner, 2026-10-24: accepted as proposed.** No: a template letter matches only a pattern letter whose base set contains all of its bases (§4.4). Biopython's IUPAC matching is intersection-based and answers the opposite way, so the reference script hand-codes §4.4's subset rule (§6). D052. |
| M8-8 | A palindromic site matches on both strands at the same place. One hit or two? | **Owner, 2026-10-24: accepted as proposed.** One, strand `both`, in the view and in `luria find`. In pBR322 that means its **single** EcoRI site at 4359 is one hit with strand `both`, not two — likewise its single BamHI (375), HindIII (29) and NdeI (2295) sites, each verified to occur exactly once in the record. D052. |
| M8-9 | `luria find` exit code when nothing matches? | **Owner, 2026-10-24: accepted as proposed.** 0 with `0 hits`, like a report rather than `grep`; exit 1 stays what `docs/cli.md` says it is — an internal error. D052. |
| M8-10 | Use the two-tier process (triage P1, P2) in this milestone: full contracts and fixtures for 8a–8g and 8j, lighter contracts for 8h, 8i, 8k1–8k3 and 8l1–8l2, and no decision entries for keys and wording? | **Owner, 2026-10-24: accepted — for milestone 8 only.** The science tier's process is unchanged; keys, colours and on-screen wording get short contracts and no decision entries, so the hand test can change them without approval rounds. Recorded as D051, which names its own scope and expires with the milestone; the rest of the triage (T1–T6, P3–P5) stays unapproved and `AGENTS.md` is untouched. |
| M8-11 | Which CDS, if any, is selected when the view opens? It decides the status line, `w`/`b`, residue numbering and the default for an AA search. | **Owner, 2026-10-24: accepted.** Exactly one usable CDS → select it. Several → select none and say `no CDS selected` in the title bar. Never pick one silently (D005). D048. |
| M8-12 | A cap on hits? An all-ambiguity pattern (`N`, `NNNN`) hits at every base, so an 8521 bp construct gives 8521 hits and `luria find` would print every one. | **Owner, 2026-10-24: accepted.** 200 in both the view and `find`; the count line reads `showing the first 200 of N hits`; exit status unchanged. Core keeps no cap (§4.4, §5.4, §5.5). D048. |
| M8-13 | `luria FILE` when stdin or stdout is not a terminal (`luria FILE > out`)? | **Owner, 2026-10-24: accepted.** Reuse `review::terminal_problem` and the usage exit code `docs/cli.md` §5 gives (§5.6). D048. |
| M8-14 | How big a file will the view and `find` accept? `luria-core` sets no sequence length limit today, so a gigabase FASTA reaches the allocator. | **Owner, 2026-10-24: accepted.** 10 000 000 bases in core, refused by name; the cli refuses on byte length before reading. Both tested at the boundary and one past it (§4.1, §5.6). D048. |
| M8-15 | What does `--cds ID` match — the display name, `/locus_tag`, `/protein_id`? Will it also take a range later (`proposal-commands.md` §3.2)? | **Owner, 2026-10-24: answered — match every id the record offers, and ask when several match; the planner's reading of "ask" for a command with no terminal is accepted: a refusal listing the candidates (id source and span), the view's `c` picker being the interactive way to choose (§5.5).** The flag's grammar is fixed now, ranges reserved for milestone 9 and refused with that message until then. D049. |
| M8-16 | Are overlapping hits reported? `AA` on `AAA` is three hits or one, depending on the answer. | **Owner, 2026-10-24: accepted as proposed.** All of them (§4.4). A search that dropped overlapping positions would hide the site the user came for. D050. |
| M8-17 | Whose residue numbers does a protein hit carry, and what is the number of a hit that runs onto the terminal stop, given D010 says the stop is not a residue? Without this, `luria find`'s AA columns are unimplementable without a guess. | **Owner, 2026-10-24: accepted as proposed.** CDS mode: the CDS's own numbering, and a hit ending on the stop carries none — printed `stop` by `find`, "…to the stop codon" in the view. Six-frame mode: codons counted within that frame from 1, and the strand and frame columns are never empty (§4.5, §5.5). D050. |
| M8-18 | In frame mode, when the length isn't a multiple of 3, is the leftover 1–2 base codon drawn as `?` or not drawn? It is also where the Python reference and Rust will disagree first, since Biopython pads and trims its own way. | **Owner, 2026-10-24: accepted as proposed.** Not drawn at all. `?` means a *complete* codon that cannot be read. The reference script maps Biopython's `X` to `?` and handles the tail by hand (§4.3, §6). D050. |
| M8-19 | When a block starts off the selected CDS's phase, split the codon or snap the block to the phase? | **Owner, 2026-10-24: accepted as proposed.** Snap. The §5.1 fallback means every residue changes which bases it sits on whenever the user single-bases past a block edge, and it doubles the layout fixtures. D050. |
| M8-20 | The view's `T` key makes a FASTA circular for the session, so the same file and pattern can give different hits in the view and in `luria find` — that is exactly the across-the-end case. | **Owner, 2026-10-24: accepted as proposed.** Linear by default in both; `luria find --circular` asks for what the view showed, and the last line names the mode (§5.5). D049 records it; nothing here reopens D005 as a whole, and an assumed-circular FASTA meets D035 again in milestone 9, which is a separate question. |
