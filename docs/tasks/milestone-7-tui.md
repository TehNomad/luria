# Milestone 7 plan: the review-oriented terminal interface

Status: **Approved by the owner, 2026-10-20.** Planner: Claude. Decision:
D034 ("Milestone 7 is the review-oriented user interface (TUI)"). Branch:
`feat/milestone-7`. M7-1 to M7-15 are answered (§4); nothing is open. The answers become D046 in 7h.

## 1. What this milestone is

Milestones 5 and 6 produce two JSON documents: the proposal of
`luria q5 interpret` (`luria.q5-interpretation/1`) and the result of
`luria q5 design` (`luria.q5-result/1`). Both are complete and
deterministic, but a person reviews them by reading JSON. The product
brief's "definition of success" asks that "the interpreted request is
visible and reviewable", that warnings, assumptions and the difference
between computational and experimental validation are "visible rather than
buried", and that "the reviewed result can be exported".

This milestone adds a terminal interface for that review:

```
luria review REQUEST.json
luria review --template FILE.gb (--text TEXT | --text-file FILE)
    --backend openai-compatible --endpoint URL --model NAME
    [--design PROFILE.json] [--allow-remote]
```

The first form runs a request and opens its result. The second interprets
text, shows the proposal (or the questions), and, on the user's explicit
approval, writes the request file and runs it, as `interpret` followed by
`design` would.

The milestone succeeds when everything a result or proposal holds can be
reviewed on screen, the screens add no value that the documents do not
hold, and exporting from the TUI gives byte for byte what the CLI gives.

## 2. What exists today, established 2026-10-20

Facts about the code and the environment, not design choices:
1. `luria_cli::run::run_with_output(path, write_edited)` returns an
   `Outcome` holding the `ResultDocument` struct; `result::render` turns it
   into the exact bytes `luria q5 design` prints. It takes a request
   **path**, not an in-memory request.
2. `luria_cli::interpret::run::interpret(&InterpretInput, &mut dyn Backend)`
   returns an `InterpretOutcome` holding the `InterpretationDocument`. The
   scripted backend of 6f lets it run without a model.
3. `DesignView`, `PrimerView` and `PartView` carry every primer's sequence,
   its template `span`, its `parts` (tail and annealing) with their spans,
   the edit span, Tm, the annealing temperature and the warnings. The edited
   sequence itself is not in the result (`--write-edited` only).
4. `luria_core::sequence::reverse_complement` and `complement_letter` are
   public.
5. Six test files in `crates/cli/tests/` run the binary through
   `CARGO_BIN_EXE_luria`, which cargo provides only to tests of the package
   that owns the binary. Moving `luria` to another package breaks them; a
   separate `luria-tui` crate that depends on `luria-cli` cannot be called
   from `luria-cli`'s binary without a package cycle.
6. **Sandbox.** A probe outside the workspace (`target/tmp/tui-probe`)
   built and ran `ratatui` 0.29 with `crossterm` 0.29, including a
   `TestBackend` terminal. No C compiler step beyond linking build scripts,
   no TLS. Two obstacles, both environmental:
   - `~/.cargo/registry/src` is read-only in the sandbox, so a crate not
     already unpacked there cannot be built (`nono why`:
     `insufficient_access`). The probe used a project-local `CARGO_HOME`
     under `target/tmp/`.
   - Linking a build script writes to `$TMPDIR`, by default `/tmp`, which
     the sandbox makes write-only; the link aborts. `TMPDIR=target/tmp/...`
     fixes it.
7. Dependency counts (`cargo tree -e normal`, unique): the workspace today
   has 29 crates; `crossterm` alone brings about 30; `ratatui` with
   `crossterm` about 70.
8. A worker's shell is not a terminal. No agent can drive the TUI
   interactively; only tests against `TestBackend` and the owner's own
   session can.
9. **Vendoring works** (probe, 2026-10-20). `cargo vendor` (run with a
   project-local cargo home to stand in for the owner's run outside the
   sandbox) and this `.cargo/config.toml`:

       [source.crates-io]
       replace-with = "vendored-sources"

       [source.vendored-sources]
       directory = "vendor"

       [env]
       TMPDIR = { value = "target", relative = true }

   built `ratatui` + `crossterm` from a clean `target/` with the default
   cargo home and no `TMPDIR` in the shell. `[env]` reaches rustc, and so
   the linker it runs; `target` exists before any build script links, so a
   fresh clone needs no extra directory. The vendor directory is **268 MB,
   80 crates**, most of it Windows libraries (`winapi-*-gnu` about 106 MB,
   `windows-sys` about 35 MB), because `cargo vendor` takes every platform.
10. Member names and template ids are core `Identifier`s: printable ASCII,
   no whitespace (`identity.rs`). A name cannot break a tab-separated row.

11. **As vendored by the owner** (checked 2026-10-20): `ratatui` 0.30.2
   with feature `crossterm_0_29`, and `crossterm` 0.29, in
   `crates/cli/Cargo.toml`. The probe used 0.29; the plan does not depend
   on the difference. The workspace now compiles 105 crates (118 over all
   targets), up from 29; `Cargo.lock` lists 200, including optional
   ratatui backends (termwiz, termina) that no target compiles. None of
   the 24 packages locked before changed version. `vendor/` holds 196
   crates, 239 MB, and is ignored. `.cargo/config.toml` is ignored too,
   (M7-14), and at first had no `[env]` line: without
   `TMPDIR` in the shell, a clean build fails linking `serde_json`'s
   build script (`Cannot create temporary file in /tmp/`). The owner then
   added the `[env]` line (M7-14), and from `cargo clean`, with no
   `TMPDIR` in the shell, all five required checks pass: 905 tests,
   1 ignored.

## 3. The trust boundary

The TUI is presentation (D003). These rules make that enforceable:
1. **No biology in the TUI.** Every value shown comes from a
   `ResultDocument`, an `InterpretationDocument`, or a `luria-core`
   function. The only sequence operations are slicing the template at spans
   the result reports and core's `reverse_complement`/`complement_letter`.
   The TUI computes no Tm, no coordinate, no codon and no translation.
2. **What you see is what you export.** Screens render the same struct
   that `result::render` and the interpretation renderer serialize. Export
   writes those functions' bytes, unchanged.
3. **State is always on screen.** Every result screen shows the state
   (`verified`, `needs_clarification`, `failed`) and "Experimental
   validation: not performed". A proposal is labelled as a proposal and is
   never shown with a result's colours. A member's design failure is shown
   as a failure, not hidden or collapsed.
4. **No silent action.** Nothing runs a design except an explicit approve
   key on a `proposed` interpretation. Approving shows the request file's
   path first; the file follows `--write-request`'s rules (template's
   directory, never overwrite). The design then runs **from that file**, so
   the result's request digest names a file on disk.
5. **The TUI edits no request field** (M7-3). Changing what is asked
   means changing the text or the profile and interpreting again.
6. **Privacy unchanged** (D023, D044). The same remote gate, key rule and
   HTTP-only backend. The screen shows the messages sent, from the document.
7. **The terminal is always restored**, on quit, on error and on panic.

## 4. Owner decisions (answered 2026-10-20)

| # | Question | Answer |
| --- | --- | --- |
| M7-1 | Scope | **Results and the interpret flow**: `luria review REQUEST.json`, and `luria review --template … --text …` → proposal → explicit approve → request file written → design from it → result. |
| M7-2 | Terminal library | **`ratatui` + `crossterm`** (about 40 new crates; 29 → about 70). How they reach the sandbox is §9 Q5. |
| M7-3 | Questions in the TUI | **Read-only.** The user edits the text or profile in the TUI and interprets again. No new origin, no change to `build`. |
| M7-4 | Export | **The result JSON, the edited constructs as FASTA, and a primer table in IDT's ordering format** (name, sequence, scale, purification). A new text output, approved here as D041 requires. Details in §5.4. |
| M7-5 | Crates in the sandbox | **`cargo vendor`, run by the owner outside the sandbox**, with its source replacement in a project `.cargo/config.toml`. Probed 2026-10-20 (§2 fact 9). |
| M7-6 | Scale and purification | **A fixed rule by primer length, never a choice on screen.** First stated as `25nm` to 100 nt and `100nm` above; corrected by M7-10 to IDT's published ranges. |
| M7-7 | Delimiter | **Tab-separated.** |
| M7-8 | Primer names | **`<template id>_<member name>_F` / `_R`**, e.g. `pBR322_A42G_F` (confirmed as M7-12). |
| M7-9 | The vendor directory (§9 Q9) | **Ignored for now.** `vendor/` is in `.gitignore`. |
| M7-10 | The scale rule (§9 Q10) | **By full primer length, all `STD`: 15–60 nt `25nm`; 61–90 `100nm`; 91–100 `250nm`.** |
| M7-11 | IDT's codes (§9 Q11) | **From IDT's Bulk Input panel, copied by the owner 2026-10-20** (§5.4). Header row `Name`, `Sequence`, `Scale`, `Purification`. |
| M7-12 | "Original sequence name" (§9 Q12) | **The template id.** |
| M7-13 | Primers over 100 nt (§9 Q13) | **An Ultramer scale is allowed for long insertion tails**: 101–200 nt `4nmU`, `STD`. IDT's Ultramer page (`sg.idtdna.com/jp/site/ultramer.html`, read 2026-10-20) gives 4 and 20 nmole, desalted, 45–200 bases, and 200 as the Ultramer limit. `4nmU` is the smaller of the two desalted scales; `4nmU` over `20nmU` confirmed as M7-15. |
| M7-14 | `.cargo/config.toml` (§9 Q14) | **Ignored**: a setting of this sandboxed checkout. A fresh clone builds from crates.io without vendoring, and the README says nothing about vendoring. It carries the source replacement and the `[env]` `TMPDIR` line of §2 fact 9. |
| M7-15 | The Ultramer scale (§9 Q15) | **`4nmU`.** |

## 5. Proposed shape

### 5.1 Where the code lives

A module `luria_cli::review` inside `luria-cli` (fact 5), with `ratatui`
and `crossterm` as `luria-cli` dependencies (amends D041/D044's dependency
rule). `luria-core` and `luria-nl` are unchanged. Split inside the module:
- `review::app`: the state machine. Pure: screens, selection, key events
  in, actions out (`RunRequest(path)`, `Export(kind)`, `Quit`). No I/O.
- `review::view`: rendering an `App` into a `ratatui` buffer. No I/O.
- `review::context`: the sequence-context rows (§5.3), from the template
  and the result, with core's functions only.
- `review::order`: the primer table of §5.4. Pure: a `ResultDocument`
  and the user's scale and purification in, the file's bytes out.
- `review::terminal`: the thin shell: raw mode, the event loop, restore on
  panic. Not unit-tested; checked by the owner by hand.

### 5.2 Screens

1. **Proposal** (`luria.q5-interpretation/1`): the user's text with each
   quote highlighted, the proposed members (token → CDS, residue, number,
   target, codon), the CDS, composition and numbering, each value's origin
   (`text` or `profile`), the assumptions, the questions with their codes,
   the messages sent and the raw reply. For `failed`, the error and the
   first ungrounded string.
2. **Result overview**: state banner, experimental-validation line,
   template id and digest, target CDS, operation warnings, one row per
   member: name, mutation, warnings count, designed or the failure code.
3. **Member detail**: the mutation (residue, original and new codon, DNA
   interval), edit warnings, then the primer pair: each primer 5'→3' with
   its tail and annealing parts marked, length, Tm and basis, the
   annealing temperature, Tm balance, pair relationship, primer warnings,
   unresolved inputs; or the design failure.
4. **Sequence context**: §5.3.
5. **Provenance**: package, algorithms and versions, request digest,
   template source and digest, buffer, imported and unused GenBank
   features.
6. **Clarification / failure**: for a `needs_clarification` result, the
   clarifications and candidates; for `failed`, the error and its details.

### 5.3 Sequence context

A window of the template around the edit span: the top strand with
coordinates, the forward primer over its span, the reverse primer under its
span written 3'→5' (the primer's own text reversed, so each base sits
under the base it pairs with; nothing is complemented), tails shown off
the template beside their primer, and the edited bases marked.

Changed by the planner in 7d0 (2026-10-21), to be confirmed by the owner:
- A member with no design (every circular template, D035) gets one line
  saying there is no context, rather than the edit span alone: the edit
  span is reported only inside a design, a circular edit can cross the
  origin, and the member screen already lists the edit's coordinates.
- The expected layouts are for three SID-1 designs (`native-01`, `del-11`,
  `ins-14`), not the linear GenBank record: their primer geometry is
  already fixed by independently verified rows of `tests/fixtures/q5-primer/`,
  while the linear record's primers are not. They are in
  `tests/fixtures/review/`, written by `reference/context.py`.

### 5.3a The redesigned member screen (owner, 2026-10-21, M7-16 to M7-22)

After using the TUI, the owner found it showed too much. Replaces §5.2
screens 2 to 4 and §5.3's layout. Answers:

| # | Question | Answer |
| --- | --- | --- |
| M7-16 | What `enter` on a member opens | **The context screen directly**, as the main member view. The technical lines (7c2's member text, provenance) go behind `t`. |
| M7-17 | The context layout | Forward primer above the top strand, the **mutant protein between the strands**, the bottom strand, the reverse primer below it; then one short line per primer (5'->3', length, Tm) and the annealing temperature. No `^` row: lower case marks the edit. The heading is the request as written. |
| M7-18 | Which protein | **Mutant only**, from the edited construct, each residue letter over the middle base of its codon in the CDS's own frame. |
| M7-19 | Indels | **Deletions** show the template with the deleted bases in lower case and a gap in the protein row; downstream residues sit over their own codons. **Insertions** of up to 30 bp are drawn inline between their flanks; longer ones collapse to `[+N bp FIRST10...LAST10]`, and `i` expands them into the sequence, scrolled like the rest; they start collapsed each time the screen opens. |
| M7-20 | Frames and stops | **Shifted frame** downstream of a frameshift. At the first stop codon, `*`, and **nothing after it**. Only **whole codons** get a letter. |
| M7-21 | Who computes it | **luria-core** (`frame::mutant_residues`, 7i1) reads the codons of the edited construct core produced, and returns each mutant residue's number, letter and the edited-construct position of its middle base. The TUI only lays those positions out as screen columns (the same placement arithmetic as 7d), never translating. The edited construct is accepted only when its digest is the member's `mutation.edited.sha256` (D005). Amends §3 rule 1. |
| M7-22 | The overview | **Members only**: the template, the target, one row per member. Warnings and provenance behind keys. |

Fixtures: `tests/fixtures/review/` is rewritten in the new layout by its
script, which translates the edited construct independently with its own
codon table.

### 5.4 The primer table (M7-4)

What IDT documents (its Japanese ordering guide,
`sg.idtdna.com/jP/Site/how_to_order_dna.html`, read 2026-10-20): the bulk
input takes rows pasted from a spreadsheet, the sequence name in column A
and the sequence 5'→3' in column B; scale and purification may follow in
columns C and D. A row with only the first two gets 25 nmol scale and
desalting. The list of accepted scale and purification codes is shown only
inside the Bulk Input pop-up, as an image. The owner copied it from that
panel on 2026-10-20 (M7-11):

| Scale code | Scale | | Purification code | Purification |
| --- | --- | --- | --- | --- |
| `25nm` | 25 nmole | | `STD` | Standard Desalting |
| `100nm` | 100 nmole | | `PAGE` | PAGE |
| `250nm` | 250 nmole | | `HPLC` | HPLC |
| `1um` | 1 µmole | | `IEHPLC` | IE HPLC |
| `2um` | 2 µmole | | `RNASE` | RNase Free HPLC |
| `5um` | 5 µmole | | `DUALHPLC` | Dual HPLC |
| `10um` | 10 µmole | | `PAGEHPLC` | Dual PAGE & HPLC |
| `4nmU` | 4 nmole Ultramer | | | |
| `20nmU` | 20 nmole Ultramer | | | |
| `PU` | PAGE Ultramer | | | |
| `25nmS` | 25 nmole Sameday | | | |

luria writes only `25nm`, `100nm`, `250nm`, `4nmU` and `STD` (M7-10,
M7-13); the rest
are recorded so the source is complete. IDT also refuses a sequence too long
for the chosen scale; that check is IDT's, and luria does not copy it.

Proposed rules:
- One row per primer of every **designed** member, forward then reverse,
  members in result order. A member with a design failure has no row; the
  export screen lists the members left out and why. A result that is not
  `verified`, or has no designed member, exports no table.
- Sequence:
  `PrimerView.sequence` exactly, tail included, upper case. A sequence with
  any letter other than `ACGT` is refused, not exported.
- Name: `<template id>_<member name>_F` and `_R` (M7-8, M7-12). Both
  parts are core `Identifier`s, so no tab or line break can enter a row.
- Scale and purification: a fixed rule by the primer's full length (M7-6),
  stated in D046 with its source, shown on the export screen beside each
  row. Not chosen on screen, not taken from the profile.
  IDT's desalted, tube-delivery ranges (`sg.idtdna.com/jp/site/dna-oligo-t.html`,
  read 2026-10-20) are: 25 nmol, 15–60 bases; 100 nmol, 10–90; 250 nmol,
  1, 5 and 10 µmol, 5–100. Above 100 bases is an Ultramer, a separate
  product with its own scales. The rule (M7-10), every row `STD`:

  | Full primer length | Scale |
  | --- | --- |
  | 15–60 nt | `25nm` |
  | 61–90 nt | `100nm` |
  | 91–100 nt | `250nm` |
  | 101–200 nt | `4nmU` (M7-13) |

  A primer the rule does not cover, under 15 nt or over 200 nt, is refused
  at export with its name and length, never given a guessed scale. The
  test that must fail with the rule disabled: one primer at each boundary
  (15, 60, 61, 90, 91, 100, 101, 200) and one on each side (14, 201).
- Format: tab-separated (M7-7), `.tsv`, UTF-8 (in practice ASCII), one
  `\n` after every row. First row the header `Name`, `Sequence`, `Scale`,
  `Purification` (M7-11), tab-separated like the rows.
- Writing: the same rules as `--write-edited`; an existing file is never
  overwritten. luria writes a file; it never submits an order (product
  brief, non-goals: "an automatic oligo-ordering service").
- Verification: the expected table for the linear GenBank fixture's
  designed result is written by hand from that fixture's hand-verified
  primers, not by running the export.

### 5.5 Testing

- `app`: key sequences on fixed documents, asserting the screen and the
  action produced. Includes: approve is refused unless the state is
  `proposed`; approve never runs a design without a request path.
- `view`: render into `TestBackend` at fixed sizes and assert that given
  strings appear (the state, "not performed", each primer sequence, each
  warning code, each question code). **No whole-screen snapshots generated
  by running the code** (AGENTS.md: expectations are never copied from
  output). Values asserted come from the existing hand-verified fixtures.
- `context`: rows compared with the hand-written layout of §7.
- End to end: the six fixture requests the CLI tests already run, opened
  in `review`, exported, compared byte for byte with `luria q5 design`'s
  stdout.
- Each trust rule of §3 names the test that fails with it disabled.
- Small terminals: below a minimum size the TUI says so rather than
  truncating a sequence silently.

## 6. Contracts (tentative)

| # | Contract | Content |
| --- | --- | --- |
| 7a0 | — | Owner, outside the sandbox: add the two dependencies, run `cargo vendor`, add `.cargo/config.toml` (§2 fact 9) and the `vendor/` ignore line. Planner then checks the required checks pass in the sandbox. Done 2026-10-20; see §2 fact 11. |
| 7a | `milestone-7a-review-skeleton.md` | `review` module skeleton, `luria review` argument parsing, refusal when stdout is not a terminal, terminal restore. |
| 7b | `milestone-7b-app.md` | The pure state machine of §5.1 with its key tests. |
| 7c | `milestone-7c-result-views.md` | Screens 2, 3, 5, 6. |
| 7d0 | `milestone-7d0-context-fixture.md` | Planner: the hand-written context layout and its verification. |
| 7d | `milestone-7d-context.md` | Screen 4. |
| 7e | `milestone-7e-proposal-view.md` | Screen 1. |
| 7f | `milestone-7f-approve-and-run.md` | Approve, write the request, run it, open the result. |
| 7g1 | `milestone-7g1-export.md` | Export of the result JSON and edited FASTA, byte-identical to the CLI. |
| 7g2 | `milestone-7g2-order-table.md` | The IDT primer table (§5.4), after the codes are recorded. |
| 7h | `milestone-7h-docs.md` | `docs/cli.md` §11, README, D046. |

## 7. Fixtures and verification

- No new scientific fixtures: the TUI shows results whose values are
  already verified (M3–M6).
- One new presentation fixture set, `tests/fixtures/review/`: the
  sequence-context layouts of three SID-1 designs from
  `tests/fixtures/q5-primer/` rows (§5.3), recorded in
  `tests/fixtures/README.md`.
- Owner acceptance in a real terminal, from a short checklist in 7h.

## 8. Not in this milestone

- Editing request fields in the TUI, or answering questions from lists
  (M7-3).
- Ordering formats for other vendors.
- A web interface (Milestone 8).
- Reopening a saved result file: results are deterministic, so the TUI
  reopens the request instead. A strict reader for `luria.q5-result/1`
  would be new code with no second consumer yet.
- A "reviewed" or "approved" state stored in any document; D020's states
  are unchanged.
- HTTPS, the Anthropic backend, annotated sites (the two `later-*` files).
- Mouse support, themes, configuration files.

## 8a. Planner choices made during implementation, to be confirmed

- **`--request FILE.json` is required in the interpret form of `luria
  review`** (7f2, drafted 2026-10-21). §1's command line names no place
  for the approved request; §3 rule 4 needs the path shown before
  approval and checked under `--write-request`'s rules. Making it a
  required option states it before the model is called and lets it be
  checked first. The alternative is a fixed name beside the template.
- **Export goes to `--export DIR`, given on the command line** (7g1,
  drafted 2026-10-21), as `<request stem>.result.json`, `<request
  stem>.order.tsv` and `<member>.fa`; nothing is written if any of them
  exists. The edited constructs come from running the request file again
  through `--write-edited`'s own code, after checking that the run gives
  the result on screen.
- **A member with no design has no sequence context** (7d0, §5.3).
- **The context fixtures use SID-1 designs, not the linear GenBank
  record** (7d0, §5.3).

## 9. Questions for the owner

All answered, as M7-1 to M7-15 (§4).

Planner defaults, stated so they can be overridden: the code lives in
`luria-cli` (§5.1); the subcommand is `luria review`; quitting exits 0, and
failures before the screen opens use the existing exit codes; no
confirmation before a loopback model call.
