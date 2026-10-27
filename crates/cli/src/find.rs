//! `luria find`: one sequence file, one pattern, the hits as tab-separated rows
//! (plan §5.5, contract 8i2 §4; D049's M8-15 and M8-20, D050, D052, D054, D056).
//!
//! The rules, in §4's order:
//!
//! * **The arguments** (§4.1) are `args::parse_find`'s; this module starts at
//!   the `--cds` range, which D049 refuses for milestone 8 with a message saying
//!   ranges come in milestone 9, so the flag's grammar never changes under a
//!   running script.
//! * **The file** is `record_file::read_record_file`'s: the byte ceiling before
//!   a byte is read, then `open_record`, whose refusal comes back unchanged
//!   (D048, D049, D056).
//! * **`--circular`** (D049's M8-20) is for FASTA only. A GenBank record states
//!   its own topology (D036), and contradicting it is refused, not ignored.
//! * **The mode** (§4.5, plan §5.4) is protein when `--aa` or `--cds` says so,
//!   and otherwise DNA unless the pattern holds a letter that cannot be an IUPAC
//!   nucleotide. So `MKSNNA` — every letter of it a nucleotide code — is a DNA
//!   pattern, and `MSIQHF` is not.
//! * **`--cds`** (§4.6) is matched by core's `name_match::match_cds`, the same
//!   matcher go-to and the feature list will use (D056). No match and several are
//!   exit 2, and several lists the candidates with the id that matched and the
//!   span it matched them at: never a guess, never a prompt (D049, D005). A value
//!   naming one unusable CDS exits with that CDS's own refusal and its
//!   category's code, because the refusal is core's judgement and the exit code
//!   belongs to it (D042, D052's M8-5).
//! * **The search** is core's: `find_dna`, or `find_protein` on the matched CDS
//!   or the six frames. Core returns every hit uncapped (D050), and the 200 rows
//!   that print are presentation (M8-12).
//! * **The output** (§4.8) names what it searched and claims nothing else
//!   (D052): the record's format, length and topology, the pattern and the mode,
//!   the CDS a `--cds` run read, and a last line for what the run did *not* look
//!   at. A search that finds nothing is a successful run, exit 0 with `0 hits`
//!   (M8-9, D052).
//!
//! The `#` lines and the rows are built by `search_record`, which reads no file
//! and prints nothing; `run` reads the file and writes the answer, and the exit
//! status is whatever the outcome says it is.

use std::io::{stderr, stdout, Write};
use std::path::Path;

use luria_core::cds::Cds;
use luria_core::error::DomainError;
use luria_core::name_match::cds_span;
use luria_core::name_match::match_cds;
use luria_core::name_match::Matched;
use luria_core::name_match::NameMatch;
use luria_core::name_match::NameSource;
use luria_core::protein_search::find_protein;
use luria_core::protein_search::ProteinHit;
use luria_core::protein_search::ProteinTarget;
use luria_core::search::find_dna;
use luria_core::search::DnaHit;
use luria_core::sequence::normalize;
use luria_core::sequence::Sequence;
use luria_core::sequence::SequenceTopology;
use luria_core::view_record::CdsStatus;
use luria_core::view_record::FeatureShape;
use luria_core::view_record::RecordFormat;
use luria_core::view_record::TopologySource;
use luria_core::view_record::ViewCds;
use luria_core::view_record::ViewRecord;

use crate::args::USAGE;
use crate::exit::ExitStatus;
use crate::record_file::read_record_file;

/// The rows one run prints, however many hits core returned: the cap is
/// presentation and never a rule of core (M8-12, D048, D050's M8-16). Past it the
/// count line says what was cut off (§4.8).
const MAX_ROWS: usize = 200;

/// The letters a DNA pattern may hold, either case: `sequence::DNA_ALPHABET`
/// with `U` added, which a pattern may hold because `search::find_dna` folds it
/// to `T` before it validates (search.rs rule 2). The mode rule reads these, and
/// nothing else, as nucleotides (plan §5.4).
const NUCLEOTIDE_LETTERS: &str = "ACGTURYSWKMBDHVN";

/// D049's M8-15, plan §5.5: `--cds` takes no range in milestone 8, and the
/// refusal says when ranges come.
const RANGE_MESSAGE: &str = "searching a range of bases comes in milestone 9";

/// What one `find` run answers, before anything is printed. `Found` is the whole
/// stdout text and exit 0, whatever the count (M8-9, D052); `Refused` is one
/// message on stderr with the status it exits with and no usage text; `Usage` is
/// a usage error, which goes with the usage text and exit 2 (docs/cli.md §3.7).
#[derive(Clone, Debug, PartialEq)]
pub enum FindOutcome {
    Found { text: String },
    Refused { message: String, status: ExitStatus },
    Usage { message: String },
}

/// What the pattern was searched for: §4.5's mode, and under `--cds` the one CDS
/// §4.6 answered. `phrase` is the `{name} ({a}..{b}, {strand})` the `# searched:`
/// line names a CDS search by, so the output says whose translation was read
/// (D052, plan §5.5).
enum What<'a> {
    Dna,
    SixFrames,
    InCds(&'a Cds, String),
}

/// §4.3's and §4.7's shared shape: core's own message, and the status its own
/// category gives, with no usage text.
fn refusal(error: &DomainError) -> FindOutcome {
    FindOutcome::Refused {
        message: error.message(),
        status: ExitStatus::for_failure(error.category()),
    }
}

/// Whether `letter` can be a nucleotide, either case (§4.5). `MKSNNA` holds only
/// IUPAC letters, so that pattern is DNA and not protein (test 7). The view's
/// search prompt reads the same rule (8l1 §4 rule 2), which is why this is
/// `pub(crate)` and the mode decision is not written twice.
pub(crate) fn nucleotide_letter(letter: char) -> bool {
    NUCLEOTIDE_LETTERS.contains(letter.to_ascii_uppercase())
}

/// Whether `value` is entirely the range form `c?DIGITS..DIGITS`, its leading and
/// trailing spaces trimmed (§4.2). The `c` is milestone 9's circular range
/// (`--cds c345..43`, plan §5.5). Anything else — a letter, a space between the
/// digits, an empty side, a third run of dots — is a name, and `match_cds`
/// decides what it names.
fn is_range(value: &str) -> bool {
    let text: &str = value.trim();
    let body: &str = match text.strip_prefix('c') {
        Some(rest) => rest,
        None => text,
    };
    let sides: Vec<&str> = body.splitn(2, "..").collect();

    sides.len() == 2
        && !sides[0].is_empty()
        && !sides[1].is_empty()
        && sides[0].bytes().all(|byte| byte.is_ascii_digit())
        && sides[1].bytes().all(|byte| byte.is_ascii_digit())
}

/// The `# record:` line's format name, as §4.8 spells it.
fn format_name(record: &ViewRecord) -> &'static str {
    match record.format {
        RecordFormat::GenBank => "GenBank",
        RecordFormat::Fasta => "FASTA",
    }
}

/// The `# record:` line's topology: a GenBank record states its own (D036); a
/// FASTA record states none, so the linear assumption is named as an assumption
/// (D049); and a `--circular` run names itself as the reason the sequence
/// searched was circular (§4.8). Only a FASTA record reaches the second arm,
/// because §4.4 refuses `--circular` for a GenBank record.
fn topology_phrase(record: &ViewRecord, circular: bool) -> String {
    match record.topology {
        TopologySource::Stated => record.sequence.topology().name(),
        TopologySource::Assumed if circular => "circular (--circular)".to_string(),
        TopologySource::Assumed => "linear (assumed: a FASTA file states no topology)".to_string(),
    }
}

/// The `# searched:` line's phrase: the mode, and for a CDS search the CDS with
/// its span and strand (D052 — a `0 hits` row must not read as "this record has
/// no such site", because the run looked at one molecule in one mode).
fn searched_as(what: &What) -> String {
    match what {
        What::Dna => "DNA on both strands".to_string(),
        What::SixFrames => "protein in six frames".to_string(),
        What::InCds(_, phrase) => format!("protein in CDS {}", phrase),
    }
}

/// The column names, which say which kind of row follows them (plan §5.5): three
/// columns for DNA, six for protein (§4.8).
fn header(what: &What) -> String {
    match what {
        What::Dna => "start\tend\tstrand".to_string(),
        What::SixFrames | What::InCds(_, _) => {
            "start\tend\tstrand\tframe\tfirst_residue\tlast_residue".to_string()
        }
    }
}

/// The 1-based inclusive last base of the span that starts at the zero-based
/// `start` and runs `width` bases: `(start + width - 1) mod L + 1`. A span of a
/// circular record that runs past the last base is numbered again from the first,
/// and its `end` is then below its `start` (§4.8, plan §5.5). `length` is never
/// zero, because a reader refuses a record with no bases (D008).
fn last_base(start: u64, width: u64, length: u64) -> u64 {
    start.saturating_add(width.saturating_sub(1)) % length + 1
}

/// The DNA rows, in the order core returned them (D054): first base, last base,
/// and the strand — `+`, `-`, or `both` for a span that matches on both, which is
/// one hit and not two (M8-8, D052).
fn dna_rows(hits: &[DnaHit], length: u64) -> Vec<String> {
    hits.iter()
        .map(|hit| {
            format!(
                "{}\t{}\t{}",
                hit.start + 1,
                last_base(hit.start, hit.length, length),
                hit.strand.symbol()
            )
        })
        .collect()
}

/// The protein rows: the same base coordinates, then the strand the reading ran
/// on, the frame, and the two residue numbers (plan §5.5).
fn protein_rows(hits: &[ProteinHit], length: u64) -> Vec<String> {
    hits.iter()
        .map(|hit| {
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                hit.start + 1,
                last_base(hit.start, hit.length, length),
                hit.strand.clone().symbol(),
                frame_cell(hit),
                residue_cell(hit.first),
                residue_cell(hit.last)
            )
        })
        .collect()
}

/// The `frame` cell: 1, 2 or 3 in six-frame mode, `-` in CDS mode, where there
/// is no frame to name (§4.8). A cell is never empty (D050's M8-17).
fn frame_cell(hit: &ProteinHit) -> String {
    match hit.frame {
        Some(frame) => frame.index().to_string(),
        None => "-".to_string(),
    }
}

/// One residue number, or the word `stop` where core says there is none: a hit
/// that runs onto a CDS's terminal stop codon has no last residue there, because
/// that codon is not a residue (D010, D050's M8-17, §4.8). A frame's codons are
/// all numbered, so its cells hold numbers (D050).
fn residue_cell(number: Option<u64>) -> String {
    match number {
        Some(number) => number.to_string(),
        None => "stop".to_string(),
    }
}

/// The two lines that close the rows: the cap's own line when core returned more
/// hits than print (M8-12), then the count, `1 hit` for one hit and `N hits` for
/// any other number — `0 hits` included, which is a successful run (D052).
fn count_lines(total: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    if total > MAX_ROWS {
        lines.push(format!("showing the first {} of {} hits", MAX_ROWS, total));
    }
    lines.push(if total == 1 {
        "1 hit".to_string()
    } else {
        format!("{} hits", total)
    });
    lines
}

/// The lines that say what the run did *not* look at, after the count (D052:
/// "the output names what it searched, and claims nothing else"). A linear
/// sequence has no hit across its end, because nothing wraps (plan §5.5); six
/// frames never run across the origin, on a circular record as on a linear one
/// (D052's M8-2).
fn caveat_lines(searched: &Sequence, six_frames: bool) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    match searched.topology() {
        SequenceTopology::Linear => {
            lines.push("linear: hits do not cross the end".to_string());
        }
        SequenceTopology::Circular => {
            if six_frames {
                lines.push("frames do not cross the origin".to_string());
            }
        }
    }
    lines
}

/// The id source §4.6 names on a candidate line: the display name the feature
/// list shows, the qualifier the record states, or the list form (D049).
fn source_name(source: NameSource) -> &'static str {
    match source {
        NameSource::DisplayName => "name",
        NameSource::Gene => "/gene",
        NameSource::LocusTag => "/locus_tag",
        NameSource::ProteinId => "/protein_id",
        NameSource::ListForm => "list form",
    }
}

/// `{name} ({a}..{b}, {strand})`: a CDS's display name (D053) and where it sits,
/// the span `name_match::cds_span` reads in D036 order and the strand the
/// feature's own location states. A CDS whose location the display parser
/// refused shows `(not drawn)` in place of the parenthetical, because an undrawn
/// location states neither a span nor a strand and `find` invents neither (D053,
/// D005); pBR322 has no such CDS, so no §5 test prints it.
fn cds_phrase(record: &ViewRecord, item: usize) -> String {
    let view: &ViewCds = &record.cdss[item];
    let place: String = match &record.features[view.feature].shape {
        FeatureShape::Drawn(shape) => match cds_span(record, item) {
            Some((first, last)) => format!(
                "({}..{}, {})",
                first,
                last,
                shape.location.strand().symbol()
            ),
            None => "(not drawn)".to_string(),
        },
        FeatureShape::NotDrawn(_refusal) => "(not drawn)".to_string(),
    };
    format!("{} {}", record.features[view.feature].name, place)
}

/// §4.6's `Several`: the line that refuses, then one line per candidate with the
/// id that matched, its source and the CDS's span. D049 refuses rather than
/// picking, because a command a script runs must fail rather than wait; choosing
/// among them by eye is the view's picker (D005).
fn candidates(file: &Path, record: &ViewRecord, value: &str, found: &[NameMatch]) -> String {
    let mut lines: Vec<String> = vec![format!(
        "'{}' names {} CDSs in {}; give one of them:",
        value,
        found.len(),
        file.display()
    )];
    for candidate in found.iter() {
        lines.push(format!(
            "  {} matched by {} '{}'",
            cds_phrase(record, candidate.item),
            source_name(candidate.source),
            candidate.name
        ));
    }
    lines.join("\n")
}

/// Everything §4 does once the record is in hand: `--circular`, the mode,
/// `--cds`, the search and the rows. This is the pure half of `find` — it reads
/// no file and prints nothing, and takes `file` only to name it in the `# file:`
/// line and in the messages, where it is printed as it was given (§4.8).
pub fn search_record(
    file: &Path,
    record: &ViewRecord,
    pattern: &str,
    aa: bool,
    cds: Option<&str>,
    circular: bool,
) -> FindOutcome {
    // §4.4 before anything that reads the record: a GenBank record states its own
    // topology, and `--circular` would overrule it (D049's M8-20, D036). Exit 2
    // with no usage text, because the command line is well formed (D056).
    if circular && record.format == RecordFormat::GenBank {
        return FindOutcome::Refused {
            message: format!(
                "'--circular' is for FASTA files; {} states its own topology",
                file.display()
            ),
            status: ExitStatus::Usage,
        };
    }

    // §4.5: protein when `--aa` or `--cds` says so, and otherwise DNA unless the
    // pattern holds a letter that cannot be a nucleotide (plan §5.4's mode rule).
    let protein: bool = aa || cds.is_some() || !pattern.chars().all(nucleotide_letter);

    // §4.4: on FASTA, the search runs on a circular copy of the same bases.
    // `sequence::normalize` is the only way to a `Sequence`, and these letters
    // came from one, so they are canonical and it cannot refuse them; if it ever
    // did, its own message is what the user would see.
    let searched: Sequence = if circular {
        match normalize(
            record.sequence.name(),
            record.sequence.canonical_text(),
            SequenceTopology::Circular,
        ) {
            Ok(copy) => copy,
            Err(error) => return refusal(&error),
        }
    } else {
        record.sequence.clone()
    };

    // §4.6: what `--cds VALUE` names, by the matcher go-to and the feature list
    // will share (D056). These three answers print no usage text, and none of
    // them guesses (D049).
    let what: What = match cds {
        None if protein => What::SixFrames,
        None => What::Dna,
        Some(value) => match match_cds(record, value) {
            Matched::Nothing => {
                return FindOutcome::Refused {
                    message: format!("no CDS in {} is named '{}'", file.display(), value),
                    status: ExitStatus::Usage,
                }
            }
            Matched::Several(found) => {
                return FindOutcome::Refused {
                    message: candidates(file, record, value, &found),
                    status: ExitStatus::Usage,
                }
            }
            Matched::One(found) => {
                let view: &ViewCds = &record.cdss[found.item];
                match &view.status {
                    // D042's refusal, D052's M8-5: the reason a CDS cannot be
                    // searched is core's, and so is the exit code its category
                    // gives. The display name is what the feature list calls it.
                    CdsStatus::Unusable(error) => {
                        return FindOutcome::Refused {
                            message: format!(
                                "CDS '{}' cannot be searched: {}",
                                record.features[view.feature].name,
                                error.message()
                            ),
                            status: ExitStatus::for_failure(error.category()),
                        }
                    }
                    CdsStatus::Usable(cds) => What::InCds(cds, cds_phrase(record, found.item)),
                }
            }
        },
    };

    // §4.7: the search. An empty pattern or a letter the mode's alphabet does not
    // hold is core's refusal, with its own category's exit code (§4.7).
    let length: u64 = searched.length();
    let (rows, total): (Vec<String>, usize) = match &what {
        What::Dna => match find_dna(&searched, pattern) {
            Err(error) => return refusal(&error),
            Ok(hits) => (dna_rows(&hits, length), hits.len()),
        },
        What::SixFrames => match find_protein(&searched, ProteinTarget::SixFrames, pattern) {
            Err(error) => return refusal(&error),
            Ok(hits) => (protein_rows(&hits, length), hits.len()),
        },
        What::InCds(cds, _) => match find_protein(&searched, ProteinTarget::Cds(cds), pattern) {
            Err(error) => return refusal(&error),
            Ok(hits) => (protein_rows(&hits, length), hits.len()),
        },
    };

    // §4.8: the `#` lines name what was read and what was searched; the header
    // says which rows follow; at most `MAX_ROWS` of them print (M8-12); and the
    // lines after the count say what the run did not look at (D052).
    let mut lines: Vec<String> = vec![
        format!("# file: {}", file.display()),
        format!(
            "# record: {}, {} bp, {}",
            format_name(record),
            length,
            topology_phrase(record, circular)
        ),
        format!("# searched: {} as {}", pattern, searched_as(&what)),
        header(&what),
    ];
    lines.extend(rows.iter().take(MAX_ROWS).cloned());
    lines.extend(count_lines(total));
    lines.extend(caveat_lines(&searched, matches!(what, What::SixFrames)));

    FindOutcome::Found {
        text: lines.join("\n") + "\n",
    }
}

/// One message on stderr; a failed write changes nothing, because the run still
/// reports the status of its own failure (as `main.rs`'s `write_err`).
fn write_err(text: &str) {
    let mut stream = stderr().lock();
    let _ = stream.write_all(text.as_bytes());
    let _ = stream.flush();
}

/// Print the answer on the stream it belongs to and give the status it exits
/// with: the rows on stdout and exit 0 (D052's M8-9), a refusal on stderr with
/// its own status and no usage text, and a usage error on stderr with the usage
/// text and exit 2 (docs/cli.md §3.7). A stdout that cannot be written is exit 1,
/// the internal error `main.rs` gives a report that cannot be made.
fn report(outcome: FindOutcome) -> ExitStatus {
    match outcome {
        FindOutcome::Found { text } => {
            let mut stream = stdout().lock();
            if stream.write_all(text.as_bytes()).is_err() {
                return ExitStatus::Internal;
            }
            if stream.flush().is_err() {
                return ExitStatus::Internal;
            }
            ExitStatus::Success
        }
        FindOutcome::Refused { message, status } => {
            write_err(format!("luria: {}\n", message).as_str());
            status
        }
        FindOutcome::Usage { message } => {
            write_err(format!("luria: {}\n\n{}", message, USAGE).as_str());
            ExitStatus::Usage
        }
    }
}

/// Run `luria find` in §4's order and print the answer: the `--cds` range check,
/// then the file, then `search_record`, then `report`. The range is refused
/// before the file is read, so a range and a missing file give the same
/// answer (test 10).
pub fn run(file: &Path, pattern: &str, aa: bool, cds: Option<&str>, circular: bool) -> ExitStatus {
    if let Some(value) = cds {
        if is_range(value) {
            return report(FindOutcome::Usage {
                message: RANGE_MESSAGE.to_string(),
            });
        }
    }

    let outcome: FindOutcome = match read_record_file(file) {
        Ok(record) => search_record(file, &record, pattern, aa, cds, circular),
        Err(problem) => {
            let (message, status): (String, ExitStatus) = problem.report(file);
            FindOutcome::Refused { message, status }
        }
    };

    report(outcome)
}
