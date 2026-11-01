---
type: Task Contract
title: "Review F1: the Q5 annealing temperature is the rounded lower Tm plus 1"
description: "D030 reports the annealing temperature as the lower Tm plus 3 °C, capped at 72 °C."
status: deprecated
---
# Review F1: the Q5 annealing temperature is the rounded lower Tm plus 1

Status: **Implemented 2026-10-18** by the local worker. Planner: Claude. Owner
decision of 2026-10-18, amending D030. Branch: `feat/milestone-5`.

Two planner errors in this contract, found by the worker and corrected during
the run: §3.6 missed `crates/cli/tests/run.rs` (two Ta values), the four
invariant tests in `crates/core/tests/q5_primer_fixtures.rs` that recompute Ta
with the old formula, and the rep-14 Ta in `tests/fixtures/q5-primer/README.md`;
and §3.6 wrongly lists `rep-01` as exercising the cap — its lower Tm of 70.53
gives exactly 72 with or without it. The cap is exercised by the unit test at
about line 2165 and by `native-03`, `del-04`, `ins-06` and `rep-07`.

## 1. What and why

D030 reports the annealing temperature as the lower Tm plus 3 °C, capped at
72 °C. The owner found, in the NEB Tm calculator's own changelog, that this
offset belongs to NEB's **old** Tm:

> v1.9.1 July 27, 2016 — The calculations in the NEB Tm calculator were
> modified to correct an error that was decreasing the effective primer
> concentration to one quarter of its input value. This correction raises Tm
> values by roughly 2°C for all polymerase reaction buffers. The suggested
> value for annealing temperature using Q5 is now Tm + 1 (was Tm + 3) to
> account for the revised Tm values.

luria's Tm (`tm.rs`) uses the primer concentration undivided, which is the
corrected form, so D030 paired a corrected Tm with the old offset and reports
Ta about 2 °C high. Against NEBaseChanger the owner measured +1.9 °C (D404N)
and +2.2 °C (D404E). NEBaseChanger's own code computes
`min(min(tmf, tmr) + 1, 72)` and `Math.round`s it for display
([docs/research/nebasechanger-client-investigation.md](/research/nebasechanger-client-investigation.md) line 269).

The owner also decided that Ta is a **whole number of degrees**, because a
thermocycler is set to that precision.

**Only Ta changes.** Every primer, span, Tm, warning and failure stays exactly
as it is: Ta is computed after sizing, from the Tms sizing already chose, and
nothing reads it back.

## 2. The rule

```
Ta = min(round_half_up(min(Tm_forward, Tm_reverse)) + 1, 72)
```

- In Rust: `(lower.round() + TA_OFFSET).min(TA_CAP)` with `TA_OFFSET = 1.0`.
  `f64::round` rounds half away from zero, which for a positive Tm is half up.
- In Python: `math.floor(lower + 0.5)`. **Not** `round()`, which rounds half
  to even and would disagree with Rust at an exact `.5`.

This equals NEBaseChanger's displayed value: rounding commutes with `min`, with
adding an integer and with an integer cap.

The planner checked the 51 rows of the four `native*.tsv` tables: no lower Tm
lies within 0.005 °C of a half degree, so the two-decimal Tm columns decide
every rounding.

## 3. Changes, file by file

### 3.1 [docs/decisions/](/decisions/) (D030 and its two references)

Make exactly these six replacements. Change nothing else in the file.

1. The D030 status line. Replace

   ```
   Status: approved by the owner, 2026-10-14. Amends D029: D029's scope, its
   failure outcomes and its abutting pair stand; its fixed, centred lengths do
   not.
   ```

   with

   ```
   Status: approved by the owner, 2026-10-14. Amends D029: D029's scope, its
   failure outcomes and its abutting pair stand; its fixed, centred lengths do
   not. Amended by the owner, 2026-10-18: the annealing temperature is the lower
   Tm rounded to a whole degree, plus 1 °C (was the lower Tm plus 3 °C).
   ```

2. The annealing-temperature bullet. Replace

   ```
   * **Annealing temperature.** Reported as the lower of the two Tms plus 3 °C,
     capped at 72 °C, using the same Tms the sizing used.
   ```

   with

   ```
   * **Annealing temperature.** Reported as the lower of the two Tms, rounded
     half up to a whole degree, plus 1 °C, capped at 72 °C, using the same Tms
     the sizing used. It is a whole number of degrees because a thermocycler
     is set to that precision (owner, 2026-10-18).
   ```

3. The annealing-temperature source. Replace

   ```
   * Annealing temperature: NEB, Q5 *PCR Optimization* (E0555): "a 10–30 second
     annealing step at 3 °C above the Tm of the lower Tm primer"; Q5
     Site-Directed Mutagenesis Kit FAQ: "For predesigned back-to-back primer sets,
     a Tm+3 rule can be applied but optimization may be necessary." NEB computes
     that Tm with its own calculator; luria applies the rule to its own Tm and
     claims nothing about NEB's number. The 72 °C cap is the top of the Q5 SDM
     protocol's 50–72 °C annealing range.
   ```

   with

   ```
   * Annealing temperature: the NEB Tm calculator's changelog, v1.9.1, 27 July
     2016, as quoted by the owner on 2026-10-18: "The calculations in the NEB Tm
     calculator were modified to correct an error that was decreasing the
     effective primer concentration to one quarter of its input value. This
     correction raises Tm values by roughly 2°C for all polymerase reaction
     buffers. The suggested value for annealing temperature using Q5 is now
     Tm + 1 (was Tm + 3) to account for the revised Tm values." luria's Tm uses
     the primer concentration undivided (`tm.rs`), which is the corrected form,
     so the +1 offset is the one that matches it. NEBaseChanger computes the
     same rule, `min(min(tmf, tmr) + 1, 72)`, and rounds it for display
     (`docs/research/nebasechanger-client-investigation.md`). Two NEB pages
     still give +3 — Q5 *PCR Optimization* (E0555), "3 °C above the Tm of the
     lower Tm primer", and the Q5 Site-Directed Mutagenesis Kit FAQ, "a Tm+3
     rule" — which D030 cited until 2026-10-18; the changelog indicates they
     describe the pre-2016 Tm. The planner could not open the calculator's page
     itself (HTTP 403), so the quotation rests on the owner's reading. The
     72 °C cap is the top of the Q5 SDM protocol's 50–72 °C annealing range.
   ```

4. The reversibility line. Replace

   ```
   Each threshold — 55, 63, 5, the 10-base 5′ flank and the +3 annealing offset — is
   ```

   with

   ```
   Each threshold — 55, 63, 5, the 10-base 5′ flank, and the +1 annealing offset with its whole-degree rounding — is
   ```

5. The "Not adopted" paragraph. Replace

   ```
   Not adopted from the reference tool: its 1 °C annealing offset (the Q5
   documentation says 3 °C), and its mismatch-aware Tm, which differs from the
   published model by several degrees (`tests/fixtures/tm/README.md`).
   ```

   with

   ```
   Not adopted from the reference tool: its mismatch-aware Tm, which differs
   from the published model by several degrees (`tests/fixtures/tm/README.md`).
   Its 1 °C annealing offset, first not adopted, was adopted on 2026-10-18 with
   the NEB calculator changelog as the source.
   ```

6. In D031, replace the two lines

   ```
     forward first, the 5 °C balance, annealing temperature as the lower Tm plus
     3 °C capped at 72 °C, ambiguity codes ending availability, the buffer named
   ```

   with

   ```
     forward first, the 5 °C balance, annealing temperature as the lower Tm
     rounded to a whole degree plus 1 °C, capped at 72 °C, ambiguity codes
     ending availability, the buffer named
   ```

   and in D033 replace `` `Ta = min(72, lower Tm + 3)` `` with
   `` `Ta = min(72, round(lower Tm) + 1)` `` (the only occurrence).

### 3.2 [docs/specs/q5-primer-design.md](/specs/q5-primer-design.md)

Two replacements:

- `and the annealing temperature is the lower Tm + 3 °C capped at 72 °C.` →
  `and the annealing temperature is the lower Tm, rounded to a whole degree,
  + 1 °C, capped at 72 °C (D030 as amended 2026-10-18).` Reflow only that
  sentence's lines to stay under 80 columns.
- In the §18 table row, `Ta = lower Tm + 3 capped at 72` →
  `Ta = rounded lower Tm + 1 capped at 72, amended 2026-10-18`.

### 3.3 `crates/core/src/primer_design.rs`

1. The constant:

   ```rust
   /// D030: annealing temperature is the lower Tm, rounded to a whole degree,
   /// plus this, °C (NEB Tm calculator v1.9.1: Q5 "Tm + 1"; amended 2026-10-18).
   pub const TA_OFFSET: f64 = 1.0;
   ```

2. `pair_summary`: the lower Tm is rounded with `f64::round` before the offset
   is added, then capped, exactly as §2 states. Keep the function's signature.
   Update its doc comment to say so.
3. The module doc sentence `The annealing temperature is the lower Tm plus
   `TA_OFFSET`, capped at `TA_CAP`.` becomes `The annealing temperature is the
   lower Tm rounded to a whole degree, plus `TA_OFFSET`, capped at `TA_CAP`.`
4. `NATIVE_DESIGN_ALGORITHM_VERSION`: `"0.6.0"` → `"0.7.0"` (D021: the output
   changed). Append to its doc comment: `0.7.0 reports the annealing
   temperature as the lower Tm rounded to a whole degree, plus 1 °C, capped at
   72 °C (D030 as amended 2026-10-18); every primer and Tm is unchanged.`
5. The test that pins the version (`design.algorithm_version() == "0.6.0"`,
   about line 2449): `"0.6.0"` → `"0.7.0"`.

No other constant changes. `Q5_WORKFLOW_VERSION` and the others stay.

### 3.4 The four reference generators, `tests/fixtures/q5-primer/reference/`

1. `native-design.py`:
   - `TA_OFFSET` in the constants line: `3.0` → `1.0`. Add `import math`.
   - Add, after the constants line, one shared helper:

     ```python
     def annealing_temp(tf, tr):
         """D030 as amended 2026-10-18: the lower Tm rounded half up to a whole
         degree, plus TA_OFFSET, capped at TA_CAP. math.floor(x + 0.5), not
         round(), which rounds half to even and would disagree with Rust's
         f64::round at an exact half."""
         return min(TA_CAP, math.floor(min(tf, tr) + 0.5) + TA_OFFSET)
     ```

   - Its `annealing_temp` cell becomes `f"{annealing_temp(tf, tr):.2f}"`.
   - Docstring line `Ta = min(72, min(Tm_f, Tm_r) + 3)` →
     `Ta = min(72, round_half_up(min(Tm_f, Tm_r)) + 1)`.
2. `native-deletion.py`, `native-insertion.py`, `native-replacement.py`: each
   `annealing_temp` cell becomes `f"{nd.annealing_temp(tf, tr):.2f}"`. Drop
   `TA_OFFSET` and `TA_CAP` from their import-from-`nd` lines if nothing else
   uses them there. In each docstring, `min Tm + 3` / `lower Tm + 3` becomes
   `round_half_up(min Tm) + 1` / `round_half_up(lower Tm) + 1`.
3. Do not touch `neb-q5-sdm-reference.py`, `unit-expectations.py` or any
   other script. (`unit-expectations.py` and `--synthetic` reach Ta through
   `native-design.py` and `native-replacement.py`, so they follow.)

Then regenerate, with the project venv (it exists; do not create or modify
it):

```bash
.venv/bin/python -I tests/fixtures/q5-primer/reference/native-design.py
.venv/bin/python -I tests/fixtures/q5-primer/reference/native-deletion.py
.venv/bin/python -I tests/fixtures/q5-primer/reference/native-insertion.py
.venv/bin/python -I tests/fixtures/q5-primer/reference/native-replacement.py
.venv/bin/python -I tests/fixtures/verify.py
```

`verify.py` must end `all 12 checks passed`. **`git diff` of the four `.tsv`
files must change the `annealing_temp` column and nothing else.** Check this
with `git diff --word-diff` or a script; if any other cell changed, stop and
report.

### 3.5 Fixture README

`tests/fixtures/q5-primer/README.md`, the column-meanings sentence
`` `forward_tm`, `reverse_tm` and `annealing_temp` are °C to two decimals at
the `` … : add after that sentence: `` `annealing_temp` is a whole number of
degrees, `min(72, round_half_up(lower Tm) + 1)` (D030 as amended
2026-10-18), written to two decimals like the Tms. ``

### 3.6 Hard-coded Ta values in tests

Each new value is `min(floor(lower + 0.5) + 1, 72)` of the two Tm figures in
the same assertion, and must also equal what the generators now print
(`native-design.py --synthetic` and `unit-expectations.py` for the unit tests,
the `.tsv` tables for the rest). The planner derived the table below; **if a
generator prints anything different, stop and report — do not choose.**

`crates/core/src/primer_design.rs` (`assert_tms(&…, forward, reverse, Ta)`):

| Line (approx.) | Tms | Ta old → new |
| --- | --- | --- |
| 1552 | 56.96, 58.73 | 59.96 → 58.00 |
| 1575 | 63.96, 62.53 | 65.53 → 64.00 |
| 1604 | 60.26, 58.73 | 61.73 → 60.00 |
| 1625 | 64.51, 62.79 | 65.79 → 64.00 |
| 1716 | 63.33, 61.85 | 64.85 → 63.00 |
| 1869 | 71.55, 55.68 | 58.68 → 57.00 |
| 1885 | 57.91, 62.65 | 60.91 → 59.00 |
| 1899 | 67.60, 70.90 | 70.60 → 69.00 |
| 1912 | 73.21, 69.59 | 72.00 → 71.00 |
| 2130 | 70.90, 68.76 | 71.76 → 70.00 |
| 2165 | 80.68, 76.28 | 72.00 → 72.00 (unchanged: still capped) |
| 2196 | 63.89, 62.17 | 65.17 → 63.00 |
| 2224 | 63.89, 62.17 | 65.17 → 63.00 |

`crates/core/src/q5_design.rs`:

| Line (approx.) | Tms | Ta old → new |
| --- | --- | --- |
| 823, 942, 1124 | 56.52, 58.66 | 59.52 → 58.00 |
| 859 | 57.83, 58.37 | 60.83 → 59.00 |
| 1057 | 68.70, 64.00 | 67.00 → 65.00 |
| 1094 | 63.36, 59.48 | 62.48 → 60.00 |

The test at about line 1912, `the_annealing_temperature_is_capped`, is no
longer capped (71.00). Rename it
`the_annealing_temperature_is_the_rounded_lower_tm_plus_one` and change the
last clause of its comment from `and an annealing temperature at the 72 °C
cap.` to `and an annealing temperature of the rounded lower Tm plus 1.` The
cap stays covered by the test at about line 2165
(`an_unequal_inline_replacement_is_sized_on_the_matched_basis`, 76.28 → 77 →
72) and by the `native-03`, `del-04`, `ins-06`, `rep-01` and `rep-07`
fixture rows.

`crates/cli/tests/end_to_end.rs` (two-decimal strings):

| Line (approx.) | Case | old → new |
| --- | --- | --- |
| 167 | native-01 | "65.90" → "64.00" |
| 177, 187 | native-02, native-03 | "72.00" → "72.00" (unchanged) |
| 257 | native-04 | "62.45" → "60.00" |
| 279 | del-11 | "68.77" → "67.00" |
| 324 | ins-14 | "65.38" → "63.00" |
| 360 | rep-14 | "65.90" → "64.00" |
| 393 | rep-15 | "68.77" → "67.00" |

Confirm each case name against the test before editing; the line numbers
are approximate.

`crates/cli/tests/result_design.rs`: line 79 `"65.90"` → `"64.00"`; line 102
`"65.38"` → `"63.00"`.

`crates/core/tests/q5_primer_fixtures.rs` reads `annealing_temp` from the
tables, so it needs no change.

### 3.7 `crates/cli/tests/documented_example.rs`

Ta is now exact, so take it out of the tolerance table: the arm
`"celsius" | "annealingTemperature" | "gcPercent" => 0.005,` becomes
`"celsius" | "gcPercent" => 0.005,`. Update the doc comment at the top if it
says temperatures are compared within a tolerance: Tms are, Ta is not.

### 3.8 [docs/specs/cli.md](/specs/cli.md)

The planner has already updated [docs/tasks/contracts/milestone-5h-cli-page.txt](/tasks/contracts/milestone-5h-cli-page.txt)
(version 0.7.0 in three places, `"annealingTemperature": 64.0`, and two
prose passages). Copy it, do not edit it:

```bash
cp docs/tasks/contracts/milestone-5h-cli-page.txt docs/specs/cli.md
diff docs/tasks/contracts/milestone-5h-cli-page.txt docs/specs/cli.md
```

### 3.9 `.gitignore`

Add the line `.venv/` after `target/`. The project venv lives there (owner,
2026-10-18).

## 4. Proofs (AGENTS.md: disable each rule once, then restore it)

Record the exact failing test and message for each, and restore after each:

1. **Rounding.** In `pair_summary`, remove `.round()`. `cargo test
   --workspace` must fail; native-01 expects 64.00 and gets 63.90.
2. **Offset.** Set `TA_OFFSET` back to `3.0`. Tests must fail.
3. **Cap.** Remove `.min(TA_CAP)`. The test at about line 2165 and the
   capped fixture rows must fail (77.00, not 72.00).
4. **The Python rounding mode.** Not a disabling proof: run
   `.venv/bin/python -I -c "import math; print(round(62.5), math.floor(62.5 + 0.5))"`
   and record the output (`62 63`), which is why the helper uses `floor`.

## 5. Required checks

From the repository root, with `export TMPDIR=$PWD/target/tmp`:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
.venv/bin/python -I tests/fixtures/verify.py
diff docs/tasks/contracts/milestone-5h-cli-page.txt docs/specs/cli.md
```

## 6. Do not

- Do not change any primer, span, Tm, sizing rule, warning or failure. If any
  value other than a Ta or the 0.6.0 version string changes anywhere, stop
  and report.
- Do not change `TA_CAP`, any other constant, or any other algorithm version.
- Do not write a Ta into any fixture or test by running luria. The values come
  from the generators and must agree with §3.6's table.
- Do not edit [docs/tasks/contracts/milestone-5h-cli-page.txt](/tasks/contracts/milestone-5h-cli-page.txt) or this contract.
- Do not add a dependency, create or modify `.venv`, or install anything.
- Do not stage or commit.

## 7. Report

Changed files; the exact commands and their real output; the four proofs
with their failures; confirmation that the `.tsv` diffs touch only
`annealing_temp`; any generator value that differed from §3.6; anything
unverified.
