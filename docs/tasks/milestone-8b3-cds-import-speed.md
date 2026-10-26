# Contract 8b3 — core: CDS import without per-CDS copies

Planner: Claude. Status: **Implemented 2026-10-26** by worker `w-8b3` in one
round, reviewed and accepted. Requested by the owner 2026-10-26 after review
of 8b2. Tier: light (D051): no behaviour changes, so no decision entry and no
fixture. Follows **8b2** (`open_record`), which calls `import_cds` twice per
record. Findings from review:

* Planner's benchmark after the change, same record shape as §1, release
  build: `import_cds(&record, &[])` 0.18 s at 1 000 CDSs and 0.44 s at 4 300
  (was 0.9 s and 61 s); `open_record` 0.78 s and 2.9 s (was 2.4 s and 156 s),
  and 5.3 s at 9 000 CDSs. The rest of `open_record` at 4 300 CDSs, about
  1 s, is outside `import_cds`, and no contract covers it.
* G3 found a test gap: making only the §4.2 step 3 site report the first
  line breaks no test. The unused list's refusal *line* is not pinned
  anywhere, only its code. Step 2's site is pinned by
  `the_synthetic_record_imports_each_name_as_expected`.
* G2 breaks 9 of the 12 `genbank_fixtures` tests as well as test 2.

## 1. Why

`import_cds` (`genbank_cds.rs:551`) costs time that grows faster than the
square of a record's CDS count. Measured by the planner in a release build,
on a synthetic record with one `gene` and one `CDS` feature per gene, 1100 bp
per gene:

| CDSs | `read_genbank` | `import_cds(&record, &[])` | `open_record` |
| --- | --- | --- | --- |
| 1 000 | 0.2 s | 0.9 s | 2.4 s |
| 4 300 | 1.0 s | 61 s | 156 s |

Importing one name costs as much as importing none: every call evaluates
every CDS (§4.1 step 3). Each evaluation copies data the size of the
record:

1. `evaluate` (`genbank_cds.rs:387`) calls `record.features()` and
   `record.sequence()` (`genbank.rs:148-158`), and both clone.
2. `shared_lines` (`genbank_cds.rs:309`) scans every feature for each CDS.
3. `translate` and `extract_cds` both reach `extraction::extract`
   (`extraction.rs:29`), which calls `sequence.letters()`
   (`sequence.rs:396`). That clones the whole sequence as a `Vec<char>`.
4. `import_cds` step 1 compares every name with every later name, step 2
   compares every name with every feature, and step 3's `used.contains` scans
   a `Vec`.

## 2. Files the worker may touch

1. `crates/core/src/sequence.rs`: add one method,
   `pub(crate) fn letter_slice(&self) -> &[char]`, which borrows the letters
   `letters()` clones. `letters()` stays as it is.
2. `crates/core/src/genbank.rs`: add two methods to `GenBankRecord`,
   `pub(crate) fn sequence_ref(&self) -> &Sequence` and
   `pub(crate) fn feature_slice(&self) -> &[GenBankFeature]`. The existing
   accessors stay as they are.
3. `crates/core/src/extraction.rs`: `extract` reads `sequence.letter_slice()`
   instead of `sequence.letters()`. Nothing else changes in that file.
4. `crates/core/src/genbank_cds.rs`: the changes in §3, and the tests in §4.

Everything else is out of scope, including the other `letters()` callers.

## 3. What changes in `genbank_cds.rs`

Every result, error, rule, line, detail and order stays exactly as it is
now. Only the work done to reach them changes.

1. **One survey per call.** `import_cds` builds, once, what every
   evaluation needs: the borrowed feature slice, the borrowed sequence, and
   a map from CDS id text to the key lines of the `CDS` features with that
   id, in file order. This is what `shared_lines` gives today. Only
   features whose key is `CDS` count, as `is_named_cds` checks; a `gene`
   feature with the same `/locus_tag` must not make a CDS ambiguous.
   `evaluate` takes these by reference instead of cloning.
   `std::collections` only.
2. **Step 1, a repeated name.** The refusal names the **first name, in
   request order, that occurs again later**: `[a, b, b, a]` is refused
   naming `a`, not `b`. Counting first and then walking the names in order
   gives this; a set filled while walking does not.
3. **Step 2.** Each name is found through a map from id text to the indices
   of `CDS` features with that id, in file order: none is `NotFound`, two or
   more is `AmbiguousName` with the second feature's line, as now.
4. **Step 3.** `used` is a set.
5. Helpers that become unused are removed, rather than left behind with an
   `allow`.

## 4. Tests the worker writes

Units in `genbank_cds.rs`'s `mod tests`. Existing tests stay exactly as they
are, and all of them must pass. They pin every result this contract must
keep: `genbank_fixtures.rs`, the `synthetic-expected.tsv` and
`synthetic-unused.tsv` ambiguous rows, `view_record_fixtures.rs`.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `the_first_name_repeated_later_is_the_one_refused` | on `synthetic.gb`, names `[p1, m1, m1, p1]` → `RepeatedName` naming `p1`, `line` and `detail` `None` |
| 2 | `a_thousand_cdss_with_their_genes_import_in_file_order` | a GenBank text built in the test: 1 000 genes, each a `gene` and a `CDS` feature on the same span with the same `/gene` and `/locus_tag` (`b00000` …), each CDS `ATG` + 331 × `GCT` + `TAA` and 101 `C`s between genes. `import_cds(&record, &[])`: 2 001 unused features (the source, 1 000 genes, 1 000 CDSs), every CDS with no refusal. `import_cds` of all 1 000 locus tags in reverse order: 1 000 CDSs returned in that reverse order, and an unused list of the source and the 1 000 genes |

Test 2 is a correctness test at scale. The old code passes it, slowly; it
pins that the gene features do not make the CDSs ambiguous.

## 5. Checks

`scripts/check`, plus `python3 -I tests/fixtures/verify.py` (still
`all 24 checks passed`). Report how long test 2 takes in the debug build.

## 6. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | Step 1 with a set filled while walking the names | 1 |
| G2 | The id map counts every feature with the id, not only `CDS` | 2, and `genbank_fixtures` |
| G3 | `AmbiguousName` reports the first feature's line, not the second's | `genbank_fixtures` (name the test) |

## 7. Do not

- No change to any result, error, rule, line, detail or order; no change to
  any existing test or fixture.
- No new dependency; no public API change (the three new methods are
  `pub(crate)`).
- Do not touch `view_record.rs` or any other `letters()` caller.

## 8. Report

Every guard experiment with its failure line, and test 2's debug-build time.
