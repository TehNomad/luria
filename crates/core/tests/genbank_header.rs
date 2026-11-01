//! What the GenBank reader keeps of a record for the writer to write it back
//! (contract 9c1, plan M9-7 and M9-15): the LOCUS line as read with its
//! molecule type, division and date, the header lines between LOCUS and
//! FEATURES, and every feature's own lines. Checked against pBR322 as NCBI
//! wrote it (`tests/fixtures/genbank/raw/J01749.1.gb`).
//!
//! The expectations are the fixture's own lines, counted by hand and given in
//! §2 of the contract: the LOCUS line is line 1, the FEATURES line is line
//! 231, the first feature line is line 232 and the ORIGIN line is line 399.
//! The molecule type, division and date come from the LOCUS line itself, read
//! by hand, and not from what luria answers.

use luria_core::genbank::read_genbank;
use luria_core::genbank::GenBankFeature;
use luria_core::genbank::GenBankRecord;
use luria_core::identity::Identifier;

const FIXTURES: &str = "../../tests/fixtures/genbank/";
const PBR322_FILE: &str = "raw/J01749.1.gb";
const LOCUS_LINE: &str =
    "LOCUS       SYNPBR322               4361 bp    DNA     circular SYN 30-SEP-2008";

/// pBR322 as it sits on disk, and the text's lines as `str::lines()` gives
/// them, so that a test may name a line by its number.
fn pbr322() -> (GenBankRecord, Vec<String>) {
    let path: String = format!("{}{}", FIXTURES, PBR322_FILE);
    let text: String = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
    let lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let record: GenBankRecord = read_genbank(
        Identifier::create("pbr322".to_string()).unwrap(),
        text.clone(),
    )
    .unwrap_or_else(|error| {
        panic!(
            "{}: read_genbank refused it: {:?}",
            PBR322_FILE,
            error.message()
        )
    });
    (record, lines)
}

/// The fixture's non-blank lines `from..=to` counted from 1, in file order.
fn file_lines(lines: &[String], from: usize, to: usize) -> Vec<String> {
    assert!(
        from >= 1 && from <= to && to <= lines.len(),
        "lines {}..{} out of a {}-line fixture",
        from,
        to,
        lines.len()
    );
    lines[from - 1..to]
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string())
        .collect()
}

/// The fixture's layout, as the contract states it: LOCUS line 1, the
/// FEATURES line 231, the first feature line 232, the last feature line 398,
/// ORIGIN line 399 and its first sequence line 400.
fn landmarks(lines: &[String]) {
    assert_eq!(lines[0], LOCUS_LINE);
    assert!(lines[230].starts_with("FEATURES"));
    assert_eq!(lines[231], "     source          1..4361");
    assert_eq!(
        lines[397],
        "                     /bound_moiety=\"echinomycin\""
    );
    assert!(lines[398].starts_with("ORIGIN"));
    assert!(lines[399].trim_start().starts_with("1 ttctcatgtt"));
}

#[test]
fn pbr322_keeps_its_header() {
    let (record, lines) = pbr322();
    landmarks(&lines);

    // The LOCUS line as read, and the tokens it states (M9-15).
    assert_eq!(record.locus_line(), LOCUS_LINE);
    assert_eq!(record.locus_line(), lines[0]);
    assert_eq!(record.molecule_type(), Some("DNA".to_string()));
    assert_eq!(record.division(), Some("SYN".to_string()));
    assert_eq!(record.date(), Some("30-SEP-2008".to_string()));

    // Every non-blank header line, verbatim, in file order (M9-7): the lines
    // between LOCUS and FEATURES, ACCESSION and VERSION among them.
    let expected: Vec<String> = file_lines(&lines, 2, 230);
    assert_eq!(record.header_lines().len(), expected.len());
    assert_eq!(record.header_lines(), expected);
    assert_eq!(
        record.header_lines()[0],
        "DEFINITION  Cloning vector pBR322, complete sequence."
    );
    assert_eq!(
        record.header_lines()[record.header_lines().len() - 1],
        lines[229]
    );
    assert!(!record
        .header_lines()
        .iter()
        .any(|line| line.starts_with("FEATURES") || line.starts_with("ORIGIN")));
}

#[test]
fn pbr322_features_keep_their_lines() {
    let (record, lines) = pbr322();
    landmarks(&lines);

    let features: Vec<GenBankFeature> = record.features();
    let kept: Vec<String> = features
        .iter()
        .flat_map(|feature| feature.lines())
        .collect();

    // Joining every feature's lines, in order, gives the whole FEATURES
    // section as the file wrote it, and nothing outside it.
    assert_eq!(kept, file_lines(&lines, 232, 398));

    // The first feature keeps its key line, and every feature starts with
    // the line the reader counted it on.
    assert_eq!(features[0].lines()[0], lines[231]);
    assert_eq!(features[0].line(), 232);
    for feature in &features {
        assert!(!feature.lines().is_empty());
        assert_eq!(
            feature.lines()[0],
            lines[(feature.line() - 1) as usize],
            "feature {:?} on line {}",
            feature.key(),
            feature.line()
        );
        // Its first line states its location: the location's first ten
        // characters are on it.
        let head: String = feature.location().chars().take(10).collect();
        assert!(
            feature.lines()[0].contains(head.as_str()),
            "feature {:?} on line {}: {:?} does not state {:?}",
            feature.key(),
            feature.line(),
            feature.lines()[0],
            head
        );
    }
}
