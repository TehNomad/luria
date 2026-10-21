//! The text of three result screens (milestone 7c1): the overview, the problem
//! screen and the provenance screen (plan §5.2 screens 2, 5 and 6). Each screen
//! is a list of plain text lines, one string per line, none holding a `\n`.
//!
//! Pure: no I/O and no `ratatui` or `crossterm` types, so every line is testable
//! as a string (plan §5.1). Every value on a line is one field of the
//! `ResultDocument`, written with `{}` (plan §3 rule 1): no coordinate, Tm,
//! codon or translation is computed, no float is rounded, and a digest is
//! shortened nowhere. The only formatting is intervals as `[start, end)`, a
//! float with `{}` (the same digits the JSON holds) and a code written as the
//! serde tag's variant name in snake case. An error, a clarification, a design
//! failure and a warning are always listed (plan §3 rule 3).
//!
//! Lines hold ASCII only: `->` for an arrow, no `→` or `°`. `view` draws these
//! lines (7c3). The state and the experimental validation are not here, because
//! 7a's banner draws them on every screen.

use crate::result::{
    DetailValue, Details, MutationView, ObstacleView, OperationWarning, Provenance, ResultDocument,
    SourceView,
};

/// One interval as `[start, end)`.
pub(crate) fn interval(interval: (u64, u64)) -> String {
    format!("[{}, {})", interval.0, interval.1)
}

/// A list of intervals, joined with `, `.
pub(crate) fn intervals(intervals: &[(u64, u64)]) -> String {
    intervals
        .iter()
        .map(|one| interval(*one))
        .collect::<Vec<String>>()
        .join(", ")
}

/// The code of an operation warning: its serde tag, the variant name in snake
/// case, written as a `match` rather than found by serializing.
pub(crate) fn operation_warning_code(warning: &OperationWarning) -> &'static str {
    match warning {
        OperationWarning::WildTypeSkipped { .. } => "wild_type_skipped",
        OperationWarning::ManyDesigns { .. } => "many_designs",
    }
}

/// An operation warning's own message.
fn operation_warning_message(warning: &OperationWarning) -> &str {
    match warning {
        OperationWarning::WildTypeSkipped { message, .. } => message.as_str(),
        OperationWarning::ManyDesigns { message, .. } => message.as_str(),
    }
}

/// The code of an obstacle: its serde tag, the variant name in snake case.
fn obstacle_code(obstacle: &ObstacleView) -> &'static str {
    match obstacle {
        ObstacleView::FivePrimeAvailability { .. } => "five_prime_availability",
        ObstacleView::ThreePrimeAvailability { .. } => "three_prime_availability",
        ObstacleView::AmbiguousTemplateBase { .. } => "ambiguous_template_base",
        ObstacleView::FivePrimeTmFloorUnreachable { .. } => "five_prime_tm_floor_unreachable",
        ObstacleView::ThreePrimeTmFloorUnreachable { .. } => "three_prime_tm_floor_unreachable",
    }
}

/// An obstacle's own message.
fn obstacle_message(obstacle: &ObstacleView) -> &str {
    match obstacle {
        ObstacleView::FivePrimeAvailability { message, .. } => message.as_str(),
        ObstacleView::ThreePrimeAvailability { message, .. } => message.as_str(),
        ObstacleView::AmbiguousTemplateBase { message, .. } => message.as_str(),
        ObstacleView::FivePrimeTmFloorUnreachable { message, .. } => message.as_str(),
        ObstacleView::ThreePrimeTmFloorUnreachable { message, .. } => message.as_str(),
    }
}

/// The mutation one member performed, as one phrase: for a residue, its CDS,
/// the residue number between the two residues, and the two codons; for a
/// substitution or replacement, its interval and the two letters; for a
/// deletion, its interval, `del` and the bases removed; for an insertion, the
/// bases written and the point. Every value is the member's own.
pub(crate) fn mutation_summary(mutation: &MutationView) -> String {
    match mutation {
        MutationView::Residue {
            cds,
            residue,
            original_residue,
            written_residue,
            original_codon,
            written_codon,
            ..
        } => format!(
            "{} {}{}{} {}->{}",
            cds, original_residue, residue, written_residue, original_codon, written_codon
        ),
        MutationView::Substitution {
            interval: span,
            original,
            written,
            ..
        }
        | MutationView::Replacement {
            interval: span,
            original,
            written,
            ..
        } => format!("{} {}->{}", interval(*span), original, written),
        MutationView::Deletion {
            interval: span,
            original,
            ..
        } => format!("{} del {}", interval(*span), original),
        MutationView::Insertion { point, written, .. } => format!("ins {} at {}", written, point),
    }
}

/// An error's details as lines, one per pair, `<indent><key>: <value>`: text as
/// itself, a number as itself, an absent value as `none`, and obstacles as the
/// word `obstacles` then one line per obstacle, `<indent>  <code>: <message>`.
pub(crate) fn details_lines(details: &Details, indent: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for (key, value) in &details.0 {
        match value {
            DetailValue::Text(text) => lines.push(format!("{}{}: {}", indent, key, text)),
            DetailValue::Count(count) => lines.push(format!("{}{}: {}", indent, key, count)),
            DetailValue::Signed(signed) => lines.push(format!("{}{}: {}", indent, key, signed)),
            DetailValue::Absent => lines.push(format!("{}{}: none", indent, key)),
            DetailValue::Obstacles(obstacles) => {
                lines.push(format!("{}{}: obstacles", indent, key));
                for obstacle in obstacles {
                    lines.push(format!(
                        "{}  {}: {}",
                        indent,
                        obstacle_code(obstacle),
                        obstacle_message(obstacle)
                    ));
                }
            }
        }
    }
    lines
}

/// The overview (plan §5.2 screen 2): the template, the target, the operation,
/// the operation's warnings, one row per member with the member at `selected`
/// marked `> `, then the keys that open the screens below. `member` rows keep
/// the order `results` holds. Nothing here is computed: a row's outcome is that
/// member's `design` or the code of its `design_failure`, and `members` is the
/// number of rows.
pub fn overview(document: &ResultDocument, selected: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    match &document.provenance.template {
        Some(template) => lines.push(format!(
            "template: {}, {}, {} bp",
            template.id, template.topology, template.length
        )),
        None => lines.push("template: none".to_string()),
    }

    match &document.target {
        Some(target) => lines.push(format!(
            "target: {}, {}, {}",
            target.id,
            target.strand,
            intervals(&target.intervals)
        )),
        None => lines.push("target: none".to_string()),
    }

    match document.provenance.operation {
        Some(operation) => lines.push(format!("operation: {}", operation)),
        None => lines.push("operation: none".to_string()),
    }

    if !document.warnings.is_empty() {
        lines.push("warnings:".to_string());
        for warning in &document.warnings {
            lines.push(format!(
                "  {}: {}",
                operation_warning_code(warning),
                operation_warning_message(warning)
            ));
        }
    }

    lines.push(format!("members: {}", document.results.len()));
    for (index, member) in document.results.iter().enumerate() {
        let marker: &str = if index == selected { "> " } else { "  " };
        // A member states one design, a design failure, or, when the run's
        // invariant is broken, neither: then the row says `no design` rather
        // than hiding the missing design (plan §3 rule 3).
        let outcome: String = match (&member.design, &member.design_failure) {
            (Some(_), _) => "designed".to_string(),
            (None, Some(problem)) => format!("design failed: {}", problem.code),
            (None, None) => "no design".to_string(),
        };
        lines.push(format!(
            "{}{}  {}  {}  warnings: {}",
            marker,
            member.name,
            mutation_summary(&member.mutation),
            outcome,
            member.warnings.len()
        ));
    }

    if document.error.is_some() || !document.clarifications.is_empty() {
        lines.push(String::new());
        lines.push("e problem".to_string());
    }
    lines.push(String::new());
    lines.push("p provenance".to_string());
    lines
}

/// The problem screen (plan §5.2 screen 6): the error with its details, the
/// clarifications and the candidates, a blank line between the groups, and the
/// single line `no problem` when there is none of the three. Every line is the
/// document's own text; nothing is summarised away (plan §3 rule 3).
pub fn problem(document: &ResultDocument) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    if let Some(error) = &document.error {
        lines.push(format!("error: {}", error.code));
        lines.push(format!("category: {}", error.category));
        lines.push(format!("message: {}", error.message));
        if let Some(rule) = error.rule {
            lines.push(format!("rule: {}", rule));
        }
        if let Some(path) = &error.path {
            lines.push(format!("path: {}", path));
        }
        if !error.details.0.is_empty() {
            lines.push("details:".to_string());
            lines.extend(details_lines(&error.details, "  "));
        }
    }

    if !document.clarifications.is_empty() {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push("clarifications:".to_string());
        for clarification in &document.clarifications {
            lines.push(format!("  {}", clarification));
        }
    }

    if !document.candidates.is_empty() {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push("candidates:".to_string());
        for candidate in &document.candidates {
            lines.push(format!("  {}", candidate));
        }
    }

    if lines.is_empty() {
        lines.push("no problem".to_string());
    }
    lines
}

/// The provenance screen (plan §5.2 screen 5): the package, the algorithms, the
/// request digest, the operation, the template with where it came from and what
/// a GenBank record imported and did not, and the resolved buffer. The template
/// digest is written in full. Floats are written with `{}`, the digits the
/// document holds.
pub fn provenance(document: &ResultDocument) -> Vec<String> {
    let source: &Provenance = &document.provenance;
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!(
        "package: {} {}",
        source.package.name, source.package.version
    ));
    lines.push("algorithms:".to_string());
    for algorithm in &source.algorithms {
        lines.push(format!(
            "  {}: {} {}",
            algorithm.role, algorithm.name, algorithm.version
        ));
    }

    match &source.request_sha256 {
        Some(digest) => lines.push(format!("request sha256: {}", digest)),
        None => lines.push("request sha256: none".to_string()),
    }

    match source.operation {
        Some(operation) => lines.push(format!("operation: {}", operation)),
        None => lines.push("operation: none".to_string()),
    }

    match &source.template {
        Some(template) => {
            lines.push(format!("template: {}", template.id));
            lines.push(format!("  sha256: {}", template.sha256));
            lines.push(format!("  length: {} bp", template.length));
            lines.push(format!("  topology: {}", template.topology));
            match &template.source {
                SourceView::Fasta { path, header } => {
                    lines.push(format!("  source: fasta {}", path));
                    lines.push(format!("  header: {}", header));
                }
                SourceView::Genbank {
                    path,
                    locus_name,
                    accession,
                    version,
                } => {
                    lines.push(format!("  source: genbank {}", path));
                    lines.push(format!("  locus: {}", locus_name));
                    lines.push(format!(
                        "  accession: {}",
                        accession.as_deref().unwrap_or("none")
                    ));
                    lines.push(format!(
                        "  version: {}",
                        version.as_deref().unwrap_or("none")
                    ));
                }
            }
            if let Some(genbank) = &template.genbank {
                lines.push("  imported cds:".to_string());
                for cds in &genbank.imported_cds {
                    lines.push(format!(
                        "    {}  {}  {}  from /{}  line {}",
                        cds.id,
                        cds.strand,
                        intervals(&cds.intervals),
                        cds.id_qualifier,
                        cds.line
                    ));
                }
                lines.push("  unused features:".to_string());
                for feature in &genbank.unused_features {
                    // One line per feature: its key and line, then its `id` and
                    // its refusal's code, each added when the document holds it.
                    let mut line: String = format!("    {}  line {}", feature.key, feature.line);
                    if let Some(id) = &feature.id {
                        line.push_str(&format!("  {}", id));
                    }
                    if let Some(refusal) = &feature.refusal {
                        line.push_str(&format!("  refused: {}", refusal.code));
                    }
                    lines.push(line);
                }
            }
        }
        None => lines.push("template: none".to_string()),
    }

    match &source.buffer {
        Some(buffer) => {
            match buffer.preset {
                Some(preset) => lines.push(format!("buffer: {}", preset)),
                None => lines.push("buffer: stated".to_string()),
            }
            lines.push(format!("  monovalent: {} mM", buffer.monovalent_millimolar));
            lines.push(format!("  primer: {} nM", buffer.primer_nanomolar));
            lines.push(format!("  dmso: {} %", buffer.dmso_percent));
        }
        None => lines.push("buffer: none".to_string()),
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::{
        BufferView, EditedView, GenBankView, MemberView, PackageView, RefusalView, TemplateView,
        UnusedFeatureView,
    };

    /// The provenance a hand-built document needs: the package that ran, and
    /// nothing else, so a test sets the one field it is about.
    fn a_provenance() -> Provenance {
        Provenance {
            package: PackageView {
                name: env!("CARGO_PKG_NAME"),
                version: env!("CARGO_PKG_VERSION"),
            },
            algorithms: Vec::new(),
            request_sha256: None,
            operation: None,
            template: None,
            buffer: None,
        }
    }

    /// A document, for the lines to be built from: no target, no member, no
    /// warning, no error, no clarification, no candidate.
    fn document() -> ResultDocument {
        ResultDocument {
            schema: crate::RESULT_SCHEMA,
            state: "verified",
            experimental_validation: "not_performed",
            provenance: a_provenance(),
            request: None,
            target: None,
            clarifications: Vec::new(),
            candidates: Vec::new(),
            warnings: Vec::new(),
            results: Vec::new(),
            error: None,
        }
    }

    /// A member with no design and no design failure, so the row's third state
    /// is reachable: `run.rs` reports that member when core's design outcome is
    /// neither a design nor a failure. The mutation is one the run of
    /// `edit-a42g.json` records, taken from the document, not a scientific
    /// claim of this test.
    fn member(name: &str) -> MemberView {
        MemberView {
            name: name.to_string(),
            mutation: MutationView::Substitution {
                state: "verified".to_string(),
                interval: (165, 168),
                original: "GCA".to_string(),
                written: "GGC".to_string(),
                coordinates: vec![166],
                edited: EditedView {
                    id: "synlin-A42G".to_string(),
                    sha256: "0".to_string(),
                    length: 393,
                },
            },
            warnings: Vec::new(),
            design: None,
            design_failure: None,
        }
    }

    /// The document with `n` members, each with no design and no failure.
    fn document_with(n: usize) -> ResultDocument {
        let mut document: ResultDocument = document();
        document.results = (0..n).map(|_| member("m")).collect();
        document
    }

    /// A GenBank template whose `genbank` import holds `features` as its only
    /// unused features, so the unused-feature lines are the ones to check.
    fn genbank_document(features: Vec<UnusedFeatureView>) -> ResultDocument {
        let mut document: ResultDocument = document();
        document.provenance.template = Some(TemplateView {
            id: "synlin".to_string(),
            sha256: "a8b4ddcfa617179143d428dc103aabb7b13cf27f7b323407d11db8a01105bbd7".to_string(),
            length: 393,
            topology: "linear".to_string(),
            source: SourceView::Genbank {
                path: "synthetic-linear.gb".to_string(),
                locus_name: "synlin".to_string(),
                accession: None,
                version: None,
            },
            genbank: Some(GenBankView {
                imported_cds: Vec::new(),
                unused_features: features,
            }),
        });
        document
    }

    /// The row of member `index` in `lines`, which the marker and the name make
    /// identifiable.
    fn row(lines: &[String], index: usize) -> String {
        lines
            .iter()
            .filter(|line| line.starts_with("> ") || line.starts_with("  "))
            .nth(index)
            .expect("a member row")
            .clone()
    }

    #[test]
    fn no_problem_says_so() {
        assert_eq!(problem(&document()), ["no problem".to_string()]);
    }

    #[test]
    fn an_operation_warning_is_listed() {
        let mut document: ResultDocument = document();
        let warning: OperationWarning = OperationWarning::ManyDesigns {
            count: 12,
            threshold: 10,
            message: "the set grew past its threshold".to_string(),
        };
        document.warnings = vec![warning.clone()];
        let lines: Vec<String> = overview(&document, 0);
        assert_eq!(lines.iter().filter(|line| **line == "warnings:").count(), 1);
        assert!(
            lines.contains(&format!(
                "  {}: {}",
                "many_designs",
                operation_warning_message(&warning)
            )),
            "the code is the serde tag in snake case: {:?}",
            lines
        );
    }

    #[test]
    fn a_stated_buffer_is_named_so() {
        let mut document: ResultDocument = document();
        document.provenance.buffer = Some(BufferView {
            preset: None,
            monovalent_millimolar: 50.0,
            primer_nanomolar: 250.0,
            dmso_percent: 10.0,
        });
        let buffer: &BufferView = document.provenance.buffer.as_ref().unwrap();
        let lines: Vec<String> = provenance(&document);
        assert_eq!(
            lines
                .iter()
                .filter(|line| **line == "buffer: stated")
                .count(),
            1,
            "{:?}",
            lines
        );
        assert!(lines.contains(&format!(
            "  monovalent: {} mM",
            buffer.monovalent_millimolar
        )));
        assert!(lines.contains(&format!("  primer: {} nM", buffer.primer_nanomolar)));
        assert!(lines.contains(&format!("  dmso: {} %", buffer.dmso_percent)));
    }

    #[test]
    fn details_lines_cover_every_value() {
        let details: Details = Details(vec![
            ("text", DetailValue::Text("synA".to_string())),
            ("count", DetailValue::Count(7)),
            ("signed", DetailValue::Signed(-3)),
            ("absent", DetailValue::Absent),
            (
                "obstacles",
                DetailValue::Obstacles(vec![ObstacleView::FivePrimeAvailability {
                    available: 4,
                    needed: 15,
                    message: "not enough bases before the edit".to_string(),
                }]),
            ),
        ]);
        assert_eq!(
            details_lines(&details, "  "),
            vec![
                "  text: synA".to_string(),
                "  count: 7".to_string(),
                "  signed: -3".to_string(),
                "  absent: none".to_string(),
                "  obstacles: obstacles".to_string(),
                "    five_prime_availability: not enough bases before the edit".to_string(),
            ],
            "one line per pair, the obstacle as <indent>  <code>: <message>"
        );
    }

    #[test]
    fn a_member_with_neither_design_nor_failure_says_no_design() {
        let mut document: ResultDocument = document_with(1);
        document.results[0].name = "A42G".to_string();
        let name: &str = &document.results[0].name;
        let summary: String = mutation_summary(&document.results[0].mutation);
        let lines: Vec<String> = overview(&document, 0);
        assert_eq!(
            row(&lines, 0),
            format!("> {}  {}  no design  warnings: 0", name, summary),
            "a member with neither states `no design`, never `designed`"
        );
        assert!(!lines.iter().any(|line| line.contains("designed")));
    }

    #[test]
    fn an_unused_feature_with_an_id_and_a_refusal_is_one_line() {
        let feature: UnusedFeatureView = UnusedFeatureView {
            index: 2,
            key: "CDS".to_string(),
            line: 40,
            id: Some("rop".to_string()),
            refusal: Some(RefusalView {
                code: "alternative_start",
                rule: None,
                message: "the start codon is not ATG".to_string(),
            }),
        };
        let lines: Vec<String> = provenance(&genbank_document(vec![feature.clone()]));
        assert_eq!(
            lines.iter().filter(|line| line.starts_with("    ")).count(),
            1,
            "{:?}",
            lines
        );
        assert_eq!(
            lines.iter().find(|line| line.starts_with("    ")).unwrap(),
            &format!(
                "    {}  line {}  {}  refused: {}",
                feature.key,
                feature.line,
                feature.id.as_ref().unwrap(),
                feature.refusal.as_ref().unwrap().code
            ),
            "key, line, id and refusal's code on one line"
        );
    }
}
