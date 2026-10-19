# Post-Milestone-5 code review: findings

Status: Phase 1 (read-only review) complete, 2026-10-18. Phase 2 (fixes) in
progress. Branch: `feat/milestone-5` (owner's choice, 2026-10-18; it is
already merged into `main` at `c9feca0`, and is merged again afterwards).
Planner: Claude.

Baseline at review start (`c9feca0`): `cargo fmt --check` clean, 719 tests
pass, `cargo clippy -D warnings` clean, `python -I tests/fixtures/verify.py`
12 of 12 pass.

Method: three read-only reviewers, each on a copy of the tracked files outside
the repository:
- (A) the core readers (`fasta.rs`, `genbank*.rs`) and their fixtures;
- (B) the error codes and rules, and the CLI's request parsing and
  preparation;
- (C) the result document, run wiring, `--write-edited`, the end-to-end tests,
  the docs, and the D030 change `819d48e`.

**Confirmed** items were reproduced by a scratch run or verified by reading.
No High-severity defect was found. Nothing panicked under fuzzing of the
readers (about 38,000 mutated GenBank texts and 190,000 location strings),
except B-1.

Each fix is its own contract, `docs/tasks/review-m5-*.md`.

---

## Owner decisions, 2026-10-18

| # | Question | Answer |
| --- | --- | --- |
| A-9 | Non-ATG start codons when `/transl_table` is absent or 1 | Require ATG for every table. Recorded as D042. |
| B-2 | What `template.sha256` means | The normalized sequence's content digest, as `provenance.template.sha256` reports it. The docs are fixed; the code is unchanged. |
| B-3 | Exit status for an unreadable request or template file | Keep exit 4 (`malformed_input`). Plan §3.7 and `docs/cli.md` are fixed: exit 1 is for internal errors and output write failures. |
| B-6 | `path` is null for errors raised while running | Fix the docs now; mapping errors to request fields belongs to Milestone 6. |
| B-8 | `..` and symlinked template paths; unbounded reads | Deferred to Milestone 6, where an LLM writes requests. |
| C-2 | Member names that collide by case | Refused with `--write-edited` only (`output_name_unusable`), and `:` `*` `?` `<` `>` `\|` too. Windows reserved names are allowed. |
| C-4 | Upper bounds on the buffer | Na⁺ ≤ 1000 mM, primer ≤ 10 µM, DMSO ≤ 20 %. Recorded as D043. |
| C-6 | Exit 1 from an I/O failure hides exit 7 | Exit 1 wins; documented. |
| C-12 | GenBank imported CDSs carry no location | Add intervals and strand to `importedCds`. |
| D030 | Cite NEB's own D404N capture | Yes: `example-construct/raw/sdmsummary.txt` shows Ta 71 = lower Tm 70 + 1. |

---

## A. Core readers and their fixtures

- **A-1, Medium, confirmed.** GenBank ORIGIN text can be silently dropped:
  `normalize` skips a line starting with `;` and trims Unicode whitespace
  after the reader has counted the characters. A 30-character ORIGIN reads as
  20 bases with LOCUS `20 bp`. Breaks D008. → `review-m5-a1-genbank-reader.md`.
- **A-2, Low, confirmed.** `/pseudo\t` (trailing tab) is read as a qualifier
  named `pseudo\t`, so a pseudo CDS imports. → `review-m5-a1`.
- **A-3, Low–Medium, confirmed.** A refused CDS location doesn't name its
  CDS or line. → `review-m5-a3-location-names-cds.md`.
- **A-4, Low, confirmed.** LOCUS topology is matched against every token, the
  name included: `LOCUS linear 30 bp DNA SYN` reads as linear. → `review-m5-a1`.
- **A-5, Low, confirmed.** An unescaped inner quote is accepted
  (`/note="a" "b"`). → `review-m5-a1`.
- **A-6, Low, confirmed.** A non-breaking space at a FASTA line's end is
  silently trimmed (the `normalize` cause of A-1). Fixed in `review-m5-a1`.
  CR-only line endings fail closed with a misleading error: accepted, not
  fixed.
- **A-7, Low, suspected.** The not-used list is quadratic in the number of
  CDSs (about 6 s for a 4,300-CDS genome, extrapolated). Plasmids are
  unaffected. Left as a refactor candidate.
- **A-8, Low, confirmed.** The test helper `fixture_id` doesn't apply G2 as
  luria does. → `review-m5-a10-genbank-tests.md`.
- **A-9, owner decision.** Non-ATG starts under tables absent or 1 (see the
  decisions table). → `review-m5-a9-start-codon.md`.
- **A-10, test gaps.** `pbr322-features.tsv` `id`, `id_source` and
  `pbr322-cds.tsv` `codon_start`, `start_codon`, `translation_matches` are
  unread; `RepeatedQualifier` is tested for `codon_start` only; an empty id
  is untested. → `review-m5-a10`.

Recorded, no change:
- A `/translation` ending in `*` is refused. NCBI never writes one.
- A syntax error in any feature, named or not, fails the whole read
  (`malformed_genbank`). D041's "never fails the run" covers
  representability, not syntax.
- "Complete" is not checked against a terminal stop codon for GenBank or FASTA
  CDSs. D042 now checks the start.

## B. Error codes, request parsing and preparation

- **B-1, Medium, confirmed.** A non-UTF-8 argument panics (exit 101) instead
  of a usage error (exit 2). → `review-m5-b1-arguments.md`.
- **B-2, Medium, confirmed (docs).** `docs/cli.md` says `template.sha256` is
  the file's digest; it is the sequence content digest. → docs contract.
- **B-3, Medium (spec conflict).** Plan §3.6 makes unreadable files
  `malformed_input` (exit 4); §3.7 says I/O is exit 1. Resolved for exit 4.
  → docs contract.
- **B-4, B-5, Low (docs).** `docs/cli.md` lists four CLI codes (there are
  six: `template_unreadable` and `output_name_unusable` are missing), and says
  `rule` is set for two error codes (it is set for six). → docs contract.
- **B-6, Low (docs).** `path` is null for errors raised during execution,
  even when the request caused them. → docs contract; mapping in M6.
- **B-7, Low.** `--` is not accepted. → `review-m5-b1`.
- **B-8, owner item.** Path confinement and size caps: deferred to M6.

Milestone 6 inputs (recorded for the M6 plan, no change now):
1. Clarifications are prose strings, and candidates are bare ids with no
   stable code.
2. Domain errors point at the edit object, not the field.
3. No stdin and no absolute paths: an adapter must write the request next to
   the template.
4. Residue letters and codons are case-sensitive; names are identifiers with
   no whitespace.
5. A UTF-8 BOM, `-0` and exponent-form integers are refused.
6. `missing_cds_attribute` also covers an unrecognized value, a misspelled
   `geneticCode` is `unsupported` (exit 6), and a GenBank `not_found` is
   `biological_validation` (exit 5).
7. `edit` and a one-member `variants` say the same thing; the adapter needs a
   canonical choice.

Other notes: plan §3.6's "26 and 18 producer sites" are now 30 and 19. The
pinned-code test's hard-coded count (52) would not catch a new variant left
off the list.

## C. Result, run, `--write-edited`, end-to-end, docs

- **C-1, Medium, confirmed.** `--write-edited` checks and then writes:
  a dangling symlink at `DIR/<name>.fa` is written through, outside DIR, and
  a file created between check and write is overwritten. Breaks G8.
  → `review-m5-c1-write-edited.md`.
- **C-2, Low–Medium.** Case-colliding names (see decisions). → `review-m5-c1`.
- **C-3, Medium (test gap).** Plan §6's minus-strand and origin-crossing
  residue edits through the executable, against `mutation/cases.tsv`, were
  dropped in 5g3 without a recorded deviation. The behavior is correct when
  run by hand. → `review-m5-c3-cli-tests.md`.
- **C-4, owner decision.** Unbounded buffer values give a "verified"
  document with a Tm of 18.9 million °C. → `review-m5-c4-buffer-bounds.md`.
- **C-5, Low (docs).** Same as B-4, B-5.
- **C-6, Low.** The I/O failure path has no test. → `review-m5-c9-run-and-imports.md`.
- **C-7, Low (test gap).** Only three cases run twice, and each pair runs
  from the same directory. → `review-m5-c3`.
- **C-8, Low (test gap).** Ta is compared at two decimals; no x.5 tie test.
  → `review-m5-c3`.
- **C-9, Low.** `run.rs:153` `unwrap()` on a core invariant. → `review-m5-c9`.
- **C-10, Low.** `dmsoPercent: -0.0` is accepted and echoed as `-0.0`.
  Recorded, no change.
- **C-11, Low.** Four payload fields that hold only interval text are left
  out of `details`. Documented as an exception in the docs contract.
- **C-12.** See decisions. → `review-m5-c9`.
- **C-13, Low (test gap).** Written deletion and insertion files are not
  checked against the document's digest. → `review-m5-c3`.

The D030 change (`819d48e`) checked out: the rule matches NEBaseChanger's
client code, `NATIVE_DESIGN_ALGORITHM_VERSION` was bumped, and the fixture
values come from independent Python generators.

Docs drift for the docs contract: B-2 to B-6, C-5, C-6, C-11; the README says
`docs/cli.md` has the schemas but `docs/cli.md` §9 says the full reference
isn't written; `docs/cli.md` §5 doesn't say the document is still printed on
exit 1; a `--write-edited` FASTA doesn't record topology; the GenBank comment
and README drift listed by reviewer A.

## Contracts, in order

| Contract | Items | Status |
| --- | --- | --- |
| `review-m5-b1-arguments.md` | B-1, B-7 | done |
| `review-m5-a1-genbank-reader.md` | A-1, A-2, A-4, A-5, A-6 | done |
| `review-m5-a9-start-codon.md` | A-9 (D042) | done |
| `review-m5-a3-location-names-cds.md` | A-3 | done |
| `review-m5-a10-genbank-tests.md` | A-8, A-10 | done |
| `review-m5-c1-write-edited.md` | C-1, C-2 | done |
| `review-m5-c4-buffer-bounds.md` | C-4 (D043) | done |
| `review-m5-c9-run-and-imports.md` | C-6, C-9, C-12 | |
| `review-m5-c3-cli-tests.md` | C-3, C-7, C-8, C-13 | |
| `review-m5-docs.md` | the docs drift above | |
