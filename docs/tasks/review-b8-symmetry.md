# Review B8: apply the symmetry correction only to a symmetric duplex

Status: **Implemented 2026-10-16** by the local worker; accepted after one follow-up (the printed selfcomp column and two tm.rs comments, at the planner's request).
Planner: Claude. Parent: `docs/tasks/review-post-m4.md` B8. Depends on B7
being committed.
Files:
- `crates/core/src/tm.rs`
- `crates/core/tests/tm_crosscheck.rs`
- `tests/fixtures/tm/reference/crosscheck.py`
- `tests/fixtures/tm/README.md`

## 1. Defect

`melting_temperature` adds `SYMMETRY` whenever the primer is
self-complementary, on either basis. The correction belongs to a duplex
whose two strands are identical. On the matched basis, that's true when the
primer equals its own reverse complement. On the mismatch-aware basis, the
duplex is the primer against the wild-type template strand. That strand
differs from the primer at the mismatch, so the duplex is not symmetric and
takes no symmetry term.

`crosscheck.py` encodes the same mistake: it passes Biopython
`selfcomp=selfcomp(primer)` even when the site differs from the primer. So
the expected value of `"one mismatch at index 9"` in `tm_crosscheck.rs`
(currently 56.390) includes the wrong term.

## 2. Required changes, in this order

1. **`crosscheck.py`:**
   - in `bio_tm`, pass `selfcomp=(selfcomp(primer) and primer == site)`;
   - change its comment to say why: only the matched duplex can be
     symmetric;
   - change nothing else.
2. Run `.tmp/tm/venv/bin/python -I tests/fixtures/tm/reference/crosscheck.py`
   (without `--check`) and record its output.
   - **Only** `"one mismatch at index 9"` may change. Every other value must
     print exactly as before:
     - `62.126`
     - `88.462`
     - `29.723`
     - `67.397`
     - `56.750`
   - If any other value changes, stop and report.
3. **`tm_crosscheck.rs`:**
   - change that one `EXPECTED` value to the new figure, rounded to 3
     decimals as the others are;
   - update the doc comment's note on `selfcomp` to match the new rule.
4. Run `cargo test --workspace` and confirm that
   `tm_crosscheck`'s mismatch case now **fails**, i.e. luria still computes
   the old value. Record the failure.
5. **`tm.rs`:**
   - add `SYMMETRY` only when `basis == TmBasis::MatchedComplement` and the
     primer is self-complementary;
   - comment the reason in a sentence or two.
6. Confirm that `crosscheck.py --check` prints `OK tm_crosscheck.rs` and
   that `cargo test --workspace` passes.
7. **`tests/fixtures/tm/README.md`:** where it explains `selfcomp` (around
   line 149), say that the flag is passed only for a matched duplex, and why.

## 3. Stop and report if

Any existing test other than the `tm_crosscheck.rs` case above changes
result.

## 4. Do not

- Change any other expected value, fixture or test.
- Change `TM_FUNCTION_VERSION` (B12 does that).

## 5. Checks

The two Python commands above, then
`.tmp/tm/venv/bin/python -I tests/fixtures/verify.py` (all `PASS`). Then the
usual cargo checks, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report:
- the changed files;
- the old and new value of the changed case;
- the output of `crosscheck.py` before and after;
- the failure recorded in step 4;
- the check results;
- any deviation.
