//! `docs/cli.md` §12 against the code (contract 7k6): the command lines the
//! page shows are the ones `luria help` prints, and every mutation line of
//! §12.2's table parses with `luria_cli::shorthand::parse_line` as its row
//! says.

use luria_cli::args::USAGE;
use luria_cli::shorthand::{parse_line, Reading};
use std::fs;

const PAGE: &str = "../../docs/cli.md";

/// Everything after §12's heading, so a marker below is found in §12 alone.
fn section_12() -> String {
    let page: String = fs::read_to_string(PAGE).expect("docs/cli.md is readable");
    page.split("## 12. Entering a design in the terminal: `luria new`")
        .nth(1)
        .expect("docs/cli.md has §12")
        .to_string()
}

/// The first fenced block after `marker` in `text`, without its fences.
fn fenced_after(text: &str, marker: &str) -> String {
    let after: &str = text
        .split(marker)
        .nth(1)
        .unwrap_or_else(|| panic!("no {:?}", marker));
    let block: &str = after.split("```").nth(1).expect("a fenced block");
    block.split_once('\n').expect("a fence line").1.to_string()
}

/// The backticked pieces of `text`, in order.
fn backticked(text: &str) -> Vec<String> {
    text.split('`')
        .skip(1)
        .step_by(2)
        .map(|piece| piece.to_string())
        .collect()
}

/// One row of §12.2's table: the backticked examples in its first column,
/// and the meaning the second column gives.
struct Row {
    examples: Vec<String>,
    meaning: String,
}

/// §12.2's table, one row after its header and separator.
fn shorthand_rows() -> Vec<Row> {
    section_12()
        .split("### 12.2 The mutation shorthand")
        .nth(1)
        .expect("§12.2")
        .lines()
        .skip_while(|line| !line.starts_with('|'))
        .take_while(|line| line.starts_with('|'))
        .skip(2)
        .map(|line| {
            let cells: Vec<&str> = line.split('|').map(|cell| cell.trim()).collect();
            Row {
                examples: backticked(cells[1]),
                meaning: cells[2].to_string(),
            }
        })
        .collect()
}

/// The reading the row's words name: `Residue` in the words, a `Base … from`
/// of one base, `replaced`, `deleted` or `inserted`; `None` when the row
/// names no kind, which a `The same` row takes from the row before.
fn expected(meaning: &str) -> Option<Reading> {
    if meaning.contains("replaced") {
        return Some(Reading::Replacement {
            start: 0,
            end: 0,
            from: String::new(),
            to: String::new(),
        });
    }
    if meaning.contains("deleted") {
        return Some(Reading::Deletion { start: 0, end: 0 });
    }
    if meaning.contains("inserted") {
        return Some(Reading::Insertion {
            point: 0,
            insert: String::new(),
        });
    }
    if meaning.contains("Residue") {
        return Some(Reading::Residue {
            number: 0,
            from: 'A',
            to: 'A',
            codon: None,
        });
    }
    if meaning.starts_with("Base ") && meaning.contains(" from") {
        return Some(Reading::Substitution {
            start: 0,
            end: 0,
            from: String::new(),
            to: String::new(),
        });
    }
    None
}

/// The kind's name, as the test words it.
fn named(reading: &Reading) -> &'static str {
    match reading {
        Reading::Residue { .. } => "Residue",
        Reading::Substitution { .. } => "Substitution",
        Reading::Replacement { .. } => "Replacement",
        Reading::Deletion { .. } => "Deletion",
        Reading::Insertion { .. } => "Insertion",
    }
}

#[test]
fn the_documented_command_lines_are_the_usage_text() {
    let section: String = section_12();
    let shown: String = fenced_after(&section, "docs/tasks/milestone-7k-new.md`.");
    let lines: Vec<&str> = shown.lines().map(|line| line.trim()).collect();
    assert_eq!(lines.len(), 2, "§12 shows two command lines");
    for line in lines {
        assert!(USAGE.contains(line), "luria help does not show {:?}", line);
    }
}

#[test]
fn the_documented_shorthand_reads_as_documented() {
    let rows: Vec<Row> = shorthand_rows();
    assert_eq!(rows.len(), 6, "§12.2's table gives six rows");
    let mut previous: Option<Reading> = None;
    for row in &rows {
        let kind: Reading = match expected(&row.meaning) {
            Some(reading) => {
                previous = Some(reading.clone());
                reading
            }
            None => {
                if row.meaning.starts_with("The same") {
                    previous
                        .clone()
                        .expect("a `The same` row follows a row that names a kind")
                } else {
                    panic!(
                        "the row {:?} names no kind in {:?}, so its example {:?} cannot be checked",
                        row.examples, row.meaning, row.examples[0]
                    )
                }
            }
        };
        // A `The same` row names the codon it claims, as `D404N:AAC` does.
        let codon: Option<String> = if row.meaning.starts_with("The same") {
            backticked(&row.meaning)
                .into_iter()
                .next()
                .filter(|named| !named.is_empty())
        } else {
            None
        };
        for example in &row.examples {
            let reading: Reading = match parse_line(example) {
                Ok(found) => found,
                Err(refusal) => panic!(
                    "the page's row {:?} says {}, so {:?} must read as {}; luria refused with {} ({})",
                    row.meaning,
                    named(&kind),
                    example,
                    named(&kind),
                    refusal.code(),
                    refusal.message()
                ),
            };
            assert_eq!(
                std::mem::discriminant(&reading),
                std::mem::discriminant(&kind),
                "the page's row {:?} says {}, so {:?} reads as {}",
                row.meaning,
                named(&kind),
                example,
                named(&reading)
            );
            if let Some(written) = &codon {
                match &reading {
                    Reading::Residue { codon: given, .. } => assert_eq!(
                        given,
                        &Some(written.clone()),
                        "the page's row {:?} says the codon is `{}`, so {:?} reads with it",
                        row.meaning,
                        written,
                        example
                    ),
                    _ => assert_eq!(named(&kind), "Residue", "only a residue line names a codon"),
                }
            }
        }
    }
}
