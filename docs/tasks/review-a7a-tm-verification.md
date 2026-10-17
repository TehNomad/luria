# Review A7a: track the Tm verification scripts, with `--check` (D039)

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review. All three checks pass; one label in crosscheck.py was aligned with tm_crosscheck.rs.
Planner: Claude. Parent: `docs/tasks/review-post-m4.md` A7 and C3. Decision:
D039.

## 1. Requirement (D039, inline)

> Every script that produced or verified a fixture expectation is tracked
> next to the fixture it serves, under that fixture's `reference/` directory.
> The Python dependencies are pinned in `tests/fixtures/requirements.txt`.
> Every generator has a `--check` mode. It regenerates in memory, compares
> with the tracked file, and exits non-zero on any difference, without
> writing anything. […] None of this is part of the required cargo checks
> (D026).

## 2. Today

These scripts live only in the git-ignored `.tmp/tm/`:
- `verify_imm.py`: checks the mismatch ΔH°/ΔS° from Biopython's `DNA_IMM1`
  against the review's ΔG°37, and **writes** `tests/fixtures/tm/nn-mismatch.tsv`
  through a path relative to `.tmp/tm/`;
- `crosscheck.py`: prints the Biopython Tm values that
  `crates/core/tests/tm_crosscheck.rs` hard-codes in `EXPECTED`.

Nothing checks `nn-matched.tsv`'s ΔH°/ΔS° against Biopython's `DNA_NN4`; the
audit did that by hand.

## 3. Required changes

The venv `.tmp/tm/venv` has Biopython 1.88 and numpy 2.5.4. Run every script
as `.tmp/tm/venv/bin/python -I <script> …`. Put scratch files under
`target/`, never `/tmp`.

1. **`tests/fixtures/requirements.txt`**: `biopython==1.88` and
   `numpy==2.5.4`, one per line, with a comment naming D039.
2. **`tests/fixtures/tm/reference/verify_imm.py`**:
   - copy it from `.tmp/tm/`;
   - derive its paths from `__file__`, so the output is `../nn-mismatch.tsv`
     relative to the script;
   - collect the TSV text in a string instead of writing line by line;
   - without `--check`, write it, as today;
   - with `--check`, compare it byte for byte with the tracked file, print
     `OK nn-mismatch.tsv` or `DIFFERS nn-mismatch.tsv` followed by the first
     differing line, exit 1 on a difference, and write nothing;
   - keep its verification printout and its algorithm unchanged.
3. **`tests/fixtures/tm/reference/verify_matched.py`** (new). Read
   `../nn-matched.tsv` and check:
   - each of the 10 Watson–Crick motifs' ΔH° and ΔS° equals Biopython's
     `MeltingTemp.DNA_NN4` entry for that motif;
   - `initiation`, `terminal_AT` and `symmetry` equal `DNA_NN4`'s `init`,
     `init_A/T` and `sym`.

   Read Biopython's key spelling from its source; don't assume it. Print one
   `OK` line, or every mismatch, and exit 1 on any mismatch. `--check` is
   accepted and means the same thing. It writes nothing.
4. **`tests/fixtures/tm/reference/crosscheck.py`**:
   - copy it from `.tmp/tm/` and keep its computation;
   - with `--check`, read `crates/core/tests/tm_crosscheck.rs` (path from
     `__file__`), extract the label and value of each of the six `EXPECTED`
     entries, and compare each with the script's value rounded to 3 decimals.
     Exit 1 on any difference or on a missing label.
   - If the Rust table can't be parsed reliably with a small regex, stop and
     report.
5. **`tests/fixtures/tm/README.md`**: replace every `.tmp/tm/…` reference
   with the tracked script, and say how to run each one. Remove the sentence
   saying the scripts aren't tracked.
6. **`crates/core/tests/tm_crosscheck.rs`**: the module doc comment names
   `.tmp/tm/crosscheck.py`. Change it to the tracked path. This is a comment
   change only.

## 4. Required verification, before handing off

- `verify_imm.py --check` prints `OK`. If it reports a difference, the
  tracked fixture differs from what the script generates. **Stop and report
  the diff.** Do not regenerate the file.
- `verify_matched.py` prints `OK`.
- `crosscheck.py --check` passes.
- Do each of these once with a deliberately wrong copy, under `target/`, to
  show it fails, then restore.

## 5. Do not

- Change any fixture data, any Rust code, or any test other than the doc
  comment in item 3.6.
- Change the algorithm of either copied script.
- Leave scratch files outside `target/`.

## 6. Checks

The three Python commands above, then the usual checks with
`TMPDIR=$PWD/target/tmp`: `cargo fmt --check`, `cargo test --workspace`,
`cargo clippy -D warnings`, `git diff --check` and `git status --short`.

## 7. Handoff

Report the files added and changed, the exact output of each Python command,
the deliberate-failure results, the cargo check results, and any deviation.
