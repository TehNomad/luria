//! The text of the proposal screen (milestone 7e) and of the four result
//! screens (milestone 7c1, 7c2): the overview, the member
//! screen, the problem screen and the provenance screen (plan §5.2 screens 1, 2,
//! 3, 5 and 6). Each screen
//! is a list of plain text lines, one string per line, none holding a `\n`.
//!
//! Pure: no I/O and no `ratatui` or `crossterm` types, so every line is testable
//! as a string (plan §5.1). Every value on a line is one field of the
//! `ResultDocument` or the `InterpretationDocument`, written with `{}` (plan §3
//! rule 1): no coordinate, Tm, codon or translation is computed, no float is
//! rounded, and a digest is shortened nowhere. The one approved rounding is the
//! sequence screen's `Tm` line, shown with `{:.1}` (7i4, plan §5.3a); the full
//! value stays on the technical screen. The only formatting is intervals
//! as `[start, end)`, the 1-based inclusive `bases_at` the screens in words use
//! (M7-24), a float with `{}` (the same digits the JSON holds) and a code
//! written as the serde tag's variant name in snake case. An error, a
//! clarification, a design failure and a warning are always listed (plan §3
//! rule 3); on the proposal screen the questions and an error come first, the
//! proposed request is shown as the exact bytes `--write-request` writes (plan
//! §3 rule 2), and the messages sent to the model are shown (plan §3 rule 6).
//!
//! Screens use Unicode: `→` (U+2192), `–` (U+2013), `°` (U+00B0), `▶` (U+25B6)
//! and `…` (U+2026); the files luria writes are unchanged (plan §5.3b, M7-24).
//! `view` draws these lines (7c3). The state and the experimental validation are
//! not here, because 7a's banner draws them on every screen.

use crate::interpret::document::{
    AssumptionView, InterpretErrorView, InterpretationDocument, OriginView, TemplateFileView,
    UngroundedView,
};
use crate::result::{
    DesignView, DetailValue, Details, EditWarningView, MemberView, MutationView, ObstacleView,
    OperationWarning, PrimerView, PrimerWarningView, Provenance, RequestEcho, ResultDocument,
    SourceView,
};
use crate::review::app::Screen;
use crate::review::edit_span;
use crate::review::export::ExportPlan;
use crate::review::MemberFrame;
use luria_core::frame::EditSpan;
use luria_core::sequence::complement_letter;
use luria_nl::backend::BackendInfo;
use luria_nl::extraction::{Composition, Extraction, Operation};
use luria_nl::prompt::Prompt;
use serde_json::to_string_pretty;
use std::collections::BTreeMap;

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

/// `[start, end)` zero-based as 1-based inclusive: `"1210–1212"`, or `"1210"`
/// for one base (plan §5.3b, M7-24; D006 keeps files zero-based). Only the
/// screens in words use it; a coordinate written by luria stays zero-based.
pub(crate) fn bases_at(start: u64, end: u64) -> String {
    if end.saturating_sub(start) == 1 {
        format!("{}", start + 1)
    } else {
        format!("{}–{}", start + 1, end)
    }
}

/// An insertion at zero-based point `p`: `"after p"`, or `"at the start"` for 0
/// (plan §5.3b, M7-24).
pub(crate) fn insertion_place(point: u64) -> String {
    if point == 0 {
        "at the start".to_string()
    } else {
        format!("after {}", point)
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

/// The code of a warning about one member's edit: its serde tag, the variant
/// name in snake case, written as a `match` rather than found by serializing.
pub(crate) fn edit_warning_code(warning: &EditWarningView) -> &'static str {
    match warning {
        EditWarningView::Frameshift { .. } => "frameshift",
        EditWarningView::InsertionFrameshift { .. } => "insertion_frameshift",
        EditWarningView::ReplacementFrameshift { .. } => "replacement_frameshift",
        EditWarningView::PrematureStop { .. } => "premature_stop",
    }
}

/// A warning about one member's edit's own message.
fn edit_warning_message(warning: &EditWarningView) -> &str {
    match warning {
        EditWarningView::Frameshift { message, .. } => message.as_str(),
        EditWarningView::InsertionFrameshift { message, .. } => message.as_str(),
        EditWarningView::ReplacementFrameshift { message, .. } => message.as_str(),
        EditWarningView::PrematureStop { message, .. } => message.as_str(),
    }
}

/// The code of a warning about one primer: its serde tag, the variant name in
/// snake case, written as a `match` rather than found by serializing.
pub(crate) fn primer_warning_code(warning: &PrimerWarningView) -> &'static str {
    match warning {
        PrimerWarningView::GcContentOutsideBand { .. } => "gc_content_outside_band",
        PrimerWarningView::SingleBaseRun { .. } => "single_base_run",
        PrimerWarningView::DinucleotideRepeat { .. } => "dinucleotide_repeat",
        PrimerWarningView::ThreePrimeGcRun { .. } => "three_prime_gc_run",
        PrimerWarningView::LengthOutsideGuideline { .. } => "length_outside_guideline",
        PrimerWarningView::AboveOrderingLimit { .. } => "above_ordering_limit",
    }
}

/// A warning about one primer's own message.
fn primer_warning_message(warning: &PrimerWarningView) -> &str {
    match warning {
        PrimerWarningView::GcContentOutsideBand { message, .. } => message.as_str(),
        PrimerWarningView::SingleBaseRun { message, .. } => message.as_str(),
        PrimerWarningView::DinucleotideRepeat { message, .. } => message.as_str(),
        PrimerWarningView::ThreePrimeGcRun { message, .. } => message.as_str(),
        PrimerWarningView::LengthOutsideGuideline { message, .. } => message.as_str(),
        PrimerWarningView::AboveOrderingLimit { message, .. } => message.as_str(),
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

/// The mutation one member performed, as one phrase in words (plan §5.3b,
/// M7-24): a residue as `A42G (GCA→GCT)`; a substitution or a replacement as
/// `GAT→AAC at 1210–1212`; a deletion as `delete GAT at 1210–1212`; an
/// insertion as `insert GATT after 1209`. Coordinates are 1-based inclusive, a
/// base text over `MEMBER_INLINE_LIMIT` is `<N> bp`, and every value is the
/// member's own.
pub(crate) fn mutation_summary(mutation: &MutationView) -> String {
    match mutation {
        MutationView::Residue {
            residue,
            original_residue,
            written_residue,
            original_codon,
            written_codon,
            ..
        } => format!(
            "{}{}{} ({}→{})",
            original_residue, residue, written_residue, original_codon, written_codon
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
        } => format!(
            "{}→{} at {}",
            bases_text(original),
            bases_text(written),
            bases_at(span.0, span.1)
        ),
        MutationView::Deletion {
            interval: span,
            original,
            ..
        } => format!(
            "delete {} at {}",
            bases_text(original),
            bases_at(span.0, span.1)
        ),
        MutationView::Insertion { point, written, .. } => {
            format!("insert {} {}", bases_text(written), insertion_place(*point))
        }
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

/// A list of numbers, joined with `, `.
fn numbers(numbers: &[u64]) -> String {
    numbers
        .iter()
        .map(|one| one.to_string())
        .collect::<Vec<String>>()
        .join(", ")
}

/// The mutation's own fields, each `  <label>: <value>`: for every kind its
/// `state`, the coordinates it recorded, and the edited construct; for a
/// residue its CDS, its residue number, its codon policy and, when the document
/// states one, the source of that policy; for an insertion the bases on each
/// side of the point. The letters and the interval are on the summary line
/// above, so they are not repeated here.
fn mutation_lines(mutation: &MutationView) -> Vec<String> {
    let (state, coordinates, edited) = match mutation {
        MutationView::Residue {
            state,
            coordinates,
            edited,
            ..
        }
        | MutationView::Substitution {
            state,
            coordinates,
            edited,
            ..
        }
        | MutationView::Deletion {
            state,
            coordinates,
            edited,
            ..
        }
        | MutationView::Insertion {
            state,
            coordinates,
            edited,
            ..
        }
        | MutationView::Replacement {
            state,
            coordinates,
            edited,
            ..
        } => (state, coordinates, edited),
    };
    let mut lines: Vec<String> = vec![
        format!("  state: {}", state),
        format!("  coordinates: {}", numbers(coordinates)),
        format!(
            "  edited: {}, {} bp, sha256 {}",
            edited.id, edited.length, edited.sha256
        ),
    ];
    match mutation {
        MutationView::Residue {
            cds,
            residue,
            codon_policy,
            codon_policy_source,
            ..
        } => {
            lines.push(format!("  cds: {}", cds));
            lines.push(format!("  residue: {}", residue));
            lines.push(format!("  codon policy: {}", codon_policy));
            if let Some(source) = codon_policy_source {
                lines.push(format!("  codon source: {}", source));
            }
        }
        MutationView::Insertion { before, after, .. } => {
            lines.push(format!("  before: {}", before));
            lines.push(format!("  after: {}", after));
        }
        _ => {}
    }
    lines
}

/// One primer as it is to be typed, written 5'->3' as the document holds it, in
/// the order the parts sit in the primer: its text, length and span, then the
/// edited span when it has one, its 5' tail, its parts in primer order, its
/// annealing portion, its Tm and why that basis was substituted, its whole
/// primer Tm, and its sequence guideline warnings. The enthalpy and the entropy
/// are not on this screen: they stay in the JSON export.
fn primer_lines(primer: &PrimerView) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    let mut head: String = format!(
        "{} 5'-{}-3'  {} nt  span {}",
        primer.role,
        primer.sequence,
        primer.length,
        interval(primer.span)
    );
    if let Some(edited_span) = primer.edited_span {
        head.push_str(&format!("  edited span {}", interval(edited_span)));
    }
    lines.push(head);

    if let Some(tail) = &primer.tail {
        lines.push(format!("  tail: {} ({})", tail.sequence, tail.reason));
    }

    lines.push("  parts:".to_string());
    for part in &primer.parts {
        let mut line: String = format!(
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
            line.push_str(&format!("  {}", interval(span)));
        }
        lines.push(line);
    }

    lines.push(format!(
        "  annealing: {}, {} nt, overlap {}, spans {}",
        primer.annealing.sequence,
        primer.annealing.length,
        primer.annealing.overlap,
        intervals(&primer.annealing.spans)
    ));

    lines.push(format!(
        "  tm: {} C, {}",
        primer.tm.celsius, primer.tm.basis
    ));
    if let Some(because) = &primer.tm.substituted_because {
        lines.push(format!("  tm substituted because: {}", because));
    }

    if let Some(whole) = &primer.whole_primer_tm {
        lines.push(format!(
            "  whole primer tm: {} C, {}",
            whole.celsius, whole.basis
        ));
    }

    if !primer.warnings.is_empty() {
        lines.push("  warnings:".to_string());
        for warning in &primer.warnings {
            lines.push(format!(
                "    {}: {}",
                primer_warning_code(warning),
                primer_warning_message(warning)
            ));
        }
    }
    lines
}

/// The member screen (plan §5.2 screen 3): the member's name, the mutation it
/// performed and its fields, the member's own warnings, then the primer pair
/// with its geometry, Tms, warnings and unresolved inputs, or the reason no
/// pair was designed. `index` is `document.results`' index; out of range the
/// screen is the single line `no such member`. A failure is listed as a failure
/// (plan §3 rule 3), and nothing on the screen is computed, rounded, reversed
/// or complemented (plan §3 rule 1). `c context` opens the sequence context, so
/// it is offered only where a design gives spans to read, and `esc back` is
/// offered on every member screen.
pub fn member(document: &ResultDocument, index: usize) -> Vec<String> {
    if index >= document.results.len() {
        return vec!["no such member".to_string()];
    }
    let member: &MemberView = &document.results[index];
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!("member: {}", member.name));
    lines.push(format!("mutation: {}", mutation_summary(&member.mutation)));
    lines.extend(mutation_lines(&member.mutation));

    if !member.warnings.is_empty() {
        lines.push("warnings:".to_string());
        for warning in &member.warnings {
            lines.push(format!(
                "  {}: {}",
                edit_warning_code(warning),
                edit_warning_message(warning)
            ));
        }
    }

    if let Some(failure) = &member.design_failure {
        lines.push(format!("design failed: {}", failure.code));
        lines.push(format!("  {}", failure.message));
        if !failure.details.0.is_empty() {
            lines.extend(details_lines(&failure.details, "  "));
        }
    }

    if let Some(design) = &member.design {
        lines.push(format!(
            "design: {}, {}, {} {}",
            design.policy, design.style, design.algorithm.name, design.algorithm.version
        ));
        lines.push(format!("  placement: {}", design.placement));
        lines.push(format!(
            "  min annealing overlap: {}",
            design.min_annealing_overlap
        ));
        lines.push(format!("  edit span: {}", interval(design.edit_span)));
        lines.push(format!(
            "  annealing temperature: {} C",
            design.annealing_temperature
        ));
        lines.push(format!(
            "  tm balance met: {}",
            if design.tm_balance_met { "yes" } else { "no" }
        ));
        let mut pair: String = format!(
            "  pair: {}, separation {}, deleted width {}",
            design.pair.relationship, design.pair.separation, design.pair.deleted_width
        );
        if let Some(boundary) = design.pair.boundary {
            pair.push_str(&format!(", boundary {}", boundary));
        }
        if design.pair.abuts {
            pair.push_str(", abuts");
        }
        lines.push(pair);

        lines.extend(primer_lines(&design.forward));
        lines.extend(primer_lines(&design.reverse));

        if !design.unresolved.is_empty() {
            lines.push("unresolved:".to_string());
            for entry in &design.unresolved {
                lines.push(format!("  {}: {}", entry.input, entry.effect));
            }
        }
    }

    lines.push(String::new());
    lines
}

/// The details screen's lines (7i4, M7-30, M7-52): 7c2's member text, one empty
/// line, then the provenance, so the record of how the result was computed is
/// read with the variant's technical lines, one key away, and `member()`'s stale
/// `c context` and `esc back` lines are gone. The empty line is added only when
/// `member()`'s last line is not already empty, so there is exactly one.
pub fn details(document: &ResultDocument, index: usize) -> Vec<String> {
    let mut lines: Vec<String> = member(document, index);
    if lines.last().is_some_and(|last| !last.is_empty()) {
        lines.push(String::new());
    }
    lines.extend(provenance(document));
    lines
}

/// The details screen with the model's reading under it (M7-64): `details`' own
/// lines, one empty line, the heading, then the model's reading with each line
/// indented by two spaces, so it sits inside the body as the provenance does. An
/// empty reading line stays empty, so the spacing `text::proposal` gave is kept.
/// Nothing is added to, changed in or removed from `details`' lines.
pub fn details_with_reading(
    document: &ResultDocument,
    index: usize,
    reading: &[String],
) -> Vec<String> {
    let mut lines: Vec<String> = details(document, index);
    lines.push(String::new());
    lines.push("the model's reading of your words:".to_string());
    for line in reading {
        if line.is_empty() {
            lines.push(String::new());
        } else {
            lines.push(format!("  {}", line));
        }
    }
    lines
}

/// A sequence-context block's width, in template columns.
const CONTEXT_WIDTH: usize = 60;
/// The width of a context row's label.
const CONTEXT_LABEL: usize = 10;
/// The line the screen says when the template's bases are not there to draw
/// (D005).
const CONTEXT_UNAVAILABLE: &str = "sequence context unavailable: the template could not be read \
                                   again, or it differs from the one this result was computed on";

/// One primer's bases, by template column: the character to show in each column
/// its parts fill, and nothing for the columns it leaves empty.
type ContextColumns = BTreeMap<usize, char>;

/// One primer's parts written into `columns`, the template columns it fills, in
/// `template_len`'s bounds; `notes` collects the lines a part that cannot be
/// written in its own span gives. `reverse` is the reverse primer, whose text
/// reads 3'->5' on the screen: its bases are written in the reverse of the order
/// they sit in the primer, so each base stands in the column of the template
/// base it pairs with, and its 5' tail starts at the column just right of its
/// last spanned column. A part with a span whose base count differs from the
/// span's width, as an unequal replacement's does, is `*` in every one of its
/// columns and named after the blocks; a tail reaching outside the template is
/// not drawn and is named there too. The bases are the part's own text, upper
/// case when it anneals and lower case when it does not: nothing is complemented
/// (plan §3 rule 1).
fn place(
    primer: &PrimerView,
    reverse: bool,
    template_len: usize,
    columns: &mut ContextColumns,
    notes: &mut Vec<String>,
) {
    // The columns the primer's spanned parts fix: an unspanned tail sits beside
    // them, so the tail's own columns are counted from them.
    let spans: Vec<(u64, u64)> = primer.parts.iter().filter_map(|part| part.span).collect();
    let first: i64 = spans.iter().map(|span| span.0 as i64).min().unwrap_or(0);
    let last: i64 = spans.iter().map(|span| span.1 as i64).max().unwrap_or(0);

    for part in &primer.parts {
        let shown: Vec<char> = if part.annealing {
            part.sequence.to_uppercase().chars().collect()
        } else {
            part.sequence.to_lowercase().chars().collect()
        };
        // The reverse primer's bases read 3'->5', so they are written in the
        // reverse of the order they sit in the primer.
        let written: Vec<char> = if reverse {
            shown.iter().rev().copied().collect()
        } else {
            shown
        };
        match part.span {
            Some((start, end)) => {
                if part.sequence.len() != (end - start) as usize {
                    for column in start..end {
                        columns.insert(column as usize, '*');
                    }
                    notes.push(format!(
                        "{} {}: {} at {}",
                        primer.role,
                        part.role,
                        part.sequence,
                        interval((start, end))
                    ));
                    continue;
                }
                for (offset, base) in written.iter().enumerate() {
                    columns.insert(start as usize + offset, *base);
                }
            }
            None => {
                // Signed arithmetic throughout, so a tail past the template's
                // start is skipped instead of underflowing.
                let from: i64 = if reverse {
                    last
                } else {
                    first - written.len() as i64
                };
                let mut outside: bool = false;
                for (offset, base) in written.iter().enumerate() {
                    let column: i64 = from + offset as i64;
                    if column < 0 || column as usize >= template_len {
                        outside = true;
                        continue;
                    }
                    columns.insert(column as usize, *base);
                }
                if outside {
                    notes.push(format!(
                        "{} tail: {} extends past the template",
                        primer.role, part.sequence
                    ));
                }
            }
        }
    }
}

/// The sequence-context screen (plan §5.2 screen 4, 7d): a designed primer pair
/// laid out against the template, 60 template columns a block, the forward
/// primer 5'->3' above the top strand and the reverse primer 3'->5' below it,
/// each base in the template column the document's span puts it in, `^` under
/// every base of the design's edit span and `+` at an insertion point. The
/// window runs from five bases before the leftmost primer column to five after
/// the rightmost, tails included, within the template.
///
/// `index` is `document.results`' index, out of range the screen is `no such
/// member`; a member with no design says so; when `template` is `None`, so the
/// template's bases are not the ones this result was computed on, the screen
/// says it is unavailable and draws no sequence (D005). `template` is what
/// `matching_template` accepts. Nothing is complemented, and no Tm, coordinate,
/// pairing or overlap is computed (plan §3 rule 1): the bases are placed by the
/// spans the document reports. The layout of the three designs the fixtures
/// record is checked by `tests/fixtures/review/reference/context.py`, not here.
pub fn context(document: &ResultDocument, index: usize, template: Option<&[char]>) -> Vec<String> {
    if index >= document.results.len() {
        return vec!["no such member".to_string()];
    }
    let member: &MemberView = &document.results[index];
    let design: &DesignView = match &member.design {
        Some(design) => design,
        None => return vec!["no design, so no sequence context".to_string()],
    };
    let letters: &[char] = match template {
        Some(letters) => letters,
        None => return vec![CONTEXT_UNAVAILABLE.to_string()],
    };

    let mut forward: ContextColumns = BTreeMap::new();
    let mut reverse: ContextColumns = BTreeMap::new();
    let mut notes: Vec<String> = Vec::new();
    place(
        &design.forward,
        false,
        letters.len(),
        &mut forward,
        &mut notes,
    );
    place(
        &design.reverse,
        true,
        letters.len(),
        &mut reverse,
        &mut notes,
    );

    // The edit row: `^` under every base of the design's own edit span, and `+`
    // at the point an insertion names.
    let mut edit: ContextColumns = BTreeMap::new();
    for column in design.edit_span.0..design.edit_span.1 {
        edit.insert(column as usize, '^');
    }
    if let MutationView::Insertion { point, .. } = &member.mutation {
        edit.insert(*point as usize, '+');
    }

    // The window, from the columns the two primers are written in, within the
    // template. A design whose parts name no template column has none to lay
    // out, so no block is drawn.
    let used: Vec<usize> = forward.keys().chain(reverse.keys()).copied().collect();
    let (lo, hi): (usize, usize) = match (used.iter().min().copied(), used.iter().max().copied()) {
        (Some(first), Some(last)) => (
            first.saturating_sub(5),
            letters.len().min(last.saturating_add(6)),
        ),
        _ => (0, 0),
    };

    let template_id: &str = match &document.provenance.template {
        Some(template) => template.id.as_str(),
        None => "none",
    };
    let mut lines: Vec<String> = vec![
        format!(
            "sequence context: {}, {} of {}",
            member.name,
            interval((lo as u64, hi as u64)),
            template_id
        ),
        "top strand 5'->3'; forward 5'->3' above it, reverse 3'->5' below it".to_string(),
        "upper case anneals, lower case does not; ^ base edited or removed, + insertion point"
            .to_string(),
    ];

    let mut start: usize = lo;
    while start < hi {
        let end: usize = (start + CONTEXT_WIDTH).min(hi);
        lines.push(String::new());
        lines.push(format!("bases {}", interval((start as u64, end as u64))));
        for (label, cells) in [
            ("template", None),
            ("forward", Some(&forward)),
            ("reverse", Some(&reverse)),
            ("edit", Some(&edit)),
        ] {
            let body: String = match cells {
                None => letters[start..end].iter().collect(),
                Some(columns) => (start..end)
                    .map(|column| *columns.get(&column).unwrap_or(&' '))
                    .collect(),
            };
            // The label padded to `CONTEXT_LABEL`, then one character per column,
            // a column the row does not fill as a space, and the trailing spaces
            // cut: a row that fills nothing is the label alone.
            let row: String = format!("{:width$}{}", label, body, width = CONTEXT_LABEL);
            lines.push(row.trim_end().to_string());
        }
        start = end;
    }

    if !notes.is_empty() {
        lines.push(String::new());
        lines.extend(notes);
    }
    lines
}

/// The base count past which the heading and the overview write a base text as
/// its length, and past which an insertion that is not expanded is collapsed
/// (`member.py`'s inline limit, plan §5.3a).
pub(crate) const MEMBER_INLINE_LIMIT: usize = 30;

/// A base text as the heading and the overview state it: the bases themselves,
/// or `<N> bp` when there are more than `MEMBER_INLINE_LIMIT` of them.
pub(crate) fn bases_text(bases: &str) -> String {
    let count: usize = bases.chars().count();
    if count > MEMBER_INLINE_LIMIT {
        format!("{} bp", count)
    } else {
        bases.to_string()
    }
}

/// A collapsed insertion's marker: `[+N bp <first 10>...<last 10>]`, its bases
/// lower case and three ASCII dots between the two halves.
fn marker(written: &[char]) -> String {
    let head: String = written.iter().take(10).collect::<String>().to_lowercase();
    let tail: String = written[written.len().saturating_sub(10)..]
        .iter()
        .collect::<String>()
        .to_lowercase();
    format!("[+{} bp {}...{}]", written.len(), head, tail)
}

/// The bottom row's character for a template-row character: `complement_letter`
/// of it, lower when the base is lower, and the character itself when it has no
/// complement, as `-` and a marker's punctuation. No complement table (plan §3
/// rule 1).
fn bottom_letter(ch: char) -> char {
    match complement_letter(ch.to_ascii_uppercase()) {
        Some(complement) => {
            if ch.is_ascii_lowercase() {
                complement.to_ascii_lowercase()
            } else {
                complement
            }
        }
        None => ch,
    }
}

/// The sequence block's geometry: the edit, the region it takes, and the number
/// of display columns the block runs over. Nothing is computed but display
/// columns (plan §3 rule 1).
struct MemberLayout {
    /// The edit's start: a template position and, on its own, a display column.
    start: usize,
    /// The edit's end: a template position.
    end: usize,
    /// The region's width: the greater of the bases removed and the bases
    /// written, or the width of a collapsed insertion's marker.
    region: usize,
    /// The whole display, in columns: the template, less the bases removed,
    /// plus the region.
    total: usize,
    /// An insertion of more than `MEMBER_INLINE_LIMIT` bases, not expanded.
    collapsed: bool,
}

impl MemberLayout {
    /// A template position as a display column: positions before the edit keep
    /// their own, positions inside it stand in the region from its start, and
    /// positions at or past its end follow the region.
    fn col(&self, position: usize) -> usize {
        if position < self.start {
            position
        } else if position < self.end {
            self.start + (position - self.start)
        } else {
            self.start + self.region + (position - self.end)
        }
    }

    /// A display column as a template position: a column inside the region is the
    /// region's start, and its end when `at_end`, because the window's end is
    /// exclusive and maps to the position after the last drawn base.
    fn template_position(&self, column: usize, at_end: bool) -> u64 {
        let position: usize =
            if column <= self.start && !(at_end && column == self.start && self.start < self.end) {
                column
            } else if column < self.start + self.region {
                if at_end {
                    self.end
                } else {
                    self.start
                }
            } else {
                self.end + (column - self.start - self.region)
            };
        position as u64
    }
}

/// One primer's parts written into `cells`, the display columns of the sequence
/// block. The forward primer reads 5'->3' and the reverse 3'->5', so a reverse
/// part's text is reversed: each base stands in the column of the template base
/// it pairs with. A spanned context part stands at `col(p)` for every `p` of its
/// span; a mutation part's written bases from the edit's start; an unspanned
/// tail just left of the forward primer's first spanned column, and, for the
/// reverse primer, reversed just right of its last. A part's bases are upper case
/// when it anneals and lower case when it does not. Inside a collapsed insertion
/// a tail is hidden whole, and any base the marker covers is hidden; both are
/// named in `notes` with the `i` key that expands. A context part whose base
/// count differs from its span is `*` over its span and named, as `place` writes
/// it; columns outside the display are not drawn.
fn member_place(
    primer: &PrimerView,
    reverse: bool,
    layout: &MemberLayout,
    cells: &mut ContextColumns,
    notes: &mut Vec<String>,
) {
    // The columns the primer's spanned parts fix: an unspanned tail sits beside
    // them, so its own columns are counted from them.
    let spans: Vec<(u64, u64)> = primer.parts.iter().filter_map(|part| part.span).collect();
    let first: i64 = spans
        .iter()
        .map(|span| layout.col(span.0 as usize) as i64)
        .min()
        .unwrap_or(0);
    let last: i64 = spans
        .iter()
        .map(|span| layout.col((span.1 - 1) as usize) as i64 + 1)
        .max()
        .unwrap_or(0);

    for part in &primer.parts {
        let shown: Vec<char> = if part.annealing {
            part.sequence.to_uppercase().chars().collect()
        } else {
            part.sequence.to_lowercase().chars().collect()
        };
        // The reverse primer's bases read 3'->5', so they are written in the
        // reverse of the order they sit in the primer.
        let written: Vec<char> = if reverse {
            shown.iter().rev().copied().collect()
        } else {
            shown
        };
        let count: usize = part.sequence.chars().count();
        let cols: Vec<i64> = match part.span {
            None => {
                if layout.collapsed {
                    // A tail is inserted bases: inside a collapsed insertion all
                    // of it is hidden.
                    notes.push(format!(
                        "{} tail: {} nt inside the insertion, i to expand",
                        primer.role, count
                    ));
                    continue;
                }
                let from: i64 = if reverse {
                    last
                } else {
                    first - written.len() as i64
                };
                (from..from + written.len() as i64).collect()
            }
            Some((start, end)) => {
                if part.role == "mutation" {
                    (layout.start as i64..layout.start as i64 + written.len() as i64).collect()
                } else if count != (end - start) as usize {
                    for position in start..end {
                        cells.insert(layout.col(position as usize), '*');
                    }
                    notes.push(format!(
                        "{} {}: {} at {}",
                        primer.role,
                        part.role,
                        part.sequence,
                        interval((start, end))
                    ));
                    continue;
                } else {
                    (start..end)
                        .map(|position| layout.col(position as usize) as i64)
                        .collect()
                }
            }
        };

        let mut hidden: bool = false;
        for (column, base) in cols.into_iter().zip(written.iter()) {
            if layout.collapsed
                && column >= layout.start as i64
                && column < layout.start as i64 + layout.region as i64
            {
                hidden = true;
                continue;
            }
            if column >= 0 && (column as usize) < layout.total {
                cells.insert(column as usize, *base);
            }
        }
        if hidden {
            notes.push(format!(
                "{} {}: {} nt inside the insertion, i to expand",
                primer.role, part.role, count
            ));
        }
    }
}

/// The redesigned member screen's sequence block (plan §5.3a), as
/// `tests/fixtures/review/reference/member.py` lays it out: the forward primer
/// above the top strand, the mutant protein between the strands, the bottom
/// strand, and the reverse primer below it, in blocks of `CONTEXT_WIDTH` display
/// columns. `index` is `document.results`' index, out of range the block is the
/// single line `no such member`; a member with no design says so; when `template`
/// is `None`, so the template's bases are not the ones this result was computed
/// on, the block says it is unavailable and draws no sequence (D005). `template`
/// is what `matching_template` accepts, and `frame` is `App::frame(index)`, this
/// member's mutant protein: a frame error, or no frame, draws the `protein` label
/// alone and names the reason as the last note. `expanded` asks for a long
/// insertion to be written out, so its marker, its hidden tails and the protein
/// inside it are drawn.
///
/// Nothing is computed but display columns (plan §3 rule 1): the bases are the
/// document's own, the protein is the one core read, and the bottom row is
/// `complement_letter` of the template row.
pub fn member_sequence(
    document: &ResultDocument,
    index: usize,
    template: Option<&[char]>,
    frame: Option<&MemberFrame>,
    expanded: bool,
) -> Vec<String> {
    if index >= document.results.len() {
        return vec!["no such member".to_string()];
    }
    let member: &MemberView = &document.results[index];
    let design: &DesignView = match &member.design {
        Some(design) => design,
        None => return vec!["no design, so no sequence".to_string()],
    };
    let letters: &[char] = match template {
        Some(letters) => letters,
        None => return vec![CONTEXT_UNAVAILABLE.to_string()],
    };

    // The edit: the span 7i2's `edit_span` maps from the mutation, and the bases
    // the variant names as written, which is that span's written length.
    let span: EditSpan = match edit_span(&member.mutation) {
        Ok(span) => span,
        Err(reason) => return vec![format!("the edit span cannot be read: {}", reason)],
    };
    let written: Vec<char> = match &member.mutation {
        MutationView::Substitution { written, .. }
        | MutationView::Replacement { written, .. }
        | MutationView::Insertion { written, .. } => written.chars().collect(),
        MutationView::Residue { written_codon, .. } => written_codon.chars().collect(),
        MutationView::Deletion { .. } => Vec::new(),
    };
    let (s, e): (usize, usize) = (span.start as usize, span.end as usize);
    let collapsed: bool = s == e && written.len() > MEMBER_INLINE_LIMIT && !expanded;
    let region: usize = if collapsed {
        marker(&written).chars().count()
    } else {
        (e - s).max(written.len())
    };
    let layout: MemberLayout = MemberLayout {
        start: s,
        end: e,
        region,
        total: letters.len() - (e - s) + region,
        collapsed,
    };

    // The template row: its bases outside the edit, at their own columns, and the
    // region's columns from the edit's start, holding the bases removed in lower
    // case (for an insertion, the bases written), then `-` past them, or the
    // collapsed marker.
    let mut template_row: ContextColumns = BTreeMap::new();
    for (position, base) in letters.iter().enumerate() {
        if position < layout.start || position >= layout.end {
            template_row.insert(layout.col(position), *base);
        }
    }
    if layout.collapsed {
        for (offset, ch) in marker(&written).chars().enumerate() {
            template_row.insert(layout.start + offset, ch);
        }
    } else {
        let shown: Vec<char> = if layout.start == layout.end {
            written
                .iter()
                .collect::<String>()
                .to_lowercase()
                .chars()
                .collect()
        } else {
            letters[layout.start..layout.end]
                .iter()
                .map(|ch| ch.to_ascii_lowercase())
                .collect()
        };
        for offset in 0..layout.region {
            let ch: char = if offset < shown.len() {
                shown[offset]
            } else {
                '-'
            };
            template_row.insert(layout.start + offset, ch);
        }
    }

    // The bottom row: the complement of every template-row character, none under
    // a collapsed marker.
    let mut bottom_row: ContextColumns = BTreeMap::new();
    for (column, ch) in &template_row {
        if layout.collapsed && *column >= layout.start && *column < layout.start + layout.region {
            continue;
        }
        bottom_row.insert(*column, bottom_letter(*ch));
    }

    // The two primer rows.
    let mut notes: Vec<String> = Vec::new();
    let mut forward: ContextColumns = BTreeMap::new();
    let mut reverse: ContextColumns = BTreeMap::new();
    member_place(&design.forward, false, &layout, &mut forward, &mut notes);
    member_place(&design.reverse, true, &layout, &mut reverse, &mut notes);

    // The protein: each residue core read over the edited construct, at the
    // display column of its codon's middle base; none inside a collapsed marker.
    let mut protein: ContextColumns = BTreeMap::new();
    if let Some(Ok(residues)) = frame {
        for residue in residues {
            let middle: usize = residue.middle as usize;
            let column: usize = if middle < layout.start {
                middle
            } else if middle < layout.start + written.len() {
                if layout.collapsed {
                    continue;
                }
                layout.start + (middle - layout.start)
            } else {
                layout.start + layout.region + (middle - layout.start - written.len())
            };
            protein.insert(column, residue.symbol);
        }
    }

    // The window: five columns on each side of the columns the two primers are
    // drawn in, within the display.
    let used: Vec<usize> = forward.keys().chain(reverse.keys()).copied().collect();
    let (lo, hi): (usize, usize) = match (used.iter().min().copied(), used.iter().max().copied()) {
        (Some(first), Some(last)) => (first.saturating_sub(5), layout.total.min(last + 6)),
        _ => (0, 0),
    };

    let template_id: &str = match &document.provenance.template {
        Some(named) => named.id.as_str(),
        None => "none",
    };
    let mut lines: Vec<String> = vec![format!(
        "{}  {}  {} bases {}",
        member.name,
        mutation_summary(&member.mutation),
        template_id,
        bases_at(
            layout.template_position(lo, false),
            layout.template_position(hi, true)
        )
    )];

    // Blocks of `CONTEXT_WIDTH` columns; a block never splits a collapsed marker,
    // so it ends where the marker starts and the next block starts at it.
    let mut start: usize = lo;
    while start < hi {
        let mut end: usize = (start + CONTEXT_WIDTH).min(hi);
        if layout.collapsed
            && start < layout.start
            && layout.start < end
            && end < layout.start + layout.region
        {
            end = layout.start;
        }
        lines.push(String::new());
        for (label, cells) in [
            ("forward", &forward),
            ("template", &template_row),
            ("protein", &protein),
            ("bottom", &bottom_row),
            ("reverse", &reverse),
        ] {
            let body: String = (start..end)
                .map(|column| *cells.get(&column).unwrap_or(&' '))
                .collect();
            let row: String = format!("{:width$}{}", label, body, width = CONTEXT_LABEL);
            lines.push(row.trim_end().to_string());
        }
        start = end;
    }

    if layout.collapsed {
        notes.insert(0, format!("insertion: {} bp, i to expand", written.len()));
    }
    // A frame that cannot be read, or none at all, is said as the last note; the
    // `protein` row is then the bare label in every block.
    if let Some(Err(reason)) = frame {
        notes.push(format!("protein: {}", reason));
    } else if frame.is_none() {
        notes.push("protein: not available".to_string());
    }
    if !notes.is_empty() {
        lines.push(String::new());
        lines.extend(notes);
    }
    lines
}

/// One primer's line in the sequence screen's primer table (M7-28, plan §5.3b):
/// its label padded to `CONTEXT_LABEL`, its `<n> nt` right-aligned in 6, its
/// `{:.1} °C` right-aligned in 9, three spaces, then its own sequence.
/// The padding counts characters, not bytes.
fn primer_table_line(label: &str, primer: &PrimerView) -> String {
    format!(
        "{:width$}{:>6}  {:>9}   {}",
        label,
        format!("{} nt", primer.length),
        format!("{:.1} °C", primer.tm.celsius),
        primer.sequence,
        width = CONTEXT_LABEL
    )
}

/// The primer table's header (M7-28): 10 spaces, `length` right-aligned in 6,
/// `Tm` right-aligned in 9, three spaces, `sequence 5'→3'`.
fn primer_table_header() -> String {
    format!(
        "{}{:>6}  {:>9}   sequence 5'→3'",
        " ".repeat(CONTEXT_LABEL),
        "length",
        "Tm"
    )
}

/// The sequence screen (`enter` on a member, plan §5.3a, 7i4, M7-28, M7-29):
/// 7i3's block, then, when the member has a design, the primer table with the
/// annealing temperature, then every warning by the document's own message, and
/// a design failure as `No primer pair could be designed` with its message. The
/// full Tm, the codes and every other value stay on the details screen (7c2's
/// `member`). `index` is `document.results`' index; out of range the screen is
/// 7i3's single line. Every value is one field of the document (plan §3 rule 1),
/// and warnings and failures are listed, never summarised away (plan §3 rule 3).
pub fn member_screen(
    document: &ResultDocument,
    index: usize,
    template: Option<&[char]>,
    frame: Option<&MemberFrame>,
    expanded: bool,
) -> Vec<String> {
    let mut lines: Vec<String> = member_sequence(document, index, template, frame, expanded);
    let member: &MemberView = match document.results.get(index) {
        Some(member) => member,
        None => return lines,
    };

    if let Some(design) = &member.design {
        lines.push(String::new());
        lines.push(primer_table_header());
        lines.push(primer_table_line("forward", &design.forward));
        lines.push(primer_table_line("reverse", &design.reverse));
        lines.push(format!(
            "annealing temperature {} °C",
            design.annealing_temperature
        ));
    }

    // The messages, in the document's own order: the member's edit warnings, then
    // the forward primer's, then the reverse primer's (M7-29): a warning is shown
    // as the document's own `message`, never as its code.
    let mut warnings: Vec<String> = Vec::new();
    for warning in &member.warnings {
        warnings.push(format!("  {}", edit_warning_message(warning)));
    }
    if let Some(design) = &member.design {
        for warning in &design.forward.warnings {
            warnings.push(format!("  forward: {}", primer_warning_message(warning)));
        }
        for warning in &design.reverse.warnings {
            warnings.push(format!("  reverse: {}", primer_warning_message(warning)));
        }
    }
    if !warnings.is_empty() {
        lines.push(String::new());
        lines.push("Warnings".to_string());
        lines.extend(warnings);
    }

    if let Some(failure) = &member.design_failure {
        lines.push(String::new());
        lines.push("No primer pair could be designed".to_string());
        lines.push(format!("  {}", failure.message));
    }
    lines
}

/// The `?` help (M7-27, M7-53): the sequence legend, on the variant screen only,
/// then `Keys` and the open screen's own keys. Each key line is two spaces, the
/// key padded to 11, then its words, and `ctrl-c` is the last line on every
/// screen. `problem` is whether the document has one, `expandable` whether `i`
/// expands on the open member; a key that does nothing is not named. Nothing of
/// the document is in it.
pub fn help(screen: Screen, problem: bool, expandable: bool) -> Vec<String> {
    const LEGEND: [&str; 10] = [
        "The sequence",
        "  forward    the forward primer, 5'→3', over the bases it covers",
        "  template   the template's top strand, 5'→3'",
        "  protein    the variant's protein, each letter over its codon's middle base",
        "  bottom     the bottom strand, 3'→5'",
        "  reverse    the reverse primer, written 3'→5' under its bases",
        "  lower case edited bases, and primer bases that do not anneal",
        "  *          a stop codon: the protein ends there",
        "  [+N bp …]  a long insertion, shown in full with i",
        "",
    ];

    /// One key line: the key padded to 11, then its words.
    fn key(key: &str, words: &str) -> String {
        format!("  {:width$}{}", key, words, width = 11)
    }

    let mut lines: Vec<String> = Vec::new();
    if matches!(screen, Screen::Member(_)) {
        lines.extend(LEGEND.map(|line| line.to_string()));
    }
    lines.push("Keys".to_string());
    match screen {
        Screen::Overview => {
            lines.push(key("enter", "open a variant"));
            lines.push(key("j k", "move"));
            lines.push(key("PgUp PgDn", "page"));
            if problem {
                lines.push(key("e", "the problem"));
            }
            lines.push(key("x", "export"));
        }
        Screen::Member(_) => {
            lines.push(key("d", "details"));
            if expandable {
                lines.push(key("i", "show or hide a long insertion"));
            }
            lines.push(key("j k", "scroll"));
            lines.push(key("PgUp PgDn", "page"));
            lines.push(key("esc", "back"));
        }
        Screen::Details(_) | Screen::Problem => {
            lines.push(key("j k", "scroll"));
            lines.push(key("PgUp PgDn", "page"));
            lines.push(key("Home", "top"));
            lines.push(key("esc", "back"));
        }
        Screen::Export => {
            lines.push(key("w o", "write the files"));
            lines.push(key("esc", "back"));
        }
        Screen::Replace => {
            lines.push(key("y", "replace the files named"));
            lines.push(key("esc", "back"));
        }
    }
    lines.push(key("q", "quit"));
    lines.push(key("ctrl-c", "quit"));
    lines
}

/// The code of the operation the model reported: its serde tag, the variant
/// name in snake case, written as a `match` rather than found by serializing.
fn operation_code(operation: &Operation) -> &'static str {
    match operation {
        Operation::Q5ResidueSubstitutions => "q5_residue_substitutions",
        Operation::Other => "other",
    }
}

/// The code of the composition the model reported: its serde tag, the variant
/// name in snake case.
fn composition_code(composition: &Composition) -> &'static str {
    match composition {
        Composition::SeparateVariants => "separate_variants",
        Composition::CombinedConstruct => "combined_construct",
    }
}

/// The questions a run is asking, first on the screen and never summarised away
/// (plan §3 rule 3): `questions:`, then one line per question, `  <code>
/// (<subject>): <message>`, each part the question's own.
fn question_lines(document: &InterpretationDocument) -> Vec<String> {
    let mut lines: Vec<String> = vec!["questions:".to_string()];
    for question in &document.questions {
        lines.push(format!(
            "  {} ({}): {}",
            question.code, question.subject, question.message
        ));
    }
    lines
}

/// Why a run ended, first on the screen (plan §3 rule 3): `error: <code>`, the
/// rule when the refusal names one, then its own message on one line; and, when
/// the reply holds a string the user's text does not, that string by its place in
/// the extraction and its own text, so a fabrication is named and not hidden.
fn error_lines(error: &InterpretErrorView, ungrounded: Option<&UngroundedView>) -> Vec<String> {
    let mut lines: Vec<String> = vec![format!("error: {}", error.code)];
    if let Some(rule) = &error.rule {
        lines.push(format!("  rule: {}", rule));
    }
    lines.push(format!("  {}", error.message));
    if let Some(problem) = ungrounded {
        lines.push(format!(
            "  not in your text: {} \"{}\"",
            problem.field, problem.value
        ));
    }
    lines
}

/// The proposed request, shown as exactly the bytes `--write-request` writes
/// (plan §3 rule 2): the header, then the request echo through
/// `serde_json::to_string_pretty`, line by line, two spaces in, so what the user
/// approves is what the file holds. A request that cannot be serialized says so
/// on one line; the echo of a parsed request always serializes, so this never
/// panics.
fn request_lines(request: &RequestEcho) -> Vec<String> {
    let mut lines: Vec<String> =
        vec!["proposed request, written exactly so when you approve it:".to_string()];
    match to_string_pretty(request) {
        Err(_) => lines.push("  the request cannot be shown".to_string()),
        Ok(pretty) => {
            for line in pretty.lines() {
                lines.push(format!("  {}", line));
            }
        }
    }
    lines
}

/// Where each value of a proposed request came from: `where each value came
/// from:`, then one line per value, `  <value>: <source>`, `source` being the
/// document's own `text` or `profile`.
fn origin_lines(origins: &[OriginView]) -> Vec<String> {
    let mut lines: Vec<String> = vec!["where each value came from:".to_string()];
    for origin in origins {
        lines.push(format!("  {}: {}", origin.value, origin.source));
    }
    lines
}

/// What a proposal took as given, for the user to check: `assumptions:`, then one
/// line per assumption, `  <code>: <message>`.
fn assumption_lines(assumptions: &[AssumptionView]) -> Vec<String> {
    let mut lines: Vec<String> = vec!["assumptions:".to_string()];
    for assumption in assumptions {
        lines.push(format!("  {}: {}", assumption.code, assumption.message));
    }
    lines
}

/// The model's reading of the user's text, as the reply wrote it, quoted so the
/// screen shows what the model claimed and not what luria made of it (plan §3
/// rule 1, D021): the operation, the CDS name as the user wrote it, one row per
/// mutation with the words that say so and the codon when the user named one, the
/// composition, the numbering, and every quote no field can hold.
fn evidence_lines(extraction: &Extraction) -> Vec<String> {
    let mut lines: Vec<String> = vec![
        "the model's reading, checked against your text:".to_string(),
        format!("  operation: {}", operation_code(&extraction.operation)),
    ];
    match &extraction.cds {
        Some(cds) => lines.push(format!("  cds: \"{}\"", cds)),
        None => lines.push("  cds: none".to_string()),
    }
    for mutation in &extraction.mutations {
        lines.push(format!(
            "  {}  quote \"{}\"",
            mutation.token, mutation.quote
        ));
        if let Some(codon) = &mutation.codon {
            lines.push(format!("  codon {}", codon));
        }
    }
    match &extraction.composition {
        Some(composition) => {
            lines.push(format!("  composition: {}", composition_code(composition)))
        }
        None => lines.push("  composition: none".to_string()),
    }
    match &extraction.numbering {
        Some(numbering) => lines.push(format!("  numbering: \"{}\"", numbering)),
        None => lines.push("  numbering: none".to_string()),
    }
    for entry in &extraction.unsupported {
        lines.push(format!("  unsupported: \"{}\"", entry));
    }
    lines
}

/// The template the run read, always named on the screen: its file name alone
/// (D012), the record's LOCUS name and the digest of its sequence, written in
/// full. A run that read no template says so.
fn template_lines(template: Option<&TemplateFileView>) -> Vec<String> {
    match template {
        Some(found) => vec![format!(
            "template: {}, LOCUS {}, sha256 {}",
            found.file, found.locus_name, found.sha256
        )],
        None => vec!["template: not read".to_string()],
    }
}

/// The model that was called and the prompt it was sent, always named on the
/// screen (plan §3 rule 6): the backend's kind, its model, the endpoint it calls
/// or `no endpoint`, and the prompt's version. Never a key: `describe` keeps
/// credentials out of the document (D003).
fn model_lines(backend: &BackendInfo, prompt_version: &str) -> Vec<String> {
    vec![format!(
        "model: {} {} at {}, prompt {}",
        backend.kind,
        backend.model,
        backend.endpoint.as_deref().unwrap_or("no endpoint"),
        prompt_version
    )]
}

/// What was sent to the model, from the document (plan §3 rule 6, D023), always
/// on the screen: the system prompt by its line count alone, it being luria's own
/// text, then the user message in full, line by line, four spaces in. A run that
/// sent nothing says so.
fn sent_lines(messages: Option<&Prompt>) -> Vec<String> {
    match messages {
        Some(sent) => {
            let mut lines: Vec<String> = vec![
                "sent to the model:".to_string(),
                format!(
                    "  system prompt: {} lines, not shown",
                    sent.system.lines().count()
                ),
                "  user message:".to_string(),
            ];
            for line in sent.user.lines() {
                lines.push(format!("    {}", line));
            }
            lines
        }
        None => vec!["nothing was sent to the model".to_string()],
    }
}

/// The model's words exactly as they came back, two spaces in, line by line: the
/// reply itself, kept apart from the request it became (D021).
fn reply_lines(reply: &str) -> Vec<String> {
    let mut lines: Vec<String> = vec!["the model's reply, as received:".to_string()];
    for line in reply.lines() {
        lines.push(format!("  {}", line));
    }
    lines
}

/// The proposal screen (plan §5.2 screen 1, 7e): the questions, the error, the
/// proposed request, each value's origin, the assumptions, the model's reading,
/// the template, the model called, the messages sent and the raw reply. Groups in
/// this order, one blank line between them, each shown only when it has content,
/// except `template`, `model` and `sent`, which are always there. Nothing is
/// parsed out of a message, re-derived or summarised (plan §3 rule 1); questions
/// and an error come before anything else (plan §3 rule 3); the request shown is
/// byte for byte the file approving writes (plan §3 rule 2); and the screen shows
/// what was sent to the model (plan §3 rule 6). The state is not here: 7a's
/// banner draws it on every screen, and a proposal is labelled as a proposal. The
/// lines are drawn by `view` (7f).
pub fn proposal(document: &InterpretationDocument) -> Vec<String> {
    let mut groups: Vec<Vec<String>> = Vec::new();
    if !document.questions.is_empty() {
        groups.push(question_lines(document));
    }
    if let Some(error) = &document.error {
        groups.push(error_lines(
            error,
            document.interpretation.ungrounded.as_ref(),
        ));
    }
    if let Some(request) = &document.request {
        groups.push(request_lines(request));
    }
    if !document.origins.is_empty() {
        groups.push(origin_lines(&document.origins));
    }
    if !document.assumptions.is_empty() {
        groups.push(assumption_lines(&document.assumptions));
    }
    if let Some(extraction) = &document.interpretation.extraction {
        groups.push(evidence_lines(extraction));
    }
    groups.push(template_lines(document.template.as_ref()));
    groups.push(model_lines(
        &document.interpretation.backend,
        document.interpretation.prompt_version,
    ));
    groups.push(sent_lines(document.interpretation.messages.as_ref()));
    if let Some(reply) = &document.interpretation.reply {
        groups.push(reply_lines(reply));
    }
    let mut lines: Vec<String> = Vec::new();
    for group in groups {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.extend(group);
    }
    lines
}

/// The overview (plan §5.3a M7-22, 7i4): the template, the target, the operation's
/// warnings, and one row per member with the member at `selected` marked `> `. No
/// operation line, no warning count on a row, and no key line for the problem or
/// provenance screen, because the screen's own key line offers both (7i4).
/// `member` rows keep the order `results` holds. Nothing here is computed: a row's
/// outcome is that member's `design` or the code of its `design_failure`, and
/// `members` is the number of rows.
pub fn overview(document: &ResultDocument, selected: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    // M7-25: the template and the target are named in two columns, the id padded
    // to the longer of the two; an absent one is `none`.
    let template_id: String = document
        .provenance
        .template
        .as_ref()
        .map(|template| template.id.clone())
        .unwrap_or("none".to_string());
    let target_id: String = document
        .target
        .as_ref()
        .map(|target| target.id.clone())
        .unwrap_or("none".to_string());
    let id_width: usize = template_id.chars().count().max(target_id.chars().count());

    let template_line: String = match &document.provenance.template {
        Some(template) => format!(
            "Template  {:width$}  {}, {} bp",
            template_id,
            template.topology,
            template.length,
            width = id_width
        ),
        None => format!("Template  {:width$}  none", template_id, width = id_width),
    };
    let target_line: String = match &document.target {
        Some(target) => {
            // M7-26: a strand in words, `+` as `forward` and `-` as `reverse`.
            let strand: &str = if target.strand == "+" {
                "forward"
            } else {
                "reverse"
            };
            format!(
                "Target    {:width$}  {} strand, bases {}",
                target_id,
                strand,
                target
                    .intervals
                    .iter()
                    .map(|one| bases_at(one.0, one.1))
                    .collect::<Vec<String>>()
                    .join(", "),
                width = id_width
            )
        }
        None => format!("Target    {:width$}  none", target_id, width = id_width),
    };
    lines.push(template_line.trim_end().to_string());
    lines.push(target_line.trim_end().to_string());
    lines.push(String::new());

    // 7i4 (plan §5.3a M7-22): the overview names the template, the target and the
    // members only, so the operation is not here; the operation warnings are, as
    // a warning is never hidden (plan §3 rule 3), and M7-25 shows the message,
    // never the code.
    if !document.warnings.is_empty() {
        lines.push("Warnings".to_string());
        for warning in &document.warnings {
            lines.push(format!("  {}", operation_warning_message(warning)));
        }
        lines.push(String::new());
    }

    // M7-26: the rows are padded so the summary and the outcome line up; the
    // selected row carries `▶`, the others two spaces.
    let name_width: usize = document
        .results
        .iter()
        .map(|member| member.name.chars().count())
        .max()
        .unwrap_or(0);
    let summaries: Vec<String> = document
        .results
        .iter()
        .map(|member| mutation_summary(&member.mutation))
        .collect();
    let summary_width: usize = summaries
        .iter()
        .map(|summary| summary.chars().count())
        .max()
        .unwrap_or(0);
    lines.push(format!("Variants ({})", document.results.len()));
    for (index, member) in document.results.iter().enumerate() {
        let marker: &str = if index == selected { "▶ " } else { "  " };
        // A member states one design, a design failure, or, when the run's
        // invariant is broken, neither: then the row says `no design` rather
        // than hiding the missing design (plan §3 rule 3).
        let outcome: &str = match (&member.design, &member.design_failure) {
            (Some(_), _) => "designed",
            (None, Some(_)) => "no primer pair",
            (None, None) => "no design",
        };
        // The row ends with its outcome, and a member's own warnings are not
        // counted here: they are on the member's screen (7i4, plan §5.3a M7-22).
        lines.push(
            format!(
                "{}{:name$}  {:sum$}  {}",
                marker,
                member.name,
                summaries[index],
                outcome,
                name = name_width,
                sum = summary_width
            )
            .trim_end()
            .to_string(),
        );
    }
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

/// The export screen (7g1, plan §5.4 "Writing", M7-4): the files `y` writes,
/// each named exactly as the plan names it, so the list is the files, not a
/// description of them. With no plan, the one line says how to enable an export.
/// `rows` is the table's lines less its header, so the count is the primers. The
/// members left out of a table that exists are named, as `order` lists them.
/// `message` is the line the last export gave, after a blank line.
pub fn export(plan: Option<&ExportPlan>, message: Option<&str>) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    match plan {
        None => {
            lines.push("start luria review with --export DIR to export this result".to_string())
        }
        Some(plan) => {
            lines.push(format!("export to {}:", plan.directory.display()));
            lines.push(format!(
                "  {}  the result, as luria q5 design prints it",
                plan.result_file
            ));
            match &plan.order {
                Ok(table) => {
                    // The header is not a primer, so the count is the lines less
                    // one (M7-7).
                    let rows: usize = table.text.lines().count() - 1;
                    lines.push(format!(
                        "  {}  IDT order table, {} primers",
                        plan.order_file, rows
                    ));
                }
                // The refusal is shown with its own message, and the other
                // files are still listed, because they are written (§2).
                Err(refusal) => {
                    lines.push(format!("order table not written: {}", refusal.message()));
                }
            }
            for name in &plan.constructs {
                lines.push(format!("  {}  edited construct", name));
            }
            if let Ok(table) = &plan.order {
                if !table.left_out.is_empty() {
                    lines.push(format!(
                        "members left out of the order table: {}",
                        table.left_out.join(", ")
                    ));
                }
            }
            lines.push(String::new());
            lines.push(
                "nothing is replaced: if any of these files exists, nothing is written".to_string(),
            );
        }
    }

    if let Some(message) = message {
        lines.push(String::new());
        lines.push(message.to_string());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpret::document::{InterpretationDocument, InterpretationView};
    use crate::result::{
        AlgorithmRef, AnnealingView, BufferView, DesignView, EditedView, ErrorView, GenBankView,
        MemberView, PackageView, PairView, PartView, PrimerView, PrimerWarningView, RefusalView,
        TailView, TemplateView, TmView, UnusedFeatureView,
    };
    use crate::review::export::export_plan;
    use crate::review::order::OrderRefusal;
    use crate::review::text;
    use luria_nl::backend::BackendInfo;
    use luria_nl::extraction::{Extraction, Mutation, Operation};
    use luria_nl::prompt::PROMPT_VERSION;
    use std::path::Path;

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

    /// A Tm, written by hand: no basis substitution, so a test sets the one
    /// field it is about.
    fn tm() -> TmView {
        TmView {
            celsius: 60.0,
            basis: "matched_complement".to_string(),
            substituted_because: None,
            enthalpy_kcal: 0.0,
            entropy_cal: 0.0,
        }
    }

    /// An annealing portion written by hand, covering `span`.
    fn annealing(text: &str, span: (u64, u64)) -> AnnealingView {
        AnnealingView {
            sequence: text.to_string(),
            length: text.len() as u64,
            overlap: text.len() as u64,
            spans: vec![span],
        }
    }

    /// One part of a primer, written by hand: the `text` at the primer indices
    /// from `primer_start`, annealing or not, sitting at the template `span`.
    fn part(text: &str, annealing: bool, span: (u64, u64), primer_start: u64) -> PartView {
        PartView {
            role: "context".to_string(),
            annealing,
            sequence: text.to_string(),
            span: Some(span),
            primer_start,
            primer_end: primer_start + text.len() as u64,
        }
    }

    /// A primer, written by hand: no tail, no whole-primer Tm and no warning, so
    /// a test sets the one field it is about. Nothing here is a scientific
    /// value, as the primers a test builds are not core's.
    fn primer(role: &str, text: &str) -> PrimerView {
        let span: (u64, u64) = (0, text.len() as u64);
        PrimerView {
            role: role.to_string(),
            sequence: text.to_string(),
            length: text.len() as u64,
            span,
            edited_span: None,
            tail: None,
            parts: vec![part(text, true, span, 0)],
            annealing: annealing(text, span),
            tm: tm(),
            whole_primer_tm: None,
            warnings: Vec::new(),
        }
    }

    /// A design carrying these two primers, with the geometry of a hand-built
    /// pair: no boundary, not abutting, nothing unresolved.
    fn design(forward: PrimerView, reverse: PrimerView) -> DesignView {
        DesignView {
            policy: "luriaNative".to_string(),
            style: "q5_polymerase".to_string(),
            algorithm: AlgorithmRef {
                name: "luria-core-primer-design-native",
                version: "0.7.0",
            },
            min_annealing_overlap: 15,
            placement: "inline".to_string(),
            edit_span: (0, 3),
            forward,
            reverse,
            annealing_temperature: 60.0,
            tm_balance_met: true,
            pair: PairView {
                deleted_width: 0,
                separation: 0,
                boundary: None,
                abuts: false,
                relationship: "hand-built pair".to_string(),
            },
            unresolved: Vec::new(),
        }
    }

    /// The one-member document whose member carries `design` and no failure, so
    /// the member screen's lines are the ones to check.
    fn design_document(design: Option<DesignView>) -> ResultDocument {
        let mut document: ResultDocument = document();
        let mut member: MemberView = member("m");
        member.design = design;
        document.results = vec![member];
        document
    }

    /// How many times `expected` appears in `lines`, so "one line per entry" is
    /// tested as one, not at least one.
    fn count(lines: &[String], expected: &str) -> usize {
        lines.iter().filter(|line| *line == expected).count()
    }

    /// The row of member `index` in `lines`, which the marker and the name make
    /// identifiable.
    fn row(lines: &[String], index: usize) -> String {
        lines
            .iter()
            .filter(|line| line.starts_with("\u{25b6} ") || line.starts_with("  "))
            .nth(index)
            .expect("a member row")
            .clone()
    }

    #[test]
    fn coordinates_on_screen_are_1_based_inclusive() {
        // M7-24 (plan §5.3b): `[start, end)` zero-based shows as 1-based inclusive
        // on screen, with one base as a single number; D006 keeps the files
        // luria writes zero-based, so only these two convert.
        assert_eq!(bases_at(1209, 1212), "1210–1212");
        assert_eq!(bases_at(45, 46), "46");
        assert_eq!(bases_at(0, 1), "1");
        assert_eq!(insertion_place(1209), "after 1209");
        assert_eq!(insertion_place(0), "at the start");
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
        assert_eq!(count(&lines, "Warnings"), 1, "one `Warnings` heading");
        assert_eq!(
            count(
                &lines,
                &format!("  {}", operation_warning_message(&warning))
            ),
            1,
            "the warning's own `message`, indented by two: {:?}",
            lines
        );
        assert!(
            !lines.iter().any(|line| line.contains("many_designs")),
            "the screen shows the message, never the code: {:?}",
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
            format!("\u{25b6} {}  {}  no design", name, summary),
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

    #[test]
    fn an_insertion_shows_before_and_after() {
        let mutation: MutationView = MutationView::Insertion {
            state: "verified".to_string(),
            point: 168,
            before: "AA".to_string(),
            after: "CC".to_string(),
            written: "G".to_string(),
            coordinates: vec![168],
            edited: EditedView {
                id: "ins-G".to_string(),
                sha256: "0".to_string(),
                length: 394,
            },
        };
        let mut document: ResultDocument = design_document(None);
        document.results[0].mutation = mutation.clone();
        let lines: Vec<String> = text::member(&document, 0);
        match &mutation {
            MutationView::Insertion { before, after, .. } => {
                assert_eq!(
                    count(&lines, &format!("  before: {}", before)),
                    1,
                    "{:?}",
                    lines
                );
                assert_eq!(
                    count(&lines, &format!("  after: {}", after)),
                    1,
                    "{:?}",
                    lines
                );
                assert_eq!(
                    count(
                        &lines,
                        &format!("mutation: {}", mutation_summary(&mutation))
                    ),
                    1,
                    "the summary names the insertion the member made: {:?}",
                    lines
                );
            }
            other => panic!("an insertion: {:?}", other),
        }
        let lines: Vec<String> = text::member(&design_document(None), 0);
        assert!(
            !lines
                .iter()
                .any(|line| line.starts_with("  before: ") || line.starts_with("  after: ")),
            "a mutation that is not an insertion shows neither: {:?}",
            lines
        );
    }

    #[test]
    fn a_tail_is_shown_with_its_reason() {
        let mut forward: PrimerView = primer("forward", "CTACGGTAAG");
        let tail: TailView = TailView {
            sequence: "GATG".to_string(),
            reason: "the inserted bases are written past the annealing portion".to_string(),
        };
        forward.tail = Some(tail.clone());
        let lines: Vec<String> = text::member(
            &design_document(Some(design(forward, primer("reverse", "GTCGCAT")))),
            0,
        );
        assert_eq!(
            count(
                &lines,
                &format!("  tail: {} ({})", tail.sequence, tail.reason)
            ),
            1,
            "the tail is the primer's own text and core's own reason: {:?}",
            lines
        );
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.starts_with("  tail: "))
                .count(),
            1,
            "the primer with no tail shows no tail line: {:?}",
            lines
        );
    }

    #[test]
    fn a_primer_warning_is_listed() {
        let mut reverse: PrimerView = primer("reverse", "GTCGCAT");
        let warning: PrimerWarningView = PrimerWarningView::SingleBaseRun {
            base: "G".to_string(),
            length: 5,
            start: 0,
            message: "a run of 5 G at position 0".to_string(),
        };
        reverse.warnings = vec![warning.clone()];
        let lines: Vec<String> = text::member(
            &design_document(Some(design(primer("forward", "CTACGGTAAG"), reverse))),
            0,
        );
        assert_eq!(count(&lines, "  warnings:"), 1, "{:?}", lines);
        assert_eq!(
            count(
                &lines,
                &format!(
                    "    {}: {}",
                    "single_base_run",
                    primer_warning_message(&warning)
                )
            ),
            1,
            "the code is the serde tag in snake case, with core's own message: {:?}",
            lines
        );
    }

    #[test]
    fn a_member_edit_warning_is_listed() {
        let warning: EditWarningView = EditWarningView::Frameshift {
            cds: "synA".to_string(),
            coding_bases_removed: 1,
            wholly_inside: true,
            message: "removing 1 coding base from synA shifts its reading frame".to_string(),
        };
        let mut document: ResultDocument = design_document(None);
        document.results[0].warnings = vec![warning.clone()];
        let lines: Vec<String> = text::member(&document, 0);
        assert_eq!(count(&lines, "warnings:"), 1, "{:?}", lines);
        assert_eq!(
            count(
                &lines,
                &format!("  {}: {}", "frameshift", edit_warning_message(&warning))
            ),
            1,
            "the code is the serde tag in snake case, with core's own message: {:?}",
            lines
        );
    }

    /// A part of a primer, written by hand, with `role`: annealing or not, its
    /// own `sequence`, sitting at the template `span` (`None` is a tail), from
    /// the primer index `primer_start`.
    fn role_part(
        role: &str,
        annealing: bool,
        text: &str,
        span: Option<(u64, u64)>,
        primer_start: u64,
    ) -> PartView {
        PartView {
            role: role.to_string(),
            annealing,
            sequence: text.to_string(),
            span,
            primer_start,
            primer_end: primer_start + text.len() as u64,
        }
    }

    /// The template's bases `context` lays a hand-built design out on: 40
    /// letters, so the window of these parts fits inside it.
    fn template(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    /// The row of `label` in `lines`, which its padded label makes identifiable.
    fn context_row(lines: &[String], label: &str) -> String {
        lines
            .iter()
            .find(|line| line.starts_with(&format!("{:width$}", label, width = CONTEXT_LABEL)))
            .expect("a row with the label")
            .clone()
    }

    #[test]
    fn an_unequal_part_is_starred_and_noted() {
        // A forward primer whose mutation part writes 6 bases over a span 3
        // columns wide: the span's columns are starred, and the part's own text
        // is named in a line after the blocks.
        let mut forward: PrimerView = primer("forward", "ACGAAAAAA");
        forward.parts = vec![
            part("ACG", true, (0, 3), 0),
            role_part("mutation", false, "AAAAAA", Some((3, 6)), 3),
        ];
        let mut document: ResultDocument =
            design_document(Some(design(forward, primer("reverse", "GTCGCAT"))));
        document.provenance.template = Some(TemplateView {
            id: "sid-1".to_string(),
            sha256: "0".to_string(),
            length: 40,
            topology: "linear".to_string(),
            source: SourceView::Fasta {
                path: "sid-1_cDNA.fa".to_string(),
                header: "a template".to_string(),
            },
            genbank: None,
        });
        let letters: Vec<char> = template("ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT");
        let lines: Vec<String> = context(&document, 0, Some(&letters));
        assert_eq!(
            context_row(&lines, "forward"),
            "forward   ACG***".to_string(),
            "the span's three columns are `*`, the rest of the row is the primer's \
             own bases: {:?}",
            lines
        );
        let note: String = "forward mutation: AAAAAA at [3, 6)".to_string();
        assert_eq!(count(&lines, &note), 1, "{:?}", lines);
        let at: usize = lines
            .iter()
            .position(|line| *line == note)
            .expect("the note");
        assert_eq!(
            lines[at - 1],
            "".to_string(),
            "a blank line runs before the notes"
        );
    }

    #[test]
    fn a_tail_past_the_start_is_noted() {
        // A forward primer whose 5' tail is longer than the column its first
        // spanned part starts at: the part reaches left of the template's start,
        // so those columns are not drawn, and the tail is named after the blocks.
        let mut forward: PrimerView = primer("forward", "AAAACGT");
        forward.parts = vec![
            role_part("tail", false, "AAA", None, 0),
            role_part("context", true, "ACGT", Some((2, 6)), 3),
        ];
        let document: ResultDocument =
            design_document(Some(design(forward, primer("reverse", "GTCGCAT"))));
        let letters: Vec<char> = template("ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT");
        let lines: Vec<String> = context(&document, 0, Some(&letters));
        let row: String = context_row(&lines, "forward");
        assert_eq!(
            row,
            "forward   aaACGT".to_string(),
            "only the tail's two columns inside the template are drawn, lower case \
             because the tail does not anneal: {:?}",
            lines
        );
        assert_eq!(
            row[CONTEXT_LABEL..].chars().filter(|ch| *ch == 'a').count(),
            2,
            "only two of the tail's three bases are drawn"
        );
        assert_eq!(
            count(&lines, "forward tail: AAA extends past the template"),
            1,
            "{:?}",
            lines
        );
    }

    #[test]
    fn out_of_range_is_no_such_member() {
        for index in 0..2 {
            assert_eq!(
                text::member(&document_with(index), index),
                ["no such member".to_string()],
                "index {} of {} members is out of range",
                index,
                index
            );
        }
    }

    /// The interpretation record a proposal document needs: a scripted backend, the
    /// prompt's own version, and nothing sent, said, read or refused, so a test
    /// sets the one field it is about.
    fn an_interpretation() -> InterpretationView {
        InterpretationView {
            backend: BackendInfo {
                kind: "scripted".to_string(),
                endpoint: None,
                model: "scripted-corpus".to_string(),
            },
            prompt_version: PROMPT_VERSION,
            messages: None,
            reply: None,
            extraction: None,
            ungrounded: None,
        }
    }

    /// A proposal document, for the lines to be built from: no request, no
    /// question, no origin, no assumption, no error and no template read, so the
    /// groups a test checks are the ones its own field makes.
    fn a_proposal_document() -> InterpretationDocument {
        InterpretationDocument {
            schema: crate::INTERPRETATION_SCHEMA,
            state: "proposed",
            package: PackageView {
                name: env!("CARGO_PKG_NAME"),
                version: env!("CARGO_PKG_VERSION"),
            },
            request: None,
            questions: Vec::new(),
            origins: Vec::new(),
            assumptions: Vec::new(),
            error: None,
            template: None,
            interpretation: an_interpretation(),
        }
    }

    /// An extraction with one mutation, `token` quoted by `quote` and the codon
    /// `codon` when the user named one, every other member unstated, so the rows a
    /// test reads are the ones this mutation gives.
    fn an_extraction(token: &str, quote: &str, codon: Option<&str>) -> Extraction {
        Extraction {
            operation: Operation::Q5ResidueSubstitutions,
            cds: None,
            mutations: vec![Mutation {
                token: token.to_string(),
                quote: quote.to_string(),
                codon: codon.map(|codon| codon.to_string()),
            }],
            composition: None,
            numbering: None,
            unsupported: Vec::new(),
        }
    }

    #[test]
    fn a_stated_codon_is_shown_beside_its_quote() {
        let mut document: InterpretationDocument = a_proposal_document();
        document.interpretation.extraction = Some(an_extraction("A42G", "A42G", Some("GGC")));
        let lines: Vec<String> = proposal(&document);
        assert_eq!(
            count(&lines, "  A42G  quote \"A42G\""),
            1,
            "the quote the model gave sits beside its token: {:?}",
            lines
        );
        assert_eq!(
            count(&lines, "  codon GGC"),
            1,
            "the codon the user named is a line of its own after its quote: {:?}",
            lines
        );
        let at: usize = lines
            .iter()
            .position(|line| *line == "  A42G  quote \"A42G\"")
            .expect("the mutation row");
        assert_eq!(
            lines[at + 1],
            "  codon GGC".to_string(),
            "the codon is shown beside its quote, not in another group"
        );
        let mut unstated: InterpretationDocument = a_proposal_document();
        unstated.interpretation.extraction = Some(an_extraction("A42G", "A42G", None));
        let lines: Vec<String> = proposal(&unstated);
        assert!(
            !lines.iter().any(|line| line.starts_with("  codon "))
                && !lines.iter().any(|line| line.contains("GGC")),
            "a mutation with no codon states none: {:?}",
            lines
        );
    }

    #[test]
    fn nothing_sent_says_so() {
        let lines: Vec<String> = proposal(&a_proposal_document());
        assert_eq!(
            count(&lines, "nothing was sent to the model"),
            1,
            "a run that sent nothing says it (plan §3 rule 6): {:?}",
            lines
        );
        assert!(
            !lines.iter().any(|line| line.contains("user message")),
            "no user message is shown when none was sent: {:?}",
            lines
        );
    }

    /// The plan `main` builds from `--export out` for the request `set.json`,
    /// built by `export_plan`, which is pure and writes nothing.
    fn plan_of(document: &ResultDocument) -> ExportPlan {
        export_plan(document, Path::new("set.json"), Path::new("out"))
    }

    #[test]
    fn no_plan_says_how_to_export() {
        assert_eq!(
            text::export(None, None),
            vec!["start luria review with --export DIR to export this result".to_string()],
            "with no --export directory, the screen says how to get one"
        );

        // A plan lists every file it will write, the table's primer count, and
        // the rule that nothing is replaced.
        let mut document: ResultDocument = document_with(2);
        document.results[0].name = "del-11".to_string();
        document.results[1].name = "ins-14".to_string();
        document.provenance.template = Some(TemplateView {
            id: "sid-1".to_string(),
            sha256: "0".to_string(),
            length: 393,
            topology: "linear".to_string(),
            source: SourceView::Fasta {
                path: "sid-1_cDNA.fa".to_string(),
                header: "a template".to_string(),
            },
            genbank: None,
        });
        document.results[0].design = Some(design(
            primer("forward", &"A".repeat(15)),
            primer("reverse", &"C".repeat(15)),
        ));
        document.results[1].design_failure = Some(ErrorView {
            code: "no_primer_design_fits",
            rule: None,
            category: "unsupported",
            message: "no primer pair fits".to_string(),
            path: None,
            details: Details(Vec::new()),
        });
        let plan: ExportPlan = plan_of(&document);
        assert_eq!(
            text::export(
                Some(&plan),
                Some("export failed: set.result.json already exists")
            ),
            vec![
                "export to out:".to_string(),
                "  set.result.json  the result, as luria q5 design prints it".to_string(),
                "  set.order.tsv  IDT order table, 2 primers".to_string(),
                "  sid-1_del-11.fa  edited construct".to_string(),
                "  sid-1_ins-14.fa  edited construct".to_string(),
                "members left out of the order table: ins-14".to_string(),
                String::new(),
                "nothing is replaced: if any of these files exists, nothing is written".to_string(),
                String::new(),
                "export failed: set.result.json already exists".to_string(),
            ],
            "the files the export writes, the table it writes, the members it \
             leaves out, then the line the last export gave: {:?}",
            text::export(
                Some(&plan),
                Some("export failed: set.result.json already exists")
            )
        );
    }

    #[test]
    fn a_refused_table_is_named() {
        // The pBR322 `T99C` shape: a verified result whose one member has no
        // design, so `order` refuses and the screen says so, while the result
        // file and the construct are still listed as written.
        let mut document: ResultDocument = document_with(1);
        document.results[0].name = "T99C".to_string();
        let plan: ExportPlan = plan_of(&document);
        assert_eq!(
            text::export(Some(&plan), None),
            vec![
                "export to out:".to_string(),
                "  set.result.json  the result, as luria q5 design prints it".to_string(),
                "order table not written: no order table: no member has a primer pair".to_string(),
                "  T99C.fa  edited construct".to_string(),
                String::new(),
                "nothing is replaced: if any of these files exists, nothing is written".to_string(),
            ],
            "the refusal's own message, with no order file listed: {:?}",
            text::export(Some(&plan), None)
        );
        assert_eq!(
            plan.order,
            Err(OrderRefusal::NothingDesigned),
            "the plan holds the refusal the screen shows"
        );
    }
}
