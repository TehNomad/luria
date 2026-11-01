# Milestone 7k: `luria new`, entering a design in the terminal (plan draft)

Status: **Approved plan, 2026-10-21** (M7-31 to M7-38). Planner: Claude. Parent:
`docs/tasks/contracts/milestone-7-tui.md` (D034: the TUI is built around the same
typed operations). Owner answers so far, 2026-10-21:

| # | Question | Answer |
| --- | --- | --- |
| M7-31 | How input is opened | **A separate `luria new`**, which hands over to the existing proposal and review screens. |
| M7-32 | The template | **A file (FASTA or GenBank) or a pasted sequence.** A paste is saved as a new FASTA file. GenBank: pick a CDS from the record's list. FASTA or paste: the CDS is stated (1-based start–end and strand, or "the whole sequence"); never guessed. |
| M7-33 | The mutations | **Shorthand read by luria, plus free text** sent to the local model as `luria q5 interpret` does. |
| M7-35 | The shorthand | **As §3 drafts it**, 1-based inclusive template positions; residues as 6's notation. |
| M7-36 | Free text without `--backend` | **Shown, disabled**, saying it needs `--backend`. |
| M7-37 | Where files go | **Beside the template** for a file template (as `--write-request` requires); for a paste, the pasted `.fa` and the request both in the **current directory**. Names editable on the form; never replacing a file. |
| M7-39 | Placement on the form | **Shown as a fixed `inline`**, with a note that confine-to-tail is not supported yet (D029: `q5 design` refuses it); the overlap stays editable. Found when the planner ran the 7k2 fixtures through `q5 design`. |
| M7-40 | Words and shorthand | **Either one, not both** (2026-10-22). Words go through `interpret` and the existing proposal screen, as `luria review --template` does; they need a GenBank template (M6-7). Supersedes §2 rule 1's "joined" wording. |
| M7-41 | Words and the form's values | **The form's overlap and codon choice are the design words are read with** (2026-10-22), so a words proposal needs no `--design`. Found in the planner's code review (7k9). |
| M7-42 | Words and numbering | **Residues count from the CDS start** unless the words say otherwise (2026-10-22), as the shorthand does (7k9). |
| M7-43 | Ctrl-C | **Quits from any screen, writing nothing** (2026-10-22), in `luria new` and `luria review` (7k10). |
| M7-44 | Overlap too large | **A blocker on Overlap**, not a panic or a wrong value (2026-10-22, 7k10). |
| M7-38 | Branch | **Continue on `feat/milestone-7`.** |
| M7-34 | Choices on the form | **Primer placement** (`inline` or `confine_to_tail`) with the minimum annealing overlap, and **codon choice** (the default policy, or a typed codon). Composition is not offered: several mutations are always separate variants, the only composition `q5 design` accepts. |

## 1. What the user does

```
luria new [--design PROFILE.json] [--backend … --endpoint URL --model NAME [--allow-remote]]
```

One form, then the screens that exist already:

```
╭ New design ──────────────────────────────────────────────────────────────╮
│ Template   ◉ file  ○ paste                                               │
│   File     sid-1_cDNA.fa                                                 │
│            sid-1, 3153 bp, FASTA                                         │
│   Shape    ◉ linear  ○ circular                                          │
│   CDS      ◉ whole sequence  ○ bases [     ]–[     ]  ◉ forward ○ reverse│
│                                                                          │
│ Mutations  D404N                                                         │
│            del 1210-1212                                                 │
│            ✓ residue 404 D→N, codon by default policy                    │
│            ✓ delete bases 1210–1212                                      │
│ In words   (only with --backend)                                         │
│                                                                          │
│ Primers    inline (confine to tail not supported yet)   overlap [15] nt  │
│ Codon      ◉ default policy  ○ type per mutation (D404N:AAC)             │
│ Save as    sid-1-d404n.json                                              │
╰──────────────────────────────────────────────────────────────────────────╯
 tab next  enter review  ? help  esc quit
```

`enter` builds the request and shows it on the existing proposal screen
(7e/7f2): the request exactly as it will be written, and where each value
came from. `a` then `y` writes it (and the pasted FASTA, if any) and runs
the design; the result opens on the review screens of 7i/7j, with
export. Nothing is written or run before `y` (plan §3 rule 4).

## 2. Rules

1. **luria builds the request, never the model alone.** Shorthand is read
   by deterministic Rust code with a live, line-by-line preview; a line it
   cannot read is shown in red with the reason and blocks `enter`. Free
   text goes through 6's interpret path unchanged (grounding, questions,
   refusals); its mutations join the shorthand's, and a mutation given
   both ways is asked about, not merged silently.
2. **Nothing scientific is defaulted silently.** Every value the request
   needs is on the form, visible, before `enter`; one that is empty
   blocks `enter` with a message saying which. A `--design` profile
   fills the form's fields, and the proposal's origins say so.
3. **Coordinates typed by the user are 1-based inclusive** (M7-24), and
   converted to the request's zero-based half-open form by one tested
   function. Residue numbers count from the CDS's first codon
   (`from-cds-start`).
4. **The CDS is the user's.** For FASTA and paste it is stated on the
   form; for GenBank it is picked from 6c's catalogue (id, gene, product,
   usable or why not). An unusable CDS cannot be picked.
5. **A paste is a file.** On `y`, a pasted sequence is written as a new
   FASTA file under `--write-request`'s rules (beside the request, never
   replacing a file, atomic); the request names it with its SHA-256, so
   the review checks it as it checks any template (D005).
6. **Same code paths.** The request is checked by `request.rs` exactly as
   a file would be, the design runs through `run_with_constructs`, and
   the screens after `enter` are the existing ones.

## 3. The shorthand (proposal)

One mutation per line; names are generated (`D404N`, `del-1210-1212`,
…) and shown, editable on the proposal screen only by changing the line.

| Line | Meaning | Request edit |
| --- | --- | --- |
| `D404N` | residue 404 D→N, codon by the form's codon choice | `residue`, `policy: default` |
| `D404N:AAC` | the same, codon `AAC` | `residue`, `codon` |
| `1210 G>A` | base 1210 G→A | `substitution` |
| `1210-1212 GAT>AAC` | bases 1210–1212 replaced | `replacement` (a request's `substitution` is one base only, so any range is a replacement, as `native-01` is) |
| `del 1210-1212` | delete bases 1210–1212 | `deletion` (the `from` bases read from the template and shown) |
| `ins GAT after 1209` | insert after base 1209 | `insertion` (`before`/`after` read from the template and shown) |

`p.` prefixes and three-letter codes are accepted as 6's notation does
(`notation.rs` is reused for residues). A stop (`*`), frameshift
notation or a range of residues is refused with a reason.

## 4. Contracts (one worker at a time)

| # | What | Tested by |
| --- | --- | --- |
| 7k1 | The shorthand parser: lines to typed edits, 1-based conversion, reasons | unit tests, a fixture table of lines and expected edits written by the planner |
| 7k2 | `NewForm` state: fields, focus, typed text, paste, validation messages; pure | unit tests |
| 7k3 | Building a request document from a valid form (FASTA, GenBank, paste) | request round-trip through `request.rs`, fixture requests |
| 7k4 | The form's text and drawing, in 7j2's frame | test backend |
| 7k5 | `luria new` wiring: args, the confirmation screen, writing the paste and request, hand-over to review (shorthand only) | args tests; the loop by the owner's hand test |
| 7k6 | Docs: `docs/specs/cli.md` §12, D047 | documented-usage test |
| 7k7 | Free text with `--backend`: the "In words" box through 6's interpret path, its mutations joined to the shorthand's (planner split from 7k5, 2026-10-22, to keep 7k5 reviewable) | interpret corpus cases through the scripted backend |

## 5. Open questions

None. Planner defaults, to be confirmed later: generated member names
(`D404N`, `G1210A`, `del1210-1212`, `ins1209GAT`); the form's own default
for the overlap is **empty**, filled only from `--design` or
by the user (rule 2).
