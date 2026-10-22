//! The text of `luria review`'s result screens (milestone 7c1, 7c2): the
//! overview, the member screen, the problem screen and the provenance screen, as
//! the lines `luria_cli::review::text` builds from the result document (plan
//! §5.2 screens 2, 3, 5 and 6).
//!
//! Every expected line is built from the document's own field, or is a fact
//! recorded in an independently verified fixture table:
//! `tests/fixtures/genbank/synthetic-linear-record.tsv` (LOCUS `synlin`, 393 bp,
//! `linear`, sha256 `a8b4ddcfa617179143d428dc103aabb7b13cf27f7b323407d11db8a01105bbd7`,
//! accession `SYN0002`, version `SYN0002.1`),
//! `tests/fixtures/genbank/synthetic-linear-cds.tsv` (`synA`, `[42, 345)`, `+`,
//! its id from `/gene`),
//! `tests/fixtures/genbank/synthetic-linear-residues.tsv` (residue 42 is `A`
//! with codon `GCA`, residue 87 is `W` with codon `TGG`),
//! `tests/fixtures/genbank/pbr322-cds.tsv` (`tet` and `bla`) and
//! `tests/fixtures/genbank/pbr322-record.tsv` (the digest below). No value is
//! copied from running the code: a line of a design is asserted only against the
//! document's own field (its primer text, length, span, parts, annealing, Tm and
//! basis as the document holds them), because the documents' primer values are
//! not independently verified for `synthetic-linear.gb`, and every other expected
//! string names a fixture fact. Every value the screen shows is one the document
//! holds, so no line asserts a Tm, coordinate, codon, translation, reverse
//! complement or rounding the code did not compute.
//!
//! The documents are `luria_cli::run::run` on a request in a directory under
//! `CARGO_TARGET_TMPDIR`, as `tests/review_view.rs` does: the two-variant set of
//! `variants-separate.json`, the brief's fail-closed clarification as
//! `tests/end_to_end.rs`'s `the_fail_closed_example_asks_and_edits_nothing` runs
//! it, the pBR322 `tet` `T99C` design failure (G7, D029) as
//! `a_fasta_and_a_genbank_template_give_the_same_result` builds it, and a
//! request path that is not in the directory.

use serde_json::{json, Value};

use luria_cli::result::{DesignView, DetailValue, EditedView, MutationView, ResultDocument};
use luria_cli::review::app::Screen;
use luria_cli::review::text;
use luria_cli::run::run;
use std::fs;
use std::path::PathBuf;

/// pBR322's digest, from `tests/fixtures/genbank/pbr322-record.tsv`.
const PBR322_DIGEST: &str = "536042a7485bdabdf49c4d21a306213a098d77d19bdba57a1c6adc76ce2b8372";

/// `synthetic-linear.gb`'s digest, from
/// `tests/fixtures/genbank/synthetic-linear-record.tsv`.
const SYNLIN_DIGEST: &str = "a8b4ddcfa617179143d428dc103aabb7b13cf27f7b323407d11db8a01105bbd7";

/// A fresh directory for one test, as `tests/review_view.rs`'s `workspace` is.
fn workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("review-text-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// The design block `tests/end_to_end.rs`'s `design()` states: the same bound
/// and buffer, so the run is the one that fixture pins.
fn design() -> Value {
    json!({
        "policy": "luriaNative",
        "style": "q5_polymerase",
        "minAnnealingOverlap": 15,
        "placement": "inline",
        "buffer": "q5_reaction"
    })
}

/// The two-variant set: `variants-separate.json` beside a copy of
/// `synthetic-linear.gb`. `state` `verified` and two members are pinned here,
/// as 7a's test and `tests/fixtures/nl/README.md` pin them.
fn set(test: &str) -> ResultDocument {
    let dir = workspace(test);
    fs::copy(
        "../../tests/fixtures/genbank/synthetic-linear.gb",
        dir.join("synthetic-linear.gb"),
    )
    .unwrap();
    fs::copy(
        "../../tests/fixtures/nl/request/variants-separate.json",
        dir.join("request.json"),
    )
    .unwrap();
    let document: ResultDocument = run(&dir.join("request.json")).document;
    assert_eq!(document.state, "verified");
    assert_eq!(document.results.len(), 2);
    document
}

/// The brief's fail-closed example, as `tests/end_to_end.rs`'s
/// `the_fail_closed_example_asks_and_edits_nothing` runs it: a residue number
/// with no CDS named, on a template annotated with two, so the run asks and
/// edits nothing (D005).
fn clarification(test: &str) -> ResultDocument {
    let dir = workspace(test);
    fs::copy(
        "../../tests/fixtures/genbank/raw/J01749.1.gb",
        dir.join("J01749.1.gb"),
    )
    .unwrap();
    let path = dir.join("request.json");
    fs::write(
        &path,
        json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": {
                "id": "pbr322",
                "genbank": "J01749.1.gb",
                "sha256": PBR322_DIGEST,
                "cds": ["tet", "bla"]
            },
            "edit": {
                "name": "G42",
                "residue": { "number": 42, "from": "A", "to": "G", "policy": "default" }
            },
            "design": design()
        })
        .to_string(),
    )
    .unwrap();
    let document: ResultDocument = run(&path).document;
    assert_eq!(document.state, "needs_clarification");
    assert_eq!(document.results.len(), 0);
    document
}

/// pBR322 `tet`, `T99C`, the GenBank request of `tests/end_to_end.rs`'s
/// `a_fasta_and_a_genbank_template_give_the_same_result`: a verified edit that
/// no primer pair can be designed for, so its one member carries a
/// `designFailure` (G7, D029).
fn design_failure(test: &str) -> ResultDocument {
    let dir = workspace(test);
    fs::copy(
        "../../tests/fixtures/genbank/raw/J01749.1.gb",
        dir.join("J01749.1.gb"),
    )
    .unwrap();
    let path = dir.join("request.json");
    fs::write(
        &path,
        json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": {
                "id": "pbr322",
                "genbank": "J01749.1.gb",
                "sha256": PBR322_DIGEST,
                "cds": ["tet"]
            },
            "scope": "tet",
            "edit": {
                "name": "T99C",
                "substitution": { "interval": [99, 100], "from": "T", "to": "C" }
            },
            "design": design()
        })
        .to_string(),
    )
    .unwrap();
    let document: ResultDocument = run(&path).document;
    assert_eq!(document.state, "verified");
    assert_eq!(document.results.len(), 1);
    assert!(document.results[0].design.is_none());
    assert!(document.results[0].design_failure.is_some());
    document
}

/// The document of a request file that is not in the directory: `failed`, with
/// no member and no design to show.
fn failed(test: &str) -> ResultDocument {
    let dir = workspace(test);
    let document: ResultDocument = run(&dir.join("no-such-request.json")).document;
    assert_eq!(document.state, "failed");
    document
}

/// The written codon a member's own residue mutation holds, read from the
/// document.
fn written_codon(mutation: &MutationView) -> String {
    match mutation {
        MutationView::Residue { written_codon, .. } => written_codon.clone(),
        other => panic!("a residue mutation: {:?}", other),
    }
}

/// The edited construct a mutation carries, which every kind has.
fn edited_of(mutation: &MutationView) -> &EditedView {
    match mutation {
        MutationView::Residue { edited, .. }
        | MutationView::Substitution { edited, .. }
        | MutationView::Deletion { edited, .. }
        | MutationView::Insertion { edited, .. }
        | MutationView::Replacement { edited, .. } => edited,
    }
}

/// A member's `designFailure` code, as the document holds it.
fn design_failure_code(document: &ResultDocument) -> &'static str {
    document.results[0]
        .design_failure
        .as_ref()
        .expect("a design failure")
        .code
}

/// The half-open interval `[start, end)` the screen writes for `span`, built
/// from the span the document holds, so an expected line names it.
fn span_text(span: (u64, u64)) -> String {
    format!("[{}, {})", span.0, span.1)
}

/// The same interval as M7-24 says the overview and the member screen write it:
/// 1-based inclusive, so `[42, 345)` is `43–345` and one base is one number.
fn bases_at_text(span: (u64, u64)) -> String {
    if span.1 - span.0 == 1 {
        format!("{}", span.0 + 1)
    } else {
        format!("{}\u{2013}{}", span.0 + 1, span.1)
    }
}

/// A member's `mutation` as M7-22 says a residue row reads it: the residues the
/// document holds, then its codons in brackets. Every value is the document's
/// own field.
fn residue_summary(mutation: &MutationView) -> String {
    match mutation {
        MutationView::Residue {
            original_residue,
            residue,
            written_residue,
            original_codon,
            written_codon,
            ..
        } => format!(
            "{}{}{} ({}\u{2192}{})",
            original_residue, residue, written_residue, original_codon, written_codon
        ),
        other => panic!("the set's members are residue edits: {:?}", other),
    }
}

/// The character column `needle` starts at in `line`, so two rows' columns can
/// be shown to line up (M7-26).
fn start_column(line: &str, needle: &str) -> usize {
    let chars: Vec<char> = line.chars().collect();
    let target: Vec<char> = needle.chars().collect();
    chars
        .windows(target.len())
        .position(|window| window == target.as_slice())
        .unwrap_or_else(|| panic!("{:?} holds {}", line, needle))
}

/// The design of member 0, which the set's members have.
fn design_of(document: &ResultDocument) -> &DesignView {
    document.results[0].design.as_ref().expect("a design")
}

/// The member rows: the marker (`\u{25b6} ` for the selected row, two spaces for
/// the others) is the only thing that makes a row recognizable, so the lines with
/// one are the rows.
fn member_rows(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .filter(|line| line.starts_with("\u{25b6} ") || line.starts_with("  "))
        .cloned()
        .collect()
}

/// How many times `expected` appears in `lines`, so "one line per entry" is
/// tested as one, not at least one.
fn count(lines: &[String], expected: &str) -> usize {
    lines.iter().filter(|line| *line == expected).count()
}

#[test]
fn the_overview_names_the_template_and_target() {
    let document: ResultDocument = set("overview");
    let template = document
        .provenance
        .template
        .as_ref()
        .expect("the set names its template");
    let target = document.target.as_ref().expect("the set has a target");
    let lines: Vec<String> = text::overview(&document, 0);
    // M7-25, M7-26: the ids padded to the longer of the two, the strand `+` as
    // `forward`, the target's interval 1-based inclusive. `synlin`, `linear`,
    // 393 bp and `synA` with CDS `[42, 345)` are facts of
    // `tests/fixtures/genbank/synthetic-linear-record.tsv` and
    // `synthetic-linear-cds.tsv`.
    let width: usize = template.id.chars().count().max(target.id.chars().count());
    assert_eq!(
        lines[0],
        format!(
            "Template  {:width$}  {}, {} bp",
            template.id,
            template.topology,
            template.length,
            width = width
        ),
        "the template line: {:?}",
        lines
    );
    assert_eq!(
        lines[1],
        format!(
            "Target    {:width$}  {} strand, bases {}",
            target.id,
            if target.strand == "+" {
                "forward"
            } else {
                "reverse"
            },
            bases_at_text(target.intervals[0]),
            width = width
        ),
        "the target line, in words: {:?}",
        lines
    );
    // 7i4 (plan §5.3a M7-22): the operation is not on the overview, which names
    // the template, the target and the members only.
    assert_eq!(lines[2], "", "a blank between the heading and the rows");
    assert_eq!(
        lines[3],
        format!("Variants ({})", document.results.len()),
        "the members are counted in words: {:?}",
        lines
    );
}

#[test]
fn the_overview_reads_in_words() {
    let document: ResultDocument = set("words");
    let lines: Vec<String> = text::overview(&document, 0);
    // M7-25, M7-26: the heading reads in words, with the template's and the
    // target's ids padded to the longer, the `+` strand as `forward`, and the
    // CDS interval as 1-based inclusive bases: `synlin` (393 bp, `linear`) and
    // `synA` with CDS `[42, 345)` are facts of `tests/fixtures/genbank/
    // synthetic-linear-record.tsv` and `synthetic-linear-cds.tsv`.
    let heading: Vec<String> = vec![
        "Template  synlin  linear, 393 bp".to_string(),
        "Target    synA    forward strand, bases 43\u{2013}345".to_string(),
        String::new(),
        "Variants (2)".to_string(),
    ];
    assert_eq!(
        &lines[..heading.len()],
        &heading[..],
        "the heading is the document's own values in the new words: {:?}",
        lines
    );
    // The rows, with every value the document's own field and only the marker,
    // the padding and the words typed here.
    let summaries: Vec<String> = document
        .results
        .iter()
        .map(|member| residue_summary(&member.mutation))
        .collect();
    let name_width: usize = document
        .results
        .iter()
        .map(|member| member.name.chars().count())
        .max()
        .expect("the set has members");
    let summary_width: usize = summaries
        .iter()
        .map(|summary| summary.chars().count())
        .max()
        .expect("the set has members");
    for (index, summary) in summaries.iter().enumerate() {
        let marker: &str = if index == 0 { "\u{25b6} " } else { "  " };
        assert_eq!(
            lines[heading.len() + index],
            format!(
                "{marker}{:name$}  {:sum$}  designed",
                document.results[index].name,
                summary,
                name = name_width,
                sum = summary_width,
                marker = marker
            ),
            "row {} reads in words: {:?}",
            index,
            lines
        );
    }
}

#[test]
fn rows_line_up() {
    let document: ResultDocument = set("line_up");
    let lines: Vec<String> = text::overview(&document, 1);
    let rows: Vec<String> = member_rows(&lines);
    assert_eq!(rows.len(), 2, "one row per member: {:?}", lines);
    let summaries: Vec<String> = document
        .results
        .iter()
        .map(|member| residue_summary(&member.mutation))
        .collect();
    let columns: Vec<usize> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| start_column(row, &summaries[index]))
        .collect();
    assert_eq!(
        columns[0], columns[1],
        "the name is padded to the longest, so both summaries start in the same column: {:?}",
        rows
    );
    let outcomes: Vec<usize> = rows
        .iter()
        .map(|row| start_column(row, "designed"))
        .collect();
    assert_eq!(
        outcomes[0], outcomes[1],
        "and the summary is padded to the longest, so both outcomes start in the same column: {:?}",
        rows
    );
}

#[test]
fn the_help_holds_the_legend() {
    let lines: Vec<String> = text::help(Screen::Member(0), false, false);
    assert!(
        lines.iter().any(|line| line.starts_with("  protein")),
        "the help explains the protein line: {:?}",
        lines
    );
    assert!(
        lines.iter().any(|line| line.starts_with("  lower case")),
        "the help explains what lower case means: {:?}",
        lines
    );
}

#[test]
fn help_names_only_the_open_screens_keys() {
    // M7-53: `?` shows the open screen's own keys, and the sequence legend is on
    // the variant screen only.
    fn has_key(lines: &[String], key: &str) -> bool {
        let prefix: String = format!("  {:width$}", key, width = 11);
        lines.iter().any(|line| line.starts_with(&prefix))
    }

    let legend: [&str; 9] = [
        "The sequence",
        "  forward",
        "  template",
        "  protein",
        "  bottom",
        "  reverse",
        "  lower case",
        "  *",
        "  [+N bp",
    ];
    for (screen, problem, expandable) in [
        (Screen::Overview, false, false),
        (Screen::Overview, true, false),
        (Screen::Member(0), false, false),
        (Screen::Member(0), true, true),
        (Screen::Details(0), true, false),
        (Screen::Problem, true, false),
        (Screen::Export, true, false),
    ] {
        let lines: Vec<String> = text::help(screen, problem, expandable);
        let legend_on_member: bool = screen == Screen::Member(0);
        for head in legend {
            let count: usize = lines.iter().filter(|line| line.starts_with(head)).count();
            assert_eq!(
                count,
                usize::from(legend_on_member),
                "the legend is on Member only: {:?} -> {:?}",
                screen,
                lines
            );
        }
        // `x` is an overview key, `w o` an export key, and each names the screen
        // it works on, never another.
        assert_eq!(
            has_key(&lines, "x"),
            screen == Screen::Overview,
            "`x` is on Overview only: {:?} -> {:?}",
            screen,
            lines
        );
        assert_eq!(
            has_key(&lines, "w o"),
            screen == Screen::Export,
            "`w o` is on Export only: {:?} -> {:?}",
            screen,
            lines
        );
        assert_eq!(
            has_key(&lines, "e"),
            screen == Screen::Overview && problem,
            "`e` appears only when there is a problem, on Overview: {:?} -> {:?}",
            screen,
            lines
        );
        assert_eq!(
            has_key(&lines, "i"),
            screen == Screen::Member(0) && expandable,
            "`i` appears only where `i` expands: {:?} -> {:?}",
            screen,
            lines
        );
        assert_eq!(
            has_key(&lines, "d"),
            screen == Screen::Member(0),
            "`d` is on the variant screen only: {:?} -> {:?}",
            screen,
            lines
        );
        assert_eq!(
            has_key(&lines, "Home"),
            screen == Screen::Details(0) || screen == Screen::Problem,
            "`Home` is on the scrolled screens that offer it: {:?} -> {:?}",
            screen,
            lines
        );
        // `ctrl-c` is the last line on every screen.
        assert_eq!(
            lines.last().map(String::as_str),
            Some("  ctrl-c     quit"),
            "`ctrl-c` is the last line: {:?} -> {:?}",
            screen,
            lines
        );
    }
}

#[test]
fn each_member_has_one_row_and_one_marker() {
    let document = set("rows");
    let lines = text::overview(&document, 1);
    let rows = member_rows(&lines);
    assert_eq!(rows.len(), 2, "one row per member: {:?}", lines);

    let marked: Vec<&String> = rows
        .iter()
        .filter(|row| row.starts_with("\u{25b6} "))
        .collect();
    assert_eq!(marked.len(), 1, "exactly one marker: {:?}", rows);
    assert_eq!(
        *marked[0], rows[1],
        "the marker is on the member at `selected`"
    );

    // M7-26: the row is the marker, the name padded to the longest name, the
    // summary padded to the longest summary and the outcome; every value is the
    // document's own field.
    let summaries: Vec<String> = document
        .results
        .iter()
        .map(|member| residue_summary(&member.mutation))
        .collect();
    let name_width: usize = document
        .results
        .iter()
        .map(|member| member.name.chars().count())
        .max()
        .expect("the set has members");
    let summary_width: usize = summaries
        .iter()
        .map(|summary| summary.chars().count())
        .max()
        .expect("the set has members");
    for (index, summary) in summaries.iter().enumerate() {
        let marker: &str = if index == 1 { "\u{25b6} " } else { "  " };
        assert_eq!(
            rows[index],
            format!(
                "{marker}{:name$}  {:sum$}  designed",
                document.results[index].name,
                summary,
                name = name_width,
                sum = summary_width,
                marker = marker
            ),
            "row {} is padded to line up: {:?}",
            index,
            rows
        );
    }
    assert!(
        rows[0].ends_with("designed"),
        "the row ends with its outcome, and its warnings are not counted on the row: {:?}",
        rows[0]
    );
}

#[test]
fn a_design_failure_is_on_the_overview_row() {
    let document = design_failure("failure_row");
    let rows: Vec<String> = member_rows(&text::overview(&document, 0));
    assert_eq!(rows.len(), 1, "one row for the one member: {:?}", rows);
    // M7-26: a design failure reads `no primer pair`; the name, the bases and the
    // edited bases are the document's own fields. `T99C` and its interval
    // `[99, 100)` are the request this helper writes.
    let (original, written, span) = match &document.results[0].mutation {
        MutationView::Substitution {
            original,
            written,
            interval,
            ..
        } => (original.clone(), written.clone(), *interval),
        other => panic!("the failure's member is a substitution: {:?}", other),
    };
    assert_eq!(
        rows[0],
        format!(
            "\u{25b6} {}  {}\u{2192}{} at {}  no primer pair",
            document.results[0].name,
            original,
            written,
            bases_at_text(span)
        ),
        "the row states the failure in words: {:?}",
        rows
    );
    assert!(
        !rows[0].contains("designed"),
        "a design failure is shown as a failure, never as a design: {:?}",
        rows[0]
    );
    assert!(
        !rows[0].contains(design_failure_code(&document)),
        "the row shows the words, not the document's failure code: {:?}",
        rows[0]
    );
}

#[test]
fn the_overview_body_never_holds_the_problem_line() {
    // 7i4 (plan §5.3a M7-22): warnings and provenance are behind keys, so the
    // overview's body carries neither the problem line nor the provenance line,
    // with a problem or without one. `the_key_line_offers_problem_only_with_a_problem`
    // (7c3) is where the key that opens the problem screen is shown.
    for (test, lines) in [
        ("set", text::overview(&set("overview_body_set"), 0)),
        (
            "clarification",
            text::overview(&clarification("overview_body_clarification"), 0),
        ),
        ("failed", text::overview(&failed("overview_body_failed"), 0)),
    ] {
        assert!(
            !lines.iter().any(|line| line.as_str() == "e problem"),
            "{}: the overview body holds no problem line: {:?}",
            test,
            lines
        );
        assert!(
            !lines.iter().any(|line| line.as_str() == "p provenance"),
            "{}: the overview body holds no provenance line: {:?}",
            test,
            lines
        );
    }
}

#[test]
fn the_problem_screen_lists_the_clarification_and_candidates() {
    let document = clarification("clarification_lines");
    let lines = text::problem(&document);
    let expected: Vec<String> = vec![
        "clarifications:".to_string(),
        format!("  {}", document.clarifications[0]),
        "candidates:".to_string(),
        "  tet".to_string(),
        "  bla".to_string(),
    ];
    let positions: Vec<usize> = expected
        .iter()
        .map(|line| {
            lines
                .iter()
                .position(|row| *row == *line)
                .unwrap_or_else(|| panic!("{} is listed: {:?}", line, lines))
        })
        .collect();
    for pair in positions.windows(2) {
        assert!(
            pair[0] < pair[1],
            "the groups come in this order: {:?}",
            lines
        );
    }
    for line in &expected {
        assert_eq!(count(&lines, line), 1, "{:?} is listed once", lines);
    }
}

#[test]
fn the_problem_screen_shows_the_error() {
    let document = failed("failed_error");
    let error = document.error.as_ref().expect("an error");
    let lines = text::problem(&document);
    assert_eq!(lines[0], format!("error: {}", error.code));
    assert_eq!(lines[1], "category: malformed_input");
    assert_eq!(lines[2], format!("message: {}", error.message));
}

#[test]
fn the_provenance_screen_shows_the_record() {
    let document = set("provenance");
    let lines = text::provenance(&document);

    for expected in [
        "template: synlin".to_string(),
        format!("  sha256: {}", SYNLIN_DIGEST),
        "  length: 393 bp".to_string(),
        "  topology: linear".to_string(),
        "  locus: synlin".to_string(),
        "  accession: SYN0002".to_string(),
        "  version: SYN0002.1".to_string(),
    ] {
        assert_eq!(
            count(&lines, &expected),
            1,
            "{} is on the screen once",
            expected
        );
    }

    // The imported CDS, as `tests/fixtures/genbank/synthetic-linear-cds.tsv`
    // records it, with the line number the document itself holds.
    let cds = document
        .provenance
        .template
        .as_ref()
        .expect("a template")
        .genbank
        .as_ref()
        .expect("a GenBank import")
        .imported_cds[0]
        .clone();
    assert_eq!(
        count(
            &lines,
            &format!("    synA  +  [42, 345)  from /gene  line {}", cds.line)
        ),
        1,
        "{:?}",
        lines
    );

    // One line per algorithm the document names, from its own fields.
    for algorithm in &document.provenance.algorithms {
        assert_eq!(
            count(
                &lines,
                &format!(
                    "  {}: {} {}",
                    algorithm.role, algorithm.name, algorithm.version
                )
            ),
            1,
            "{} is named once",
            algorithm.role
        );
    }
    assert_eq!(count(&lines, "algorithms:"), 1);
}

#[test]
fn the_member_screen_names_the_edit() {
    let document = set("member_screen");
    let member = &document.results[0];
    let lines = text::member(&document, 0);

    assert_eq!(
        count(&lines, &format!("member: {}", member.name)),
        1,
        "the screen names the member: {:?}",
        lines
    );
    // `synA`, residue 42 being `A` and its codon `GCA` are facts of
    // `tests/fixtures/genbank/synthetic-linear-cds.tsv` and
    // `synthetic-linear-residues.tsv`; the written codon and the edited
    // construct are the document's own fields.
    // M7-22: the summary is the residues and the codons the document holds; `GCA`
    // is codon 42 of `tests/fixtures/genbank/synthetic-linear-residues.tsv`.
    let (original_residue, residue, written_residue) = match &member.mutation {
        MutationView::Residue {
            original_residue,
            residue,
            written_residue,
            ..
        } => (original_residue.clone(), *residue, written_residue.clone()),
        other => panic!("the member's mutation is a residue: {:?}", other),
    };
    assert_eq!(
        count(
            &lines,
            &format!(
                "mutation: {}{}{} (GCA\u{2192}{})",
                original_residue,
                residue,
                written_residue,
                written_codon(&member.mutation)
            )
        ),
        1,
        "the mutation line is the member's own edit: {:?}",
        lines
    );
    let edited = edited_of(&member.mutation);
    for expected in [
        "  state: verified".to_string(),
        "  cds: synA".to_string(),
        "  residue: 42".to_string(),
        // codon 42 is the half-open interval [165, 168) of the residues table.
        "  coordinates: 165, 166, 167".to_string(),
        format!(
            "  edited: {}, {} bp, sha256 {}",
            edited.id, edited.length, edited.sha256
        ),
    ] {
        assert_eq!(
            count(&lines, &expected),
            1,
            "{} is on the screen once",
            expected
        );
    }

    // The design the document carries is what used to make `c context` worth
    // offering; M7-56 takes both stale lines out of `member`, so the screen's
    // lines are the record only.
    assert!(member.design.is_some(), "the set's member 0 has a design");
    for stale in ["c context", "esc back"] {
        assert!(
            !lines.iter().any(|line| line.as_str() == stale),
            "`{}` is not a line of `member`: {:?}",
            stale,
            lines
        );
    }
}

#[test]
fn each_primer_is_written_five_to_three() {
    let document = set("primer_written");
    let design = design_of(&document);
    let lines = text::member(&document, 0);
    assert_eq!(
        (design.forward.role.as_str(), design.reverse.role.as_str()),
        ("forward", "reverse"),
        "the document names the two primers"
    );

    for primer in [&design.forward, &design.reverse] {
        let mut expected = format!(
            "{} 5'-{}-3'  {} nt  span {}",
            primer.role,
            primer.sequence,
            primer.length,
            span_text(primer.span)
        );
        if let Some(edited_span) = primer.edited_span {
            expected.push_str(&format!("  edited span {}", span_text(edited_span)));
        }
        assert_eq!(
            count(&lines, &expected),
            1,
            "one line, holding the primer's own text, written 5' to 3', with its own length and span: {:?}",
            lines
        );
    }
}

#[test]
fn each_part_is_listed_in_primer_order() {
    let document = set("parts_order");
    let design = design_of(&document);
    let lines = text::member(&document, 0);
    assert!(
        !design.forward.parts.is_empty() && !design.reverse.parts.is_empty(),
        "both primers carry parts in the document"
    );
    assert_eq!(
        count(&lines, "  parts:"),
        2,
        "one header per primer: {:?}",
        lines
    );

    let mut positions: Vec<usize> = Vec::new();
    for part in &design.forward.parts {
        let mut expected = format!(
            "    {}-{}  {} {}  {}",
            part.primer_start,
            part.primer_end,
            part.role,
            if part.annealing {
                "annealing"
            } else {
                "not annealing"
            },
            part.sequence
        );
        if let Some(span) = part.span {
            expected.push_str(&format!("  {}", span_text(span)));
        }
        let position = lines
            .iter()
            .position(|line| *line == expected)
            .unwrap_or_else(|| {
                panic!(
                    "the part's own line is listed: {:?} vs {:?}",
                    expected, lines
                )
            });
        assert_eq!(
            count(&lines, &expected),
            1,
            "one line per part: {:?}",
            lines
        );
        positions.push(position);
    }
    for pair in positions.windows(2) {
        assert!(
            pair[0] < pair[1],
            "the forward primer's parts come in the order the document holds them: {:?}",
            lines
        );
    }
}

#[test]
fn the_tm_and_its_basis_are_shown() {
    let document = set("tm_shown");
    let forward = &design_of(&document).forward;
    let lines = text::member(&document, 0);

    let tm_line = format!("  tm: {} C, {}", forward.tm.celsius, forward.tm.basis);
    let position = lines
        .iter()
        .position(|line| *line == tm_line)
        .unwrap_or_else(|| {
            panic!(
                "the Tm line holds the document's own value and basis: {:?}",
                lines
            )
        });
    assert_eq!(count(&lines, &tm_line), 1, "{:?}", lines);

    // Checked in the document, not assumed: the set's forward Tm of member 0 is
    // a substituted basis (a tandem mismatch), so its reason must be shown.
    assert!(
        forward.tm.substituted_because.is_some(),
        "the forward Tm states why its basis was substituted: {:?}",
        forward.tm
    );
    assert_eq!(
        lines[position + 1],
        format!(
            "  tm substituted because: {}",
            forward
                .tm
                .substituted_because
                .as_ref()
                .expect("a substituted basis")
        ),
        "the line after the Tm shows why: {:?}",
        lines
    );
}

#[test]
fn a_design_failure_replaces_the_pair() {
    let document = design_failure("failure_screen");
    let failure = document.results[0]
        .design_failure
        .as_ref()
        .expect("a design failure");
    let lines = text::member(&document, 0);

    assert_eq!(
        count(&lines, &format!("design failed: {}", failure.code)),
        1,
        "the failure is named by the document's own code: {:?}",
        lines
    );
    assert_eq!(
        count(&lines, &format!("  {}", failure.message)),
        1,
        "with the document's own message: {:?}",
        lines
    );
    for (key, value) in &failure.details.0 {
        match value {
            DetailValue::Text(text) => assert_eq!(
                count(&lines, &format!("  {}: {}", key, text)),
                1,
                "the details are listed, one line per pair: {:?}",
                lines
            ),
            other => panic!("this design failure's details are text: {:?}", other),
        }
    }

    assert!(
        !lines.iter().any(|line| line.starts_with("forward 5'-")),
        "a member with no pair shows no primer: {:?}",
        lines
    );
    // M7-56: `member` names no key, so neither stale line is on it, design or
    // no design.
    for stale in ["c context", "esc back"] {
        assert!(
            !lines.iter().any(|line| line.as_str() == stale),
            "no context to read, and no key line on the member's text: {:?}",
            lines
        );
    }
}

#[test]
fn the_stale_lines_are_gone() {
    for (test, document) in [
        ("the set", set("back_set")),
        ("a design failure", design_failure("back_failure")),
    ] {
        let lines = text::member(&document, 0);
        // M7-56: `c context` and `esc back` are not lines of `member`.
        for stale in ["c context", "esc back"] {
            assert!(
                !lines.iter().any(|line| line.as_str() == stale),
                "{}'s member text holds no `{}`: {:?}",
                test,
                stale,
                lines
            );
        }
    }
}
