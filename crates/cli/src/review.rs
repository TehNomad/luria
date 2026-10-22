//! `luria review` (milestone 7a): run a request as `luria q5 design` does and
//! show its result on the terminal screen. The state and the experimental
//! validation shown are the document's own strings (plan §3 rule 1: the view
//! invents nothing), and every result screen shows both (plan §3 rule 3).
//!
//! The layout is `interpret.rs`'s: `app` is the state and its key rule,
//! `view` is the drawing, and `terminal` is the only file with I/O (plan §5.1),
//! so the screen's content is testable without a terminal. `terminal` is not
//! unit-tested; it keeps to the terminal session and the event loop.

use crate::args::UsageError;
use crate::interpret::document::InterpretationDocument;
use crate::interpret::run::{request_location_problem, request_text, write_request_file};
use crate::prepare::prepare;
use crate::result::{
    CdsView, EditedView, MemberView, MutationView, RequestEcho, ResultDocument, TemplateView,
};
use crate::run::{run_with_constructs, Outcome};
use luria_core::cds::Cds;
use luria_core::frame::mutant_residues;
use luria_core::frame::EditSpan;
use luria_core::frame::FrameResidue;
use luria_core::sequence::Sequence;

use std::path::Path;

pub mod app;
pub mod export;
pub mod order;
pub mod proposal;
pub mod terminal;
pub mod text;
pub mod view;

#[derive(Clone, Debug, PartialEq)]
/// What an approved proposal gave: the result of running the request the
/// approval wrote, and, for 7d's sequence-context screen, the template's bases
/// when they match the result (D005).
pub struct Approved {
    pub document: ResultDocument,
    pub template: Option<Vec<char>>,
    /// Every member's mutant protein, in `document.results` order (7i2, plan
    /// §5.3a): one entry per member, each the residues core read, or the reason
    /// they cannot be read.
    pub frames: Vec<MemberFrame>,
}

/// Write the proposed request at `request` and run it, as `luria q5 design`
/// would run that file: `run_with_output(request, None)`, so the result's
/// request digest names a file on disk (plan §3 rule 4).
///
/// Refuses, writing nothing, when the document is not `proposed` or names no
/// request, and when the location is not usable now: `request_location_problem`
/// is checked again here, because a file may have appeared since the screen
/// opened, and `write_request_file` never replaces one (C-1). The `Err` is one
/// line for the screen.
pub fn approve(
    document: &InterpretationDocument,
    template: &Path,
    request: &Path,
) -> Result<Approved, String> {
    // The plan §3 rule 4 check, the same condition `handle` and `view` use.
    if !proposal::approvable(document) {
        return Err("there is no proposed request to approve".to_string());
    }
    let echo: &RequestEcho = document
        .request
        .as_ref()
        .expect("a proposed document names its request");

    // The location, checked again: the rules 7f1 states, with the state of the
    // file system as it is now.
    if let Some((_code, message)) = request_location_problem(template, request) {
        return Err(message);
    }

    // A new file only, written atomically: a name `request_location_problem`
    // accepted can still be taken by the time this writes.
    if let Err(reason) = write_request_file(request, &request_text(echo)) {
        let name: &str = request
            .file_name()
            .expect("a file name")
            .to_str()
            .expect("a UTF-8 file name");
        return Err(format!("could not write {}: {}", name, reason));
    }

    Ok(run_approved(request))
}

/// The design run from a request file on disk, the code `approve` does after its
/// write, shared with `new`'s approval (7k5): `run_with_constructs`, so the
/// result's request digest names a file on disk (plan §3 rule 4), and the
/// template read again, so the context screen draws the sequence this result was
/// computed on and no other (7d, D005). The run's own edited constructs are kept
/// beside it, so the member screen shows the protein core made for each member
/// and not a translation of its own (7i2, plan §5.3a, M7-21).
pub(crate) fn run_approved(request: &Path) -> Approved {
    let (outcome, constructs): (Outcome, Vec<(String, Sequence)>) = run_with_constructs(request);
    let (letters, frames): (Option<Vec<char>>, Vec<MemberFrame>) = match prepare(request) {
        Ok(prepared) => (
            matching_template(&outcome.document, &prepared.template),
            member_frames(
                &outcome.document,
                &prepared.template,
                &prepared.annotations,
                &constructs,
            ),
        ),
        Err(_problem) => (None, Vec::new()),
    };
    Approved {
        document: outcome.document,
        template: letters,
        frames,
    }
}

/// One member's mutant protein: each residue core read, from
/// `frame::mutant_residues` over the whole edited construct (plan §5.3a,
/// M7-18, M7-21). The `Err` is the reason the frame cannot be read, in one
/// line for the screen.
pub type MemberFrame = Result<Vec<FrameResidue>, String>;

/// The `edited` block of a member's mutation: every `MutationView` variant
/// carries it, so this is the digest rule applied to a member of any kind.
fn edited_of(mutation: &MutationView) -> &EditedView {
    match mutation {
        MutationView::Substitution { edited, .. }
        | MutationView::Residue { edited, .. }
        | MutationView::Deletion { edited, .. }
        | MutationView::Insertion { edited, .. }
        | MutationView::Replacement { edited, .. } => edited,
    }
}

/// The edit, in template coordinates, as the member's mutation states it
/// (rule 6 of §2.2): an interval `[start, end)` with its written length, an
/// insertion as the point with its written length, and a residue substitution
/// as the width of its coordinates, which keeps the length (D040).
pub(crate) fn edit_span(mutation: &MutationView) -> Result<EditSpan, String> {
    let span: EditSpan = match mutation {
        MutationView::Substitution {
            interval, written, ..
        }
        | MutationView::Replacement {
            interval, written, ..
        } => EditSpan {
            start: interval.0,
            end: interval.1,
            written_length: written.len() as u64,
        },
        MutationView::Deletion { interval, .. } => EditSpan {
            start: interval.0,
            end: interval.1,
            written_length: 0,
        },
        MutationView::Insertion { point, written, .. } => EditSpan {
            start: *point,
            end: *point,
            written_length: written.len() as u64,
        },
        MutationView::Residue { coordinates, .. } => {
            if coordinates.is_empty() {
                return Err("no edit coordinates".to_string());
            }
            // `coordinates` are template positions in D040's order, so the
            // edited region is the least of them to one past the greatest.
            let start: u64 = *coordinates
                .iter()
                .min()
                .expect("a non-empty list has a least");
            let end: u64 = *coordinates
                .iter()
                .max()
                .expect("a non-empty list has a greatest")
                + 1;
            EditSpan {
                start,
                end,
                written_length: end - start,
            }
        }
    };
    Ok(span)
}

/// One member's mutant protein, by the rules of §2.2, in order: the first that
/// applies gives `Err` with its own line. The construct core made for this
/// member is read, and only when its digest is the one the result reports and
/// the template's digest is the one the result was computed on (D005, fail
/// closed): the CDS is taken from the template's annotations for its
/// coordinates and strand alone.
fn member_frame(
    document: &ResultDocument,
    template: &Sequence,
    annotations: &[Cds],
    member: &MemberView,
    constructs: &[(String, Sequence)],
) -> MemberFrame {
    let target: &CdsView = document
        .target
        .as_ref()
        .ok_or_else(|| "no target CDS, so no protein".to_string())?;
    let cds: &Cds = annotations
        .iter()
        .find(|annotated| annotated.id().value() == target.id)
        .ok_or_else(|| "the target CDS is not among the template's annotations".to_string())?;
    let construct: &Sequence = constructs
        .iter()
        .find(|(name, _)| name == &member.name)
        .map(|(_, sequence)| sequence)
        .ok_or_else(|| "no edited construct".to_string())?;
    if construct.content_hash().hex_digest() != edited_of(&member.mutation).sha256 {
        return Err("the edited construct differs from the one this result reports".to_string());
    }
    let named: &TemplateView = document.provenance.template.as_ref().ok_or_else(|| {
        "the template differs from the one this result was computed on".to_string()
    })?;
    if named.sha256 != template.content_hash().hex_digest() {
        return Err("the template differs from the one this result was computed on".to_string());
    }
    let span: EditSpan = edit_span(&member.mutation)?;
    match mutant_residues(construct, cds, span, 0, construct.length()) {
        Ok(residues) => Ok(residues),
        Err(error) => Err(error.message()),
    }
}

/// Every member's mutant protein, in `document.results` order (plan §5.3a).
/// Nothing is translated here: each frame is core's `frame::mutant_residues`
/// over the construct core produced for that member, read over the whole
/// construct (M7-21). No codon, no translation, no column is computed.
pub fn member_frames(
    document: &ResultDocument,
    template: &Sequence,
    annotations: &[Cds],
    constructs: &[(String, Sequence)],
) -> Vec<MemberFrame> {
    document
        .results
        .iter()
        .map(|member| member_frame(document, template, annotations, member, constructs))
        .collect()
}

/// The template's bases when `sequence` is the one the result was computed on:
/// its digest equals `document.provenance.template.sha256`. `None` when the
/// document names no template, or the digests differ (D005), in which case the
/// sequence-context screen says so and draws no sequence (7d).
pub fn matching_template(document: &ResultDocument, sequence: &Sequence) -> Option<Vec<char>> {
    let named: &TemplateView = document.provenance.template.as_ref()?;
    // The digest the document reports is the one `Sequence::content_hash` gives
    // and `prepare` compares `template.sha256` with (5e2).
    if named.sha256 != sequence.content_hash().hex_digest() {
        return None;
    }
    Some(sequence.letters())
}

/// Why `luria review` cannot open its screen here, or `None` when it can
/// (plan §3 rule 7). Both stdin (keys) and stdout (the screen) must be
/// terminals. Decided before the request is read.
pub fn terminal_problem(stdin_is_terminal: bool, stdout_is_terminal: bool) -> Option<UsageError> {
    if stdin_is_terminal && stdout_is_terminal {
        return None;
    }
    Some(UsageError {
        message: "'review' needs a terminal on stdin and stdout; 'q5 design' writes the same result as JSON".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use luria_core::identity::Identifier;
    use luria_core::sequence::{normalize, SequenceTopology};

    /// The message §3.2 names, exactly.
    const MESSAGE: &str =
        "'review' needs a terminal on stdin and stdout; 'q5 design' writes the same result as JSON";

    /// A document whose provenance names the template `digest`, so the one
    /// field `matching_template` reads is the one under test.
    fn document_with_template(digest: &str) -> ResultDocument {
        let mut document: ResultDocument = ResultDocument {
            schema: crate::RESULT_SCHEMA,
            state: "verified",
            experimental_validation: "not_performed",
            provenance: crate::result::Provenance {
                package: crate::result::PackageView {
                    name: env!("CARGO_PKG_NAME"),
                    version: env!("CARGO_PKG_VERSION"),
                },
                algorithms: Vec::new(),
                request_sha256: None,
                operation: None,
                template: None,
                buffer: None,
            },
            request: None,
            target: None,
            clarifications: Vec::new(),
            candidates: Vec::new(),
            warnings: Vec::new(),
            results: Vec::new(),
            error: None,
        };
        document.provenance.template = Some(crate::result::TemplateView {
            id: "sid-1".to_string(),
            sha256: digest.to_string(),
            length: 4,
            topology: "linear".to_string(),
            source: crate::result::SourceView::Fasta {
                path: "sid-1_cDNA.fa".to_string(),
                header: "a template".to_string(),
            },
            genbank: None,
        });
        document
    }

    /// A DNA sequence, built by core's own normalizer, so its digest is core's
    /// and the letters `matching_template` returns are the ones asked for.
    fn sequence(text: &str) -> Sequence {
        normalize(
            Identifier::create("sid-1".to_string()).expect("an identifier"),
            text.to_string(),
            SequenceTopology::Linear,
        )
        .expect("a DNA sequence")
    }

    #[test]
    fn matching_template_needs_the_same_digest() {
        let letters: Sequence = sequence("ACGT");
        let digest: String = letters.content_hash().hex_digest().to_string();
        assert_eq!(
            matching_template(&document_with_template(&digest), &letters),
            Some(vec!['A', 'C', 'G', 'T']),
            "the sequence whose digest the document reports gives its bases"
        );

        let changed: Sequence = sequence("CCGT");
        assert_eq!(
            matching_template(&document_with_template(&digest), &changed),
            None,
            "a sequence with one base changed is not the template the result \
             was computed on"
        );

        let mut no_template: ResultDocument = document_with_template(&digest);
        no_template.provenance.template = None;
        assert_eq!(
            matching_template(&no_template, &letters),
            None,
            "a document naming no template has no template to show"
        );
    }

    #[test]
    fn review_needs_a_terminal_on_both_streams() {
        for (stdin, stdout) in [(false, true), (true, false), (false, false)] {
            assert_eq!(
                terminal_problem(stdin, stdout),
                Some(UsageError {
                    message: MESSAGE.to_string()
                }),
                "stdin {} stdout {}",
                stdin,
                stdout
            );
        }
        assert_eq!(terminal_problem(true, true), None);
    }
}
