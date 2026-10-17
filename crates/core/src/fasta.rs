//! Reading one FASTA record (D041).
//!
//! The reader takes text, never a file: opening a file belongs to the caller.
//! Its rules, in order:
//!
//! 1. The text is split with `str::lines()`, which drops the `\n` and the
//!    `\r` of a `\r\n`. Line numbers count from 1, as an editor numbers them;
//!    they are not sequence coordinates.
//! 2. A line is a header line when it starts with `>` once leading spaces and
//!    tabs are removed.
//! 3. Every line before the first header line must be blank: empty, or made
//!    only of spaces, tabs and `\r`. The first line that breaks this is
//!    refused with `TextBeforeHeader` and its line number. A `;` comment line
//!    and a byte-order mark are neither skipped nor repaired (D008).
//! 4. No header line, and every line blank (an empty text included), is
//!    refused with `NoRecord`.
//! 5. A header line after the first is refused with `SecondRecord`: one
//!    record per file is what this milestone reads.
//! 6. The header is the first header line with leading spaces and tabs
//!    removed, the `>` removed, and spaces, tabs and `\r` trimmed from both
//!    ends. Internal whitespace is kept, and an empty header is allowed.
//! 7. The body is every line after the header line, joined with `\n`, and
//!    handed to `sequence::normalize` unchanged: its `;` comment-line skip,
//!    blank handling, case folding and alphabet check apply there, so
//!    normalization stays in one place (D008).
//!
//! Topology and identity come from the caller (D041, D007): the reader never
//! infers either, and the header is source metadata, not the sequence's name.

use crate::error::DomainError;
use crate::error::FastaRule;
use crate::error::MalformedFastaProblem;
use crate::identity::Identifier;
use crate::sequence::normalize;
use crate::sequence::Sequence;
use crate::sequence::SequenceTopology;

/// One FASTA record: its header line as source metadata, and its sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct FastaRecord {
    header: String,
    sequence: Sequence,
}

impl FastaRecord {
    /// The header line without its `>`, blanks at either end removed. Source
    /// metadata only (D007); the sequence's identifier is the caller's.
    pub fn header(&self) -> String {
        self.header.clone()
    }

    pub fn sequence(&self) -> Sequence {
        self.sequence.clone()
    }
}

/// True for an empty line, or one made only of spaces, tabs and `\r`.
fn is_blank(line: &str) -> bool {
    for ch in line.chars() {
        if !is_blank_char(ch) {
            return false;
        }
    }
    true
}

/// True for a blank character: a space, a tab or a carriage return.
fn is_blank_char(ch: char) -> bool {
    ch == ' ' || ch == '\t' || ch == '\r'
}

/// `text` with spaces, tabs and `\r` removed from both ends, internal
/// whitespace kept (rule 6).
fn trim_blanks(text: String) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut from: usize = 0;
    while from < chars.len() && is_blank_char(chars[from]) {
        from += 1;
    }
    let mut to: usize = chars.len();
    while to > from && is_blank_char(chars[to - 1]) {
        to -= 1;
    }
    chars[from..to].iter().collect()
}

/// The header text of a header line (rules 2 and 6): leading spaces and tabs
/// removed, the `>` removed, and spaces, tabs and `\r` trimmed from both
/// ends. `None` when the line is not a header line (rule 2).
fn header_of(line: &str) -> Option<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut lead: usize = 0;
    while lead < chars.len() && (chars[lead] == ' ' || chars[lead] == '\t') {
        lead += 1;
    }
    if lead == chars.len() || chars[lead] != '>' {
        return None;
    }
    let after_mark: String = chars[(lead + 1)..].iter().collect();
    Some(trim_blanks(after_mark))
}

/// Read a text that holds exactly one FASTA record. The topology and the
/// sequence's identifier come from the caller (D041, D007); the header line
/// is kept as source metadata only.
pub fn read_fasta(
    name: Identifier,
    text: String,
    topology: SequenceTopology,
) -> Result<FastaRecord, DomainError> {
    let mut have_header: bool = false;
    let mut header: String = "".to_string();
    let mut body_lines: Vec<String> = Vec::new();
    let mut second_record: Option<u64> = None;
    let mut number: u64 = 0;

    for line in text.lines() {
        number += 1;
        match header_of(line) {
            Some(header_text) => {
                if have_header {
                    // Rule 5: the first header line after the first.
                    if second_record.is_none() {
                        second_record = Some(number);
                    }
                } else {
                    header = header_text;
                    have_header = true;
                }
            }
            None => {
                if have_header {
                    // Rule 7: the body is every line after the header line.
                    body_lines.push(line.to_string());
                } else if !is_blank(line) {
                    // Rule 3: the first line before the header that is not
                    // blank. A `;` comment line and a byte-order mark fail
                    // here; neither is repaired or skipped (D008).
                    return Err(DomainError::MalformedFasta(MalformedFastaProblem {
                        sequence: name.value().to_string(),
                        rule: FastaRule::TextBeforeHeader,
                        line: Some(number),
                    }));
                }
            }
        }
    }

    // Rule 4: no header line at all, so every line was blank.
    if !have_header {
        return Err(DomainError::MalformedFasta(MalformedFastaProblem {
            sequence: name.value().to_string(),
            rule: FastaRule::NoRecord,
            line: None,
        }));
    }

    if second_record.is_some() {
        return Err(DomainError::MalformedFasta(MalformedFastaProblem {
            sequence: name.value().to_string(),
            rule: FastaRule::SecondRecord,
            line: second_record,
        }));
    }

    // Rule 7: the body goes to `normalize` unchanged, and so do its errors.
    let body: String = body_lines.join("\n");
    let sequence: Sequence = normalize(name, body, topology)?;
    Ok(FastaRecord { header, sequence })
}

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
        let expected = normalize(name(), "ACGTNNAC".to_string(), SequenceTopology::Linear).unwrap();
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
