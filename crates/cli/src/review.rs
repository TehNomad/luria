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
use crate::result::{ResultDocument, TemplateView};
use luria_core::sequence::Sequence;

pub mod app;
pub mod order;
pub mod terminal;
pub mod text;
pub mod view;

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
