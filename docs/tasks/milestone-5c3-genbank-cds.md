# Milestone 5c3: importing named CDSs from a GenBank record

Status: **Implemented 2026-10-17** by the local worker, across two runs (the first hit the turn limit with the integration test unfinished); accepted on review. Finding: the §4.1 `AmbiguousName` check is redundant, because §4.2 step 3 gives the same error for a named import, so disabling it alone fails no test (§6.5). It is kept as a shortcut.
Planner: Claude. Parent: `docs/tasks/milestone-5-cli.md` §3.3 (GenBank) and
§5 (5c3). Builds on 5c1 (`genbank_location.rs`) and 5c2 (`genbank.rs`), both
committed.

## 1. What and why

A request names the CDSs it wants from a GenBank record, by id. This
contract turns those features into `Cds` values, each checked against every
rule luria applies, and lists every other feature as *not used*. For an
unused CDS, the list also gives the refusal luria would have given it. A
problem in an unused feature never fails the run.

## 2. Requirements this contract applies (`docs/decisions.md` D041)

- **G1:** only `CDS` features are candidates. Each requested id must match
  exactly one CDS feature; zero or more than one is an error. A named CDS
  luria can't represent fails the run with that feature's own error. Every
  other feature, of any type, is listed as not used, with the reason when
  it's a CDS luria couldn't have used.
- **G2:** a CDS's id is the first of `/locus_tag`, `/gene`, `/label`
  (priority order, not file order). The result records which qualifier it
  came from. A CDS with none of the three can't be named.
- **G3:** when a named CDS has `/translation`, it must equal luria's
  translation without the terminal stop. A mismatch refuses the CDS, and the
  error names the first differing residue.
- **G5 (added 2026-10-17):** `/transl_table` absent or `1` is the standard
  code. `11` is accepted only when the CDS's first codon is `ATG`, because
  tables 1 and 11 assign every codon the same amino acid and differ only in
  alternative starts. The CDS is then imported as the standard code, and the
  record's table is kept for provenance. Any other start under 11, and every
  other table, is refused.
- **Plan §3.3:** `/codon_start` other than 1 and `/pseudo` are refused for a
  named CDS. Other qualifiers are ignored.
- **D006, D036:** coordinates come from `parse_genbank_location` (5c1) as
  zero-based half-open pairs in D036 order.

## 3. Error variants (in `crates/core/src/error.rs`)

```rust
/// Which rule refused a GenBank CDS. Stable: `code()` is part of the CLI's
/// output contract (D041), and renaming one needs approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenBankCdsRule {
    NotFound,
    AmbiguousName,
    RepeatedName,
    NoId,
    RepeatedQualifier,
    Pseudo,
    CodonStart,
    TranslTable,
    AlternativeStart,
}
```

`impl GenBankCdsRule` has `code(self) -> &'static str` and
`category(self) -> ErrorCategory`, each an exhaustive match with no `_` arm:

| Rule | Code | Category |
| --- | --- | --- |
| `NotFound` | `not_found` | `BiologicalValidation` |
| `AmbiguousName` | `ambiguous_name` | `BiologicalValidation` |
| `RepeatedName` | `repeated_name` | `MalformedInput` |
| `NoId` | `no_id` | `Unsupported` |
| `RepeatedQualifier` | `repeated_qualifier` | `MalformedInput` |
| `Pseudo` | `pseudo` | `Unsupported` |
| `CodonStart` | `codon_start` | `Unsupported` |
| `TranslTable` | `transl_table` | `Unsupported` |
| `AlternativeStart` | `alternative_start` | `Unsupported` |

```rust
/// Payload: a GenBank CDS luria cannot import. `cds` is the requested name
/// or the feature's id (empty for `NoId`). `line` is the feature's key line,
/// `None` for `NotFound` and `RepeatedName`. `detail` is the offending
/// value: the qualifier's value for `CodonStart` and `TranslTable` (`None`
/// for a value-less flag), the first codon for `AlternativeStart`, the
/// qualifier name for `RepeatedQualifier` and `Pseudo`; `None` otherwise.
#[derive(Clone, Debug, PartialEq)]
pub struct UnusableGenBankCdsProblem {
    pub cds: String,
    pub rule: GenBankCdsRule,
    pub line: Option<u64>,
    pub detail: Option<String>,
}

/// Payload: a CDS whose `/translation` differs from luria's translation
/// without the terminal stop (D041 G3). `residue` is the first differing
/// residue, counted from 1; `stated` and `translated` are the letters there,
/// `None` where one translation has already ended.
#[derive(Clone, Debug, PartialEq)]
pub struct GenBankTranslationMismatchProblem {
    pub cds: String,
    pub line: u64,
    pub residue: u64,
    pub stated: Option<String>,
    pub translated: Option<String>,
}
```

Add `DomainError::UnusableGenBankCds(UnusableGenBankCdsProblem)` and then
`DomainError::GenBankTranslationMismatch(GenBankTranslationMismatchProblem)`
as the last two variants, after `MalformedGenBank`. Then:
- `category()`: `problem.rule.category()` for the first, and
  `BiologicalValidation` for the second;
- `code()`: `"unusable_genbank_cds"` and `"genbank_translation_mismatch"`;
- `message()`. Below, `{c}` is `cds` and `{d}` is `detail`, or `(no value)`
  when `detail` is `None`. Append ` (line {n})` whenever `line` is
  `Some(n)`, and always for the mismatch.

| Rule | Message |
| --- | --- |
| `NotFound` | `GenBank CDS '{c}' is not the id of any CDS feature in the record` |
| `AmbiguousName` | `GenBank CDS '{c}' is the id of more than one CDS feature` |
| `RepeatedName` | `GenBank CDS '{c}' is named more than once in the request` |
| `NoId` | `a GenBank CDS has none of /locus_tag, /gene or /label with a value, so it cannot be named` |
| `RepeatedQualifier` | `GenBank CDS '{c}' repeats /{d}` |
| `Pseudo` | `GenBank CDS '{c}' is marked /{d}, so it is not a coding sequence luria edits` |
| `CodonStart` | `GenBank CDS '{c}' has /codon_start {d}; only 1 is supported` |
| `TranslTable` | `GenBank CDS '{c}' has /transl_table {d}; only 1, and 11 with an ATG start, are supported` |
| `AlternativeStart` | `GenBank CDS '{c}' has /transl_table 11 and starts with {d}; table 11 is read as the standard code only for an ATG start` |

Mismatch: `GenBank CDS '{c}' states a /translation that differs from luria's at residue {residue}: the record has {s}, luria has {t} (line {line})`, where `{s}` and `{t}` are the letters, or `nothing` for `None`.

## 4. The import: `crates/core/src/genbank_cds.rs`

Add `pub mod genbank_cds;` to `lib.rs` in alphabetical order.

```rust
/// Which qualifier a CDS's id came from (D041 G2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CdsIdQualifier { LocusTag, Gene, Label }
impl CdsIdQualifier {
    /// `locus_tag`, `gene` or `label`, as the qualifier is written.
    pub fn name(self) -> &'static str;
}

/// A CDS imported from a GenBank feature, and where it came from.
#[derive(Clone, Debug, PartialEq)]
pub struct ImportedCds { cds: Cds, id_qualifier: CdsIdQualifier, index: usize, line: u64, transl_table: Option<String> }
impl ImportedCds {
    pub fn cds(&self) -> Cds;
    pub fn id_qualifier(&self) -> CdsIdQualifier;
    /// The feature's position in `GenBankRecord::features()`, from 0.
    pub fn index(&self) -> usize;
    /// The feature's key line.
    pub fn line(&self) -> u64;
    /// The record's `/transl_table` value as written, `None` when absent.
    pub fn transl_table(&self) -> Option<String>;
}

/// A feature the request did not import.
#[derive(Clone, Debug, PartialEq)]
pub struct UnusedFeature { index: usize, key: String, line: u64, id: Option<String>, refusal: Option<DomainError> }
impl UnusedFeature {
    pub fn index(&self) -> usize;
    pub fn key(&self) -> String;
    pub fn line(&self) -> u64;
    /// For a CDS, its id under G2 when it has one; `None` for other keys.
    pub fn id(&self) -> Option<String>;
    /// For a CDS, the error importing it would give; `None` if it could have
    /// been imported, and for every other key.
    pub fn refusal(&self) -> Option<DomainError>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct GenBankImport { cds: Vec<ImportedCds>, unused: Vec<UnusedFeature> }
impl GenBankImport {
    /// In the order the request named them.
    pub fn cds(&self) -> Vec<ImportedCds>;
    /// Every other feature, in file order.
    pub fn unused(&self) -> Vec<UnusedFeature>;
}

pub fn import_cds(record: &GenBankRecord, names: &[Identifier]) -> Result<GenBankImport, DomainError>;
```

### 4.1 `import_cds`

1. If a name appears more than once in `names`, refuse the first repeated
   one: `RepeatedName`, `line: None`, `detail: None`. This check comes
   before anything else.
2. For each name, in order, find the **candidates**: features whose key is
   exactly `CDS` and whose id text (§4.2 step 1) equals `name.value()`.
   - None → `NotFound`, `line: None`.
   - Two or more → `AmbiguousName`, with the line of the second candidate in
     file order.
   - One → evaluate it (§4.2). An error is returned as the result.
3. The unused list: every feature not imported, in file order. A `CDS`
   feature is evaluated (§4.2): `id` is its id text if step 1 found one,
   and `refusal` is `Some(error)` if evaluation fails, `None` if it succeeds.
   For any other key, both are `None`. Evaluating unused features never
   makes `import_cds` fail.

`names` may be empty. The import then has no CDSs, and every feature is in
the unused list.

### 4.2 Evaluating one CDS feature

The steps run in this order, and the first failure is the result. `seq` is
`record.sequence()`. `line` is the feature's key line. `{id}` is the id text
from step 1.

1. **Id:** the first of the names `locus_tag`, `gene`, `label` that appears
   among the qualifiers, and the value of its first occurrence. If none
   appears, or that value is `None`, the result is `NoId` with `cds: ""`.
2. `Identifier::create(id)`, whose errors are passed through
   (`EmptyIdentifier`, `InvalidIdentifierCharacters`).
3. **Shared id:** if two or more `CDS` features in the record have this id
   text, the result is `AmbiguousName` with the line of the second such
   feature. (For a named import, §4.1 step 2 has already reported this. The
   step matters for the unused list.)
4. **Repeated qualifier:** check, in this order, the id qualifier's name,
   `codon_start`, `transl_table` and `translation`. The first that occurs
   more than once gives `RepeatedQualifier`, with that name as the detail.
5. **Pseudo:** `pseudo`, then `pseudogene`. The first present, with or
   without a value, gives `Pseudo` with that name as the detail.
6. **Codon start:** `codon_start` present with a value other than
   `Some("1")` gives `CodonStart`, with the value as the detail.
7. **Table:** `transl_table` present with a value other than `Some("1")` or
   `Some("11")` gives `TranslTable`, with the value as the detail.
8. **Location:** `parse_genbank_location(&feature.location(), seq.length(),
   seq.topology())`, whose errors are passed through.
9. **CDS:** build each interval with `interval::create(&seq.reference(),
   start as i64, end as i64)`, then call `Cds::create(&seq, id, intervals,
   strand, Completeness::Complete, STANDARD_CODE_NAME.to_string(),
   Numbering::FromCdsStart)`. Errors are passed through.
10. **Translate:** `translate(&seq, &cds)`, whose errors are passed through
    (`CdsLengthNotDivisibleByThree`, `InternalStopCodon` and so on).
11. **Start under table 11:** if `transl_table` is `Some("11")` and the
    first three letters of `extract_cds(&seq, &cds)` are not `ATG`, the
    result is `AlternativeStart` with those three letters as the detail.
12. **`/translation`:** if present, `stated` is its value (`""` for a flag)
    with every space removed, and `translated` is the translation's
    `residues_text()`. If they differ, the result is
    `GenBankTranslationMismatch` at the first index where they differ, or at
    the shorter length + 1 when one is a prefix of the other.
13. Otherwise, the result is
    `ImportedCds { cds, id_qualifier, index, line, transl_table }`.

Matching is exact and case-sensitive. Nothing is repaired or defaulted
beyond what §2 states.

Write a module doc comment summarizing §4.1 and §4.2, citing D041 (G1, G2,
G3, G5), D006 and D036.

## 5. Fixtures (planner-written; copy them in)

The planner prepared every fixture file under `target/tmp/genbank-staging/`.
Copy that tree to `tests/fixtures/genbank/` unchanged. Afterwards, from the
repository root, `cd tests/fixtures/genbank && find . -type f | sort | xargs
sha256sum` must print exactly:

```
9943f8440d2a0e3b51b8d13ec344b43b3306c27500ec63ffff33d6177813f4fb  ./pbr322-cds.tsv
78ac721e2bb6152ee39da8c3b949a3440a6b1046f2ee3ff4174b7d67bce3029a  ./pbr322-features.tsv
7bf3fb382b03dab0fcdefd882a44cecb396bfbb65d952c08042379fc5acddb0c  ./pbr322-record.tsv
7660b0fe95a91d492b102192dc72924508a69fda762988aa0923b3d5cc2073f6  ./raw/J01749.1.gb
711de28856684d73577a9cb67feb335ba5ba1ed20ac81f4b261731ef359d96d8  ./README.md
e07be135056206ca8b47557a46e257d098be9086960262f77dff855fcca1d3db  ./reference/pbr322.py
dddf2eb2e71c84e69a449ccfbb246677bb97501ae16e4175c1c506eeee40e7de  ./reference/verify_synthetic.py
5f0feab1f2977123f5452ed16fc905f6196f18a3735d5c01a7faed88b4b451c8  ./synthetic-expected.tsv
0c59619aa84f37cc1395667e56fd4b8fc7fed7e357655ffdce65de3598c234f3  ./synthetic.gb
0a44f0138bcee5fb961f8b1c9f8ccbc6bff0367aa90289ecf85f1accdfc4964f  ./synthetic-unused.tsv
```

If any digest differs, stop and report. Never edit these files. Read
`tests/fixtures/genbank/README.md`, which explains every expectation and how
it was verified.

Then:
- `tests/fixtures/verify.py`: append `"genbank/reference/pbr322.py",` and
  `"genbank/reference/verify_synthetic.py",` to `CHECKS`, in that order,
  after the FASTA entry. Change nothing else.
- `tests/fixtures/README.md`: add this row after the `fasta/` row:

  ```markdown
  | [`genbank/`](./genbank/) | pBR322 (NCBI J01749.1) read by Biopython, and a hand-written synthetic record with one CDS per accepted form and per refusal. Drives `crates/core/tests/genbank_fixtures.rs`. |
  ```

  and in the "Rerunning fixture verification" paragraph, change "the
  codon-usage parser, and the FASTA digest check." to "the codon-usage
  parser, the FASTA digest check, and the two GenBank checks."

The pinned Python environment is at `target/venv` (Biopython 1.88). Run
`target/venv/bin/python -I tests/fixtures/verify.py`. All 12 checks must
pass.

### 5.1 Table formats

`synthetic-expected.tsv` has one row per request name, imported on its own
with `import_cds(&record, &[name])`:
- `error`: `-` for success, otherwise the error's `code()`;
- `rule`: the rule's `code()` for `unusable_genbank_cds` and
  `unusable_genbank_location`, otherwise `-`;
- `intervals`: `start-end` pairs joined by `,`; `strand`: `plus` or
  `minus`; `id_qualifier`: `CdsIdQualifier::name()`; `transl_table`: the
  value or `-`. All are `-` on failure;
- `residues`: on success, `translate(&seq, &cds).residues_text()`; for a
  mismatch, luria's translation. `-` otherwise;
- `detail`:
  - for `unusable_genbank_cds`, `problem.detail` or `-`;
  - for `genbank_translation_mismatch`, `{residue}:{stated or -}:{translated or -}`;
  - for `internal_stop_codon`, `problem.codon_number`;
  - for `cds_length_not_divisible_by_three`, `problem.length`;
  - otherwise `-`;
- `line`: `problem.line` for `unusable_genbank_cds` (when `Some`) and for
  the mismatch, otherwise `-`.

`synthetic-unused.tsv` lists `import_cds(&record, &[])`'s unused list:
`index`, `key`, `id` (or `-`), and `refusal`, which is `-`, the error's
`code()`, or `code:rule` when the error has a rule (`unusable_genbank_cds`,
`unusable_genbank_location`).

## 6. Tests

### 6.1 Integration test: `crates/core/tests/genbank_fixtures.rs`

Write it in the style of `crates/core/tests/fasta_fixtures.rs`, with a module
doc comment saying what is checked against what and that the Python checks
are not part of the cargo checks. Use a small helper to parse each TSV,
checking its header row exactly. Tests, with these names:

1. `pbr322_reads_as_biopython_reads_it`: `read_genbank` on
   `raw/J01749.1.gb` (identifier `pbr322`). Compare with `pbr322-record.tsv`:
   locus name, accession, version, length, `topology().name()`, content
   digest, and `features().len()`. Then compare with `pbr322-features.tsv`:
   the same keys in the same order.
2. `pbr322_imports_tet_and_bla_as_biopython_reads_them`: `import_cds` with
   `["tet", "bla"]`. Two CDSs, in that order. For each, its
   `pbr322-cds.tsv` row gives `index`, `[(start, end)]`, strand,
   `id_qualifier` (`gene`) and `transl_table` (`Some("11")`), and
   `translate(...).residue_count()` equals `residues`.
3. `pbr322_lists_every_other_feature_as_unused`: the same import. The unused
   list's indices are exactly 0..50 without 10 and 38, in order. The keys
   match `pbr322-features.tsv`. Every non-CDS has `id` and `refusal` `None`.
   The CDS at index 26 (ROP) has `id` `None`, and its refusal is
   `UnusableGenBankCds` with rule `NoId` and line equal to its `line()`.
4. `the_synthetic_record_imports_each_name_as_expected`: one
   `import_cds(&record, &[name])` per row of `synthetic-expected.tsv`,
   checked against every column per §5.1.
5. `the_synthetic_record_lists_every_feature_as_unused`:
   `import_cds(&record, &[])` against `synthetic-unused.tsv`.
6. `imported_cdss_keep_the_request_order_and_leave_the_unused_list`:
   `["m1", "p1"]` gives `m1` then `p1`. The unused list has 25 entries and
   contains neither index 1 nor index 2.
7. `a_name_repeated_in_the_request_is_refused`: `["p1", "m1", "p1"]` gives
   `UnusableGenBankCds` with `RepeatedName`, `cds` `p1`, `line` `None`.
8. `origin_joins_are_refused_on_a_linear_copy`: the synthetic text with
   `text.replacen("circular", "linear", 1)` (the first occurrence is on the
   LOCUS line). `pj`, `mj1` and `mj2` each give `UnusableGenBankLocation`
   with rule `OriginJoinOnLinear`. `p1` still imports, and its CDS's
   reference has `Linear` topology.

If any comparison fails, do not change a fixture or an expectation. Stop
and report the row, the expected value and what luria returned.

### 6.2 Unit tests in `genbank_cds.rs` (planner-written; add verbatim)

The tiny record is `ATGGCTTGGTAA` (M A W stop), linear. Its single feature
key is on line 3.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cds::Completeness;
    use crate::cds::Numbering;
    use crate::error::GenBankCdsRule;
    use crate::error::UnusableGenBankCdsProblem;
    use crate::genbank::read_genbank;
    use crate::genetic_code::GeneticCode;

    fn key(name: &str, location: &str) -> String {
        format!("     {:<16}{}", name, location)
    }

    fn q(text: &str) -> String {
        format!("{:21}{}", "", text)
    }

    fn record(features: &[String]) -> GenBankRecord {
        let mut lines: Vec<String> = vec![
            "LOCUS       tiny                      12 bp    DNA     linear   SYN 17-OCT-2026"
                .to_string(),
            "FEATURES             Location/Qualifiers".to_string(),
        ];
        lines.extend(features.iter().cloned());
        lines.push("ORIGIN".to_string());
        lines.push("        1 atggcttggt aa".to_string());
        lines.push("//".to_string());
        read_genbank(Identifier::create("tiny".to_string()).unwrap(), lines.join("\n")).unwrap()
    }

    fn names(list: &[&str]) -> Vec<Identifier> {
        list.iter()
            .map(|name| Identifier::create(name.to_string()).unwrap())
            .collect()
    }

    fn cds_problem(error: DomainError) -> UnusableGenBankCdsProblem {
        match error {
            DomainError::UnusableGenBankCds(problem) => problem,
            other => panic!("expected UnusableGenBankCds, got {:?}", other),
        }
    }

    #[test]
    fn a_pseudogene_qualifier_is_refused_by_its_name() {
        let found = record(&[key("CDS", "1..12"), q("/gene=\"g\""), q("/pseudogene=\"unitary\"")]);
        let problem = cds_problem(import_cds(&found, &names(&["g"])).unwrap_err());
        assert_eq!(problem.rule, GenBankCdsRule::Pseudo);
        assert_eq!(problem.detail, Some("pseudogene".to_string()));
        assert_eq!(problem.line, Some(3));
    }

    #[test]
    fn a_codon_start_without_a_value_is_refused() {
        let found = record(&[key("CDS", "1..12"), q("/gene=\"g\""), q("/codon_start")]);
        let problem = cds_problem(import_cds(&found, &names(&["g"])).unwrap_err());
        assert_eq!(problem.rule, GenBankCdsRule::CodonStart);
        assert_eq!(problem.detail, None);
    }

    #[test]
    fn table_1_is_the_standard_code_and_is_recorded() {
        let found = record(&[key("CDS", "1..12"), q("/gene=\"g\""), q("/transl_table=1")]);
        let imported = import_cds(&found, &names(&["g"])).unwrap();
        assert_eq!(imported.cds()[0].transl_table(), Some("1".to_string()));
    }

    #[test]
    fn a_wrapped_translation_is_compared_without_its_spaces() {
        let found = record(&[
            key("CDS", "1..12"),
            q("/gene=\"g\""),
            q("/translation=\"MA"),
            q("W\""),
        ]);
        assert!(import_cds(&found, &names(&["g"])).is_ok());
    }

    #[test]
    fn a_valueless_id_qualifier_cannot_name_a_cds() {
        let found = record(&[key("CDS", "1..12"), q("/gene")]);
        let problem = cds_problem(import_cds(&found, &names(&["g"])).unwrap_err());
        assert_eq!(problem.rule, GenBankCdsRule::NotFound);
        let unused = import_cds(&found, &[]).unwrap().unused();
        let refusal = cds_problem(unused[0].refusal().unwrap());
        assert_eq!(refusal.rule, GenBankCdsRule::NoId);
        assert_eq!(refusal.cds, "");
        assert_eq!(unused[0].id(), None);
    }

    #[test]
    fn the_imported_cds_is_complete_standard_and_numbered_from_its_start() {
        let found = record(&[key("CDS", "1..12"), q("/label=g")]);
        let imported = import_cds(&found, &names(&["g"])).unwrap().cds();
        let cds = imported[0].cds();
        assert_eq!(cds.id().value(), "g");
        assert_eq!(cds.reference(), found.sequence().reference());
        assert_eq!(cds.completeness(), Completeness::Complete);
        assert_eq!(cds.genetic_code(), GeneticCode::Standard);
        assert_eq!(cds.numbering(), Numbering::FromCdsStart);
        assert_eq!(imported[0].id_qualifier(), CdsIdQualifier::Label);
        assert_eq!(imported[0].id_qualifier().name(), "label");
        assert_eq!(imported[0].line(), 3);
        assert_eq!(imported[0].index(), 0);
        assert_eq!(imported[0].transl_table(), None);
    }

    #[test]
    fn each_message_reads_as_the_contract_states() {
        let found = record(&[key("CDS", "1..12"), q("/gene=\"g\""), q("/translation=\"MAV\"")]);
        assert_eq!(
            import_cds(&found, &names(&["g"])).unwrap_err().message(),
            "GenBank CDS 'g' states a /translation that differs from luria's at residue 3: the record has V, luria has W (line 3)"
        );
        assert_eq!(
            import_cds(&found, &names(&["h"])).unwrap_err().message(),
            "GenBank CDS 'h' is not the id of any CDS feature in the record"
        );
        let anonymous = record(&[key("CDS", "1..12")]);
        let unused = import_cds(&anonymous, &[]).unwrap().unused();
        assert_eq!(
            unused[0].refusal().unwrap().message(),
            "a GenBank CDS has none of /locus_tag, /gene or /label with a value, so it cannot be named (line 3)"
        );
        let table = record(&[key("CDS", "1..12"), q("/gene=\"g\""), q("/transl_table=4")]);
        assert_eq!(
            import_cds(&table, &names(&["g"])).unwrap_err().message(),
            "GenBank CDS 'g' has /transl_table 4; only 1, and 11 with an ATG start, are supported (line 3)"
        );
    }
}
```

`cargo fmt` may reflow; do not change content. If an accessor named here
has a different name in core (for example `Cds::genetic_code`), stop and
report rather than changing the test.

### 6.3 `error.rs` tests

- In `every_variant_has_its_pinned_code`, add two cases after
  `MalformedGenBank`: `UnusableGenBankCds(UnusableGenBankCdsProblem { cds:
  s(), rule: GenBankCdsRule::NotFound, line: None, detail: None })` with code
  `"unusable_genbank_cds"`, and `GenBankTranslationMismatch(
  GenBankTranslationMismatchProblem { cds: s(), line: 1, residue: 1, stated:
  None, translated: None })` with code `"genbank_translation_mismatch"`.
  Change the count from 50 to 52.
- Add after `every_genbank_rule_has_its_pinned_code`:

```rust
    #[test]
    fn every_genbank_cds_rule_has_its_pinned_code_and_category() {
        use ErrorCategory::*;
        use GenBankCdsRule::*;
        let cases: [(GenBankCdsRule, &str, ErrorCategory); 9] = [
            (NotFound, "not_found", BiologicalValidation),
            (AmbiguousName, "ambiguous_name", BiologicalValidation),
            (RepeatedName, "repeated_name", MalformedInput),
            (NoId, "no_id", Unsupported),
            (RepeatedQualifier, "repeated_qualifier", MalformedInput),
            (Pseudo, "pseudo", Unsupported),
            (CodonStart, "codon_start", Unsupported),
            (TranslTable, "transl_table", Unsupported),
            (AlternativeStart, "alternative_start", Unsupported),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (rule, code, category) in cases {
            assert_eq!(rule.code(), code, "{:?}", rule);
            assert!(rule.category() == category, "{:?}", rule);
            assert!(seen.insert(code), "duplicate code {}", code);
        }
    }
```

### 6.4 The three `variant_name` matches

Each gains two arms after `MalformedGenBank`:
`DomainError::UnusableGenBankCds(_) => "UnusableGenBankCds",` and
`DomainError::GenBankTranslationMismatch(_) => "GenBankTranslationMismatch",`.

### 6.5 Proving each rule is load-bearing (AGENTS.md)

For each rule, disable its check, run `cargo test -p luria-core --test
genbank_fixtures` together with `cargo test -p luria-core genbank_cds`,
confirm that a named test fails, then restore. Report the failing tests for
each.

| Rule | A test that must fail |
| --- | --- |
| `RepeatedName` | `a_name_repeated_in_the_request_is_refused` |
| `NotFound` | `the_synthetic_record_imports_each_name_as_expected` (row `nosuch`) |
| `AmbiguousName` (both §4.1 and §4.2 step 3) | `the_synthetic_record_imports_each_name_as_expected` (row `dup`) and `the_synthetic_record_lists_every_feature_as_unused` (indices 15, 16) |
| `NoId` | `pbr322_lists_every_other_feature_as_unused` |
| `RepeatedQualifier` | `the_synthetic_record_imports_each_name_as_expected` (row `rep`) |
| `Pseudo` | `a_pseudogene_qualifier_is_refused_by_its_name` |
| `CodonStart` | `a_codon_start_without_a_value_is_refused` |
| `TranslTable` | `each_message_reads_as_the_contract_states` |
| `AlternativeStart` | `the_synthetic_record_imports_each_name_as_expected` (row `alt11`) |
| translation mismatch | `the_synthetic_record_imports_each_name_as_expected` (rows `badtr`, `shorttr`) |

Disable `AmbiguousName` in §4.1 and in §4.2 step 3 as two separate rounds.

## 7. Do not

- Edit files other than `crates/core/src/error.rs`,
  `crates/core/src/genbank_cds.rs` (new), `crates/core/src/lib.rs`,
  `crates/core/tests/genbank_fixtures.rs` (new), the three `variant_name`
  arms, `tests/fixtures/verify.py`, `tests/fixtures/README.md`, and the
  copied `tests/fixtures/genbank/` tree.
- Edit any copied fixture file, or regenerate one with the scripts. The
  scripts' `--check` mode is the only way to run them.
- Change `genbank.rs`, `genbank_location.rs`, `cds.rs`, `translation.rs` or
  any other existing behavior. If one gets in the way, stop and report.
- Add a genetic code, or accept `/transl_table=11` with a non-ATG start.
- Add dependencies.

## 8. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`
- `target/venv/bin/python -I tests/fixtures/verify.py`, with all 12 checks
  passing.
- the `sha256sum` listing in §5.

## 9. Handoff

Report the changed and new files, the exact commands and results, the §6.5
disable-and-restore results, and anything that did not match this contract.
