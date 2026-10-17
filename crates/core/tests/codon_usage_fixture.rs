//! The 64 codon counts in `crates/core/src/codon_usage.rs` were transcribed by
//! hand from the Kazusa Codon Usage Database page for *Escherichia coli* W3110
//! on 2026-10-14. `tests/fixtures/codon-usage/` keeps a copy of that page and
//! parses it with `reference/parse_kazusa.py`, a script that shares no code
//! with `crates/core`; the first test below is what holds the transcription
//! against the source (D039).
//!
//! The third test reads the residue column the page prints and checks it
//! against `genetic_code::lookup`. That is a check of luria's own genetic-code
//! table against an external source, not a restatement of it: the page states
//! the residue of every codon under genetic code 1, so a wrong entry in
//! `STANDARD_GENETIC_CODE` shows up as a disagreement here.
//!
//! The parse itself is rerun from the repository root, not by cargo:
//!
//! ```
//! python -I tests/fixtures/codon-usage/reference/parse_kazusa.py --check
//! ```
//!
//! That Python check is not part of the required cargo checks (D039, D026).

use luria_core::codon_usage::ECOLI_K12_CODON_COUNTS;
use luria_core::genetic_code::lookup;
use luria_core::genetic_code::CodonOutcome;

const FIXTURE: &str = "../../tests/fixtures/codon-usage/counts.tsv";

/// The published total the counts in the fixture and in `codon_usage.rs` must
/// both reach: the codons of the 4332 CDSs the entry covers.
const PUBLISHED_CODONS: u64 = 1372057;

/// One row of `counts.tsv`: the codon as DNA, the one-letter residue the page
/// prints for it (`*` for stop), and its count.
struct Row {
    codon: String,
    residue: String,
    count: u64,
}

/// The rows of `counts.tsv`, skipping `#` comment lines and the header.
fn rows() -> Vec<Row> {
    let text: String = std::fs::read_to_string(FIXTURE)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", FIXTURE, error));
    let mut out: Vec<Row> = Vec::new();
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("codon\t") {
            continue;
        }
        let fields: Vec<&str> = trimmed.split('\t').collect();
        assert!(
            fields.len() == 3,
            "{}: row {:?} has {} fields, expected 3",
            FIXTURE,
            fields.first(),
            fields.len(),
        );
        out.push(Row {
            codon: fields[0].to_string(),
            residue: fields[1].to_string(),
            count: fields[2].parse::<u64>().unwrap_or_else(|_| {
                panic!(
                    "{}: {:?} count {:?} is not a number",
                    FIXTURE,
                    fields.first(),
                    fields[2]
                )
            }),
        });
    }
    out
}

/// The transcription in `codon_usage.rs` against the parsed page, codon by codon
/// and count by count, in standard table order: the capture is read in the same
/// order the embedded table is written in, so a row that is out of place is a
/// disagreement at that position.
#[test]
fn the_embedded_table_equals_the_kazusa_capture() {
    let capture: Vec<Row> = rows();
    assert!(
        capture.len() == ECOLI_K12_CODON_COUNTS.len(),
        "the capture has {} rows, codon_usage.rs has {}",
        capture.len(),
        ECOLI_K12_CODON_COUNTS.len(),
    );
    for ((codon, count), row) in ECOLI_K12_CODON_COUNTS.iter().zip(capture.iter()) {
        assert!(
            *codon == row.codon,
            "codon_usage.rs has codon {}, the capture has {}",
            *codon,
            row.codon,
        );
        assert!(
            *count == row.count,
            "{}: codon_usage.rs has count {}, the capture has {}",
            *codon,
            *count,
            row.count,
        );
    }
}

#[test]
fn the_capture_totals_the_published_codon_count() {
    let mut total: u64 = 0;
    for row in rows().iter() {
        total += row.count;
    }
    assert!(
        total == PUBLISHED_CODONS,
        "the capture sums to {}, the entry publishes {}",
        total,
        PUBLISHED_CODONS,
    );
}

/// Every row states the residue its codon stands for under genetic code 1
/// (standard), `*` for stop. `genetic_code::lookup` must give the same result.
#[test]
fn the_capture_agrees_with_the_standard_genetic_code() {
    let capture: Vec<Row> = rows();
    assert!(
        capture.len() == 64,
        "the capture has {} rows, expected 64",
        capture.len(),
    );
    for row in capture.iter() {
        let outcome: CodonOutcome = lookup(row.codon.clone());
        if row.residue == "*" {
            assert!(
                outcome == CodonOutcome::Stop,
                "{}: the capture prints stop, lookup gives {:?}",
                row.codon,
                outcome,
            );
        } else {
            let agreed: bool = match &outcome {
                CodonOutcome::Residue(residue) => residue.symbol() == row.residue,
                CodonOutcome::Stop | CodonOutcome::Ambiguous | CodonOutcome::Invalid => false,
            };
            assert!(
                agreed,
                "{}: the capture prints {}, lookup gives {:?}",
                row.codon, row.residue, outcome,
            );
        }
    }
}
