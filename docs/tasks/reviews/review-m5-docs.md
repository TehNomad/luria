---
type: Task Contract
title: "Review M5 docs: the CLI page, the GenBank fixture notes and D030's evidence"
description: "docs/specs/cli.md says template.sha256 is checked against the file's digest; it is checked against the SHA-256 of the normalized sequence."
status: deprecated
tags: [milestone-5]
---
# Review M5 docs: the CLI page, the GenBank fixture notes and D030's evidence

Status: **Implemented 2026-10-18** by the local worker; accepted after two small review rounds (the CLI codes' `details`, the verify script comment). Planner: Claude. Parent:
[docs/tasks/reviews/review-post-m5.md](/tasks/reviews/review-post-m5.md): B-2 to B-6, C-5, C-6, C-11, the owner
answers of 2026-10-18 (B-2, B-3, B-6, C-6, D030), and reviewer A's doc drift.
Docs and comments only: no code, test or fixture value changes.

Files:
- [docs/specs/cli.md](/specs/cli.md)
- `README.md` (one sentence, §2.2)
- [docs/decisions/](/decisions/) (D030's Evidence, exact text in §2.3)
- `tests/fixtures/genbank/README.md`
- `tests/fixtures/genbank/reference/verify_synthetic.py` (one comment)
- `crates/core/src/genbank_location.rs` (module doc comment only)
- `crates/core/tests/genbank_fixtures.rs` (module doc comment only)

Before writing each item, read the code it describes and make the text match
the code. If the code disagrees with this contract, stop and report.

## 1. What is wrong

- **B-2.** [docs/specs/cli.md](/specs/cli.md) says `template.sha256` is checked against "the
  file's digest". It is checked against the SHA-256 of the **normalized
  sequence** (upper-case bases, line breaks and whitespace removed): the
  value `provenance.template.sha256` reports, not `sha256sum` of the file.
  `provenance.requestSha256`, by contrast, is over the request file's raw
  bytes. (Owner, 2026-10-18: keep the content digest; fix the docs.)
- **B-3, C-6.** §5 says exit 1 is "an internal error or an I/O failure". An
  unreadable request or template file is `request_unreadable` /
  `template_unreadable`, category `malformed_input`, exit 4 (owner: keep
  that). Exit 1 is an internal error or a failure to write output, either
  stdout or a `--write-edited` file. On exit 1 the document is still printed
  to stdout when it can be, and a one-line `luria: …` note goes to stderr.
  When a `--write-edited` write fails, exit 1 replaces the exit the run
  would otherwise have had, 7 included (owner, 2026-10-18); the document
  still shows each member's `design` or `designFailure`.
- **B-4, C-5.** §6 says four codes are the CLI's own. There are six:
  `template_unreadable` and `output_name_unusable` are missing.
  `output_name_unusable` is given when a result name holds a path separator
  (`/` or `\`) or one of `:` `*` `?` `<` `>` `|`, or when two result names
  differ only in case. All three are checked only with `--write-edited`, and
  nothing is written. `output_exists` also covers a path that holds a
  symlink, a dangling one included. Read `crates/cli/src/request.rs`
  (`RequestErrorCode`), `prepare.rs` and `run.rs` for each code's exact
  condition.
- **B-5.** §6 says `rule` is set only for `invalid_mutation_request` and
  `invalid_primer_candidate`. It is set for six codes: those two plus
  `malformed_fasta`, `unusable_genbank_location`, `malformed_genbank` and
  `unusable_genbank_cds`. Confirm the list against `rule_code` in
  `crates/cli/src/result.rs`.
- **B-6.** §6 says `path` is given "for an error the request itself caused".
  It is given only for errors found while the request is read and prepared.
  An error raised while the edit runs has `path: null`, even when the
  request caused it (for example `expected_residue_mismatch`). Mapping those
  errors to request fields is planned for Milestone 6.
- **C-11.** §6 says `details` holds "the error's own fields". Four payload
  fields that hold only interval text are left out of `details`, and
  `primer_pair_coverage` has empty `details`; their intervals are in
  `message` only. Read `error_details` in `result.rs` (its doc comment says
  which four) and name the codes concerned.
- **Undocumented behavior:**
  - a `--write-edited` FASTA file's header is the result name, and the file
    records no topology;
  - `provenance.template.genbank.importedCds` entries now carry `intervals`
    and `strand` (C-12), and a refused CDS location's `details` carry `cds`
    and `line` (A-3);
  - a GenBank CDS is imported only from an ATG start, under every
    `/transl_table` (D042);
  - a buffer above 1000 mM monovalent salt, 10000 nM primer or 20 % DMSO is
    refused as `invalid_buffer_model` (D043).
- **README.** It says [docs/specs/cli.md](/specs/cli.md) has "the request and result schemas",
  but [docs/specs/cli.md](/specs/cli.md) §9 says the full reference isn't written yet.
- **Reviewer A's drift:**
  - `verify_synthetic.py` says Biopython warns about "the remote and
    between-base locations, which are in the record on purpose".
    `synthetic.gb` has a remote location but no between-base (`^`) one.
  - `tests/fixtures/genbank/README.md` says the record has "one per refused
    qualifier and location form". The location refusals `between_bases`,
    `position_zero`, `range_ends_before_start`, `past_sequence_end` and
    `syntax`, and the `/pseudogene` qualifier, are tested only in the unit
    tests of `genbank_location.rs` and `genbank_cds.rs`, not through the
    record.
  - `genbank_location.rs`'s module doc, step 1, says "Every ASCII
    whitespace character" and then lists four. Form feed is ASCII
    whitespace and is not removed; it gives `Syntax`. Describe what the code
    does.
  - `genbank_fixtures.rs`'s module doc shows
    `target/venv/bin/python -I tests/fixtures/verify.py`; every other place
    shows `python -I tests/fixtures/verify.py` run in the pinned
    environment.

## 2. Required change

### 2.1 [docs/specs/cli.md](/specs/cli.md)

Fix each item in §1 in the section where the topic already lives:
- the `template` row of §7 and the `template_digest_mismatch` row of §6
  (B-2);
- the §5 table's exit-1 row, plus a short paragraph under the table (B-3,
  C-6);
- §6's CLI-code table, the "Four codes" sentence, and the `rule`, `path` and
  `details` rows (B-4, B-5, B-6, C-11, C-5);
- §7's `design` row, for D043's limits;
- §8's `provenance` row, for `importedCds`;
- the GenBank sentences, wherever the page describes GenBank import (D042),
  or §7's `template` row if it has none;
- the `--write-edited` description in §1 or §3.1 (header and topology).

Keep the page's existing style: short table cells, plain sentences. Don't
restructure sections, and don't touch the worked example's JSON. Add no new
section except, if needed, one short paragraph under §5.

### 2.2 `README.md`

Change "See [docs/specs/cli.md](/specs/cli.md) for the request and result schemas, a worked
example and the exit codes" so it no longer claims full schemas. For
example: "See [docs/specs/cli.md](/specs/cli.md) for a worked example, the exit codes and error
codes, and a summary of the request and result".

### 2.3 [docs/decisions/](/decisions/), D030

In D030's Evidence, in the "Annealing temperature" bullet, insert this
sentence right after "…so the quotation rests on the owner's reading." and
before "The 72 °C cap…":

```
NEBaseChanger's own SID-1 D404N result, captured in
`tests/fixtures/example-construct/raw/sdmsummary.txt`, shows the same rule
independently of the changelog: Tms of 74 and 70 °C and a Ta of 71 °C, the
lower Tm plus 1 (plus 3 would give 73).
```

Rewrap the bullet's lines to the file's width; change no other words.

### 2.4 The GenBank fixture notes and comments

Make the four drift fixes in §1, describing what the code and the record
actually do. In `tests/fixtures/genbank/README.md`, replace the overstated
claim with what `synthetic.gb` holds, and add one sentence naming the
refusals that are tested only in the unit tests.

## 3. Checks

No test should change. From the repository root, after
`export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace` (the documented-example test reads
  [docs/specs/cli.md](/specs/cli.md); it must still pass)
- `cargo clippy --workspace --all-targets -- -D warnings`
- `.tmp/tm/venv/bin/python -I tests/fixtures/verify.py`
- `git diff --check`
- `git status --short`
- Confirm with `git diff` that every change to a `.rs` or `.py` file is in a
  comment.

## 4. Do not

- Change code, tests, fixture values or the worked example's JSON.
- Edit any other file.
- Describe behavior you haven't confirmed in the code.

## 5. Handoff

Report the changed files, a one-line summary of each fix with the section it
went into, anything in §1 the code contradicted, and the check results.
