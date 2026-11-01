# Milestone 7m: the owner's first hand test, as changes

Status: **Implemented, 2026-10-22** (7m1, 7m1c, 7m2 to 7m6; the terminal session awaits the owner's hand test). Planner: Claude.

The owner tried `luria new` and `luria review` in a terminal and asked for
the changes below. The planner's questions and the owner's answers are
recorded as M7-45 to M7-58. Where an answer reverses an earlier one, the
row names it.

## 1. Decisions

| Item | Topic | Answer (owner, 2026-10-22) |
| --- | --- | --- |
| M7-45 | Overlap | **Prefilled with 15 nt**, shown and editable; a `--design` profile's value replaces it. Its place in the focus order is its place on screen. Reverses M7-34's "empty until given". |
| M7-46 | Codon | **No codon field.** A line with `:CODON` uses that codon; a line without one uses the default codon policy. |
| M7-47 | Words | **One Mutations box.** A line luria cannot read at all (refusal `unreadable`) makes the whole box words. With `--backend` the box goes to the model; without it, it is a blocker that names `--backend`. A line luria reads but refuses (a stop, wrong bases, position 0, …) stays a red error and is never sent to the model. Words still need a GenBank template. Replaces M7-36's and M7-40's separate "In words" box. |
| M7-48 | Approval | **A pop-up on the form** asks for the request's file name (and shows the pasted FASTA's name). `enter` writes and runs it, and `esc` closes it. The confirmation screens go. With words, the proposal screen stays, so the model's reading and questions are seen; its approval opens the same pop-up. |
| M7-49 | Overwrite | **Ask, then replace.** When a file `luria new` or the export would write exists, a prompt names it and `y` replaces it; any other key writes nothing. `luria q5 interpret --write-request` and `luria q5 design --write-edited` still never replace. This narrows D046's and D047's "a file is never replaced". |
| M7-50 | File picker | **A filtered list under `File`**: `.fa .fasta .fna .gb .gbk .genbank` files and folders in the directory being typed, filtered as you type, `↑`/`↓` to pick, `enter` to choose. `tab` still moves on. |
| M7-51 | Form page keys | **`PgUp`/`PgDn`**: inside the paste or mutation box, the cursor moves 10 lines; elsewhere, focus jumps to the first or the last field. |
| M7-52 | Provenance | **The Provenance screen goes**; its lines move to the bottom of Details, so they are still one key away. The `p` key goes. |
| M7-53 | Help | **`?` shows the open screen's own keys**, and the sequence legend only on the variant screen. |
| M7-54 | Colours | **Terminal colour names**, which follow the user's theme, mapped to the owner's palette: edited bases yellow bold, protein magenta, annealing primer bases light blue, primer tails yellow, a stop and errors red bold, warnings yellow. Red and green are never the only difference between two things. The form's `✓` is light blue. |
| M7-55 | Primer table | **`nt` and `°C` are not coloured**; the annealing temperature is bold. |
| M7-56 | Details | **Scrolling stops at the bottom**, so `k` moves at once; on every scrolled screen. The stale `c context` and `esc back` lines go. |
| M7-57 | Export names | **`<template id>_<member>.fa`**, e.g. `sid-1_cDNA_D404N.fa`, as the order table already names primers. |
| M7-58 | Export keys | **`w` or `o` writes, `esc` goes back**, as nano and vim do. |
| M7-59 | Words and the CDS | (owner, 2026-10-22, after the backend hand test) **In `luria new`, words that name no CDS use the CDS picked on the form**, recorded as an assumption; words naming a different CDS than the form's are asked about. M6-9 still holds for `luria q5 interpret` and `luria review --template`, which have no CDS field. Contract 7m7. |
| M7-60 | Words preview | (owner, 2026-10-22, after the backend hand test) **When the model's reading can be approved, `luria new` runs the design in memory and shows the variant screens as a preview**: primers, both strands and the mutant protein in frame, so the mutation is checked before saving. Nothing is written until `a` and the save pop-up. This narrows D046's "nothing runs until you approve" to "nothing is written until you approve" for `luria new` only; `luria review --template` is unchanged. A reading with questions or an error still opens on the proposal text. Contracts 7m8a, 7m8b. |
| M7-61 | Proposal details | **The model's reading (questions, request, origins, quotes, prompt, reply) moves behind `d`**: in the preview it is at the bottom of each variant's Details screen, as M7-52 put the provenance. |
| M7-62 | Words flow | (owner, 2026-10-22, after the preview hand test) **A words reading with no questions is written and run at once**, under the form's file name, and the result screens are where the mutation is checked. If the file exists, a prompt names it and only `y` replaces it. A reading with questions or an error is shown and writes nothing. Supersedes M7-60's preview. Contract 7m9a. |
| M7-63 | Shorthand flow | **The same for mutation lines**: `[ Save and run ]` writes and runs at once; the name pop-up goes, and `Save as` is a form field again, after Overlap. The only pop-up left asks before replacing a file. Supersedes M7-48's name pop-up. Contract 7m9a. |
| M7-64 | Model reading | **At the bottom of each variant's Details screen** on the result screens, as M7-61 put it. Contract 7m9a. |
| M7-65 | Preview code | **The preview screens go** (title, `a` key, preview help); 7m8a's in-memory run goes too, as nothing else uses it. Contracts 7m9a, 7m9b. |

## 2. Contracts, in order

1. `7m1`, the review screens: M7-52, M7-53, M7-54, M7-55, M7-56, and
   M7-58's keys.
2. `7m2`, export: M7-57 and the export half of M7-49.
3. `7m3`, the form's fields: M7-45, M7-46, M7-47 and M7-51.
4. `7m4`, saving: M7-48 and the `luria new` half of M7-49.
5. `7m5`, the file picker: M7-50.
6. `7m6`, docs: `docs/specs/cli.md` §11 and §12, D046 and D047.

One worker at a time, because each contract changes tests the next one
runs.
