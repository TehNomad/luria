# Milestone 5c2: the GenBank record reader

Status: **Implemented 2026-10-17** by the local worker; accepted after one review round (§4.6, §4.7 clarified; two tests added).
Planner: Claude. Parent: [docs/tasks/contracts/milestone-5-cli.md](/tasks/contracts/milestone-5-cli.md) §3.3 (GenBank) and
§5 (5c2). Follows 5c1 (`genbank_location.rs`, committed).

## 1. What and why

This contract reads the *structure* of one GenBank flat-file record: the
LOCUS line (name, length, topology), ACCESSION and VERSION, the FEATURES
table (keys, raw location text, qualifiers), the ORIGIN sequence and the
`//` terminator. It does **not** parse locations or apply any CDS rule.
5c3 does that, for the CDS features a request names, so that a problem in a
feature nobody asked for never fails a run (D041). Feature locations are
therefore kept as text here.

## 2. Requirements this contract applies

- **D007:** the sequence carries a caller-supplied identifier. The LOCUS
  name, ACCESSION and VERSION are source metadata, not identity.
- **D008:** nothing is repaired. The ORIGIN letters go through
  `sequence::normalize` unchanged, so case folding and the alphabet check
  stay in one place.
- **D009, D041 (G4):** topology comes from the LOCUS line. A LOCUS line that
  states neither `linear` nor `circular` is refused.
- **D041:** one record per file. A second record after `//` is refused.
  Header sections other than ACCESSION and VERSION (DEFINITION, KEYWORDS,
  SOURCE, REFERENCE, COMMENT, …) are not interpreted.

## 3. Error variant and rule (in `crates/core/src/error.rs`)

```rust
/// Which rule refused a GenBank text. Stable: `code()` is part of the CLI's
/// output contract (D041), and renaming one needs approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenBankRule {
    NoLocus,
    LocusLength,
    TopologyMissing,
    TopologyConflict,
    RepeatedSection,
    LineLayout,
    QualifierSyntax,
    NoOrigin,
    OriginLine,
    OriginNumbering,
    LengthMismatch,
    MissingEnd,
    SecondRecord,
    TextAfterEnd,
}
```

`impl GenBankRule { pub fn code(self) -> &'static str }`: exhaustive, literal
strings, no `_` arm. Codes: `no_locus`, `locus_length`, `topology_missing`,
`topology_conflict`, `repeated_section`, `line_layout`, `qualifier_syntax`,
`no_origin`, `origin_line`, `origin_numbering`, `length_mismatch`,
`missing_end`, `second_record`, `text_after_end`.

```rust
/// Payload: a GenBank text that is not one readable record. `line` counts
/// text lines from 1; it is not a sequence coordinate. It is `None` when
/// there is no line to point at (an empty text, or a text that ends early).
#[derive(Clone, Debug, PartialEq)]
pub struct MalformedGenBankProblem {
    pub sequence: String,
    pub rule: GenBankRule,
    pub line: Option<u64>,
}
```

Add `DomainError::MalformedGenBank(MalformedGenBankProblem)` as the last
variant, after `UnusableGenBankLocation`. Then:
- `category()`: `MalformedInput` for every rule;
- `code()`: `"malformed_genbank"`;
- `message()`: `GenBank text for '{sequence}': {reason}`, followed by
  ` (line {n})` when `line` is `Some(n)`. Reasons:

| Rule | Reason |
| --- | --- |
| `NoLocus` | `it does not start with a LOCUS line` |
| `LocusLength` | `the LOCUS line does not give a length in bp` |
| `TopologyMissing` | `the LOCUS line states neither linear nor circular` |
| `TopologyConflict` | `the LOCUS line states more than one topology` |
| `RepeatedSection` | `a section that appears once per record appears again` |
| `LineLayout` | `a line fits no part of the record layout` |
| `QualifierSyntax` | `a qualifier is not /name or /name=value, or its quoted value is not closed` |
| `NoOrigin` | `the record has no ORIGIN sequence` |
| `OriginLine` | `a line in ORIGIN is not a position number followed by bases` |
| `OriginNumbering` | `a line in ORIGIN is numbered other than one past the bases before it` |
| `LengthMismatch` | `the LOCUS line's length differs from the ORIGIN sequence's` |
| `MissingEnd` | `the record does not end with a // line` |
| `SecondRecord` | `a second record starts after //; only one record is read` |
| `TextAfterEnd` | `text follows the record's // line` |

## 4. The reader: `crates/core/src/genbank.rs`

Add `pub mod genbank;` to `lib.rs` in alphabetical order.

```rust
/// One qualifier of a feature: `/name` (value `None`) or `/name=value`.
#[derive(Clone, Debug, PartialEq)]
pub struct GenBankQualifier { name: String, value: Option<String> }
impl GenBankQualifier {
    pub fn name(&self) -> String;
    pub fn value(&self) -> Option<String>;
}

/// One entry of the FEATURES table. The location is text: 5c3 parses it,
/// and only for the CDSs a request names.
#[derive(Clone, Debug, PartialEq)]
pub struct GenBankFeature {
    key: String,
    location: String,
    qualifiers: Vec<GenBankQualifier>,
    line: u64,
}
impl GenBankFeature {
    pub fn key(&self) -> String;
    /// The location text, its continuation lines appended with no separator.
    pub fn location(&self) -> String;
    /// In the order written.
    pub fn qualifiers(&self) -> Vec<GenBankQualifier>;
    /// The text line of the feature key, counted from 1.
    pub fn line(&self) -> u64;
}

/// One GenBank record: source metadata, the sequence, and the feature table.
#[derive(Clone, Debug, PartialEq)]
pub struct GenBankRecord {
    locus_name: String,
    accession: Option<String>,
    version: Option<String>,
    sequence: Sequence,
    features: Vec<GenBankFeature>,
}
impl GenBankRecord {
    pub fn locus_name(&self) -> String;
    pub fn accession(&self) -> Option<String>;
    pub fn version(&self) -> Option<String>;
    pub fn sequence(&self) -> Sequence;
    pub fn features(&self) -> Vec<GenBankFeature>;
}

pub fn read_genbank(name: Identifier, text: String) -> Result<GenBankRecord, DomainError>;
```

### 4.1 Lines

Split `text` with `str::lines()`. Line numbers count from 1. A line is
**blank** if it is empty or holds only spaces, tabs and `\r`. Blank lines
are skipped everywhere, before, inside and after the record. A trailing
`\r` is removed from every line before the rules below look at it. Columns
count spaces only. A tab is never treated as indentation.

### 4.2 Before LOCUS

The first non-blank line must start with `LOCUS` followed by a space or the
end of the line. If it doesn't, the result is `NoLocus` with that line. A
text with no non-blank line is `NoLocus` with `line: None`.

### 4.3 The LOCUS line

Split it on runs of spaces into tokens. `tokens[0]` is `LOCUS`.
1. **Length:** find the first token exactly `bp` at index ≥ 2. The token
   before it must be all ASCII digits and fit in `u64`. Otherwise →
   `LocusLength`.
2. **Topology:** count the tokens that are exactly `linear` or exactly
   `circular`. Zero → `TopologyMissing`; more than one (both, or one
   repeated) → `TopologyConflict`. Matching is exact and lower case.
3. **Name:** `tokens[1]` if the length token is at index ≥ 2 and is not
   `tokens[1]`; otherwise the name is `""`. (In practice it is `tokens[1]`.)

Every LOCUS refusal carries the LOCUS line's number.

### 4.4 Sections

After the LOCUS line, each non-blank line is one of the following:
- **`//`**, the line with trailing spaces removed being exactly `//`, ends
  the record (§4.8).
- A **section line**: its first character is an ASCII upper-case letter. The
  section keyword is the text up to the first space, or the whole line.
- A **continuation line**: its first character is a space. It belongs to the
  current section.
- Anything else, such as a tab, a lower-case letter, a digit or other
  punctuation in column 1, → `LineLayout`.

Keywords `LOCUS`, `ACCESSION`, `VERSION`, `FEATURES` and `ORIGIN` may each
appear once. A second one, before `//`, → `RepeatedSection` with its line.
(The first `LOCUS` is the one §4.2 read.)

- **ACCESSION** and **VERSION:** the value is the first token after the
  keyword on the keyword line, or `None` if there is none. It is kept
  exactly, even if it is `.`. Continuation lines are skipped.
- **FEATURES:** the rest of the keyword line is ignored. Continuation lines
  form the feature table (§4.5).
- **ORIGIN:** the rest of the keyword line is ignored. The sequence lines
  follow (§4.7).
- **Any other keyword** (DEFINITION, KEYWORDS, SOURCE, REFERENCE, COMMENT,
  BASE COUNT, CONTIG, …): the line and its continuation lines are skipped.

### 4.5 The FEATURES table

Each continuation line of FEATURES is one of these:
- A **key line**: exactly 5 spaces, then a non-space. The key is the first
  token. The location starts with the rest of the line after the key, with
  spaces removed from both ends. If that is empty → `LineLayout`.
- A **qualifier-area line**: at least 21 spaces. Its **text** is the line
  with spaces removed from both ends.
- Any other indentation (1–4 or 6–20 spaces) → `LineLayout`.

Qualifier-area lines before the first key line → `LineLayout`.

Each qualifier-area line is read in this order. The first case that applies
decides.
1. If the current qualifier has an **open quoted value** (§4.6), the text is
   a continuation of that value, even if it starts with `/`.
2. Else, if the text starts with `/`, it starts a **new qualifier**. The name
   is the text after `/` up to the first `=` (value follows) or the end
   (value `None`). An empty name, or a name containing a space or `"`, →
   `QualifierSyntax`.
3. Else, if the current feature has no qualifier yet, the text is a
   **location continuation**. It is appended to the location with no
   separator.
4. Else, if the current qualifier's value is unquoted (and not `None`), the
   text is appended to it, separated by one space.
5. Else (the current qualifier is a closed quoted value, or a `/name` with
   no value) → `QualifierSyntax`.

A key line, a section line or `//` ends the current feature. If a quoted
value is still open at that point, the result is `QualifierSyntax` with the
line number of the qualifier's `/name=` line.

### 4.6 Qualifier values

- A value whose first character is `"` is **quoted**. Its text lines are
  joined with one space. It is **open** while the number of `"` characters
  in the joined text is odd: the opening quote, any `""` escapes, and the
  closing quote make an even count. Once the count is even, the joined
  text must end with `"`. If it doesn't (for example `"a"b`), the result is
  `QualifierSyntax`. The stored value is the text inside the outer quotes,
  with each `""` replaced by `"`.
- Any other value is **unquoted**. It may not contain `"`; if it does, the
  result is `QualifierSyntax`. Continuation lines are joined to it with one
  space (§4.5 case 4). A continuation line that holds a `"` is refused the
  same way, with the continuation line's number.
- `/name=` with nothing after it is the value `Some("")`.

Joining with a space is right for wrapped prose. For `/translation`, whose
lines wrap without spaces, 5c3 removes the spaces. The reader treats every
qualifier the same way.

### 4.7 ORIGIN

Each continuation line of ORIGIN is a **sequence line**. Split it on runs of
spaces:
- the first token must be all ASCII digits and fit in `u64`. If not (or the
  line has no token), the result is `OriginLine`;
- that number must equal the count of sequence characters before this line,
  plus 1. If not, the result is `OriginNumbering`;
- the remaining tokens are the line's sequence characters. Their character
  count adds to the running total. A line with a number and no bases is
  allowed.

Any non-blank line in ORIGIN that is neither a continuation line nor `//`
→ `OriginLine`. That includes section lines with any keyword, among them
`ACCESSION`, `VERSION`, `FEATURES`, `ORIGIN` and `LOCUS`. Inside ORIGIN this
takes precedence over `RepeatedSection`, because ORIGIN runs to `//` and
nothing else may follow it. *(Clarified 2026-10-17 in review.)*

### 4.8 The end

- `//` with no ORIGIN section seen → `NoOrigin`, with the `//` line.
- The text ends before `//` → `MissingEnd`, `line: None`.
- After `//`, a non-blank line that starts with `LOCUS` → `SecondRecord`;
  any other non-blank line → `TextAfterEnd`, with its line. Blank lines
  after `//` are fine.
- Then the ORIGIN sequence characters, each line's tokens joined with no
  separator and the lines joined with `"\n"`, go to `normalize(name, text,
  topology)` with the topology from LOCUS. Its errors
  (`EmptySequenceContent`, `InvalidSequenceCharacters`) are returned
  unchanged.
- Last, the normalized length must equal the LOCUS length. If not, the
  result is `LengthMismatch` with the LOCUS line.

Errors found while reading lines are reported as they are reached, in line
order. The end checks run in the order listed.

`MalformedGenBankProblem.sequence` is `name.value()`. Write a module doc
comment that summarizes §4.1 to §4.8 and cites D007, D008, D009 and D041.

## 5. Tests

### 5.1 Unit tests in `genbank.rs` (planner-written; add verbatim)

Every expected value below was written out by hand from the sample text.
The sample's ORIGIN is `seq_a` from the other modules' tests,
`ATGGCTTGGAATCAGCAGCAGCAGCAGCAG`, in three lines of ten. Line numbers are
the sample's index plus 1: `source` is at line 9, `CDS` at 12,
`misc_feature` at 18, ORIGIN's second sequence line at 24, `//` at 26.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCategory;
    use crate::error::GenBankRule;
    use crate::error::GenBankRule::*;
    use crate::sequence::normalize;

    fn s(text: &str) -> String {
        text.to_string()
    }

    fn key(name: &str, location: &str) -> String {
        format!("     {:<16}{}", name, location)
    }

    fn q(text: &str) -> String {
        format!("{:21}{}", "", text)
    }

    fn locus(rest: &str) -> String {
        format!("LOCUS       test1                     {}", rest)
    }

    fn sample() -> Vec<String> {
        vec![
            locus("30 bp    DNA     circular SYN 17-OCT-2026"),
            s("DEFINITION  A synthetic test record."),
            s("ACCESSION   TEST0001"),
            s("VERSION     TEST0001.1"),
            s("KEYWORDS    ."),
            s("SOURCE      synthetic"),
            s("  ORGANISM  synthetic"),
            s("FEATURES             Location/Qualifiers"),
            key("source", "1..30"),
            q("/organism=\"synthetic\""),
            q("/mol_type=\"other DNA\""),
            key("CDS", "1..12"),
            q("/locus_tag=\"tst_0001\""),
            q("/codon_start=1"),
            q("/note=\"a note that wraps onto"),
            q("a second line\""),
            q("/translation=\"MAWN\""),
            key("misc_feature", "join(25..30,"),
            q("1..3)"),
            q("/label=wrap"),
            q("/pseudo"),
            s("ORIGIN"),
            s("        1 atggcttgga"),
            s("       11 atcagcagca"),
            s("       21 gcagcagcag"),
            s("//"),
        ]
    }

    fn name() -> Identifier {
        Identifier::create("tmpl".to_string()).unwrap()
    }

    fn read(lines: &[String]) -> Result<GenBankRecord, DomainError> {
        read_genbank(name(), lines.join("\n") + "\n")
    }

    fn pairs(feature: &GenBankFeature) -> Vec<(String, Option<String>)> {
        feature
            .qualifiers()
            .iter()
            .map(|qualifier| (qualifier.name(), qualifier.value()))
            .collect()
    }

    fn some(text: &str) -> Option<String> {
        Some(text.to_string())
    }

    fn refusal(found: Result<GenBankRecord, DomainError>) -> (GenBankRule, Option<u64>) {
        match found {
            Err(error) => {
                assert!(error.category() == ErrorCategory::MalformedInput);
                assert_eq!(error.code(), "malformed_genbank");
                match error {
                    DomainError::MalformedGenBank(problem) => {
                        assert_eq!(problem.sequence, "tmpl");
                        (problem.rule, problem.line)
                    }
                    other => panic!("expected MalformedGenBank, got {:?}", other),
                }
            }
            Ok(record) => panic!("accepted: {:?}", record),
        }
    }

    #[test]
    fn the_sample_gives_its_metadata_and_sequence() {
        let record = read(&sample()).unwrap();
        assert_eq!(record.locus_name(), "test1");
        assert_eq!(record.accession(), some("TEST0001"));
        assert_eq!(record.version(), some("TEST0001.1"));
        let expected = normalize(
            name(),
            "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Circular,
        )
        .unwrap();
        assert_eq!(record.sequence(), expected);
    }

    #[test]
    fn the_sample_gives_its_features_in_order() {
        let features = read(&sample()).unwrap().features();
        let keys: Vec<String> = features.iter().map(|feature| feature.key()).collect();
        assert_eq!(keys, vec![s("source"), s("CDS"), s("misc_feature")]);
        let lines: Vec<u64> = features.iter().map(|feature| feature.line()).collect();
        assert_eq!(lines, vec![9, 12, 18]);
        assert_eq!(features[0].location(), "1..30");
        assert_eq!(features[1].location(), "1..12");
        assert_eq!(features[2].location(), "join(25..30,1..3)");
        assert_eq!(
            pairs(&features[0]),
            vec![(s("organism"), some("synthetic")), (s("mol_type"), some("other DNA"))]
        );
        assert_eq!(
            pairs(&features[1]),
            vec![
                (s("locus_tag"), some("tst_0001")),
                (s("codon_start"), some("1")),
                (s("note"), some("a note that wraps onto a second line")),
                (s("translation"), some("MAWN")),
            ]
        );
        assert_eq!(
            pairs(&features[2]),
            vec![(s("label"), some("wrap")), (s("pseudo"), None)]
        );
    }

    #[test]
    fn the_topology_comes_from_the_locus_line() {
        let mut lines = sample();
        lines[0] = locus("30 bp    DNA     linear   SYN 17-OCT-2026");
        assert_eq!(read(&lines).unwrap().sequence().topology(), SequenceTopology::Linear);
    }

    #[test]
    fn crlf_blank_lines_and_a_missing_final_newline_read_the_same() {
        let unix = read(&sample()).unwrap();
        let mut spaced: Vec<String> = vec![s(""), s("   ")];
        for line in sample() {
            spaced.push(line);
            spaced.push(s(""));
        }
        let windows = read_genbank(name(), spaced.join("\r\n")).unwrap();
        assert_eq!(unix.locus_name(), windows.locus_name());
        assert_eq!(unix.accession(), windows.accession());
        assert_eq!(unix.version(), windows.version());
        assert_eq!(unix.sequence(), windows.sequence());
        // Feature line numbers differ, because blank lines were added; every
        // other part of each feature is the same.
        let shape = |record: &GenBankRecord| -> Vec<(String, String, Vec<(String, Option<String>)>)> {
            record
                .features()
                .iter()
                .map(|feature| (feature.key(), feature.location(), pairs(feature)))
                .collect()
        };
        assert_eq!(shape(&unix), shape(&windows));
        assert_eq!(windows.features()[1].line(), 25);
    }

    #[test]
    fn quoted_values_unescape_and_may_continue_with_a_slash() {
        let mut lines = sample();
        lines[14] = q("/note=\"say \"\"hi\"\"\"");
        lines[15] = q("/note=\"see");
        lines.insert(16, q("/path/to/thing\""));
        let features = read(&lines).unwrap().features();
        let notes: Vec<(String, Option<String>)> = pairs(&features[1])
            .into_iter()
            .filter(|(name, _)| name == "note")
            .collect();
        assert_eq!(
            notes,
            vec![(s("note"), some("say \"hi\"")), (s("note"), some("see /path/to/thing"))]
        );
    }

    #[test]
    fn a_wrapped_translation_is_joined_with_a_space() {
        let mut lines = sample();
        lines[16] = q("/translation=\"MAWN");
        lines.insert(17, q("QQ\""));
        let features = read(&lines).unwrap().features();
        assert_eq!(pairs(&features[1])[3], (s("translation"), some("MAWN QQ")));
    }

    #[test]
    fn an_unquoted_value_continues_with_a_space_and_an_empty_value_is_kept() {
        let mut lines = sample();
        lines[19] = q("/label=wrap");
        lines.insert(20, q("around"));
        lines.insert(21, q("/note="));
        let features = read(&lines).unwrap().features();
        assert_eq!(
            pairs(&features[2]),
            vec![(s("label"), some("wrap around")), (s("note"), some("")), (s("pseudo"), None)]
        );
    }

    #[test]
    fn a_record_without_features_or_accession_reads() {
        let mut lines = sample();
        lines.drain(7..21);
        lines.drain(2..4);
        let record = read(&lines).unwrap();
        assert!(record.features().is_empty());
        assert_eq!(record.accession(), None);
        assert_eq!(record.version(), None);
        assert_eq!(record.sequence().length(), 30);
    }

    #[test]
    fn an_accession_line_without_a_value_is_none_and_a_dot_is_kept() {
        let mut lines = sample();
        lines[2] = s("ACCESSION");
        lines[3] = s("VERSION     .");
        let record = read(&lines).unwrap();
        assert_eq!(record.accession(), None);
        assert_eq!(record.version(), some("."));
    }

    #[test]
    fn a_text_that_does_not_start_with_locus_is_refused() {
        assert_eq!(refusal(read_genbank(name(), s(""))), (NoLocus, None));
        assert_eq!(refusal(read_genbank(name(), s(" \n\t\n"))), (NoLocus, None));
        let mut lines = sample();
        lines.insert(0, s("DEFINITION  first"));
        assert_eq!(refusal(read(&lines)), (NoLocus, Some(1)));
        let mut lower = sample();
        lower[0] = lower[0].replace("LOCUS", "locus");
        assert_eq!(refusal(read(&lower)), (NoLocus, Some(1)));
        let mut joined = sample();
        joined[0] = s("LOCUSX      test1   30 bp    DNA     circular");
        assert_eq!(refusal(read(&joined)), (NoLocus, Some(1)));
    }

    #[test]
    fn a_locus_line_without_a_length_in_bp_is_refused() {
        for rest in [
            "DNA     circular SYN 17-OCT-2026",
            "30 aa    circular",
            "x30 bp    DNA     circular",
            "99999999999999999999 bp    DNA     circular",
        ] {
            let mut lines = sample();
            lines[0] = locus(rest);
            assert_eq!(refusal(read(&lines)), (LocusLength, Some(1)), "{}", rest);
        }
    }

    #[test]
    fn a_locus_line_without_one_topology_is_refused() {
        let mut lines = sample();
        lines[0] = locus("30 bp    DNA     SYN 17-OCT-2026");
        assert_eq!(refusal(read(&lines)), (TopologyMissing, Some(1)));
        lines[0] = locus("30 bp    DNA     Circular SYN 17-OCT-2026");
        assert_eq!(refusal(read(&lines)), (TopologyMissing, Some(1)));
        lines[0] = locus("30 bp    DNA     linear circular SYN 17-OCT-2026");
        assert_eq!(refusal(read(&lines)), (TopologyConflict, Some(1)));
        lines[0] = locus("30 bp    DNA     circular circular SYN 17-OCT-2026");
        assert_eq!(refusal(read(&lines)), (TopologyConflict, Some(1)));
    }

    #[test]
    fn a_section_that_appears_once_may_not_repeat() {
        let mut version = sample();
        version.insert(4, s("VERSION     TEST0001.2"));
        assert_eq!(refusal(read(&version)), (RepeatedSection, Some(5)));
        let mut locus_again = sample();
        locus_again.insert(21, locus("30 bp    DNA     circular"));
        assert_eq!(refusal(read(&locus_again)), (RepeatedSection, Some(22)));
        let mut features = sample();
        features.insert(21, s("FEATURES             Location/Qualifiers"));
        assert_eq!(refusal(read(&features)), (RepeatedSection, Some(22)));
    }

    #[test]
    fn a_line_that_fits_no_part_of_the_layout_is_refused() {
        let cases: Vec<(usize, String)> = vec![
            (11, s("    gene            1..3")),
            (11, s("          gene      1..3")),
            (11, s("     gene")),
            (8, q("/note=\"x\"")),
            (11, s("\tgene\t1..3")),
            (2, s("definition  lower case")),
            (2, s("1 stray")),
        ];
        for (index, line) in cases {
            let mut lines = sample();
            lines.insert(index, line.clone());
            assert_eq!(
                refusal(read(&lines)),
                (LineLayout, Some(index as u64 + 1)),
                "{:?}",
                line
            );
        }
    }

    #[test]
    fn a_malformed_qualifier_is_refused() {
        let mut unterminated = sample();
        unterminated[14] = q("/note=\"never closed");
        unterminated.remove(15);
        assert_eq!(refusal(read(&unterminated)), (QualifierSyntax, Some(15)));

        let mut trailing = sample();
        trailing[16] = q("/translation=\"MAWN\"x");
        assert_eq!(refusal(read(&trailing)), (QualifierSyntax, Some(17)));

        let cases: [(usize, &str); 4] = [
            (17, "extra"),
            (17, "/=x"),
            (17, "/bad name=x"),
            (21, "after a flag"),
        ];
        for (index, text) in cases {
            let mut lines = sample();
            lines.insert(index, q(text));
            assert_eq!(
                refusal(read(&lines)),
                (QualifierSyntax, Some(index as u64 + 1)),
                "{:?}",
                text
            );
        }

        let mut quote_in_bare = sample();
        quote_in_bare[19] = q("/label=wr\"ap");
        assert_eq!(refusal(read(&quote_in_bare)), (QualifierSyntax, Some(20)));
    }

    #[test]
    fn a_record_without_origin_is_refused() {
        let mut lines = sample();
        lines.drain(21..25);
        assert_eq!(refusal(read(&lines)), (NoOrigin, Some(22)));
    }

    #[test]
    fn an_origin_line_without_its_position_is_refused() {
        for line in ["          atcagcagca", "       1x atcagcagca", "BASE COUNT 10 a"] {
            let mut lines = sample();
            lines[23] = s(line);
            assert_eq!(refusal(read(&lines)), (OriginLine, Some(24)), "{:?}", line);
        }
    }

    #[test]
    fn an_origin_line_numbered_wrongly_is_refused() {
        let mut lines = sample();
        lines[23] = s("       12 atcagcagca");
        assert_eq!(refusal(read(&lines)), (OriginNumbering, Some(24)));
        let mut merged = sample();
        merged[22] = s("        1 atggcttgga atcagcagca");
        assert_eq!(refusal(read(&merged)), (OriginNumbering, Some(24)));
    }

    #[test]
    fn a_locus_length_other_than_the_sequence_is_refused() {
        let mut lines = sample();
        lines[0] = locus("31 bp    DNA     circular SYN 17-OCT-2026");
        assert_eq!(refusal(read(&lines)), (LengthMismatch, Some(1)));
    }

    #[test]
    fn a_record_must_end_with_its_terminator_and_nothing_after() {
        let mut missing = sample();
        missing.pop();
        assert_eq!(refusal(read(&missing)), (MissingEnd, None));

        let mut second = sample();
        second.push(locus("30 bp    DNA     circular"));
        assert_eq!(refusal(read(&second)), (SecondRecord, Some(27)));

        let mut junk = sample();
        junk.push(s(""));
        junk.push(s("junk"));
        assert_eq!(refusal(read(&junk)), (TextAfterEnd, Some(28)));

        let mut blank = sample();
        blank.push(s(""));
        blank.push(s("   "));
        assert!(read(&blank).is_ok());
    }

    #[test]
    fn the_origin_letters_go_through_normalize() {
        let mut lines = sample();
        lines[24] = s("       21 gcagcagcax");
        match read(&lines).unwrap_err() {
            DomainError::InvalidSequenceCharacters(problem) => {
                assert_eq!(problem.position, 29);
                assert_eq!(problem.characters, "X");
            }
            other => panic!("expected InvalidSequenceCharacters, got {:?}", other),
        }
        let mut empty = sample();
        empty.drain(22..25);
        empty[0] = locus("0 bp    DNA     circular");
        assert!(matches!(
            read(&empty).unwrap_err(),
            DomainError::EmptySequenceContent(_)
        ));
    }

    // Added in review: ORIGIN runs to `//`, so a section line inside it is
    // an ORIGIN line that is not one, whether or not its keyword was seen.
    // `drain(2..4)` removes ACCESSION and VERSION, so ORIGIN is index 19
    // and index 21 sits between its first and second sequence lines.
    #[test]
    fn a_section_line_inside_origin_is_an_origin_line() {
        for line in [
            "ACCESSION   TEST0002",
            "VERSION     TEST0002.1",
            "FEATURES             Location/Qualifiers",
            "ORIGIN",
            "LOCUS       x 30 bp DNA circular",
        ] {
            let mut lines = sample();
            lines.drain(2..4);
            lines.insert(21, s(line));
            assert_eq!(refusal(read(&lines)), (OriginLine, Some(22)), "{:?}", line);
        }
    }

    // Added in review: `/label=wrap` is at index 19, so its continuation is
    // line 21.
    #[test]
    fn an_unquoted_continuation_may_not_hold_a_quote() {
        let mut lines = sample();
        lines.insert(20, q("ar\"ound"));
        assert_eq!(refusal(read(&lines)), (QualifierSyntax, Some(21)));
    }

    #[test]
    fn each_message_names_the_line_when_there_is_one() {
        assert_eq!(
            read_genbank(name(), s("")).unwrap_err().message(),
            "GenBank text for 'tmpl': it does not start with a LOCUS line"
        );
        let mut lines = sample();
        lines[0] = locus("31 bp    DNA     circular");
        assert_eq!(
            read(&lines).unwrap_err().message(),
            "GenBank text for 'tmpl': the LOCUS line's length differs from the ORIGIN sequence's (line 1)"
        );
    }
}
```

Notes on the cases, so you can check them against §4 before running:
- In `a_line_that_fits_no_part_of_the_layout_is_refused`, index 11 is inside
  FEATURES, between `mol_type` and the `CDS` key line. Index 8 is right after
  the `FEATURES` line, before any key line. Index 2 is in the header, after
  DEFINITION.
- In `a_malformed_qualifier_is_refused`, index 17 is directly after the
  closed `/translation="MAWN"`, and index 21 is directly after `/pseudo`.
  `/=x` is an empty name, and `/bad name=x` has a space in the name.
  `unterminated` leaves the `/note="never closed` value open (an odd quote
  count) through `/translation="MAWN"` (2 more quotes, still odd) until the
  `misc_feature` key line, so the error is on the `/note=` line, 15.
- In `an_origin_line_numbered_wrongly_is_refused`, `merged` puts 20 bases on
  line 23, so line 24 should say 21, not 11.
- In `the_origin_letters_go_through_normalize`, the `x` is the 30th letter,
  canonical position 29, and `normalize` reports it upper-cased. `empty`
  removes all three sequence lines and declares 0 bp. `normalize` refuses
  the empty sequence before the length comparison runs.

`cargo fmt` may reflow these; do not change content. If a test fails, stop
and report the test and what the reader returned. Do not change the test.

### 5.2 `error.rs` tests

- In `every_variant_has_its_pinned_code`, add after the
  `UnusableGenBankLocation` case
  `DomainError::MalformedGenBank(MalformedGenBankProblem { sequence: s(),
  rule: GenBankRule::NoLocus, line: None })` with code `"malformed_genbank"`,
  and change the count from 49 to 50.
- Add after `every_genbank_location_rule_has_its_pinned_code_and_category`:

```rust
    #[test]
    fn every_genbank_rule_has_its_pinned_code() {
        use GenBankRule::*;
        let cases: [(GenBankRule, &str); 14] = [
            (NoLocus, "no_locus"),
            (LocusLength, "locus_length"),
            (TopologyMissing, "topology_missing"),
            (TopologyConflict, "topology_conflict"),
            (RepeatedSection, "repeated_section"),
            (LineLayout, "line_layout"),
            (QualifierSyntax, "qualifier_syntax"),
            (NoOrigin, "no_origin"),
            (OriginLine, "origin_line"),
            (OriginNumbering, "origin_numbering"),
            (LengthMismatch, "length_mismatch"),
            (MissingEnd, "missing_end"),
            (SecondRecord, "second_record"),
            (TextAfterEnd, "text_after_end"),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (rule, code) in cases {
            assert_eq!(rule.code(), code, "{:?}", rule);
            assert!(seen.insert(code), "duplicate code {}", code);
        }
    }
```

### 5.3 The three `variant_name` matches

`fn variant_name` in `crates/core/tests/core_foundation.rs`,
`mutation_fixtures.rs` and `mutation_invariants.rs` each gain one arm,
`DomainError::MalformedGenBank(_) => "MalformedGenBank",`, after the
`UnusableGenBankLocation` arm. Nothing else in those files.

### 5.4 Proving each rule is load-bearing (AGENTS.md)

For each rule, disable its check (it never refuses for that reason), run
`cargo test -p luria-core genbank::`, confirm the named test fails, restore.
A different error, or a panic, in place of the expected one counts as the
named test failing. Report the failing tests for each.

| Rule | Test that must fail |
| --- | --- |
| `NoLocus` | `a_text_that_does_not_start_with_locus_is_refused` |
| `LocusLength` | `a_locus_line_without_a_length_in_bp_is_refused` |
| `TopologyMissing` | `a_locus_line_without_one_topology_is_refused` |
| `TopologyConflict` | `a_locus_line_without_one_topology_is_refused` |
| `RepeatedSection` | `a_section_that_appears_once_may_not_repeat` |
| `LineLayout` | `a_line_that_fits_no_part_of_the_layout_is_refused` |
| `QualifierSyntax` | `a_malformed_qualifier_is_refused` |
| `NoOrigin` | `a_record_without_origin_is_refused` |
| `OriginLine` | `an_origin_line_without_its_position_is_refused` |
| `OriginNumbering` | `an_origin_line_numbered_wrongly_is_refused` |
| `LengthMismatch` | `a_locus_length_other_than_the_sequence_is_refused` |
| `MissingEnd` | `a_record_must_end_with_its_terminator_and_nothing_after` |
| `SecondRecord` | `a_record_must_end_with_its_terminator_and_nothing_after` |
| `TextAfterEnd` | `a_record_must_end_with_its_terminator_and_nothing_after` |

## 6. Do not

- Edit files other than `crates/core/src/error.rs`,
  `crates/core/src/genbank.rs` (new), `crates/core/src/lib.rs`, and the three
  `variant_name` arms in §5.3.
- Parse locations, call `parse_genbank_location`, or interpret any qualifier.
  That is 5c3.
- Change any existing behavior, message or test beyond §5.2 and §5.3.
- Repair input: no lower-casing of keywords, no tab expansion, no skipping
  of a misnumbered ORIGIN line, no reading past `//`.
- Add dependencies or make the reader read files: it takes text.

## 7. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 8. Handoff

Report the changed and new files, the exact commands and results, the §5.4
disable-and-restore results per rule, and anything that did not match this
contract.
