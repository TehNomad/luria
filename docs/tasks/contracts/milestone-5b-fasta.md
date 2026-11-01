# Milestone 5b: the FASTA reader

Status: **Implemented 2026-10-17** by the local worker; accepted after one scope amendment (§5, three `variant_name` arms).
Planner: Claude. Parent: [docs/tasks/contracts/milestone-5-cli.md](/tasks/contracts/milestone-5-cli.md) §3.3 (FASTA) and
§5 (5b). Follows 5a1–5a3 (error codes and rules, committed).

## 1. Requirements this contract applies

From [docs/decisions/](/decisions/):
- **D007:** a sequence artifact records its identifier, normalized text,
  topology, length, *source metadata when available*, and a SHA-256 content
  hash. Two artifacts with different content are never interchangeable.
- **D008:** normalization happens at a documented boundary and never repairs
  content, deletes unexpected symbols, or infers topology. The normalized
  artifact and its source representation stay distinguishable.
- **D009:** topology is explicit, linear or circular.
- **D041:** FASTA is read in this milestone, **one record per file**. The
  reader lives in `luria-core`, which stays dependency-free. A FASTA
  template's topology is declared by the caller (the request), never
  inferred.

From the plan, §3.3: zero records, more than one record, or text before the
header are refused as `MalformedInput`. The body goes through
`sequence::normalize` **unchanged**, so D008's rules stay in one place. The
header is kept as source metadata and is **not** the sequence's identity;
the identifier comes from the caller.

## 2. New error variant and rule (in `crates/core/src/error.rs`)

Follow the pattern of `MutationRequestRule` and `PrimerCandidateRule`.

```rust
/// Which rule refused a FASTA text. Stable: `code()` is part of the CLI's
/// output contract (D041), and renaming one needs approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FastaRule {
    NoRecord,
    TextBeforeHeader,
    SecondRecord,
}
```

`impl FastaRule { pub fn code(self) -> &'static str }`, exhaustive, literal
strings, no `_` arm: `no_record`, `text_before_header`, `second_record`.

```rust
/// Payload: a FASTA text that is not exactly one record. `line` counts text
/// lines from 1, as an editor does; it is not a sequence coordinate. It is
/// `None` for `NoRecord`, which has no line to point at.
#[derive(Clone, Debug, PartialEq)]
pub struct MalformedFastaProblem {
    pub sequence: String,
    pub rule: FastaRule,
    pub line: Option<u64>,
}
```

Add `DomainError::MalformedFasta(MalformedFastaProblem)` as the **last**
variant of the enum, after `DegenerateExpansionTooLarge`. Then:
- `category()`: `MalformedInput`;
- `code()`: `"malformed_fasta"`;
- `message()`, by rule (`{s}` is `problem.sequence`, `{n}` the line):
  - `NoRecord`: `FASTA text for '{s}' holds no record: it has no '>' header line`
  - `TextBeforeHeader`: `FASTA text for '{s}' has text on line {n} before its '>' header line`
  - `SecondRecord`: `FASTA text for '{s}' starts a second record on line {n}; only one record is read`

  If `line` is `None` for a rule that needs it, print `?` for `{n}`; do not
  panic.

## 3. The reader: `crates/core/src/fasta.rs`

Add `pub mod fasta;` to `crates/core/src/lib.rs` in alphabetical order.

```rust
/// One FASTA record: its header line as source metadata, and its sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct FastaRecord {
    header: String,
    sequence: Sequence,
}

impl FastaRecord {
    /// The header line without its `>`, blanks at either end removed. Source
    /// metadata only (D007); the sequence's identifier is the caller's.
    pub fn header(&self) -> String;
    pub fn sequence(&self) -> Sequence;
}

/// Read a text that holds exactly one FASTA record.
pub fn read_fasta(
    name: Identifier,
    text: String,
    topology: SequenceTopology,
) -> Result<FastaRecord, DomainError>;
```

Rules, in this order:
1. Split `text` with `str::lines()` (which removes `\n` and `\r\n`). Line
   numbers count from 1.
2. A line is a **header line** if, after removing leading spaces and tabs,
   it starts with `>`.
3. Every line before the first header line must be **blank**: empty, or
   only spaces, tabs and `\r`. The first non-blank one is refused with
   `TextBeforeHeader` and its line number. This includes a `;` comment line
   and a byte-order mark: neither is repaired or skipped (D008).
4. If there is no header line, and every line is blank (including an empty
   text), refuse with `NoRecord`, `line: None`.
5. Any header line after the first is refused with `SecondRecord` and its
   line number, even if the second record has no sequence lines.
6. The **header** is the first header line with leading spaces and tabs
   removed, the `>` removed, and then spaces, tabs and `\r` removed from both
   ends. Internal whitespace is kept exactly. An empty header is allowed.
7. The **body** is every line after the header line, joined with `"\n"`, and
   passed to `normalize(name, body, topology)` unchanged. Its errors
   (`EmptySequenceContent`, `InvalidSequenceCharacters`) are returned as they
   are. Do not restate any of `normalize`'s rules: its `;` comment-line skip,
   blank handling and case folding apply inside the body because `normalize`
   applies them.

`MalformedFastaProblem.sequence` is `name.value()`.

Write a module doc comment stating rules 1–7 briefly, and that topology and
identity come from the caller (D041, D007).

## 4. Tests

### 4.1 Unit tests in `fasta.rs` (planner-written; add verbatim)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCategory;
    use crate::error::FastaRule;
    use crate::error::MalformedFastaProblem;
    use crate::sequence::normalize;

    fn name() -> Identifier {
        Identifier::create("tmpl".to_string()).unwrap()
    }

    fn read(text: &str) -> Result<FastaRecord, DomainError> {
        read_fasta(name(), text.to_string(), SequenceTopology::Linear)
    }

    fn fasta_problem(error: DomainError) -> MalformedFastaProblem {
        assert!(error.category() == ErrorCategory::MalformedInput);
        assert_eq!(error.code(), "malformed_fasta");
        match error {
            DomainError::MalformedFasta(problem) => problem,
            other => panic!("expected MalformedFasta, got {:?}", other),
        }
    }

    #[test]
    fn one_record_gives_its_header_and_the_normalized_sequence() {
        let record = read(">seq one  here\nacgt\nNNAC\n").unwrap();
        assert_eq!(record.header(), "seq one  here");
        let expected =
            normalize(name(), "ACGTNNAC".to_string(), SequenceTopology::Linear).unwrap();
        assert_eq!(record.sequence(), expected);
        assert_eq!(record.sequence().length(), 8);
    }

    #[test]
    fn the_topology_is_the_callers() {
        let record =
            read_fasta(name(), ">h\nACGT\n".to_string(), SequenceTopology::Circular).unwrap();
        assert_eq!(record.sequence().topology(), SequenceTopology::Circular);
    }

    #[test]
    fn crlf_and_a_missing_final_newline_read_the_same() {
        let unix = read(">h\nACG\nT\n").unwrap();
        let windows = read(">h\r\nACG\r\nT").unwrap();
        assert_eq!(unix, windows);
        assert_eq!(windows.header(), "h");
    }

    #[test]
    fn header_blanks_are_trimmed_and_an_empty_header_is_allowed() {
        assert_eq!(read(">  h x \t\nA").unwrap().header(), "h x");
        assert_eq!(read(">\nACGT").unwrap().header(), "");
        assert_eq!(read("  >h\nACGT").unwrap().header(), "h");
    }

    #[test]
    fn blank_lines_before_the_header_and_in_the_body_are_accepted() {
        let record = read("\n  \n>h\nAC\n\nGT\n").unwrap();
        assert_eq!(record.sequence().canonical_text(), "ACGT");
    }

    // The `;` skip is `normalize`'s rule, applied to the body unchanged.
    #[test]
    fn a_comment_line_in_the_body_is_skipped_by_normalize() {
        let record = read(">h\nAC\n;note\nGT").unwrap();
        assert_eq!(record.sequence().canonical_text(), "ACGT");
    }

    #[test]
    fn no_header_line_is_no_record() {
        for text in ["", "\n", " \n\t\n"] {
            let problem = fasta_problem(read(text).unwrap_err());
            assert_eq!(problem.rule, FastaRule::NoRecord, "{:?}", text);
            assert_eq!(problem.line, None);
            assert_eq!(problem.sequence, "tmpl");
        }
    }

    #[test]
    fn text_before_the_header_is_refused() {
        let cases: [(&str, u64); 4] = [
            ("ACGT\n>h\nACGT", 1),
            (";comment\n>h\nAC", 1),
            ("\n\nx\n>h\nA", 3),
            ("\u{feff}>h\nACGT", 1),
        ];
        for (text, line) in cases {
            let problem = fasta_problem(read(text).unwrap_err());
            assert_eq!(problem.rule, FastaRule::TextBeforeHeader, "{:?}", text);
            assert_eq!(problem.line, Some(line), "{:?}", text);
        }
    }

    // Sequence text with no header at all is text before a header that never
    // comes, not "no record": the caller supplied something, and line 1 is it.
    #[test]
    fn bare_sequence_text_is_text_before_the_header() {
        let problem = fasta_problem(read("ACGT\n").unwrap_err());
        assert_eq!(problem.rule, FastaRule::TextBeforeHeader);
        assert_eq!(problem.line, Some(1));
    }

    #[test]
    fn a_second_header_is_a_second_record() {
        for text in [">a\nAC\n>b\nGT", ">a\nAC\n>b\n", ">a\nAC\n  >b\nGT"] {
            let problem = fasta_problem(read(text).unwrap_err());
            assert_eq!(problem.rule, FastaRule::SecondRecord, "{:?}", text);
            assert_eq!(problem.line, Some(3), "{:?}", text);
        }
    }

    #[test]
    fn a_header_with_no_sequence_is_an_empty_sequence() {
        let error = read(">h\n").unwrap_err();
        assert!(matches!(error, DomainError::EmptySequenceContent(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    // `>` inside a sequence line does not start a record; it is a character
    // outside the alphabet, at canonical position 2.
    #[test]
    fn letters_outside_the_alphabet_are_refused_by_normalize() {
        for (text, position, characters) in [(">h\nACGU", 3, "U"), (">h\nAC>GT", 2, ">")] {
            match read(text).unwrap_err() {
                DomainError::InvalidSequenceCharacters(problem) => {
                    assert_eq!(problem.position, position, "{:?}", text);
                    assert_eq!(problem.characters, characters, "{:?}", text);
                }
                other => panic!("expected InvalidSequenceCharacters, got {:?}", other),
            }
        }
    }

    #[test]
    fn each_message_names_the_line() {
        assert_eq!(
            read("x\n>h\nA").unwrap_err().message(),
            "FASTA text for 'tmpl' has text on line 1 before its '>' header line"
        );
        assert_eq!(
            read(">a\nA\n>b\nC").unwrap_err().message(),
            "FASTA text for 'tmpl' starts a second record on line 3; only one record is read"
        );
        assert_eq!(
            read("").unwrap_err().message(),
            "FASTA text for 'tmpl' holds no record: it has no '>' header line"
        );
    }
}
```

`cargo fmt` may reflow these; do not change their content. If one fails, or
does not compile because an accessor named here has another signature, stop
and report: do not change the test.

### 4.2 `error.rs` tests

- In `every_variant_has_its_pinned_code`, add one case after
  `DegenerateExpansionTooLarge`:
  `DomainError::MalformedFasta(MalformedFastaProblem { sequence: s(), rule:
  FastaRule::NoRecord, line: None })` with code `"malformed_fasta"`, and
  change `assert_eq!(cases.len(), 47)` to `48`.
- Add, after `every_primer_candidate_rule_has_its_pinned_code`:

```rust
    #[test]
    fn every_fasta_rule_has_its_pinned_code() {
        use FastaRule::*;
        let cases: [(FastaRule, &str); 3] = [
            (NoRecord, "no_record"),
            (TextBeforeHeader, "text_before_header"),
            (SecondRecord, "second_record"),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (rule, code) in cases {
            assert_eq!(rule.code(), code, "{:?}", rule);
            assert!(seen.insert(code), "duplicate code {}", code);
        }
    }
```

### 4.3 Fixture: independent digests of the two SID-1 FASTA files

Create these files with exactly this content.

**`tests/fixtures/fasta/reference/digests.py`**: copy verbatim from
§8 of this contract.

**`tests/fixtures/fasta/expected.tsv`**: generate it by running
`python3 -I tests/fixtures/fasta/reference/digests.py` from the repository
root, then run it again with `--check`, which must print `OK expected.tsv`.
The script uses the Python standard library only and shares no code with
`crates/core`. The result must be byte-identical to this (tabs between
columns, one trailing newline):

```
file	header	length	sha256
example-construct/input/sid-1_cDNA.fa	3xFLAG-mVenus-sid-1 cDNA	3153	54704740a9f04b79d17147ef95788ad9f0f11e5263843a7ed490924d43b55e3a
example-construct/raw/sdmfinalseq.fa	SID-1_D404N  Replace gat between bases 1209-1213 with AAC in SID-1 (D404N) replaced 1210 to 1212 with AAC	3153	6ab7d5a6fdd995692ed5cda5079db64c97a70935deb9edfd10fe402628b7ff50
```

(The second header has two spaces after `SID-1_D404N`. If the generated file
differs from this block in any way, stop and report; do not edit either.)

**`tests/fixtures/fasta/README.md`**:

```markdown
# FASTA reader fixture

`expected.tsv` records, for each single-record FASTA input, the header (the
first line without `>`, blanks at both ends removed), the sequence length,
and the SHA-256 of the canonical text: the letters, upper-cased, with line
breaks and blanks removed (D007). `crates/core/tests/fasta_fixtures.rs`
reads each file with `luria_core::fasta::read_fasta` and compares all three.

The inputs are the SID-1 files in `../example-construct/`: the wild-type cDNA
(`input/sid-1_cDNA.fa`, upper case with one lower-case run) and the
NEBaseChanger output (`raw/sdmfinalseq.fa`, all lower case, no final newline,
two spaces inside its header). Both lengths, 3153, are the ones that
fixture's README already records.

## Verification

`reference/digests.py` computes all three columns with the Python standard
library only, sharing no code with `crates/core`. The digests were also
checked by hand on 2026-10-17 with

    grep -v '^>' FILE | tr -d '\n\r \t' | tr a-z A-Z | sha256sum

which printed the same two values. Rerun the script's check from the
repository root:

    python -I tests/fixtures/fasta/reference/digests.py --check
```

**`tests/fixtures/README.md`**: add this row to the fixture index table,
after the `example-construct/` row:

```markdown
| [`fasta/`](./fasta/) | Header, length and SHA-256 of the two SID-1 FASTA files, computed independently. Drives `crates/core/tests/fasta_fixtures.rs`. |
```

and in the paragraph under "Rerunning fixture verification" that lists what
`verify.py` runs, change "the five q5-primer generators, and the codon-usage
parser." to "the five q5-primer generators, the codon-usage parser, and the
FASTA digest check."

**`tests/fixtures/verify.py`**: append `"fasta/reference/digests.py",` as the
last entry of `CHECKS`. Change nothing else in it.

### 4.4 Integration test: `crates/core/tests/fasta_fixtures.rs`

Write it yourself, in the style of `crates/core/tests/codon_usage_fixture.rs`
(module doc comment saying what is checked against what, and that the
Python check is not part of the cargo checks). It must:
- read `../../tests/fixtures/fasta/expected.tsv`, check its header row is
  exactly `file\theader\tlength\tsha256`, and that it has exactly 2 data rows;
- for each row, read `../../tests/fixtures/<file>` with `std::fs`, call
  `read_fasta` with identifier `fixture` and `SequenceTopology::Linear`, and
  assert `header()`, `sequence().length()` and
  `sequence().content_hash().hex_digest()` equal the row, with a message
  naming the file.

### 4.5 Proving each rule is load-bearing (AGENTS.md)

For each of the three rules, temporarily disable the check in `read_fasta`,
run `cargo test -p luria-core fasta`, confirm the named test fails, and
restore the check. Report each result (the failing test names) in the
handoff.

| Rule | Test that must fail when the check is disabled |
| --- | --- |
| `NoRecord` | `no_header_line_is_no_record` |
| `TextBeforeHeader` | `text_before_the_header_is_refused` |
| `SecondRecord` | `a_second_header_is_a_second_record` |

"Disabled" means the condition never refuses (for example `if false &&
…`). If disabling one rule makes the reader panic, that counts as a failing
test; report it.

## 5. Do not

- Edit files other than: `crates/core/src/error.rs`,
  `crates/core/src/fasta.rs` (new), `crates/core/src/lib.rs`,
  `crates/core/tests/fasta_fixtures.rs` (new), and the fixture files named in
  §4.3.
- Change `sequence::normalize` or any other existing behavior, message or
  test, apart from the two `error.rs` test changes in §4.2 and the one
  exception below.
- *Amended 2026-10-17 after the worker reported it:* the exhaustive
  `fn variant_name` matches in `crates/core/tests/core_foundation.rs`,
  `mutation_fixtures.rs` and `mutation_invariants.rs` each gain exactly one
  arm, `DomainError::MalformedFasta(_) => "MalformedFasta",`, after the
  `DegenerateExpansionTooLarge` arm. No wildcard arm, and nothing else in
  those files changes.
- Change `q5_primer_fixtures.rs`, which strips headers by hand. Switching it
  to `read_fasta` is not in this contract.
- Add dependencies, or make the reader read files: it takes text.
- Repair input: no BOM skipping, no `;` skipping before the header, no
  joining of records.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`
- `python3 -I tests/fixtures/fasta/reference/digests.py --check`, which must
  print `OK expected.tsv`.

(The full `verify.py` needs the pinned Biopython venv, which is not set up
here; don't try to install it.)

## 7. Handoff

Report the changed and new files, the exact commands and results, the §4.5
disable-and-restore results, and anything that did not match this contract.

## 8. `tests/fixtures/fasta/reference/digests.py`

```python
"""Write ../expected.tsv: the header, length and SHA-256 of each FASTA input (D039).

luria's sequence digest is SHA-256 over the canonical text: the sequence
letters with ASCII lower case folded to upper case and every line break and
blank removed (`crates/core/src/sequence.rs`, D007). This script computes the
same quantity with the Python standard library only, sharing no code with
`crates/core`, so `crates/core/tests/fasta_fixtures.rs` checks the Rust FASTA
reader against an independent reading of the same files.

Each input is a single-record FASTA file. The header is the first line with
its leading `>` removed and surrounding blanks stripped; every later line is
sequence text. The script asserts that shape rather than handling anything
else: it is a check on two known files, not a FASTA parser.

Run from the repository root with `python -I`:

    python -I tests/fixtures/fasta/reference/digests.py
    python -I tests/fixtures/fasta/reference/digests.py --check

Without --check, writes ../expected.tsv. With --check, regenerates the same
text in memory, compares it byte for byte with that tracked file, prints `OK
expected.tsv` or `DIFFERS expected.tsv` followed by the first differing line,
exits 1 on a difference, and writes nothing (D039).
"""
import hashlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.dirname(HERE)
FIXTURES = os.path.dirname(FIXTURE)
OUT = os.path.join(FIXTURE, "expected.tsv")

# Paths relative to tests/fixtures/, as the Rust test names them.
INPUTS = [
    "example-construct/input/sid-1_cDNA.fa",
    "example-construct/raw/sdmfinalseq.fa",
]


def describe(relative):
    with open(os.path.join(FIXTURES, relative), "rb") as f:
        text = f.read().decode("ascii")
    lines = text.splitlines()
    assert lines and lines[0].startswith(">"), relative + ": no header line"
    assert not any(line.startswith(">") for line in lines[1:]), relative + ": second record"
    header = lines[0][1:].strip(" \t")
    body = "".join("".join(line.split()) for line in lines[1:])
    assert body.isalpha() and body.isascii(), relative + ": non-letter in sequence text"
    canonical = body.upper()
    digest = hashlib.sha256(canonical.encode("ascii")).hexdigest()
    return [relative, header, str(len(canonical)), digest]


def render():
    rows = [["file", "header", "length", "sha256"]]
    rows += [describe(relative) for relative in INPUTS]
    return "".join("\t".join(row) + "\n" for row in rows)


def main():
    text = render()
    if "--check" in sys.argv[1:]:
        with open(OUT, "r", encoding="ascii", newline="") as f:
            tracked = f.read()
        if tracked == text:
            print("OK expected.tsv")
            return 0
        for number, (want, have) in enumerate(
            zip(text.splitlines(), tracked.splitlines()), start=1
        ):
            if want != have:
                print("DIFFERS expected.tsv line %d: %r != %r" % (number, have, want))
                return 1
        print("DIFFERS expected.tsv: line counts differ")
        return 1
    with open(OUT, "w", encoding="ascii", newline="") as f:
        f.write(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
```
