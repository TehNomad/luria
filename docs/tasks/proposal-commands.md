# Proposal: luria's commands for the sequence-first workflow

Status: **Draft for the owner, 2026-10-23. Not approved and not a task
contract.** If approved, this replaces the command list in `docs/cli.md`
§1 and amends D041 (output), D046 and D047 (`review`, `new`).
Companion to `proposal-product-direction.md` and
`proposal-sequence-view.md`.

## 1. What the commands have to cover

The new workflow has two kinds of user and one file between them:

- **At the bench:** open a construct, find the place, add edits, see
  primers, export the order. This is interactive, and should take one
  command to start.
- **Scripting and records:** design from the command line (a residue scan
  over 40 positions, for example), re-run a saved design, export from it.
- **The design file** connects the two. It's what the view saves and
  what scripts read: the template's path and digest, the CDS, the
  variants, and the design settings. Today that is the
  `luria.q5-request/1` request, unchanged. A result is never stored as
  the record: it is recomputed from the design file, which is
  deterministic (D012), so a saved design can't drift from its primers.

## 2. Today, and what goes wrong

| Today | Problem in the new workflow |
| --- | --- |
| `luria new` | A form that comes before the sequence. The view replaces it. |
| `luria review REQUEST.json` | A second interactive program with its own screens. Reopening a design should go back to the same view the user built it in. |
| `luria review --template … --text …` | Natural language, now frozen (direction §3). |
| `luria q5 design REQUEST.json` | Fine for scripts, but prints JSON only (D041), and needs a hand-written request. |
| `luria q5 interpret …` | Frozen. |

## 3. Proposed commands

Owner answers, 2026-10-23: keep a workflow level, because NEBuilder
assembly is planned; the design file's name is left as it is
(`*.request.json`); the design command prints a table; `luria find` is in;
other commands (`translate`, `orfs`) later.

### 3.0 The rule: generic verbs, workflow verbs

- **Generic commands** work on any sequence or any saved design, whatever
  the workflow: viewing, searching, running and exporting. A design file
  already names its workflow in its schema (`luria.q5-request/1`; later
  something like `luria.nebuilder-request/1`), so `run` and `export` read
  it from there and need no workflow in the command.
- **Workflow commands** create something only one workflow understands,
  so they sit under the workflow's name: `luria q5 design` takes
  mutations; a later `luria nebuilder design` would take fragments and a
  vector. Their arguments have nothing in common, so one shared `design`
  command would only get confusing.

```
luria                                   start screen
luria FILE                              open a sequence (.gb .gbk .fa .fasta, later .dna)
luria DESIGN.json                       reopen a saved design in the view
luria find FILE PATTERN [--aa] [--cds ID]
luria run DESIGN.json [--json]          any workflow, read from the file
luria export DESIGN.json [--order FILE.tsv] [--genbank DIR] [--fasta DIR]

luria q5 design FILE MUTATION... [--cds ID] [--overlap N]
                [--save DESIGN.json] [--order FILE.tsv] [--json]
luria q5 design REQUEST.json            today's form, kept (= luria run --json)
luria q5 interpret ...                  frozen, as today

later: luria nebuilder design ...

luria version
luria help [COMMAND]
```

`luria q5 design` tells its two forms apart by the first argument: a
`.json` file is a saved design, and anything else is a sequence followed
by mutations. A sequence file is never `.json`, so the two can't be
confused, and the existing CLI tests and fixtures stay as they are.

### 3.1 Interactive: `luria`, `luria FILE`, `luria DESIGN.json`

**Why a bare file, with no `open`.** On Linux, a program whose main job is
showing a file takes the file directly (`less FILE`, `vim FILE`). Tools
with subcommands (`git`, `cargo`) use them only. Tools that are both
viewers and have a few subcommands, such as `bat` (`bat FILE`, `bat
cache`) and `tig` (`tig`, `tig log`), do both: a bare argument is a file,
unless it is a subcommand's name. luria is that kind of tool, so:

- A first argument that is a subcommand's name (`find`, `run`, `export`,
  `q5`, `version`, `help`) is the subcommand. Anything else is a file to
  open.
- A file whose name clashes with a subcommand is opened as `./find`, as
  with `bat`. No separate `open` command is needed. A first argument that
  is neither a subcommand nor an existing file is an error that suggests
  the closest subcommand (`luria fnd` → "did you mean `luria find`?"), so
  a typo never looks like a missing file.

The interactive forms:

- **`luria`** alone opens a start screen: open a file (with the filtered
  file list `luria new` has now), paste a sequence, or pick a recent
  design from the current directory.
- **`luria FILE`** opens the sequence view on a construct
  (`proposal-sequence-view.md`).
- **`luria DESIGN.json`** opens the view on the design's template, with
  its CDS selected, its variants in the side pane, and their primers
  designed and drawn. This replaces `luria review REQUEST.json`. The
  details, provenance and export screens move into the view as panes or
  pop-ups.

Inside the view: `s` saves the design file beside the template,
prompting before it replaces a file (D047 M7-49). `x` exports, as
`luria review` does now. When NEBuilder exists, the view gets a second
way to act on a selection; the view itself is shared.

### 3.2 Scripting: `luria q5 design`

The shorthand from `luria new`, on the command line, so scripts never
have to write request JSON:

```
luria q5 design pET28-SID1.gb D404N V405A:GCC --cds sid1
luria q5 design pET28-SID1.gb "del 1210-1212" "ins GGSGGS after 1209" --cds sid1
luria q5 design sid1.fa D404N --save d404n.request.json --order d404n.tsv
```

- Each argument is one variant, as each line is in `luria new` (D047).
- **Output is a table for people** by default: variant, primer, sequence
  5′→3′, length, Tm, annealing temperature, warnings. `--json` prints the
  result document, as `luria q5 design REQUEST.json` does now. This amends D041's
  "JSON only" rule.
- `--cds` may be left out when the file has exactly one usable CDS, and
  the output says which one was used (triage T4). With several, a residue
  edit is refused, and the message lists the CDS ids. Base-position edits
  never need one.
- A FASTA file has no CDS. It can be given as `--cds 1..3153` (1-based,
  strand `+`) or `--cds c3153..1` (minus), or picked from detected ORFs
  in the view (triage T2).
- `--save` writes the design file, so `luria DESIGN.json` opens what was
  just designed on the command line.

### 3.3 Scripting: `luria run`, `luria export`, `luria find`

- **`luria run DESIGN.json`** runs any saved design, choosing the workflow
  from the file's schema. It prints a table by default and the result
  document with `--json`, as `luria q5 design` does. Exit codes are
  unchanged (`docs/cli.md` §5).
- **`luria export DESIGN.json`** writes the files the view's `x` writes,
  without a terminal. `--genbank DIR` writes each edited construct as
  GenBank with its primers as `primer_bind` features (milestone 10).
- **`luria find FILE PATTERN`** prints the search hits the view would
  show, with the same rules (`proposal-sequence-view.md` §4): DNA with
  IUPAC codes unless the pattern has a letter that can't be DNA, or
  `--aa`. Cheap, since the search lives in luria-core, and useful for
  checking a primer against a plasmid.

## 4. What happens to the old commands

| Old | New | How |
| --- | --- | --- |
| `luria new` | `luria` (start screen) and `luria FILE` | Removed when milestone 9 lands. |
| `luria review REQUEST.json` | `luria REQUEST.json` | Removed when milestone 9 lands. |
| `luria q5 design REQUEST.json` | unchanged | Gains the `FILE MUTATION...` form beside it. |
| `luria q5 interpret`, `luria review --text` | `luria q5 interpret` | Frozen, left out of the main help, until structure-aware natural language is designed (direction §3). `luria review --text` goes with `review`. |

luria has no released users, so nothing here needs a long deprecation.
The request and result schemas don't change, and neither do the
existing CLI tests.

## 5. Order

| Milestone | Commands |
| --- | --- |
| 8 | `luria FILE` (read-only view), `luria find` |
| 9 | `luria` start screen, `luria DESIGN.json`, `luria q5 design FILE MUTATION...`, `luria run`; remove `new` and `review` |
| 10 | `luria export`, GenBank export |
| later | `luria translate`, `luria orfs`, `luria nebuilder design` |

## 6. Owner answers

Both answered by the owner on 2026-10-23: the bare-file rule of §3.1
(subcommand names win, `./NAME` for a clash, no `open`) is approved, and
so is the split of §3.0 (generic `run`, `export` and `find`; workflow
commands such as `q5 design`).
