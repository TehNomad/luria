# Review refactor E4 and fix B11: the `apply_*` scope check, and `verify` without `unreachable!`

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review.
Planner: Claude. Parent: `docs/tasks/review-post-m4.md` findings E4 and B11.

**Behavior-preserving.** Every existing test must pass unchanged. Every error
payload must be byte-identical, and the order of checks must not change.

## 1. The duplication, and B11

1. `crates/core/src/mutation.rs`: `apply_nucleotide`, `apply_deletion` and
   `apply_replacement` each contain the same block:

   ```rust
   if let Some(cds) = scope.clone() {
       require_sequence(cds.reference().clone(), sequence)?;
       let label: String = cds.id().value().to_string();
       require_within_feature(<interval>, label, sequence.display_name(), cds.intervals().as_slice())?;
   }
   ```

   `apply_insertion` has the same block, calling
   `require_point_within_feature` instead.
2. **B11.** `verify` returns early for `Deletion`, `Insertion` and
   `Replacement`, then runs the substitution checks. Its second `match` holds
   three `unreachable!` arms. If a sixth result kind forgot its early branch,
   the code would compile and then panic at runtime.

## 2. Required changes, `crates/core/src/mutation.rs` only

**2.1 Scope helpers.** Add these, and use them in the four `apply_*`
functions at the same point in each function where the block sits today:

```rust
fn require_in_scope(sequence: &Sequence, scope: &Option<Cds>, target: NucleotideInterval)
    -> Result<(), DomainError>;
fn require_point_in_scope(sequence: &Sequence, scope: &Option<Cds>, point: InsertionPoint)
    -> Result<(), DomainError>;
```

Each does exactly what the block did: `require_sequence` first, then the
containment check, with the same label and the same sequence name.

**2.2 `verify` with one exhaustive `match`.**
- Move the substitution-only checks into one private helper that returns the
  differing positions or the existing error:
  - the length check, with its `"a substitution preserves length"` reason;
  - computing `got`, the positions where source and edited differ.
- Then write a single `match result.clone()` with one arm per kind:
  - `Deletion`, `Insertion`, `Replacement`: call their `verify_*` exactly as
    today;
  - `Nucleotide`: the helper, then the exact-set comparison with its current
    reason text, then `verify_nucleotide_claims`;
  - `Residue`: the helper, then the empty check, the subset check and
    `verify_residue_claims`, each with its current reason text.
- The `require_sequence(result.source(), source)?` check stays first.
- Every arm ends in `Ok(accepted(result.clone()))`.
- No `unreachable!` remains in `verify`.

Keep the order of checks inside each arm exactly as it is today.

## 3. Guards

- `python3 -I target/api_guard.py crates/core/src/mutation.rs` prints nothing
  and exits 0. The script is in `docs/tasks/review-e1-design-paths.md` §3.
- There is no change below `mutation.rs`'s `#[cfg(test)]` line, and none
  under `crates/core/tests/`.
- `grep -n 'unreachable!' crates/core/src/mutation.rs` shows no hit in
  non-test code.

## 4. Do not

- Edit any other file.
- Merge the whole-sequence refusals of deletion and replacement. Their
  messages differ.
- Change `verify_deletion`, `verify_insertion`, `verify_replacement`,
  `verify_nucleotide_claims` or `verify_residue_claims`.
- Change behavior, messages or tests.

## 5. Checks

Same as E1 §5, with `mutation.rs` passed to the API guard.

## 6. Handoff

Report the changed file, the exact check results (the test count is unchanged
from the start of this contract), and any deviation.
