# Review M5 A-9: every imported GenBank CDS starts with ATG (D042)

Status: **Implemented 2026-10-18** by the local worker; accepted after one review round (the `AlternativeStart` message wording). Planner: Claude. Parent:
`docs/tasks/reviews/review-post-m5.md`, owner item A-9 (the reviewer's owner item 1).
Owner answer, 2026-10-18: "Require ATG for every table." Recorded by this
contract as D042. Amends `docs/tasks/contracts/milestone-5c3-genbank-cds.md` §4.2 step
11, which stays a historical record: do not edit it.

Files:
- `crates/core/src/genbank_cds.rs`
- `crates/core/src/error.rs` (doc comments only)
- `docs/decisions/` (append D042, text in §2.4)
- `tests/fixtures/genbank/synthetic-expected.tsv`,
  `tests/fixtures/genbank/synthetic-unused.tsv`,
  `tests/fixtures/genbank/README.md`,
  `tests/fixtures/genbank/reference/verify_synthetic.py`

## 1. Defect

§4.2 step 11 checks the first codon only under `/transl_table=11`. A named
CDS with no table, or table 1, that starts with GTG, TTG, CTG or any other
codon is imported, and its residue 1 is translated literally (V, L, …).
A cell translates every start codon as methionine. So residue 1 is wrong,
and a residue edit numbered against it would assert the wrong original. With
an NCBI-style `/translation` (which starts with M), the CDS is refused
instead, but as a `/translation` mismatch at residue 1, which is the wrong
diagnosis.

## 2. Required change

### 2.1 `genbank_cds.rs`

Step 11 applies to every CDS that reaches it, whatever its `/transl_table`
(absent, 1 or 11). A first codon other than `ATG` is
`GenBankCdsRule::AlternativeStart`, with the three letters as `detail` and
the feature's key line as `line`, exactly as the table-11 case is today. The
step order is unchanged, so step 11 still runs before the `/translation`
check (step 12).

Update the module doc comment:
- step 7: drop "and 11 only for an ATG start, which step 11 checks";
  say that 11 is accepted because tables 1 and 11 assign the same residue to
  every codon, and step 11 requires an ATG start for every table (D042);
- step 11: "A first codon other than `ATG` is `AlternativeStart`, with those
  three letters as the detail, under every table (D042): luria translates
  codon 1 literally, and a cell reads every start codon as methionine."
- the step 7 inline comment in `evaluate` and the step 11 inline comment.

Keep the rule's name and code (`alternative_start`).

### 2.2 `error.rs`

Doc comments only. Wherever `AlternativeStart` or G5 is described as
table-11-only, say it applies under every table (D042). Change no code.

### 2.3 Fixtures (expectations derived by the planner, §3)

In `synthetic-expected.tsv`, replace the `mj1` and `mj2` rows with:

```
mj1	unusable_genbank_cds	alternative_start	-	-	-	-	-	CAT	22
mj2	unusable_genbank_cds	alternative_start	-	-	-	-	-	CAT	24
```

In `synthetic-unused.tsv`, rows with index 4 and 5 get the refusal
`unusable_genbank_cds:alternative_start`.

In `README.md` (`tests/fixtures/genbank/`):
- the `mj1`/`mj2` paragraph: keep the derivation and add that the reverse
  complement starts `CAT`, so both are refused as `alternative_start` under
  D042, with detail `CAT`, at key lines 22 and 24. Both still go through the
  location reader first, so both minus-strand spellings of an origin join are
  still read;
- where the list says `/transl_table=11` with a GTG start (`alt11`) is
  refused, add that a non-ATG start is refused under every table (D042).

In `reference/verify_synthetic.py`, `check_expected`:
- for an imported row (`error == "-"`), the check that an imported CDS
  starts with `ATG` applies whatever the table: replace the `table == "11"
  and …` condition with a check on the start alone, with the message
  `name + ": imported without an ATG start"`;
- for `rule == "alternative_start"`, drop the table-11 condition: the row is
  right when the start is not `ATG` and equals `row["detail"]`. Message:
  `name + ": start is not " + row["detail"]`.
- In `check_unused`, add: when a row's refusal is
  `unusable_genbank_cds:alternative_start`, the feature's extracted start
  must not be `ATG` (message `"feature %s: starts with ATG"`).
Update the module docstring only if it mentions table 11 for the start
rule.

### 2.4 `docs/decisions/`

Append, after D041, exactly:

```markdown
## D042: Every imported GenBank CDS starts with ATG

Status: approved by the owner, 2026-10-18 (`docs/tasks/reviews/review-post-m5.md`
A-9). Amends D041's `/transl_table` rule.

Decision:

* A CDS a request names from a GenBank record is imported only when its
  first codon is `ATG`, whatever its `/transl_table` (absent, 1 or 11). Any
  other first codon is refused as `alternative_start`, with the codon as the
  detail.
* Reason: luria translates codon 1 literally. A cell reads every start codon
  as methionine, so a GTG, TTG or CTG start would give a wrong residue 1, and
  a residue edit numbered against it would assert the wrong original. Table
  11 was already limited to ATG starts (G5); tables absent and 1 now follow
  the same rule.
* FASTA CDSs declared in a request are unchanged.

Reversibility:

Accepting alternative starts needs approval and a rule for residue 1.
```

## 3. Tests (write first; confirm they fail before the change)

In `genbank_cds.rs`'s test module, using `record`, `key`, `q`, `names` and
`cds_problem`. The `record` helper's sequence is `ATGGCTTGGTAA`. `4..12` is
`[3, 12)`, `GCTTGGTAA`, first codon `GCT` (planner-checked with Biopython:
it reads A W stop).
1. `key("CDS", "4..12")`, `q("/gene=\"g\"")`, no table: importing `g` is
   `AlternativeStart`, detail `Some("GCT")`, line `Some(3)`.
2. The same with `q("/transl_table=1")` added: the same refusal.
3. The same with `q("/translation=\"MW\"")` added and no table: the
   refusal is `AlternativeStart` (step 11 comes before step 12), not
   `GenBankTranslationMismatch`.
4. **Keeps the origin-crossing minus-strand import covered**, which `mj1`
   and `mj2` no longer do. Add a second helper, `circular_record(features)`,
   identical to `record` but with the LOCUS line
   `LOCUS       tiny                      12 bp    DNA     circular SYN 17-OCT-2026`
   and the ORIGIN line `        1 agccatgggt ta`. The sequence is
   `AGCCATGGGTTA`. Feature: `key("CDS", "complement(join(10..12,1..6))")`,
   `q("/gene=\"mo\"")`, `q("/translation=\"MA\"")`. Planner-derived
   (checked with Biopython): `10..12` is `[9, 12)` = `TTA`, `1..6` is
   `[0, 6)` = `AGCCAT`; joined `TTAAGCCAT`, reverse complement `ATGGCTTAA`,
   M A stop. Importing `mo` succeeds with one CDS: intervals `[9, 12)` then
   `[0, 6)` (D036, D040 order), minus strand, and translation residues `MA`.
   Use the accessors the existing import tests use; if there is no way to
   read the residues, assert the intervals and strand only and report it.

`crates/core/tests/genbank_fixtures.rs` reads the TSVs and must pass with
the updated rows, unchanged. If it doesn't, stop and report.

Run `.tmp/tm/venv/bin/python -I tests/fixtures/verify.py` and confirm all
checks pass, including `genbank/reference/verify_synthetic.py`.

## 4. Do not

- Add dependencies.
- Change any file not listed above, or any Rust test other than adding the
  four in §3.
- Rename `AlternativeStart` or change its code or category.
- Change the step order.
- Change any other fixture row.

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `.tmp/tm/venv/bin/python -I tests/fixtures/verify.py`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, each new test and whether it failed before the
change, the check results, and any deviation.
