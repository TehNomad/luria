//! Q5 site-directed mutagenesis: the accepted input scope of the workflow.
//!
//! This is the first element of the Q5 workflow boundary in
//! `docs/product-brief.md`: the workflow must identify its *accepted input
//! scope* before anything is designed. Nothing here designs a primer,
//! generates a candidate, or calculates a thermodynamic value, and nothing
//! here should: the other entries on that boundary list (primer-generation
//! method, sequence constraints, thermodynamic calculation method,
//! candidate rejection criteria) are design rules this repository has not
//! written down, and inventing them is out of scope.
//!
//! What this module establishes is the typed invocation of D014 and D016: a
//! Q5 SDM request names the DNA template artifact, the mutation intent, and
//! optionally the CDS the caller means. Validating that request answers one
//! question and nothing else: *which supported CDS does this request target?*
//!
//! The distinction that matters is between a failure and a clarification
//! (D020, D005):
//!
//! * A `DomainError` is something wrong with the request or the artifact:
//!   the artifact carries no CDS annotation at all, an annotation points at
//!   a different artifact, or the request names two different CDSs at once.
//!   No choice by the caller makes any of those representable.
//! * `NeedsClarification` is ambiguity the caller *can* resolve by choosing:
//!   the record carries several CDSs and the request did not say which one
//!   is meant. D005 forbids picking one, and D020 gives ambiguity its own
//!   lifecycle state rather than an error, so a clarification is a result.
//!
//! A clarification result performs no sequence calculation. It resolves no
//! CDS, and the gate never reads sequence content at all: it works on
//! artifact identity and annotation metadata, so the product brief's
//! "No sequence calculation performed" holds structurally rather than by
//! convention.
//!
//! `Validated` means the required inputs are present, explicit, mutually
//! consistent, and identify exactly one CDS on exactly one artifact. It
//! makes no claim about the mutation: the expected-residue check, the DNA
//! edit, and independent verification are Milestone 2's, and the numbering
//! and genetic-code rules are Milestone 1's, enforced where a CDS is built
//! and where it is translated. This module declares the workflow's scope;
//! it does not re-implement either rule.

use crate::cds::Cds;
use crate::error::CdsMismatchProblem;
use crate::error::DomainError;
use crate::error::MissingCdsAttributeProblem;
use crate::identity::Identifier;
use crate::identity::SequenceReference;
use crate::interval::require_sequence;
use crate::mutation::NucleotideMutationIntent;
use crate::mutation::ResidueMutationIntent;
use crate::sequence::Sequence;

/// Identity of this workflow's code path (D021).
pub const Q5_WORKFLOW_NAME: &str = "luria-core-q5-sdm";

/// Version of this workflow's code path (D021).
pub const Q5_WORKFLOW_VERSION: &str = "0.1.0";

/// A mutation intent as the Q5 workflow carries it (D011, D016).
///
/// Both branches are Milestone 2's intent types, unchanged. The workflow
/// names which operation it is asking for; it does not redefine either one,
/// and it does not add a third: a codon change is a `ResidueMutationIntent`
/// whose policy is `Supplied`, as Milestone 2 established.
#[derive(Clone, Debug, PartialEq)]
pub enum Q5MutationIntent {
    Nucleotide(NucleotideMutationIntent),
    Residue(ResidueMutationIntent),
}

impl Q5MutationIntent {
    /// The CDS this intent names, when it names one.
    ///
    /// A residue intent names its CDS through its `ResiduePosition`, which
    /// carries the CDS identifier as part of the position. A nucleotide
    /// intent addresses the reference sequence, so it names no CDS at all:
    /// for that intent, the scope of the request is what says which
    /// annotation the edit must fall inside.
    pub fn named_cds(&self) -> Option<Identifier> {
        match self {
            Q5MutationIntent::Nucleotide(_intent) => None,
            Q5MutationIntent::Residue(intent) => Some(intent.position().cds()),
        }
    }

    /// The artifact this intent addresses, when it addresses one.
    ///
    /// A nucleotide intent carries a `SequenceReference` as its source, so it
    /// claims a particular artifact and that claim can be checked against the
    /// request. A residue intent carries only a `ResiduePosition`, whose CDS
    /// half is an identifier with no artifact identity attached (D007 keeps
    /// artifacts apart by name *and* digest, and a bare identifier carries
    /// neither half of that pair). A residue intent therefore names no
    /// artifact of its own: its artifact binding comes from the annotation the
    /// request resolves to, which the gate checks against the template.
    pub fn source_reference(&self) -> Option<SequenceReference> {
        match self {
            Q5MutationIntent::Nucleotide(intent) => Some(intent.source()),
            Q5MutationIntent::Residue(_intent) => None,
        }
    }
}

/// A Q5 SDM request: the typed capability invocation of D014 and D016.
///
/// The product brief lists four required inputs for this workflow: the DNA
/// template, an identified target CDS, a numbering convention, and an
/// explicit mutation intent. Three are fields of this record. The target
/// CDS is optional on purpose: an absent one is the only way D005's
/// ambiguity case ("multiple supported CDSs could contain residue 42")
/// can be expressed at all, and refusing to build the record without one
/// would make that case unrepresentable rather than clarifiable.
#[derive(Clone, Debug, PartialEq)]
pub struct Q5SdmRequest {
    template: Sequence,
    scope: Option<Identifier>,
    intent: Q5MutationIntent,
}

impl Q5SdmRequest {
    /// Build a request, rejecting one that contradicts itself.
    ///
    /// Two contradictions are possible, and neither is reconciled:
    ///
    /// * the intent addresses a different artifact than the template. A
    ///   nucleotide intent carries its own `SequenceReference`, and
    ///   `NucleotideMutationIntent::create` only guarantees that reference
    ///   agrees with the intent's *own* interval, not that it agrees with the
    ///   request. Pairing such an intent with this template would otherwise
    ///   resolve a CDS and report `Validated` for an edit that belongs to
    ///   another artifact, so D007's "two artifacts are not interchangeable
    ///   because they share a display name" is enforced here by
    ///   `interval::require_sequence`, Milestone 1's rule and its
    ///   `SequenceReferenceMismatch`. A residue intent names no artifact
    ///   (see `source_reference`), so it has no claim to check.
    /// * a residue intent names one CDS and the scope names another, which is
    ///   the same contradiction Milestone 2 reports as
    ///   `ResiduePositionWrongCds`.
    pub fn create(
        template: Sequence,
        scope: Option<Identifier>,
        intent: Q5MutationIntent,
    ) -> Result<Q5SdmRequest, DomainError> {
        let claimed: Option<SequenceReference> = intent.source_reference();
        if claimed.is_some() {
            require_sequence(claimed.clone().unwrap(), &template)?;
        }

        let named: Option<Identifier> = intent.named_cds();
        if named.is_some() && scope.is_some() {
            let wanted: Identifier = scope.clone().unwrap();
            let claimed: Identifier = named.clone().unwrap();
            if wanted != claimed {
                let problem = CdsMismatchProblem {
                    expected: wanted.value().to_string(),
                    actual: claimed.value().to_string(),
                };
                return Err(DomainError::ResiduePositionWrongCds(problem));
            }
        }
        Ok(Q5SdmRequest {
            template,
            scope,
            intent,
        })
    }

    pub fn template(&self) -> Sequence {
        self.template.clone()
    }

    /// The template artifact, by identity (D007).
    pub fn template_reference(&self) -> SequenceReference {
        self.template.reference()
    }

    pub fn scope(&self) -> Option<Identifier> {
        self.scope.clone()
    }

    pub fn intent(&self) -> Q5MutationIntent {
        self.intent.clone()
    }

    /// The CDS the caller means, from either half of the request.
    ///
    /// `create` refuses a request whose two halves name different CDSs, so
    /// whichever half is present is the caller's one and only claim.
    pub fn target_cds_name(&self) -> Option<Identifier> {
        let named: Option<Identifier> = self.intent().named_cds();
        if named.is_some() {
            return named;
        }
        self.scope.clone()
    }
}

/// The workflow-level subset of the D020 product lifecycle this slice can
/// reach.
///
/// The other states belong elsewhere and are not invented here: `proposed`
/// is a natural-language interpretation, which is Milestone 5; `calculated`
/// and `verified` are mutation-domain states, whose distinction D020 makes
/// is enforced by `mutation::MutationState`; `failed` is carried by a
/// `DomainError` rather than by a result.
#[derive(Clone, Debug, PartialEq)]
pub enum Q5WorkflowState {
    NeedsClarification,
    Validated,
}

impl Q5WorkflowState {
    pub fn name(&self) -> String {
        match self {
            Q5WorkflowState::NeedsClarification => "needs_clarification".to_string(),
            Q5WorkflowState::Validated => "validated".to_string(),
        }
    }

    pub fn is_validated(&self) -> bool {
        match self {
            Q5WorkflowState::Validated => true,
            Q5WorkflowState::NeedsClarification => false,
        }
    }

    pub fn is_needs_clarification(&self) -> bool {
        match self {
            Q5WorkflowState::NeedsClarification => true,
            Q5WorkflowState::Validated => false,
        }
    }
}

/// The outcome of validating a Q5 request against an artifact's annotations.
///
/// The request is part of the result so it stays reviewable, as D004 and
/// D016 require of an interpreted request: what the application understood
/// must be inspectable alongside what it decided.
#[derive(Clone, Debug, PartialEq)]
pub struct Q5WorkflowResult {
    state: Q5WorkflowState,
    request: Q5SdmRequest,
    target: Option<Cds>,
    clarifications: Vec<String>,
    candidates: Vec<String>,
}

impl Q5WorkflowResult {
    pub fn state(&self) -> Q5WorkflowState {
        self.state.clone()
    }

    pub fn needs_clarification(&self) -> bool {
        self.state().is_needs_clarification()
    }

    /// The one CDS the request targets, when the scope resolved to one.
    pub fn target(&self) -> Option<Cds> {
        self.target.clone()
    }

    /// Why the scope did not resolve, in the caller's terms.
    pub fn clarifications(&self) -> Vec<String> {
        self.clarifications.clone()
    }

    /// Every CDS identifier the template artifact is annotated with.
    pub fn candidates(&self) -> Vec<String> {
        self.candidates.clone()
    }

    pub fn request(&self) -> Q5SdmRequest {
        self.request.clone()
    }

    /// Short label for reporting. Never used for classification.
    pub fn describe(&self) -> String {
        let state: String = self.state().name();
        let resolved: Option<Cds> = self.target();
        if resolved.is_none() {
            return format!(
                "{} against {}",
                state,
                self.request().template_reference().describe(),
            );
        }
        format!(
            "{} against {}",
            state,
            resolved.clone().unwrap().id().value()
        )
    }
}

/// Validate a Q5 request against the CDS annotations of its template (D016).
///
/// This is the whole of the accepted input scope this slice defines, and it
/// checks exactly three things:
///
/// * that the artifact carries a CDS annotation at all — an unannotated
///   record is a missing attribute, not an empty CDS, which is Milestone 1's
///   rule and its error;
/// * that every annotation names this artifact by identity, which is
///   Milestone 1's `SequenceReferenceMismatch` rule, so an annotation of a
///   different artifact is refused rather than quietly dropped from the
///   candidate list;
/// * that the request identifies exactly one of those annotations, either
///   because it named one or because the artifact has only one.
///
/// Sequence content is never read. Nothing is designed, calculated, or
/// verified here. The intent's own artifact claim, when it has one, is bound
/// to the template by `Q5SdmRequest::create` and is not restated here (D003).
pub fn validate_input_scope(
    request: Q5SdmRequest,
    annotations: Vec<Cds>,
) -> Result<Q5WorkflowResult, DomainError> {
    let template: Sequence = request.template();

    if annotations.is_empty() {
        let problem = MissingCdsAttributeProblem {
            cds: template.display_name(),
            attribute: "a target CDS".to_string(),
        };
        return Err(DomainError::MissingCdsAttribute(problem));
    }

    for annotation in annotations.iter() {
        require_sequence(annotation.reference(), &template)?;
    }

    let wanted: Option<Identifier> = request.target_cds_name();
    let mut matches: Vec<Cds> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    let mut index: usize = 0;
    while index < annotations.len() {
        let annotation: Cds = annotations[index].clone();
        names.push(annotation.id().value().to_string());
        if wanted.is_none() || annotation.id() == wanted.clone().unwrap() {
            matches.push(annotation.clone());
        }
        index += 1;
    }

    if matches.len() == 1 {
        let resolved: Cds = matches[0].clone();
        return Ok(Q5WorkflowResult {
            state: Q5WorkflowState::Validated,
            request: request.clone(),
            target: Some(resolved.clone()),
            clarifications: Vec::new(),
            candidates: names.clone(),
        });
    }

    // Zero or several candidates. Both are ambiguity the caller resolves by
    // naming a CDS, so D005 refuses to choose and D020 makes this a result
    // state rather than an error.
    let display: String = template.display_name();
    let listed: String = join_names(names.clone());
    let mut clarifications: Vec<String> = Vec::new();
    if wanted.is_none() {
        clarifications.push(format!(
            "the request does not name a CDS, and '{}' is annotated with {} of them: {}",
            display,
            matches.len(),
            listed,
        ));
    } else {
        let named: String = wanted.clone().unwrap().value().to_string();
        if matches.is_empty() {
            clarifications.push(
                format!(
                    "the request names CDS '{}', which '{}' is not annotated with; it is annotated with {}",
                    named,
                    display,
                    listed,
                ),
            );
        } else {
            clarifications.push(
                format!(
                    "more than one CDS annotation on '{}' is named {}, so the record does not identify one CDS",
                    display,
                    named,
                ),
            );
        }
    }

    Ok(Q5WorkflowResult {
        state: Q5WorkflowState::NeedsClarification,
        request: request.clone(),
        target: None,
        clarifications: clarifications.clone(),
        candidates: names.clone(),
    })
}

/// Candidate CDS identifiers as text, for a clarification the caller reads.
fn join_names(names: Vec<String>) -> String {
    let mut text: String = String::new();
    let mut index: usize = 0;
    while index < names.len() {
        if index > 0 {
            text += ", ";
        }
        text = format!("{}'{}'", text, names[index].clone());
        index += 1;
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cds::Cds;
    use crate::cds::Completeness;
    use crate::cds::Numbering;
    use crate::cds::ResiduePosition;
    use crate::error::DomainError;
    use crate::error::ErrorCategory;
    use crate::identity::Identifier;
    use crate::identity::SequenceReference;
    use crate::interval::create;
    use crate::interval::NucleotideInterval;
    use crate::interval::OrderedIntervals;
    use crate::mutation::CodonSelectionPolicy;
    use crate::mutation::NucleotideMutationIntent;
    use crate::mutation::ResidueMutationIntent;
    use crate::sequence::normalize;
    use crate::sequence::Sequence;
    use crate::sequence::SequenceTopology;
    use crate::strand::Strand;

    // "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG" is the Milestone 2 fixture text: over
    // a whole 30-nucleotide plus-strand CDS it translates to M A W N Q Q Q
    // Q Q Q, so residue 2 is A. Coordinate 3 holds G.
    fn template() -> Sequence {
        normalize(
            Identifier::create("q5-a".to_string()).unwrap(),
            "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    // A different artifact that shares the display name, so an annotation of
    // it cannot pass the identity check.
    fn decoy() -> Sequence {
        normalize(
            Identifier::create("q5-a".to_string()).unwrap(),
            "ATTCCAAGCCATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    // A third artifact with a different display name, so a mismatch is
    // detectable by name as well as by digest.
    fn other() -> Sequence {
        normalize(
            Identifier::create("q5-b".to_string()).unwrap(),
            "ATGGCTTGGAATCAGCAGCAGCAGCAGCA".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    fn whole(length: i64) -> OrderedIntervals {
        vec![create(&template().reference(), 0, length).unwrap()]
    }

    fn cds_of(name: &str, length: i64) -> Cds {
        Cds::create(
            &template(),
            Identifier::create(name.to_string()).unwrap(),
            whole(length),
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap()
    }

    fn cds_of_decoy(name: &str, length: i64) -> Cds {
        let intervals: OrderedIntervals = vec![create(&decoy().reference(), 0, length).unwrap()];
        Cds::create(
            &decoy(),
            Identifier::create(name.to_string()).unwrap(),
            intervals,
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap()
    }

    fn cds_name(name: &str) -> Identifier {
        Identifier::create(name.to_string()).unwrap()
    }

    fn nucleotide_intent() -> Q5MutationIntent {
        nucleotide_intent_on(template())
    }

    // A nucleotide intent that addresses `artifact` rather than the template:
    // the intent is well formed on its own, so `NucleotideMutationIntent::create`
    // accepts it. Only the request can say whether it belongs here.
    fn nucleotide_intent_on(artifact: Sequence) -> Q5MutationIntent {
        let interval: OrderedIntervals = vec![create(&artifact.reference(), 3, 4).unwrap()];
        let position: NucleotideInterval = interval[0].clone();
        Q5MutationIntent::Nucleotide(
            NucleotideMutationIntent::create(
                artifact.reference(),
                position,
                "G".to_string(),
                "C".to_string(),
            )
            .unwrap(),
        )
    }

    fn residue_intent(cds: &str) -> Q5MutationIntent {
        let position: ResiduePosition = ResiduePosition::create(cds_name(cds), 2).unwrap();
        Q5MutationIntent::Residue(
            ResidueMutationIntent::create(
                position,
                "A".to_string(),
                "W".to_string(),
                CodonSelectionPolicy::Default,
            )
            .unwrap(),
        )
    }

    #[test]
    fn a_request_that_names_one_of_several_cdss_resolves_to_it() {
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), Some(cds_name("cds-2")), nucleotide_intent()).unwrap();
        let result: Q5WorkflowResult =
            validate_input_scope(request, vec![cds_of("cds-1", 30), cds_of("cds-2", 12)]).unwrap();
        assert!(result.state() == Q5WorkflowState::Validated);
        let resolved: Cds = result.target().unwrap();
        assert!(resolved.id() == cds_name("cds-2"));
        assert!(resolved.total_length() == 12);
        assert!(result.clarifications().is_empty());
    }

    #[test]
    fn an_unnamed_cds_on_an_artifact_with_several_cdss_needs_clarification() {
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, nucleotide_intent()).unwrap();
        let result: Q5WorkflowResult =
            validate_input_scope(request, vec![cds_of("cds-1", 30), cds_of("cds-2", 12)]).unwrap();
        assert!(result.state() == Q5WorkflowState::NeedsClarification);
        assert!(result.needs_clarification());
        assert!(result.target().is_none());
        assert!(result.clarifications().len() == 1);
        assert!(result.candidates() == vec!["cds-1".to_string(), "cds-2".to_string()]);
    }

    #[test]
    fn an_artifact_annotated_with_one_cds_needs_no_naming() {
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, residue_intent("cds-1")).unwrap();
        let result: Q5WorkflowResult =
            validate_input_scope(request, vec![cds_of("cds-1", 30)]).unwrap();
        assert!(result.state() == Q5WorkflowState::Validated);
        let resolved: Cds = result.target().unwrap();
        assert!(resolved.id() == cds_name("cds-1"));
    }

    #[test]
    fn a_request_naming_an_unannotated_cds_needs_clarification() {
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), Some(cds_name("cds-9")), nucleotide_intent()).unwrap();
        let result: Q5WorkflowResult =
            validate_input_scope(request, vec![cds_of("cds-1", 30), cds_of("cds-2", 12)]).unwrap();
        assert!(result.state() == Q5WorkflowState::NeedsClarification);
        assert!(result.target().is_none());
        // The clarification names what is available, so the caller can fix it.
        assert!(result.clarifications()[0].contains("cds-9"));
        assert!(result.clarifications()[0].contains("'cds-1'"));
        assert!(result.candidates() == vec!["cds-1".to_string(), "cds-2".to_string()]);
    }

    #[test]
    fn duplicate_cds_identifiers_are_not_deduplicated() {
        // Two annotations sharing an identifier do not identify one CDS, so
        // the record is ambiguous rather than quietly reduced to one.
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), Some(cds_name("cds-1")), nucleotide_intent()).unwrap();
        let result: Q5WorkflowResult =
            validate_input_scope(request, vec![cds_of("cds-1", 30), cds_of("cds-1", 12)]).unwrap();
        assert!(result.state() == Q5WorkflowState::NeedsClarification);
        assert!(result.target().is_none());
        assert!(result.candidates() == vec!["cds-1".to_string(), "cds-1".to_string()]);
    }

    #[test]
    fn an_unannotated_artifact_is_a_missing_attribute() {
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, nucleotide_intent()).unwrap();
        let error: DomainError = validate_input_scope(request, Vec::new()).unwrap_err();
        assert!(matches!(error, DomainError::MissingCdsAttribute(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    #[test]
    fn an_annotation_of_another_artifact_is_refused() {
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), Some(cds_name("cds-1")), nucleotide_intent()).unwrap();
        let error: DomainError =
            validate_input_scope(request, vec![cds_of_decoy("cds-1", 18)]).unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn a_scope_and_a_residue_intent_naming_different_cdss_are_refused() {
        let built: Result<Q5SdmRequest, DomainError> =
            Q5SdmRequest::create(template(), Some(cds_name("cds-1")), residue_intent("cds-2"));
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::ResiduePositionWrongCds(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn a_scope_that_agrees_with_the_residue_intent_is_accepted() {
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), Some(cds_name("cds-1")), residue_intent("cds-1"))
                .unwrap();
        let result: Q5WorkflowResult =
            validate_input_scope(request, vec![cds_of("cds-1", 30), cds_of("cds-2", 12)]).unwrap();
        assert!(result.state() == Q5WorkflowState::Validated);
        let resolved: Cds = result.target().unwrap();
        assert!(resolved.id() == cds_name("cds-1"));
    }

    #[test]
    fn a_clarification_resolves_no_cds_and_reads_no_bases() {
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, nucleotide_intent()).unwrap();
        let result: Q5WorkflowResult =
            validate_input_scope(request, vec![cds_of("cds-1", 30), cds_of("cds-2", 12)]).unwrap();
        assert!(result.needs_clarification());
        assert!(result.target().is_none());
        // The template is still identified by identity, so the clarification
        // says which artifact it was about (D007).
        assert!(result.request().template_reference() == template().reference());
    }

    #[test]
    fn needs_clarification_and_validated_are_distinct_states() {
        let unnamed: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, nucleotide_intent()).unwrap();
        let ambiguous: Q5WorkflowResult =
            validate_input_scope(unnamed, vec![cds_of("cds-1", 30), cds_of("cds-2", 12)]).unwrap();
        let named: Q5SdmRequest =
            Q5SdmRequest::create(template(), Some(cds_name("cds-2")), nucleotide_intent()).unwrap();
        let decided: Q5WorkflowResult =
            validate_input_scope(named, vec![cds_of("cds-1", 30), cds_of("cds-2", 12)]).unwrap();
        assert!(ambiguous.state() != decided.state());
        assert!(ambiguous != decided);
        assert!(ambiguous.needs_clarification());
        assert!(decided.state().is_validated());
    }

    #[test]
    fn a_validated_result_records_the_artifact_it_resolved_against() {
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, residue_intent("cds-1")).unwrap();
        let result: Q5WorkflowResult =
            validate_input_scope(request.clone(), vec![cds_of("cds-1", 30)]).unwrap();
        // The resolved CDS names the very artifact the request was about, by
        // name and digest, so a later calculation can cite it (D021).
        let resolved: Cds = result.target().unwrap();
        assert!(resolved.reference() == request.template_reference());
        assert!(result.request().template_reference() == template().reference());
    }

    #[test]
    fn a_nucleotide_intent_on_another_artifact_of_the_same_name_is_refused() {
        let built: Result<Q5SdmRequest, DomainError> =
            Q5SdmRequest::create(template(), None, nucleotide_intent_on(decoy()));
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn a_nucleotide_intent_on_another_artifact_never_reaches_a_validated_result() {
        let built: Result<Q5SdmRequest, DomainError> = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            nucleotide_intent_on(other()),
        );
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
    }

    #[test]
    fn a_residue_intent_binds_to_the_template_through_its_annotation() {
        // A residue position names a CDS identifier, not an artifact, so its
        // artifact binding comes from the annotation the request resolves to.
        // An annotation of a different artifact is refused, so the residue
        // path cannot reach a validated result either.
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, residue_intent("cds-1")).unwrap();
        let error: DomainError =
            validate_input_scope(request, vec![cds_of_decoy("cds-1", 18)]).unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn a_residue_intent_names_no_artifact_of_its_own() {
        let residue: Q5MutationIntent = residue_intent("cds-1");
        assert!(residue.source_reference().is_none());
        assert!(residue.named_cds() == Some(cds_name("cds-1")));
        let nucleotide: Q5MutationIntent = nucleotide_intent();
        let claimed: SequenceReference = nucleotide.source_reference().unwrap();
        assert!(claimed == template().reference());
    }

    #[test]
    fn an_intent_that_agrees_with_the_template_is_accepted() {
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, nucleotide_intent_on(template())).unwrap();
        let result: Q5WorkflowResult =
            validate_input_scope(request.clone(), vec![cds_of("cds-1", 30)]).unwrap();
        assert!(result.state() == Q5WorkflowState::Validated);
        assert!(request.template_reference() == template().reference());
    }
}
