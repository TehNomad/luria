# Review fix B4: residue verification must check the annotation, not just its name

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review.
Planner: Claude. Parent: [docs/tasks/reviews/review-post-m4.md](/tasks/reviews/review-post-m4.md) finding B4.

## 1. The defect

`verify_residue_claims` (`crates/core/src/mutation.rs`, around line 1858)
accepts any `Cds` whose `id()` equals the result's `cds()`. `ResidueMutation`
records only the CDS identifier, so a different annotation that happens to
share the name is accepted as the reference. Examples: other intervals,
another strand, another genetic code. The residue and codon claims are then
re-derived over the wrong annotation.

Two recorded claims are also never re-checked:

- `original_codon`, the codon the source held at the residue;
- `actual_original`, the residue the source held.

[docs/specs/q5-sdm.md](/specs/q5-sdm.md) §8 justifies `verify` by results that "arrive from
elsewhere", and that is exactly where these gaps matter.

## 2. Requirement (D013, inline)

> Generation and verification are separate responsibilities. Verification
> re-derives the result's claims from the artifacts rather than trusting
> them.

A residue result's claims are claims about one annotation. Verification must
use that annotation, and every recorded claim it can read back from the
artifacts must be checked.

## 3. Required behaviour, `crates/core/src/mutation.rs` only

**3.1 Record the annotation.**
- Add a private field `annotation: Cds` to `ResidueMutation`, holding the
  exact `Cds` value `apply_residue` was given. Set it in `apply_residue`
  (around line 1115).
- Add a public accessor next to the existing ones:
  `pub fn annotation(&self) -> Cds { self.annotation.clone() }`.
- Keep the `cds: Identifier` field and its accessor unchanged.

**3.2 `verify_residue_claims`.** After the existing identifier check, and
before any translation, refuse when the annotation handed in is not the
recorded one:

```rust
if annotation != record.annotation() { … }
```

The error is `DomainError::MutationVerificationFailure` with
`result: record.edited.reference().describe()` and the reason
`"the CDS verified against is not the annotation the result was applied to"`.
Keep the identifier check before it, so a wrong name still reports the
existing reason.

**3.3 Two more read-back checks**, after the existing
`before_residues[offset] != record.expected_original()` check:

1. `before_residues[offset] != record.actual_original()` refuses with reason
   `"the source does not hold the recorded actual original residue"`.
2. The codon the **source** holds at the residue must equal
   `record.original_codon()`. Compute it the same way the existing edited
   codon check does: `extract_cds(source, &annotation)`, then `codon_slice`
   with the same `number` and label. On a mismatch, refuse with reason
   `"the codon read from the source CDS is not the recorded original codon"`.

Every refusal uses the same error variant and `result` value as the existing
checks.

**3.4 Doc comments.** Update the `verify` doc comment's residue sentence and
`ResidueMutation`'s doc comment to say the result records the annotation and
`verify` requires it. Keep the edits small.

## 4. Required tests, in `mutation.rs`'s test module

Write these first. Tests 1–3 must fail before the fix: today they either
pass verification or don't compile because `annotation()` doesn't exist yet.
Build tampered records with struct update syntax inside the test module, as
the existing `tampered_*` helpers do.

1. `a_residue_result_is_not_verified_against_a_same_named_annotation_with_other_coordinates`:
   - apply a residue mutation over a CDS on a sequence long enough to have
     two valid CDS annotations with the **same identifier** but different
     intervals or strand. Use the module's existing sequence and CDS helpers,
     or a minimal new helper;
   - call `verify` with the other annotation;
   - expect `Err(DomainError::MutationVerificationFailure(_))`.

   If no two such annotations can both translate cleanly on the available
   test sequences, use the same intervals on the **opposite strand**, or stop
   and report.
2. `a_residue_result_with_a_tampered_original_codon_is_refused`: set
   `original_codon` to a different valid codon and expect
   `MutationVerificationFailure`.
3. `a_residue_result_with_a_tampered_actual_original_is_refused`: set
   `actual_original` to a different residue and expect
   `MutationVerificationFailure`.
4. `a_residue_result_records_the_annotation_it_was_applied_to`:
   `record.annotation() == <the Cds passed to apply_residue>`.

All existing tests must keep passing unchanged. If one fails, for example
because it verifies against a rebuilt or different-but-same-named
annotation, stop and report. Do not change it.

## 5. Do not

- Edit any file other than `crates/core/src/mutation.rs`.
- Add dependencies or error variants, or change other result kinds.
- Re-derive the residue-to-coordinate mapping in `verify`. It stays the
  generator's responsibility (`q5-sdm.md` §8).
- Change existing tests or fixtures.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 7. Handoff

Report the changed file, the new tests and their pre-fix failures, the exact
check results, and any deviation.
