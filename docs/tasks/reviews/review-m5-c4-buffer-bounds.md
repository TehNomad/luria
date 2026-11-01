# Review M5 C-4: upper limits on the buffer model (D043)

Status: **Implemented 2026-10-18** by the local worker; accepted on planner review. Planner: Claude. Parent:
`docs/tasks/reviews/review-post-m5.md` C-4. Owner answer, 2026-10-18: "Na+ ≤ 1000
mM, primer ≤ 10 µM, DMSO ≤ 20 %". Recorded by this contract as D043.

Files:
- `crates/core/src/tm.rs`
- `docs/decisions/` (append D043, text in §2.3)
- `docs/specs/cli.md` (the two `"version": "0.2.0"` lines of the Tm algorithm,
  §2.2)

## 1. Defect

`BufferModel::create` requires only finite, positive values (DMSO ≥ 0). With
`primerNanomolar: 1e300` a run is `verified`, exits 0, and reports a reverse
Tm of 18,873,539 °C and a 1954-nt primer. Physically meaningless values give
a "verified" document.

## 2. Required change

### 2.1 `tm.rs`, `BufferModel::create`

After each existing check, add an upper limit, inclusive:
- `monovalent_millimolar > 1000.0` → `InvalidBufferModel`, field
  `"monovalent"`, reason
  `format!("{} mM is above 1000 mM, the most luria accepts (D043)", value)`;
- `primer_nanomolar > 10000.0` → field `"primer"`, reason
  `format!("{} nM is above 10000 nM (10 µM), the most luria accepts (D043)", value)`;
- `dmso_percent > 20.0` → field `"dmso"`, reason
  `format!("{} % is above 20 %, the most luria accepts (D043)", value)`.

Update the doc comment of `create` to state the three limits and cite D043.

### 2.2 Version

Bump `TM_FUNCTION_VERSION` from `"0.2.0"` to `"0.3.0"` (D021: the accepted
input domain changed). Add a one-line note to its doc comment, in the style
of the other version constants' notes. Then update `docs/specs/cli.md`: the Tm
algorithm's `"version": "0.2.0"` appears in two blocks, around lines 133 and
445 (the `tm` entry: confirm by its `name`). Change only those two lines,
to `"0.3.0"`. The primer algorithm's `0.2.0` lines are not the Tm's: leave
them.

### 2.3 `docs/decisions/`

Append, after the last decision, exactly:

```markdown
## D043: Upper limits on the primer-design buffer

Status: approved by the owner, 2026-10-18 (`docs/tasks/reviews/review-post-m5.md`
C-4). Amends D041's buffer rule.

Decision:

* A buffer is refused (`invalid_buffer_model`) when its monovalent salt is
  above 1000 mM, its primer concentration above 10000 nM (10 µM), or its
  DMSO above 20 %. Each limit is inclusive. The existing lower limits stay:
  salt and primer above zero, DMSO at least zero.
* Reason: the Tm model accepted any finite positive value, so an absurd
  buffer gave a verified result with meaningless Tms and primers. 1000 mM is
  about the top of the Owczarzy salt correction's validated range; the other
  two are well above any real PCR condition.

Reversibility:

Changing a limit changes which requests are accepted and needs approval.
```

## 3. Tests (write first; confirm they fail before the change)

In `tm.rs`'s test module, following the existing `BufferModel::create`
tests:
1. `create(1000.0, 500.0, 0.0)`, `create(150.0, 10000.0, 0.0)` and
   `create(150.0, 500.0, 20.0)` succeed.
2. `create(1000.5, 500.0, 0.0)` is `InvalidBufferModel` with field
   `"monovalent"`; `create(150.0, 10000.5, 0.0)` with field `"primer"`;
   `create(150.0, 500.0, 20.5)` with field `"dmso"`.
3. `create(150.0, 1e300, 0.0)` is refused with field `"primer"`.

`crates/cli/tests/cli.rs` checks `version` against the constant and needs no
change. The documented-example test must pass after the `docs/specs/cli.md` edit;
if it fails for any other reason, stop and report.

## 4. Do not

- Add dependencies.
- Change any lower limit, the preset, or any other file.
- Change any existing test.

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, each new test and whether it failed before the
change, the check results, and any deviation.
