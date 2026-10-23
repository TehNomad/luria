//! Milestone 7k1's proof (contract 7k1 §3.1): every line of
//! `tests/fixtures/new/shorthand.tsv` reads exactly as the fixture says.
//! The fixture is written by `tests/fixtures/new/reference/shorthand.py`,
//! independently of `crates/`, so its columns are the expected reading:
//! zero-based half-open start and end (D006), the bases upper case, and a
//! residue number as written. A refusal row names its code in the `from`
//! column (index 4); an insertion's bases are in the `to` column.

use luria_cli::shorthand::parse_line;
use luria_cli::shorthand::Reading;
use std::fs;

const FIXTURE: &str = "../../tests/fixtures/new/shorthand.tsv";
const HEADER: &str = "line\tkind\tstart\tend\tfrom\tto\tcodon";

/// Every fixture row, split into its seven columns.
fn rows() -> Vec<Vec<String>> {
    let text: String = fs::read_to_string(FIXTURE)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", FIXTURE, error));
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], HEADER);
    lines[1..]
        .iter()
        .map(|line| {
            let row: Vec<String> = line.split('\t').map(|field| field.to_string()).collect();
            assert_eq!(row.len(), 7, "the row {:?} is not seven columns", row[0]);
            row
        })
        .collect()
}

/// A position column, as the number it states.
fn position(row: &[String], column: usize, line: &str) -> i64 {
    row[column]
        .parse::<i64>()
        .unwrap_or_else(|error| panic!("{:?}: {} is not a position: {}", line, row[column], error))
}

/// A residue column, as its one-letter code.
fn letter(row: &[String], column: usize, line: &str) -> char {
    match row[column].chars().collect::<Vec<char>>().as_slice() {
        [letter] => *letter,
        _ => panic!("{:?}: {} is not one residue letter", line, row[column]),
    }
}

/// The name of the line being checked, for every failure message.
fn named(row: &[String]) -> String {
    format!("the line {:?} (kind {})", row[0], row[1])
}

/// The reading luria must give for this row; a refusal is a failure that
/// names the line, the code and the kind the row expects.
fn reading_of(row: &[String]) -> Reading {
    match parse_line(&row[0]) {
        Ok(found) => found,
        Err(refusal) => panic!(
            "{}: expected kind {}; luria refused with {} ({})",
            named(row),
            row[1],
            refusal.code(),
            refusal.message()
        ),
    }
}

#[test]
fn every_shorthand_row_matches() {
    for row in rows() {
        let line: &str = row[0].as_str();
        match row[1].as_str() {
            "refused" => match parse_line(line) {
                Err(refusal) => assert_eq!(refusal.code(), row[4], "{}", named(&row)),
                Ok(found) => panic!(
                    "{}: expected the refusal {}; luria read {:?}",
                    named(&row),
                    row[4],
                    found
                ),
            },
            "residue" => {
                let expected: Reading = Reading::Residue {
                    number: position(&row, 2, line),
                    from: letter(&row, 4, line),
                    to: letter(&row, 5, line),
                    codon: if row[6] == "policy" {
                        None
                    } else {
                        Some(row[6].clone())
                    },
                };
                assert_eq!(reading_of(&row), expected, "{}", named(&row));
            }
            "substitution" => {
                let expected: Reading = Reading::Substitution {
                    start: position(&row, 2, line),
                    end: position(&row, 3, line),
                    from: row[4].clone(),
                    to: row[5].clone(),
                };
                assert_eq!(reading_of(&row), expected, "{}", named(&row));
            }
            "replacement" => {
                let expected: Reading = Reading::Replacement {
                    start: position(&row, 2, line),
                    end: position(&row, 3, line),
                    from: row[4].clone(),
                    to: row[5].clone(),
                };
                assert_eq!(reading_of(&row), expected, "{}", named(&row));
            }
            "deletion" => {
                let expected: Reading = Reading::Deletion {
                    start: position(&row, 2, line),
                    end: position(&row, 3, line),
                };
                assert_eq!(reading_of(&row), expected, "{}", named(&row));
            }
            "insertion" => {
                let expected: Reading = Reading::Insertion {
                    point: position(&row, 2, line),
                    insert: row[5].clone(),
                };
                assert_eq!(reading_of(&row), expected, "{}", named(&row));
            }
            kind => panic!("{}: the fixture has no kind {}", named(&row), kind),
        }
    }
}
