# Review fix C7, C10: unasserted error variants and weak assertions

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review. All three C7 variants are unreachable through the public API and are now documented as defensive checks.
Planner: Claude. Parent: `docs/tasks/review-post-m4.md` findings C7, C10.

## 1. C7: three primer error variants that no test asserts

These are raised in `crates/core/src/primer.rs` and asserted by no test
outside the exhaustive name maps:

| Variant | Raised at (approx.) |
| --- | --- |
| `PrimerAnnealingNotTemplate` | `construct_primer`, line 895 |
| `PrimerPartition` | `construct_primer`, line 905 |
| `PrimerOrientationConflict` | `assemble_pair`, line 1275 |

For **each** variant, do one of these:

- **Reachable.** If some input through the public API (`construct_primer`,
  `construct_tailed_primer`, `assemble_pair`) reaches it, add a test to
  `primer.rs`'s test module that triggers it and asserts the variant and its
  `category()`. Derive the input by reasoning about the code, and show the
  reasoning in a test comment.
- **Unreachable.** If no such input exists because earlier checks make it
  impossible, write down why in a comment placed directly above the check.
  Start the comment with `Defensive check:` and name the earlier check or
  checks that already exclude this case. Do not remove the check.

Planner's starting reading, to confirm or refute:
- `PrimerOrientationConflict` looks unreachable. Orientation is derived from
  the role (`oriented_of`), and `assemble_pair` checks both roles before it
  compares orientations.
- The other two may be unreachable because the text-at-span check runs
  first.

Report your conclusion for each variant with its reasoning. If you can't
decide, stop and report rather than guessing.

## 2. C10: weak assertions to tighten

Each expected variant below follows from a documented rule, not from running
the code. If the code returns something else, stop and report.

1. `crates/core/src/cds.rs`, test `near_matches_are_not_accepted`. Both
   cases (`"STANDARD"` and `"iso-1557-1"`) must be
   `DomainError::UnsupportedGeneticCode(_)` with category `Unsupported`.
   Genetic-code names are compared exactly (D010), and the test just above
   it asserts the same variant for an unknown code.
2. `crates/core/src/translation.rs`, test
   `residues_are_addressed_by_one_based_positions`. Add
   `matches!(error, DomainError::InvalidResiduePosition(_))` next to the
   existing category assertion. `ResiduePosition::create` refuses 0 with
   that variant (its doc: "Zero and negative numbers are rejected").
3. `crates/core/tests/mutation_fixtures.rs`, the `narrower:` branch (around
   line 961). Where the variant is asserted, also assert that
   `problem.category() == ErrorCategory::BiologicalValidation`, which is the
   category `tests/fixtures/mutation/README.md` lists for this rule. Check
   the README line (around 181) first. If it says something else, stop and
   report.

Change only those assertions: add to them, and remove nothing.

## 3. Do not

- Edit files other than `primer.rs`, `cds.rs`, `translation.rs` and
  `mutation_fixtures.rs`.
- Change non-test code, except the `Defensive check:` comments in §1.
- Remove or weaken an assertion, or change a fixture.

## 4. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 5. Handoff

Report the changed files and, for each of the three C7 variants, whether it
is reachable, with the evidence. Also report the exact check results and any
deviation.
