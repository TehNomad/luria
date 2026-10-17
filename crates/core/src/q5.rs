//! Q5 site-directed mutagenesis: the accepted input scope of the workflow.
//!
//! This module owns the typed Q5 requests (`Q5SdmRequest`, `Q5MutationIntent`,
//! `ResidueIntentWithoutCds`, `Q5VariantSetRequest`, `Q5Variant`), the
//! target-resolution gate (`validate_input_scope`), the run results
//! (`Q5WorkflowResult`, `Q5WorkflowExecution`, `Q5VariantResult`,
//! `Q5VariantSetExecution`) and the edit warnings (`Q5EditWarning`).
//!
//! `execute` runs one request through D015's deterministic vertical slice —
//! gate, edit, independent verification, structured result — and `execute_set`
//! runs the members of a `SeparateVariants` set (D011, D016).
//! `Q5VariantSetRequest::create` refuses a `CombinedConstruct` with
//! `UnsupportedComposition`: composing mutations into one construct needs
//! overlap rules no source states, and D014 requires every design rule to
//! name a documented source. Primer design is composed in `q5_design.rs`,
//! which pairs this module with `primer_design.rs` (Tm in `tm.rs`, D029–D033);
//! nothing here designs a primer, generates a candidate or calculates Tm.
//!
//! A failure and a clarification are different (D020, D005). A `DomainError`
//! is something wrong with the request or the artifact — no CDS annotation at
//! all, an annotation of another artifact, two different CDS claims at once —
//! and no choice by the caller makes those representable. `NeedsClarification`
//! is ambiguity the caller can resolve by naming a CDS: D005 forbids picking
//! one and D020 gives ambiguity its own lifecycle state, so it is a result.
//! It resolves no CDS, performs no edit and reads no sequence content, so the
//! brief's "No sequence calculation performed" holds structurally. `Validated`
//! means the inputs are present, explicit, mutually consistent, and identify
//! exactly one CDS on exactly one artifact; it claims nothing about the
//! mutation.
//!
//! Rules this module decides, with their sources (the owner settled §10
//! items 1–6 on 2026-10-14):
//!
//! * A DNA edit is checked only against the scope the request stated; an
//!   unscoped DNA edit outside the resolved CDS proceeds (§4, §10 item 1).
//! * Candidates are every annotation on the artifact, not narrowed by whether
//!   one could hold a requested residue (§3, §5, §10 item 2); a named CDS
//!   matching no annotation is a clarification (§6, §10 item 3); one
//!   annotation is the target when the caller names none (§5, §10 item 4);
//!   two members sharing a result identifier are refused at construction with
//!   `InvalidMutationRequest`, compared exactly (§11, §10 item 6).
//! * A residue request whose expected original equals its replacement is legal
//!   when the codon changes (§10 item 5).
//! * A frameshift edit is kept and warned: deletion D031, insertion D032,
//!   replacement D033's addenda — owner decisions of 2026-10-15. A CDS here
//!   has no intron: `interval::validate_structure` refuses gaps between
//!   intervals.
//!
//! Other modules own the rest: the expected-original check, the DNA edit and
//! independent verification are `mutation::apply_*` and `mutation::verify`;
//! numbering and genetic-code rules are enforced where a CDS is built and
//! where it is translated; primer geometry and Tm are `primer_design.rs` and
//! `tm.rs` (D029–D033).

use crate::cds::Cds;
use crate::cds::ResiduePosition;
use crate::error::CdsMismatchProblem;
use crate::error::DomainError;
use crate::error::InvalidMutationRequestProblem;
use crate::error::MissingCdsAttributeProblem;
use crate::error::UnsupportedCompositionProblem;
use crate::genetic_code::Residue;
use crate::identity::Identifier;
use crate::identity::SequenceReference;
use crate::interval::create;
use crate::interval::holds_point;
use crate::interval::holds_position;
use crate::interval::holds_region;
use crate::interval::require_sequence;
use crate::interval::NucleotideInterval;
use crate::interval::OrderedIntervals;
use crate::mutation::apply_deletion;
use crate::mutation::apply_insertion;
use crate::mutation::apply_nucleotide;
use crate::mutation::apply_replacement;
use crate::mutation::apply_residue;
use crate::mutation::parse_residue;
use crate::mutation::verify;
use crate::mutation::CodonSelectionPolicy;
use crate::mutation::MutationResult;
use crate::mutation::MutationState;
use crate::mutation::NucleotideDeletionIntent;
use crate::mutation::NucleotideInsertionIntent;
use crate::mutation::NucleotideMutationIntent;
use crate::mutation::NucleotideReplacementIntent;
use crate::mutation::ResidueMutationIntent;
use crate::sequence::Sequence;
use crate::translation::translate;

/// Identity of this workflow's code path (D021).
pub const Q5_WORKFLOW_NAME: &str = "luria-core-q5-sdm";

/// Version of this workflow's code path (D021).
pub const Q5_WORKFLOW_VERSION: &str = "0.5.0";

/// A mutation intent as the Q5 workflow carries it (D011, D016).
///
/// Every branch is the mutation domain's intent type unchanged: the workflow
/// names which operation it is asking for, it does not redefine any one of
/// them, and it adds no mutation kind — a codon change is a
/// `ResidueMutationIntent` whose policy is `Supplied`. Four branches address
/// the reference sequence by coordinate rather than a CDS by residue: a
/// substitution, a deletion, an insertion and a replacement. An insertion
/// addresses the gap between two bases, so it asserts the base on each side
/// of that gap rather than the bases it replaces (D032); a replacement
/// asserts the whole text it overwrites and writes a text of any length in
/// its place (D033).
///
/// `ResidueWithoutCds` is not a separate operation. It is the residue
/// substitution the product brief's fail-closed example needs ("mutate
/// residue 42 to glycine"), missing exactly one field: the CDS the residue
/// number counts over. D006 requires a residue *position* to identify its
/// CDS, so `ResiduePosition` stays total and the "no CDS yet" state exists
/// only here, in the workflow's intent carrier. Which CDS a residue request
/// is about is target resolution's question, not a claim the intent makes
/// (see `named_cds`).
#[derive(Clone, Debug, PartialEq)]
pub enum Q5MutationIntent {
    Nucleotide(NucleotideMutationIntent),
    Deletion(NucleotideDeletionIntent),
    Insertion(NucleotideInsertionIntent),
    Replacement(NucleotideReplacementIntent),
    Residue(ResidueMutationIntent),
    ResidueWithoutCds(ResidueIntentWithoutCds),
}

impl Q5MutationIntent {
    /// The CDS this intent names, when it names one.
    ///
    /// Only a named residue intent names one, through the CDS identifier its
    /// `ResiduePosition` carries as part of the position. The other five
    /// variants name none: the four DNA-level intents address the reference
    /// sequence by coordinate, and `ResidueWithoutCds` is a residue number
    /// the caller has not tied to any CDS. For those, the request's `scope`
    /// is the only half that can identify a target, and an absent one states
    /// no preference — which is what makes the brief's ambiguous residue
    /// request a clarification rather than an unrepresentable request.
    pub fn named_cds(&self) -> Option<Identifier> {
        match self {
            Q5MutationIntent::Nucleotide(_intent) => None,
            Q5MutationIntent::Deletion(_intent) => None,
            Q5MutationIntent::Insertion(_intent) => None,
            Q5MutationIntent::Replacement(_intent) => None,
            Q5MutationIntent::Residue(intent) => Some(intent.position().cds()),
            Q5MutationIntent::ResidueWithoutCds(_intent) => None,
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
            Q5MutationIntent::Deletion(intent) => Some(intent.source()),
            Q5MutationIntent::Insertion(intent) => Some(intent.source()),
            Q5MutationIntent::Replacement(intent) => Some(intent.source()),
            Q5MutationIntent::Residue(_intent) => None,
            Q5MutationIntent::ResidueWithoutCds(_intent) => None,
        }
    }
}

/// A residue substitution the caller has not tied to a CDS (D011, D016).
///
/// This carries the residue assertion of D011 — the residue number, the
/// expected original residue, the replacement residue, and the codon
/// policy — and deliberately carries no CDS: the caller did not supply
/// one, and inventing one is D005's to refuse. It is a proposal, not an
/// operation: `ResidueMutationIntent` is what an operation needs, and it
/// cannot be built until a CDS identifier exists.
///
/// The residue letters are validated at construction, through
/// `mutation::parse_residue`, so no residue rule is restated here. The
/// residue *number* is not: `ResiduePosition::create` is the rule that
/// rejects a number below 1, and its payload names the CDS the position
/// belongs to, which does not exist until `bind` resolves one. A bad
/// number is therefore refused at bind time, before any edit, with
/// `InvalidResiduePosition` naming the CDS the request was resolved
/// against.
///
/// Neither this carrier nor `bind` rejects a request whose expected
/// original equals its replacement: such a request is legal when the codon
/// the policy selects differs from the codon already at that residue, which
/// is how a silent mutation is made (`docs/q5-sdm.md` §10 item 5, owner
/// 2026-10-14). A DNA-level no-op — the selected codon equal to the codon
/// already there — is refused by `mutation::apply_residue` with
/// `InvalidMutationRequest`.
#[derive(Clone, Debug, PartialEq)]
pub struct ResidueIntentWithoutCds {
    number: i64,
    expected_original: Residue,
    replacement: Residue,
    policy: CodonSelectionPolicy,
}

impl ResidueIntentWithoutCds {
    /// Build one from the caller's residue letters and number.
    pub fn create(
        number: i64,
        expected_original: String,
        replacement: String,
        policy: CodonSelectionPolicy,
    ) -> Result<ResidueIntentWithoutCds, DomainError> {
        let expected: Residue = parse_residue(expected_original.clone())?;
        let desired: Residue = parse_residue(replacement.clone())?;
        Ok(ResidueIntentWithoutCds {
            number,
            expected_original: expected,
            replacement: desired,
            policy,
        })
    }

    pub fn number(&self) -> i64 {
        self.number
    }

    pub fn expected_original(&self) -> Residue {
        self.expected_original.clone()
    }

    pub fn replacement(&self) -> Residue {
        self.replacement.clone()
    }

    pub fn policy(&self) -> CodonSelectionPolicy {
        self.policy.clone()
    }

    /// Short label for reporting. Never used for classification.
    pub fn describe(&self) -> String {
        let label: String = "residue ".to_string();
        label + &self.number().to_string()
    }

    /// Turn this proposal into Milestone 2's intent, against a CDS the
    /// workflow resolved (D003).
    ///
    /// This composes rather than adds: it calls Milestone 1's
    /// `ResiduePosition::create`, which is where the residue number is
    /// validated, and Milestone 2's `ResidueMutationIntent::create`,
    /// which is where the residue letters are. Nothing here validates a
    /// biological rule of its own, and nothing here chooses a CDS: the
    /// identifier is the gate's answer, supplied by the caller of `bind`.
    pub fn bind(&self, cds: Identifier) -> Result<ResidueMutationIntent, DomainError> {
        let position: ResiduePosition = ResiduePosition::create(cds.clone(), self.number)?;
        ResidueMutationIntent::create(
            position.clone(),
            self.expected_original().symbol(),
            self.replacement().symbol(),
            self.policy(),
        )
    }
}

/// A Q5 SDM request: the typed capability invocation of D014 and D016.
///
/// The product brief lists four required inputs: the DNA template, an
/// identified target CDS, a numbering convention, and an explicit mutation
/// intent. The DNA template and the intent are fields of this record; the
/// numbering convention belongs to the annotation, which carries it as
/// `Cds::numbering` (D006, Milestone 1's `Numbering`). The identified
/// target CDS is not a field of its own: `scope` and the intent are two
/// halves that each can identify it, and `target_cds_name` prefers the
/// intent's claim and falls back to `scope`.
///
/// * A named residue intent names its CDS through the identifier its
///   `ResiduePosition` carries, so its target is identified with or
///   without `scope`; `scope` only has to agree, and `create` refuses a
///   disagreement.
/// * `ResidueWithoutCds` names none: the caller gave a residue number and
///   no annotation to count it over, so `scope` is the only half that can
///   identify a target, and an absent one states no preference.
/// * A DNA-level intent names none: it addresses the reference sequence,
///   so `scope` is the only half that can say which annotation the edit
///   belongs to, and an absent one states no preference. `scope` is
///   optional so the ambiguous case stays clarifiable rather than
///   unrepresentable, and an unscoped DNA edit gains no containment rule
///   from the gate's resolution (`docs/q5-sdm.md` §4, §10 item 1, owner
///   2026-10-14).
///
/// The ambiguity this record can express is what those two halves leave
/// undecided: no claim while the artifact carries several annotations — the
/// product brief's fail-closed exchange, "mutate residue 42 to glycine" —
/// or a named identifier that matches no annotation, or matches several. A
/// residue position with no CDS is not modelled: D006 requires a residue
/// position to identify its CDS and numbering origin, so `ResiduePosition`
/// stays total and the unidentified residue target lives in
/// `ResidueIntentWithoutCds`, which `bind` ties to the resolved annotation.
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
    ///   DNA-level intent carries its own `SequenceReference`, which its own
    ///   constructor checked only against the intent's own interval, so
    ///   D007's "two artifacts are not interchangeable because they share a
    ///   display name" is enforced here by `interval::require_sequence` and
    ///   its `SequenceReferenceMismatch`. A residue intent names no artifact
    ///   (see `source_reference`), so it has no claim to check.
    /// * a residue intent names one CDS and the scope names another:
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
/// is a natural-language interpretation, a later milestone (D034's order);
/// `calculated` and `verified` are mutation-domain states, whose
/// distinction D020 makes is enforced by `mutation::MutationState`; `failed`
/// is carried by a `DomainError` rather than by a result.
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

/// A warning a run carries for an edit it still applied (D031, D032 and the
/// owner decisions of 2026-10-15).
///
/// A deletion inside a CDS whose width is not a multiple of 3 shifts that CDS's
/// reading frame, and so does an insertion of `k` bases whose `k` is not a
/// multiple of 3, and a replacement of `w` bases by `m` bases whose `m − w`
/// is not a multiple of 3. All three owner decisions keep the edit rather than
/// refusing it: a frameshift is sometimes the intent, as in a knockout, so luria
/// warns and still applies it. The warning is a result and not a `DomainError` —
/// nothing the caller supplied is wrong with the request — and each variant
/// records what the edit did to that CDS: how many of the removed bases were
/// coding and whether any of them lay outside the CDS, how many bases an
/// insertion put into one CDS's gap, or how many bases a replacement overwrote
/// and how many it wrote in their place.
///
/// The `PrematureStop` variant is the fourth warning, and it is not a frame
/// warning (D037, the owner decision of 2026-10-16). A DNA edit that lies wholly
/// inside the resolved CDS and keeps its reading frame re-translates that CDS on
/// the edited artifact, and a stop that is not that CDS's last codon is this
/// warning: the edit is still applied, because a nonsense mutation is sometimes
/// the intent, as a frameshift one is. A residue intent states its replacement
/// residue explicitly and gets no such warning, and `verify` stays a DNA-level
/// check, so the rule is applied at this layer. A frameshift warning and a
/// `PrematureStop` cannot both fire for one edit: one needs a frame change and
/// the other needs none.
#[derive(Clone, Debug, PartialEq)]
pub enum Q5EditWarning {
    Frameshift {
        cds: Identifier,
        coding_bases_removed: u64,
        wholly_inside: bool,
    },
    InsertionFrameshift {
        cds: Identifier,
        bases_inserted: u64,
    },
    ReplacementFrameshift {
        cds: Identifier,
        bases_replaced: u64,
        bases_written: u64,
        wholly_inside: bool,
    },
    PrematureStop {
        cds: Identifier,
        codon_number: u64,
    },
}

impl Q5EditWarning {
    /// Reporting text: what was removed, and from where.
    pub fn describe(&self) -> String {
        match self {
            Q5EditWarning::Frameshift {
                cds,
                coding_bases_removed,
                wholly_inside,
            } => {
                let text: String = "deleting ".to_string()
                    + &coding_bases_removed.to_string()
                    + " coding bases from "
                    + cds.value()
                    + " shifts its reading frame";
                if *wholly_inside {
                    return text;
                }
                text + "; the deletion also extends outside the CDS"
            }
            Q5EditWarning::InsertionFrameshift {
                cds,
                bases_inserted,
            } => {
                "inserting ".to_string()
                    + &bases_inserted.to_string()
                    + " bases into "
                    + cds.value()
                    + " shifts its reading frame"
            }
            Q5EditWarning::ReplacementFrameshift {
                cds,
                bases_replaced,
                bases_written,
                wholly_inside,
            } => {
                let text: String = "replacing ".to_string()
                    + &bases_replaced.to_string()
                    + " bases with "
                    + &bases_written.to_string()
                    + " in "
                    + cds.value()
                    + " shifts its reading frame";
                if *wholly_inside {
                    return text;
                }
                text + "; the replacement also extends outside the CDS"
            }
            Q5EditWarning::PrematureStop { cds, codon_number } => {
                "the edit puts a stop codon at codon ".to_string()
                    + &codon_number.to_string()
                    + " of "
                    + cds.value()
                    + ", before its last codon"
            }
        }
    }
}

/// The warnings one edit carries against the resolved CDS.
///
/// For a deletion (D031's related owner decision, 2026-10-15): count the
/// deleted positions that lie inside some interval of the resolved CDS. A
/// count of 0 means the edit lies outside the CDS: the run proceeds and
/// warns nothing (`docs/q5-sdm.md` §10 item 1, owner 2026-10-14). A count
/// that is a multiple of 3 keeps the reading frame; any other count shifts
/// it, and the warning says whether the deletion lay wholly inside the CDS.
/// The only position that does not count lies outside the CDS:
/// `interval::validate_structure` refuses gaps between intervals, so a CDS
/// here has no intron.
///
/// The warning one insertion carries against the resolved CDS (D032's
/// related owner decision, 2026-10-15).
///
/// `k` bases inserted into a gap inside the CDS shift its reading frame
/// whenever `k` is not a multiple of 3, and the edit is kept rather than
/// refused, so the warning is a result and not a `DomainError`. The gap at
/// `p` is inside only when some interval of the CDS has `start < p < end`,
/// so both bases beside the point are coding: an insertion at a CDS edge,
/// or one wholly outside it, has no coding run of bases to shift and warns
/// nothing. `mutation::require_point_within_feature` applies the same test
/// when the edit itself was scoped to an annotation.
///
/// The warning one replacement carries against the resolved CDS (D033's
/// addenda, the owner decision of 2026-10-15).
///
/// A replacement overwrites `w` bases and writes `m` in their place, so the
/// text of the CDS changes length by `m − w` and its reading frame shifts
/// whenever that difference is not a multiple of 3, in either direction; a
/// difference of 0 edits nothing that matters for the frame. The edit is
/// kept rather than refused, so the warning is a result and not a
/// `DomainError`. Only the region positions that lie inside some interval of
/// the resolved CDS are coding, so a replacement wholly outside it has no
/// frame to shift and warns nothing, which is what `docs/q5-sdm.md` §10
/// item 1 (owner 2026-10-14) lets an unscoped DNA edit do. The warning
/// counts the whole region width and the text written, and `wholly_inside`
/// says whether every overwritten base was coding.
///
/// A nucleotide and a residue substitution carry no warning.
///
/// The `PrematureStop` warning (D037, the owner decision of 2026-10-16) is
/// computed by `premature_stop_warning`, which re-translates the resolved CDS
/// on the edited artifact; this function returns a `DomainError` only for what
/// that re-translation reports other than an internal stop, which cannot occur
/// for an in-frame ACGT edit of a CDS that translated before the edit.
fn edit_warnings(
    template: &Sequence,
    mutation: &MutationResult,
    target: &Cds,
) -> Result<Vec<Q5EditWarning>, DomainError> {
    let premature: Option<Q5EditWarning> = premature_stop_warning(template, mutation, target)?;

    let mut warnings: Vec<Q5EditWarning> = match mutation {
        MutationResult::Deletion(_record) => {
            let removed: Vec<u64> = mutation.target_coordinates();
            let intervals: OrderedIntervals = target.intervals();

            let mut coding: u64 = 0;
            for position in removed.iter() {
                if holds_position(intervals.as_slice(), *position) {
                    coding += 1;
                }
            }

            if coding == 0 || coding.is_multiple_of(3) {
                Vec::new()
            } else {
                let wholly_inside: bool = coding == removed.len() as u64;
                vec![Q5EditWarning::Frameshift {
                    cds: target.id(),
                    coding_bases_removed: coding,
                    wholly_inside,
                }]
            }
        }

        MutationResult::Insertion(record) => {
            let position: u64 = record.point().position();
            let bases: u64 = record.inserted().len() as u64;
            let intervals: OrderedIntervals = target.intervals();

            let inside: bool = holds_point(intervals.as_slice(), position);
            if inside && !bases.is_multiple_of(3) {
                vec![Q5EditWarning::InsertionFrameshift {
                    cds: target.id(),
                    bases_inserted: bases,
                }]
            } else {
                Vec::new()
            }
        }

        MutationResult::Replacement(record) => {
            let positions: Vec<u64> = mutation.target_coordinates();
            let intervals: OrderedIntervals = target.intervals();

            let replaced: u64 = positions.len() as u64;
            let written: u64 = record.replacement().len() as u64;

            let mut coding: u64 = 0;
            for position in positions.iter() {
                if holds_position(intervals.as_slice(), *position) {
                    coding += 1;
                }
            }

            let change: i64 = (written as i64) - (replaced as i64);
            if coding == 0 || change % 3 == 0 {
                Vec::new()
            } else {
                vec![Q5EditWarning::ReplacementFrameshift {
                    cds: target.id(),
                    bases_replaced: replaced,
                    bases_written: written,
                    wholly_inside: coding == replaced,
                }]
            }
        }

        MutationResult::Nucleotide(_record) => Vec::new(),
        MutationResult::Residue(_record) => Vec::new(),
    };

    match premature {
        None => (),
        Some(warning) => warnings.push(warning),
    }

    Ok(warnings)
}

/// The `PrematureStop` warning one edit carries against the resolved CDS (D037, the
/// owner decision of 2026-10-16).
///
/// The rule runs in this order:
///
/// * the edit's half-open span on the template and the length change `delta` it
///   makes: a substitution writes `[p, p + 1)` and changes 0, a deletion removes
///   `[s, e)` and changes `−w`, an insertion into the gap at `p` changes `+k` and
///   spans the gap itself, a replacement overwrites `[s, e)` by `m` bases and
///   changes `m − w`, and a residue intent states its replacement residue
///   explicitly, so D037 gives it no warning;
/// * the edit must lie wholly inside one interval of the CDS — `holds_position`,
///   `holds_region` and `holds_point` from `interval.rs` — and `delta` must be a
///   multiple of 3, or the rule does not apply: an edit only partly inside the
///   CDS and a frameshift are both recorded as open in D037, and the frameshift
///   warning of D031–D033 already names the second;
/// * the CDS must have translated before the edit, or there is nothing to
///   compare the edited CDS against;
/// * the CDS's intervals are rebuilt on the edited artifact in the original
///   order: the interval holding the edit keeps its start and grows its end by
///   `delta`, an interval starting at or past the edit's end moves by `delta`,
///   any other interval is unchanged, and an interval the edit removed whole —
///   one that would come out empty — is simply dropped. If that leaves no
///   interval at all, the edit removed the whole CDS: there is nothing left to
///   re-translate, so the rule warns nothing and the edit still applies;
/// * the rebuilt CDS is translated. An internal stop is this warning, naming the
///   CDS and the codon number on the edited CDS; a translation that succeeds warns
///   nothing, and any other failure is returned as it stands — it cannot occur for
///   an in-frame ACGT edit of a CDS that translated.
fn premature_stop_warning(
    template: &Sequence,
    mutation: &MutationResult,
    target: &Cds,
) -> Result<Option<Q5EditWarning>, DomainError> {
    let intervals: OrderedIntervals = target.intervals();

    let edit: Option<(u64, u64, i64)> = match mutation {
        MutationResult::Nucleotide(record) => {
            let position: u64 = record.position().start();
            if holds_position(intervals.as_slice(), position) {
                Some((position, position + 1, 0))
            } else {
                None
            }
        }

        MutationResult::Deletion(record) => {
            let region: NucleotideInterval = record.region();
            let delta: i64 = -(region.length() as i64);
            if holds_region(intervals.as_slice(), region.start(), region.end()) {
                Some((region.start(), region.end(), delta))
            } else {
                None
            }
        }

        MutationResult::Insertion(record) => {
            let position: u64 = record.point().position();
            let delta: i64 = record.inserted().len() as i64;
            if holds_point(intervals.as_slice(), position) {
                Some((position, position, delta))
            } else {
                None
            }
        }

        MutationResult::Replacement(record) => {
            let region: NucleotideInterval = record.region();
            let delta: i64 = (record.replacement().len() as i64) - (region.length() as i64);
            if holds_region(intervals.as_slice(), region.start(), region.end()) {
                Some((region.start(), region.end(), delta))
            } else {
                None
            }
        }

        MutationResult::Residue(_record) => None,
    };

    let span: (u64, u64, i64) = match edit {
        None => {
            // The edit is not a DNA edit wholly inside one interval of the
            // resolved CDS: D037's rule does not apply.
            return Ok(None);
        }
        Some(value) => value,
    };
    let edit_start: u64 = span.0;
    let edit_end: u64 = span.1;
    let delta: i64 = span.2;

    if delta % 3 != 0 {
        // The edit changes the CDS's length by a non-multiple of 3, so it shifts
        // the reading frame: D037 re-translates only an edit that keeps it, and
        // the frameshift warning already names this edit.
        return Ok(None);
    }

    if translate(template, target).is_err() {
        // The CDS did not translate before the edit, so the rule has nothing to
        // compare against: D037 records that case as open.
        return Ok(None);
    }

    let edited: Sequence = mutation.edited();
    let mut shifted: OrderedIntervals = Vec::new();
    for interval in intervals.iter() {
        let from: i64 = interval.start() as i64;
        let to: i64 = interval.end() as i64;
        let (start, end): (i64, i64) =
            if interval.start() <= edit_start && edit_end <= interval.end() {
                // The interval that holds the edit: it keeps its start.
                (from, to + delta)
            } else if interval.start() >= edit_end {
                // An interval at or past the edit's end moves with the edit.
                (from + delta, to + delta)
            } else {
                // An interval that ends before the edit starts: unchanged.
                (from, to)
            };
        if start == end {
            // The edit removed this interval whole: no interval is left there,
            // so the rebuilt CDS simply does not carry one.
            continue;
        }
        let moved: NucleotideInterval = create(&edited.reference(), start, end)?;
        shifted.push(moved);
    }

    if shifted.is_empty() {
        // The edit removed the whole CDS: there is nothing left to re-translate,
        // so the rule warns nothing and the edit still applies.
        return Ok(None);
    }

    // The same arguments `mutation::rebuild_cds` uses, so the rebuilt CDS is that
    // annotation carried over the edited artifact and nothing else about it
    // changes.
    let rebuilt: Cds = Cds::create(
        &edited,
        target.id(),
        shifted,
        target.strand(),
        target.completeness(),
        target.genetic_code().name(),
        target.numbering(),
    )?;

    match translate(&edited, &rebuilt) {
        Ok(_translation) => Ok(None),
        Err(error) => match error {
            DomainError::InternalStopCodon(problem) => Ok(Some(Q5EditWarning::PrematureStop {
                cds: target.id(),
                codon_number: problem.codon_number as u64,
            })),
            other => Err(other),
        },
    }
}

/// A Q5 run: the gate's decision plus the deterministic slice of D015.
///
/// The gate result is carried whole rather than flattened, so what the
/// application understood stays inspectable alongside what it decided (D004,
/// D016). The mutation result is Milestone 2's `MutationResult` unchanged:
/// this layer names the annotation the mutation was applied to and adds
/// nothing to that record, so the selected codon, the coordinates, the
/// expected and actual originals, and the lifecycle state are still
/// Milestone 2's claims.
///
/// `mutation` is present exactly when the scope resolved. D005 forbids
/// choosing an annotation and D020 gives that ambiguity its own state, so a
/// clarification run performs no edit, produces no artifact, and holds no
/// mutation claim at all.
#[derive(Clone, Debug, PartialEq)]
pub struct Q5WorkflowExecution {
    scope: Q5WorkflowResult,
    mutation: Option<MutationResult>,
    warnings: Vec<Q5EditWarning>,
}

impl Q5WorkflowExecution {
    /// The gate's decision, exactly as `validate_input_scope` produced it.
    pub fn scope(&self) -> Q5WorkflowResult {
        self.scope.clone()
    }

    pub fn needs_clarification(&self) -> bool {
        self.scope().needs_clarification()
    }

    /// The mutation result, when the run performed one.
    pub fn mutation(&self) -> Option<MutationResult> {
        self.mutation.clone()
    }

    /// The warnings this run carries (D031–D033). A run that performed no
    /// edit carries none, and an edit that keeps the reading frame carries
    /// none either.
    pub fn warnings(&self) -> Vec<Q5EditWarning> {
        self.warnings.clone()
    }

    /// The lifecycle state of the mutation, when there is one (D020).
    pub fn mutation_state(&self) -> Option<MutationState> {
        self.mutation().map(|record: MutationResult| record.state())
    }

    /// Whether the independent verification pass accepted the edit (D013).
    /// A run that performed no edit is not verified, and a run that only
    /// calculated one is not verified either.
    pub fn is_verified(&self) -> bool {
        match self.mutation_state() {
            None => false,
            Some(state) => state.is_verified(),
        }
    }

    /// The edited artifact, when the run produced one (D007).
    pub fn edited(&self) -> Option<Sequence> {
        self.mutation()
            .map(|record: MutationResult| record.edited())
    }

    /// The artifact the run was about, by identity (D007).
    pub fn template_reference(&self) -> SequenceReference {
        self.scope().request().template_reference()
    }

    /// Short label for reporting. Never used for classification.
    pub fn describe(&self) -> String {
        let gate: Q5WorkflowResult = self.scope();
        let state: Option<MutationState> = self.mutation_state();
        if state.is_none() {
            return gate.describe();
        }
        format!("{} {}", gate.state().name(), state.clone().unwrap().name())
    }
}

/// Run the deterministic vertical slice of D015 against a Q5 request.
///
/// The slice is composed, not re-implemented: the gate is
/// `validate_input_scope`, and the edit and the independent verification are
/// the `mutation::apply_*` operation for the intent and `mutation::verify`,
/// called as they stand. Nothing here re-derives a residue, a codon, a
/// coordinate or a verification claim, and nothing here designs a primer or
/// calculates Tm: those are `primer_design.rs` and `tm.rs` under D029–D033,
/// composed in `q5_design.rs`.
///
/// Three things are decided here, and nothing else is:
///
/// * which annotation the mutation applies to. That is the gate's answer,
///   reused rather than re-derived. A clarification stops here, so D005's
///   "No sequence calculation performed" holds by construction: no edit is
///   attempted, so no base is read for an edit.
/// * what the edited artifact is called. `result` is the caller's, exactly
///   as Milestone 2's fixture table supplies a `result_id` per case: a
///   display name is the caller's to choose, and D007 keeps two artifacts
///   apart by name *and* digest rather than by name alone, so no naming rule
///   is needed here and none is invented.
/// * whether a DNA edit is checked against the resolved annotation's
///   intervals. Scoping is a parameter of the operation, not a property of
///   the edit, and an unscoped DNA request is a legal operation (Milestone
///   2, fixture `nuc-a-unscoped`), so this layer carries the request's own
///   scope through unchanged: a request that scoped an annotation is checked
///   against it, and an unscoped DNA edit outside the resolved CDS proceeds
///   (`docs/q5-sdm.md` §4, §10 item 1, owner 2026-10-14). A residue
///   mutation is different in kind: a named residue intent already names its
///   CDS, so the resolved annotation is always its scope, and an unbound one
///   takes the gate's answer through `bind`, which is also where Milestone 1
///   validates the residue number. Either way `verify` is handed that
///   annotation, because a residue result's claims are checked against the
///   annotation it was applied to.
///
/// A `DomainError` from the edit or the verification pass is returned as it
/// stands, with its own variant and category. A `Validated` gate result that
/// carried no target is refused with `MissingCdsAttribute` rather than
/// unwrapped: that pairing cannot be produced through the public API, and
/// error.rs's contract is that every failure this model produces is a
/// `DomainError`, so it is a structured failure and not a panic.
pub fn execute(
    request: Q5SdmRequest,
    annotations: Vec<Cds>,
    result: Identifier,
) -> Result<Q5WorkflowExecution, DomainError> {
    let gate: Q5WorkflowResult = validate_input_scope(request.clone(), annotations.clone())?;

    if gate.needs_clarification() {
        // D005: nothing was chosen, so nothing is calculated.
        return Ok(Q5WorkflowExecution {
            scope: gate,
            mutation: None,
            warnings: Vec::new(),
        });
    }

    // `Validated` is produced alongside exactly one resolved CDS and
    // `NeedsClarification` alongside none, so the state says whether the
    // target is here. That invariant is module-local, so it is checked rather
    // than unwrapped: a `Validated` result with no target is a structured
    // failure, not a panic (error.rs: every failure this model produces is a
    // `DomainError`).
    let resolved: Option<Cds> = gate.target();
    if resolved.is_none() {
        let problem = MissingCdsAttributeProblem {
            cds: request.template().display_name(),
            attribute: "a resolved target CDS".to_string(),
            value: None,
        };
        return Err(DomainError::MissingCdsAttribute(problem));
    }
    let target: Cds = resolved.unwrap();

    let template: Sequence = request.template();
    let verified: MutationResult = run_resolved(request, target.clone(), result)?;

    let warnings: Vec<Q5EditWarning> = edit_warnings(&template, &verified, &target)?;

    Ok(Q5WorkflowExecution {
        scope: gate,
        mutation: Some(verified),
        warnings,
    })
}

/// Run the mutation slice for one request whose target the caller resolved.
///
/// This is the part of `execute` after the gate, and what `execute_set`
/// reuses per member: the edit and the independent verification are the
/// `mutation::apply_*` operation for the intent and `mutation::verify`,
/// called as they stand, and the only decision made here is which
/// annotation the operation and the verification pass are handed. `execute`
/// reaches it with the target its own gate resolved; `execute_set` reaches
/// it once per member with the target the *set* resolved, so a set resolves
/// a target once and every member is applied to that same annotation.
/// Nothing here re-derives a residue, a codon, a coordinate or a
/// verification claim, and no biological rule is restated at this layer.
fn run_resolved(
    request: Q5SdmRequest,
    target: Cds,
    result: Identifier,
) -> Result<MutationResult, DomainError> {
    let template: Sequence = request.template();

    let dna_scope: Option<Cds> = nucleotide_scope(request.scope(), target.clone());
    let calculated: MutationResult = match request.intent() {
        Q5MutationIntent::Nucleotide(intent) => {
            apply_nucleotide(&template, dna_scope.clone(), result.clone(), intent)?
        }
        Q5MutationIntent::Deletion(intent) => {
            apply_deletion(&template, dna_scope.clone(), result.clone(), intent)?
        }
        Q5MutationIntent::Insertion(intent) => {
            apply_insertion(&template, dna_scope.clone(), result.clone(), intent)?
        }
        Q5MutationIntent::Replacement(intent) => {
            apply_replacement(&template, dna_scope.clone(), result.clone(), intent)?
        }
        Q5MutationIntent::Residue(intent) => {
            apply_residue(&template, target.clone(), result.clone(), intent)?
        }
        Q5MutationIntent::ResidueWithoutCds(intent) => {
            // The caller resolved the annotation; `bind` only states it in the
            // form Milestone 2's operation takes, through Milestone 1's and
            // Milestone 2's own constructors.
            let bound: ResidueMutationIntent = intent.bind(target.id())?;
            apply_residue(&template, target.clone(), result.clone(), bound)?
        }
    };

    // A residue result names the CDS it was applied to, so `verify` checks
    // its residue and codon claims against that annotation: it reads the
    // scope only on the residue branch. A DNA-level result records only the
    // coordinate it wrote, and `verify` never reads the scope on those
    // branches, so passing `dna_scope` changes no check; it is passed rather
    // than `None` so the value stays the request's own claim. That inertness
    // is a limitation recorded in `docs/q5-sdm.md` §8, and verification
    // behavior is unchanged.
    let verify_scope: Option<Cds> = match request.intent() {
        Q5MutationIntent::Nucleotide(_intent) => dna_scope.clone(),
        Q5MutationIntent::Deletion(_intent) => dna_scope.clone(),
        Q5MutationIntent::Insertion(_intent) => dna_scope.clone(),
        Q5MutationIntent::Replacement(_intent) => dna_scope.clone(),
        Q5MutationIntent::Residue(_intent) => Some(target.clone()),
        Q5MutationIntent::ResidueWithoutCds(_intent) => Some(target.clone()),
    };

    verify(&template, verify_scope.clone(), calculated)
}

/// The scope a DNA edit is checked against, mirroring the request.
///
/// Scoping is a parameter of the operation, not a property of the edit, and
/// an unscoped DNA request is a legal operation (Milestone 2, fixture
/// `nuc-a-unscoped`). This layer passes the request's own scope through: a
/// request that scoped an annotation is checked against it, and a request
/// that scoped nothing gains no containment rule from the gate's resolution,
/// so an unscoped DNA edit outside the resolved CDS proceeds
/// (`docs/q5-sdm.md` §4, §10 item 1, owner 2026-10-14).
fn nucleotide_scope(scope: Option<Identifier>, resolved: Cds) -> Option<Cds> {
    if scope.is_some() {
        return Some(resolved.clone());
    }
    None
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
            value: None,
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
    // state rather than an error: a named CDS matching no annotation is a
    // clarification (§10 item 3, owner 2026-10-14), and candidates are not
    // narrowed by whether they could hold a requested residue (§10 item 2,
    // owner 2026-10-14).
    let display: String = template.display_name();
    let listed: String = join_names(names.clone());
    let mut clarifications: Vec<String> = Vec::new();
    if wanted.is_none() {
        // A residue request that tied its number to no CDS names the residue
        // in the clarification as well as the annotations on offer; every
        // other request names only the CDS. The gate reads no sequence
        // content, so candidates are not narrowed by whether an annotation
        // could hold the number (§3, §5, §10 item 2, owner 2026-10-14).
        // Text only: classification is by state.
        let missing: String = missing_claim(request.intent());
        clarifications.push(format!(
            "the request does not {}, and '{}' is annotated with {} of them: {}",
            missing,
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

/// What the request left unidentified, in a clarification's words.
///
/// Reporting text only. A residue request that named no CDS says which residue
/// has no annotation to count it over; every other request says only that no
/// CDS was named.
fn missing_claim(intent: Q5MutationIntent) -> String {
    match intent {
        Q5MutationIntent::ResidueWithoutCds(unbound) => {
            "tie ".to_string() + &unbound.describe() + " to a CDS"
        }
        Q5MutationIntent::Nucleotide(_intent) => "name a CDS".to_string(),
        Q5MutationIntent::Deletion(_intent) => "name a CDS".to_string(),
        Q5MutationIntent::Insertion(_intent) => "name a CDS".to_string(),
        Q5MutationIntent::Replacement(_intent) => "name a CDS".to_string(),
        Q5MutationIntent::Residue(_intent) => "name a CDS".to_string(),
    }
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

/// How a multi-variant request composes its members (D011).
///
/// D011 states that separate variants and combined mutations are different
/// operations and that the parser and the typed request model must preserve
/// that distinction, so the mode is a field of the set request rather than
/// something inferred from how many members it carries: a one-member set
/// still names a mode, and a set of one is not a hidden single-request call
/// with its rules left undecided.
///
/// Only `SeparateVariants` is executable. `CombinedConstruct` is
/// representable so a caller can ask for it and be refused rather than be
/// quietly reinterpreted: composing mutations into one construct needs
/// overlap rules no milestone has defined (D014 requires every design rule to
/// name a documented source, and none is written for those), and executing a
/// combined construct as separate variants would answer a different question
/// than the one the caller asked. The refusal happens at construction, before
/// any member resolves a target or edits anything.
#[derive(Clone, Debug, PartialEq)]
pub enum Q5Composition {
    SeparateVariants,
    CombinedConstruct,
}

impl Q5Composition {
    pub fn name(&self) -> String {
        match self {
            Q5Composition::SeparateVariants => "separate_variants".to_string(),
            Q5Composition::CombinedConstruct => "combined_construct".to_string(),
        }
    }

    pub fn is_separate_variants(&self) -> bool {
        match self {
            Q5Composition::SeparateVariants => true,
            Q5Composition::CombinedConstruct => false,
        }
    }

    pub fn is_combined_construct(&self) -> bool {
        match self {
            Q5Composition::SeparateVariants => false,
            Q5Composition::CombinedConstruct => true,
        }
    }
}

/// One member of a variant set: a Q5 request plus the caller's name for the
/// artifact that member's edit produces (D016, D007, M2's `result_id`).
///
/// The intent-to-result-identifier pairing lives here and nowhere else: the
/// set keeps no parallel list of names, generates none, and derives none from
/// the mutation text. Each member names its own result exactly as the
/// single-request `execute` takes a `result: Identifier`, and the name is
/// still the caller's: D007 keeps artifacts apart by name *and* digest, so no
/// naming rule is needed here and none is invented.
///
/// The request is this module's `Q5SdmRequest` unchanged, so a member carries
/// its own template artifact, its own scope, and its own intent: the set
/// layer adds no intent form and restates no rule of its own.
#[derive(Clone, Debug, PartialEq)]
pub struct Q5Variant {
    request: Q5SdmRequest,
    result: Identifier,
}

impl Q5Variant {
    /// Pair an already-built request with the caller's result name.
    ///
    /// Nothing is validated here and nothing can fail: the request was
    /// validated by `Q5SdmRequest::create` and the name by
    /// `Identifier::create`, and the set's own invariants are checked when
    /// the set is built, not per member.
    pub fn create(request: Q5SdmRequest, result: Identifier) -> Q5Variant {
        Q5Variant { request, result }
    }

    pub fn request(&self) -> Q5SdmRequest {
        self.request.clone()
    }

    /// The caller's display name for this member's edited artifact.
    pub fn result_name(&self) -> Identifier {
        self.result.clone()
    }

    /// Short label for reporting. Never used for classification.
    pub fn describe(&self) -> String {
        format!(
            "{} against {}",
            self.result_name().value(),
            self.request().template_reference().describe(),
        )
    }
}

/// A multi-variant request: an explicit composition mode and ordered members
/// (D011, D016).
///
/// One typed request, not a caller's loop over the single-request API: the
/// set-level invariants below are what a loop cannot establish, and the
/// composition mode is what a loop would silently drop.
#[derive(Clone, Debug, PartialEq)]
pub struct Q5VariantSetRequest {
    composition: Q5Composition,
    variants: Vec<Q5Variant>,
}

impl Q5VariantSetRequest {
    /// Build a set request, refusing one that cannot be represented or run.
    ///
    /// The checks are set-level invariants, in this order, and none is a
    /// biological rule — residue, codon, coordinate, numbering, and
    /// verification rules stay Milestone 1's and Milestone 2's, reached when
    /// a member runs:
    ///
    /// 1. no members: a set with nothing to run is not representable, so
    ///    `InvalidMutationRequest` (`MalformedInput`) at construction.
    /// 2. the composition mode is a required parameter of `create`, so it is
    ///    always supplied explicitly and never inferred from the member count
    ///    (D011's distinction, preserved in the typed model).
    /// 3. `CombinedConstruct` is out of the supported scope: refused with
    ///    `UnsupportedComposition` (`Unsupported`) before any member resolves
    ///    a target or edits anything, and never run as separate variants.
    /// 4. every member must address one template artifact, checked with
    ///    Milestone 1's `interval::require_sequence` and its
    ///    `SequenceReferenceMismatch` (D007: name *and* digest, so a
    ///    same-named different artifact is refused rather than merged).
    /// 5. explicit CDS claims must agree, reported with Milestone 2's
    ///    `ResiduePositionWrongCds`, the same rule `Q5SdmRequest::create`
    ///    uses for a scope that disagrees with a residue position. A member
    ///    that claims no CDS is not a disagreement: it takes the agreed claim
    ///    when the set runs.
    /// 6. every member's result identifier must be distinct, refused with
    ///    `InvalidMutationRequest` (`MalformedInput`) as the empty set is.
    ///    Owner decision, 2026-10-14 (`docs/q5-sdm.md` §10 item 6): two primer
    ///    pairs sharing a label could be confused when oligos are ordered.
    ///    Identifiers are compared exactly, with no case folding: no source
    ///    states a folding rule, and D007 compares names exactly.
    pub fn create(
        composition: Q5Composition,
        variants: Vec<Q5Variant>,
    ) -> Result<Q5VariantSetRequest, DomainError> {
        if variants.is_empty() {
            let problem = InvalidMutationRequestProblem {
                request: "q5-variant-set".to_string(),
                reason: "a variant set must carry at least one variant".to_string(),
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }

        if composition.is_combined_construct() {
            let problem = UnsupportedCompositionProblem {
                composition: composition.name(),
                variant_count: variants.len(),
                reason: "combined constructs need overlap rules no milestone has defined"
                    .to_string(),
            };
            return Err(DomainError::UnsupportedComposition(problem));
        }

        let established: SequenceReference = variants[0].request().template_reference();
        let agreed: Option<Identifier> = set_target_claim(variants.clone());
        for variant in variants.iter() {
            require_sequence(established.clone(), &variant.request().template())?;
            let claimed: Option<Identifier> = variant.request().target_cds_name();
            if claimed.is_some() && agreed != claimed {
                let problem = CdsMismatchProblem {
                    expected: agreed.clone().unwrap().value().to_string(),
                    actual: claimed.clone().unwrap().value().to_string(),
                };
                return Err(DomainError::ResiduePositionWrongCds(problem));
            }
        }

        let mut seen: Vec<Identifier> = Vec::new();
        for variant in variants.iter() {
            let name: Identifier = variant.result_name();
            if seen.contains(&name) {
                let problem = InvalidMutationRequestProblem {
                    request: "q5-variant-set".to_string(),
                    reason: format!(
                        "two variants are both named '{}'; result names in a set must be distinct",
                        name.value()
                    ),
                };
                return Err(DomainError::InvalidMutationRequest(problem));
            }
            seen.push(name);
        }

        Ok(Q5VariantSetRequest {
            composition,
            variants,
        })
    }

    /// The composition mode the caller supplied.
    pub fn composition(&self) -> Q5Composition {
        self.composition.clone()
    }

    /// The members, in request order.
    pub fn variants(&self) -> Vec<Q5Variant> {
        self.variants.clone()
    }

    pub fn variant_count(&self) -> usize {
        self.variants.len()
    }

    /// The one template artifact every member addresses, by identity (D007).
    pub fn template_reference(&self) -> SequenceReference {
        self.variants[0].request().template_reference()
    }

    /// The CDS claim the set makes: the agreed explicit claim of its members,
    /// or none when no member claims one. `create` refused a set whose
    /// members claim different CDSs, so the first claim is the agreed one.
    pub fn target_cds_name(&self) -> Option<Identifier> {
        set_target_claim(self.variants.clone())
    }

    /// Short label for reporting. Never used for classification.
    pub fn describe(&self) -> String {
        format!(
            "{} variants composed as {} against {}",
            self.variant_count(),
            self.composition().name(),
            self.template_reference().describe(),
        )
    }
}

/// The CDS claim a set's members make, in request order.
///
/// Reporting and resolution both need the same thing: the first explicit
/// claim, which `create` has already required every other explicit claim to
/// agree with. A member that claims no CDS contributes nothing here, which
/// is what lets an unbound member take the set's agreed claim rather than
/// clarify for itself.
fn set_target_claim(variants: Vec<Q5Variant>) -> Option<Identifier> {
    for variant in variants.iter() {
        let claimed: Option<Identifier> = variant.request().target_cds_name();
        if claimed.is_some() {
            return claimed;
        }
    }
    None
}

/// The request a set's target resolution runs against.
///
/// Target resolution is `validate_input_scope`'s question, and it is asked
/// once for the set rather than once per member: the request carried to the
/// gate is the shared template artifact plus the set's agreed CDS claim, so
/// an unbound member resolves through the set's claim instead of clarifying
/// for itself. The intent is the first member's, which the gate reads only to
/// phrase a clarification: an intent claims no CDS here, since a claim that
/// the set carries comes from the set's members and `create` required them to
/// agree. The members themselves are not rewritten, so a member's own scope
/// stays its own claim and §10 item 1's containment rule (owner 2026-10-14)
/// reads the same for a set member as for a single request.
pub fn set_gate_request(set: Q5VariantSetRequest) -> Result<Q5SdmRequest, DomainError> {
    let members: Vec<Q5Variant> = set.variants();
    let first: Q5Variant = members[0].clone();
    Q5SdmRequest::create(
        first.request().template(),
        set.target_cds_name(),
        first.request().intent(),
    )
}

/// One member's outcome: the caller's result name and Milestone 2's result
/// record, paired so a set's results stay in request order and each result
/// keeps the identifier its own variant supplied.
///
/// The mutation record is Milestone 2's `MutationResult` unchanged: this
/// layer adds nothing to it, and the identifier is carried beside it rather
/// than merged into it, so the record stays exactly the one a single
/// `execute` returns.
#[derive(Clone, Debug, PartialEq)]
pub struct Q5VariantResult {
    identifier: Identifier,
    mutation: MutationResult,
    warnings: Vec<Q5EditWarning>,
}

impl Q5VariantResult {
    /// The caller's name for this member's edited artifact.
    pub fn identifier(&self) -> Identifier {
        self.identifier.clone()
    }

    pub fn mutation(&self) -> MutationResult {
        self.mutation.clone()
    }

    /// The warnings this member's edit carries against the set's resolved CDS.
    /// Each member carries its own, so a set whose members differ does not
    /// report one warning for all of them.
    pub fn warnings(&self) -> Vec<Q5EditWarning> {
        self.warnings.clone()
    }

    pub fn edited(&self) -> Sequence {
        self.mutation().edited()
    }

    /// Identity of this member's edited artifact (D007).
    pub fn reference(&self) -> SequenceReference {
        self.mutation().reference()
    }

    pub fn state(&self) -> MutationState {
        self.mutation().state()
    }

    /// Whether this member's independent verification pass accepted the edit
    /// (D013). A calculated result is not verified (D020).
    pub fn is_verified(&self) -> bool {
        self.state().is_verified()
    }

    /// Short label for reporting. Never used for classification.
    pub fn describe(&self) -> String {
        format!(
            "{} ({})",
            self.identifier().value(),
            self.mutation().describe(),
        )
    }
}

/// A Q5 variant-set run: the set's one scope decision plus every member's
/// result, in request order.
///
/// `results` is empty exactly when the set's target did not resolve: a
/// clarification performs no edit for any member, produces no artifact for
/// any member, and holds no mutation claim at all (D005, D020). A run that
/// did resolve carries one result per member, in request order, each verified
/// by Milestone 2's own pass — a set never reports a partial success, because
/// a member that failed returned before any set result was built.
#[derive(Clone, Debug, PartialEq)]
pub struct Q5VariantSetExecution {
    request: Q5VariantSetRequest,
    scope: Q5WorkflowResult,
    results: Vec<Q5VariantResult>,
}

impl Q5VariantSetExecution {
    /// The set request, carried whole so what the application understood
    /// stays inspectable alongside what it decided (D004, D016).
    pub fn request(&self) -> Q5VariantSetRequest {
        self.request.clone()
    }

    /// The set-level gate decision, exactly as `validate_input_scope`
    /// produced it for the set's agreed claim.
    pub fn scope(&self) -> Q5WorkflowResult {
        self.scope.clone()
    }

    pub fn needs_clarification(&self) -> bool {
        self.scope().needs_clarification()
    }

    /// The one CDS every member was applied to, when the set resolved one.
    pub fn target(&self) -> Option<Cds> {
        self.scope().target()
    }

    /// The members' results, in request order.
    pub fn results(&self) -> Vec<Q5VariantResult> {
        self.results.clone()
    }

    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// The edited artifacts, in request order. Empty for a clarification.
    pub fn edited_artifacts(&self) -> Vec<Sequence> {
        let mut artifacts: Vec<Sequence> = Vec::new();
        for result in self.results.iter() {
            artifacts.push(result.edited());
        }
        artifacts
    }

    /// Whether every member's independent verification pass accepted its
    /// edit (D013). A set that performed no edit is not verified, and a set
    /// any of whose members is only calculated is not verified either (D020).
    pub fn is_verified(&self) -> bool {
        if self.results.is_empty() {
            return false;
        }
        for result in self.results.iter() {
            if !result.is_verified() {
                return false;
            }
        }
        true
    }

    /// The artifact the set was about, by identity (D007), in every state.
    pub fn template_reference(&self) -> SequenceReference {
        self.request().template_reference()
    }

    pub fn composition(&self) -> Q5Composition {
        self.request().composition()
    }

    /// Short label for reporting. Never used for classification.
    pub fn describe(&self) -> String {
        if self.results.is_empty() {
            return self.scope().describe();
        }
        format!("{} {}", self.scope().state().name(), self.result_count())
    }
}

/// Run every member of a variant set (D011, D015, D016).
///
/// The slice is composed, not re-implemented: the gate is
/// `validate_input_scope` reused as it stands, and each member's edit and
/// independent verification are the `mutation::apply_*` operation for the
/// intent and `mutation::verify`, called as they stand. Nothing here re-derives
/// a residue, a codon, a coordinate or a verification claim, and no
/// biological rule is added at this layer.
///
/// What this function decides, and nothing else:
///
/// * **the target, once.** The set's agreed CDS claim (refused at
///   construction if its members disagree) is the claim the gate is asked
///   with, so one annotation is resolved for the whole set. A member that
///   named no CDS is applied to that annotation; a member that named one
///   named it, and `create` required the two to agree. The members' own
///   requests are not rewritten, so a member's own scope stays its own claim
///   and §10 item 1's containment rule (owner 2026-10-14) reads the same
///   here as for a single request.
/// * **clarification stops the whole set.** D005 forbids choosing an
///   annotation and D020 gives that ambiguity its own state, so an
///   unresolved set returns no results and produces no edited artifact for
///   any member.
/// * **each member starts from the pristine template.** Every member is
///   handed the template its own request names, never a previous member's
///   edited artifact, so separate variants are independent: variant 2 is not
///   variant 1 plus another edit. Identical members do produce identical
///   artifacts by name and digest, which is D007's identity semantics rather
///   than a sharing rule this layer adds.
/// * **failure is atomic.** Members run in request order and a member that
///   fails returns that original `DomainError` — Milestone 1's or Milestone
///   2's variant and category, unchanged — so no earlier member's partial
///   results escape as a successful set and no per-variant failure carrier
///   exists.
///
/// `CombinedConstruct` never reaches here: `Q5VariantSetRequest::create`
/// refuses it, so a combined construct is never executed as separate
/// variants.
pub fn execute_set(
    set: Q5VariantSetRequest,
    annotations: Vec<Cds>,
) -> Result<Q5VariantSetExecution, DomainError> {
    let gate: Q5WorkflowResult = validate_input_scope(set_gate_request(set.clone())?, annotations)?;

    if gate.needs_clarification() {
        // D005: nothing was chosen, so nothing is calculated for any member.
        return Ok(Q5VariantSetExecution {
            request: set,
            scope: gate,
            results: Vec::new(),
        });
    }

    // The same module-local invariant `execute` guards rather than unwraps:
    // a `Validated` gate result with no target is a structured failure,
    // because error.rs's contract is that every failure this model produces
    // is a `DomainError`.
    let resolved: Option<Cds> = gate.target();
    if resolved.is_none() {
        let problem = MissingCdsAttributeProblem {
            cds: set.template_reference().name().value().to_string(),
            attribute: "a resolved target CDS".to_string(),
            value: None,
        };
        return Err(DomainError::MissingCdsAttribute(problem));
    }
    let target: Cds = resolved.unwrap();

    let mut results: Vec<Q5VariantResult> = Vec::new();
    for variant in set.variants().iter() {
        // The member's own request, its own result name, and the template its
        // own request names: the set's resolved annotation is what supplies a
        // member's CDS, never what a previous member edited.
        let mutation: MutationResult =
            run_resolved(variant.request(), target.clone(), variant.result_name())?;
        let template: Sequence = variant.request().template();
        results.push(Q5VariantResult {
            identifier: variant.result_name(),
            mutation: mutation.clone(),
            warnings: edit_warnings(&template, &mutation, &target)?,
        });
    }

    Ok(Q5VariantSetExecution {
        request: set,
        scope: gate,
        results: results.clone(),
    })
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
    use crate::interval::create_point;
    use crate::interval::NucleotideInterval;
    use crate::interval::OrderedIntervals;
    use crate::mutation::apply_residue;
    use crate::mutation::Codon;
    use crate::mutation::CodonSelectionPolicy;
    use crate::mutation::MutationResult;
    use crate::mutation::MutationState;
    use crate::mutation::NucleotideDeletionIntent;
    use crate::mutation::NucleotideInsertionIntent;
    use crate::mutation::NucleotideMutationIntent;
    use crate::mutation::NucleotideReplacementIntent;
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

    // A template long enough that residue 42 is a real possibility, so the
    // product brief's fail-closed reason ("multiple supported CDSs could
    // contain residue 42") can be true of a fixture rather than only asserted
    // in a comment: 41 M codons, then GCT as residue 42 and TGG as residue 43,
    // so 129 nucleotides and 43 codons, with no stop and no ambiguous codon.
    // Under FromCdsStart numbering over a whole annotation, residue 42 is A at
    // reference coordinates 123, 124, 125.
    fn long_text() -> String {
        let mut text: String = String::new();
        let mut index: usize = 0;
        while index < 41 {
            text += "ATG";
            index += 1;
        }
        text + "GCT" + "TGG"
    }

    fn long_template() -> Sequence {
        normalize(
            Identifier::create("q5-long".to_string()).unwrap(),
            long_text(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    // The long artifact with residue 42's codon replaced by the `Default`
    // policy's choice for W: GCT becomes TGG, so residues 42 and 43 are both
    // W and the length is unchanged.
    fn long_edited_text() -> String {
        let mut text: String = String::new();
        let mut index: usize = 0;
        while index < 41 {
            text += "ATG";
            index += 1;
        }
        text + "TGG" + "TGG"
    }

    // An annotation of the long template. `whole` builds intervals against
    // `template()`, so these CDSs need their own interval.
    fn long_cds_of(name: &str, length: i64) -> Cds {
        let intervals: OrderedIntervals =
            vec![create(&long_template().reference(), 0, length).unwrap()];
        Cds::create(
            &long_template(),
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

    // The intent Milestone 2 takes, unwrapped, so a test can compare a run
    // against a bare `apply_residue` call. Residue 2 of the template's whole
    // CDS is A, and the named `Default` policy gives W as TGG.
    fn residue_mutation_intent(
        cds: &str,
        expected: String,
        replacement: String,
        policy: CodonSelectionPolicy,
    ) -> ResidueMutationIntent {
        let position: ResiduePosition = ResiduePosition::create(cds_name(cds), 2).unwrap();
        ResidueMutationIntent::create(position, expected, replacement, policy).unwrap()
    }

    fn residue_intent(cds: &str) -> Q5MutationIntent {
        Q5MutationIntent::Residue(residue_mutation_intent(
            cds,
            "A".to_string(),
            "W".to_string(),
            CodonSelectionPolicy::Default,
        ))
    }

    // The same residue substitution with no CDS named: the product brief's
    // "mutate residue 42 to glycine" shape. The caller gave a residue number
    // and no annotation to count it over, so nothing here claims a CDS.
    fn unbound_residue(
        number: i64,
        expected: String,
        replacement: String,
        policy: CodonSelectionPolicy,
    ) -> Q5MutationIntent {
        Q5MutationIntent::ResidueWithoutCds(
            ResidueIntentWithoutCds::create(number, expected, replacement, policy).unwrap(),
        )
    }

    fn unbound_residue_intent() -> Q5MutationIntent {
        unbound_residue(
            2,
            "A".to_string(),
            "W".to_string(),
            CodonSelectionPolicy::Default,
        )
    }

    // A DNA edit at an arbitrary coordinate of the template. Coordinate 3 is
    // inside a whole-CDS annotation and coordinate 20 is outside a 12-nucleotide
    // one; both coordinates hold G in the template.
    fn nucleotide_mutation_intent_at(position: u64) -> NucleotideMutationIntent {
        let interval: OrderedIntervals = vec![create(
            &template().reference(),
            position as i64,
            (position + 1) as i64,
        )
        .unwrap()];
        NucleotideMutationIntent::create(
            template().reference(),
            interval[0].clone(),
            "G".to_string(),
            "C".to_string(),
        )
        .unwrap()
    }

    fn nucleotide_intent_at(position: u64) -> Q5MutationIntent {
        Q5MutationIntent::Nucleotide(nucleotide_mutation_intent_at(position))
    }

    // A DNA-level deletion on `artifact`: the region `[start, end)` and the
    // exact text removed, as Milestone 4a's intent asserts it. The intent is
    // well formed on its own, so only the request can say whether it belongs
    // here — the pattern `nucleotide_intent_on` establishes.
    fn deletion_intent_on(
        artifact: Sequence,
        start: u64,
        end: u64,
        bases: String,
    ) -> Q5MutationIntent {
        let intervals: OrderedIntervals =
            vec![create(&artifact.reference(), start as i64, end as i64).unwrap()];
        Q5MutationIntent::Deletion(
            NucleotideDeletionIntent::create(artifact.reference(), intervals[0].clone(), bases)
                .unwrap(),
        )
    }

    fn deletion_intent(start: u64, end: u64, bases: String) -> Q5MutationIntent {
        deletion_intent_on(template(), start, end, bases)
    }

    // The caller's name for the edited artifact, as Milestone 2's fixture
    // table supplies a `result_id` per case.
    fn edited_name() -> Identifier {
        Identifier::create("q5-edited".to_string()).unwrap()
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
    fn an_unbound_residue_request_resolves_when_the_artifact_has_one_cds() {
        // The caller named no CDS, but the artifact identifies one, so the
        // target is resolved rather than guessed, and the residue number is
        // bound to that annotation only after resolution.
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, unbound_residue_intent()).unwrap();
        assert!(request.target_cds_name().is_none());
        let result: Q5WorkflowResult =
            validate_input_scope(request.clone(), vec![cds_of("cds-1", 30)]).unwrap();
        assert!(result.state() == Q5WorkflowState::Validated);
        let resolved: Cds = result.target().unwrap();
        assert!(resolved.id() == cds_name("cds-1"));
    }

    #[test]
    fn an_unbound_residue_request_against_several_cdss_needs_clarification() {
        // The generic gate rule for an unbound residue request: several
        // annotations and no claim identify no target, so D005 refuses to
        // choose and D020 reports a clarification that names the residue and
        // lists every candidate. The residue number here is 42, but the
        // annotations hold 10 and 4 residues, so this fixture is *not* the
        // brief's exchange: no CDS here could contain residue 42, so the
        // clarification is not reached because residue 42 is feasible in
        // several CDSs. The two tests below that use `long_template` are the
        // ones in which that reason is true of the fixture.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            unbound_residue(
                42,
                "A".to_string(),
                "G".to_string(),
                CodonSelectionPolicy::Default,
            ),
        )
        .unwrap();
        let result: Q5WorkflowResult =
            validate_input_scope(request, vec![cds_of("cds-1", 30), cds_of("cds-2", 12)]).unwrap();
        assert!(result.state() == Q5WorkflowState::NeedsClarification);
        assert!(result.target().is_none());
        assert!(result.clarifications()[0].contains("residue 42"));
        assert!(result.candidates() == vec!["cds-1".to_string(), "cds-2".to_string()]);
    }

    #[test]
    fn an_unbound_residue_request_with_a_scope_resolves_through_the_scope() {
        // The caller named no CDS in the residue intent but named one as the
        // request's scope, so the scope is the only claim and it resolves.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-2")),
            unbound_residue_intent(),
        )
        .unwrap();
        let result: Q5WorkflowResult =
            validate_input_scope(request, vec![cds_of("cds-1", 30), cds_of("cds-2", 12)]).unwrap();
        assert!(result.state() == Q5WorkflowState::Validated);
        let resolved: Cds = result.target().unwrap();
        assert!(resolved.id() == cds_name("cds-2"));
        assert!(resolved.total_length() == 12);
    }

    #[test]
    fn a_named_and_an_unbound_residue_request_resolve_to_the_same_annotation() {
        // Intent representation differs; resolution does not. Naming the CDS
        // in the residue position and naming it as the request's scope are
        // two claims that settle on one annotation the same way.
        let annotations: Vec<Cds> = vec![cds_of("cds-1", 30), cds_of("cds-2", 12)];
        let named: Q5WorkflowResult = validate_input_scope(
            Q5SdmRequest::create(template(), None, residue_intent("cds-2")).unwrap(),
            annotations.clone(),
        )
        .unwrap();
        let scoped: Q5WorkflowResult = validate_input_scope(
            Q5SdmRequest::create(
                template(),
                Some(cds_name("cds-2")),
                unbound_residue_intent(),
            )
            .unwrap(),
            annotations.clone(),
        )
        .unwrap();
        assert!(named.state() == scoped.state());
        let first: Cds = named.target().unwrap();
        let second: Cds = scoped.target().unwrap();
        assert!(first == second);
    }

    #[test]
    fn a_residue_request_without_a_cds_never_selects_one_and_never_edits() {
        // The run, not just the gate: no annotation was chosen, so no edit is
        // attempted, no artifact is produced, and no mutation claim exists.
        // Same fixture as the gate test above — annotations of 30 and 12
        // nucleotides, so 10 and 4 residues — so this is the generic
        // "several annotations, no claim" rule rather than the brief's
        // residue-42 exchange, whose stated reason needs CDSs that could
        // actually hold residue 42.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            unbound_residue(
                42,
                "A".to_string(),
                "G".to_string(),
                CodonSelectionPolicy::Default,
            ),
        )
        .unwrap();
        let annotations: Vec<Cds> = vec![cds_of("cds-1", 30), cds_of("cds-2", 12)];
        let run: Q5WorkflowExecution = execute(request, annotations, edited_name()).unwrap();
        assert!(run.needs_clarification());
        assert!(run.mutation().is_none());
        assert!(run.edited().is_none());
        assert!(run.mutation_state().is_none());
        assert!(!run.is_verified());
        assert!(run.scope().target().is_none());
        assert!(run.scope().candidates() == vec!["cds-1".to_string(), "cds-2".to_string()]);
        assert!(run.template_reference() == template().reference());
    }

    // The brief's fail-closed exchange over a fixture where its stated reason
    // is true: both annotations can hold residue 42, because 129 nucleotides
    // is 43 codons and 126 is 42. The residue letters play no part in
    // resolution, so the request keeps the brief's glycine.
    #[test]
    fn an_unbound_residue_42_request_over_two_cdss_that_could_hold_it_needs_clarification() {
        let first: Cds = long_cds_of("cds-1", 129);
        let second: Cds = long_cds_of("cds-2", 126);
        // The fixture's premise, checked rather than asserted in prose: each
        // annotation has at least 42 residues.
        assert!(first.total_length() / 3 >= 42);
        assert!(second.total_length() / 3 >= 42);
        let request: Q5SdmRequest = Q5SdmRequest::create(
            long_template(),
            None,
            unbound_residue(
                42,
                "A".to_string(),
                "G".to_string(),
                CodonSelectionPolicy::Default,
            ),
        )
        .unwrap();
        let result: Q5WorkflowResult = validate_input_scope(request, vec![first, second]).unwrap();
        assert!(result.state() == Q5WorkflowState::NeedsClarification);
        assert!(result.target().is_none());
        assert!(result.clarifications()[0].contains("residue 42"));
        assert!(result.candidates() == vec!["cds-1".to_string(), "cds-2".to_string()]);
    }

    #[test]
    fn a_residue_42_request_scoped_to_one_of_two_cdss_that_could_hold_it_edits_and_verifies() {
        // The execution path for the same exchange: the caller named one of the
        // two annotations, so the target resolved, `bind` supplied that
        // annotation's identifier, and residue 42 is a real residue of it. The
        // replacement residue is W rather than the brief's G because the residue
        // letters play no part in resolution and W's `Default` codon (TGG)
        // differs from the codon GCT already at residue 42 in every letter, so
        // all three written coordinates are visible in the edited text.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            long_template(),
            Some(cds_name("cds-1")),
            unbound_residue(
                42,
                "A".to_string(),
                "W".to_string(),
                CodonSelectionPolicy::Default,
            ),
        )
        .unwrap();
        let run: Q5WorkflowExecution = execute(
            request,
            vec![long_cds_of("cds-1", 129), long_cds_of("cds-2", 126)],
            edited_name(),
        )
        .unwrap();
        assert!(!run.needs_clarification());
        assert!(run.is_verified());
        let resolved: Cds = run.scope().target().unwrap();
        assert!(resolved.id() == cds_name("cds-1"));
        assert!(resolved.total_length() == 129);
        let done: MutationResult = run.mutation().unwrap();
        assert!(done.state() == MutationState::Verified);
        assert!(!done.is_nucleotide());
        assert!(done.source() == long_template().reference());
        assert!(done.target_coordinates() == vec![123_u64, 124_u64, 125_u64]);
        let edited: Sequence = run.edited().unwrap();
        assert!(edited.canonical_text() == long_edited_text());
        assert!(edited.length() == long_template().length());
    }

    #[test]
    fn an_unbound_residue_request_binds_to_the_cds_the_gate_resolved() {
        // Over the whole 30-nucleotide plus-strand CDS the template
        // translates to M A W N Q Q Q Q Q Q, so residue 2 is A and the named
        // `Default` policy gives W as TGG: the edit writes T G G over the
        // codon GCT at reference coordinates 3, 4, 5. The gate resolved the
        // only annotation, and that is the annotation the edit was applied to.
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, unbound_residue_intent()).unwrap();
        let run: Q5WorkflowExecution =
            execute(request, vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(!run.needs_clarification());
        assert!(run.is_verified());
        let resolved: Cds = run.scope().target().unwrap();
        assert!(resolved.id() == cds_name("cds-1"));
        assert!(resolved.total_length() == 30);
        let done: MutationResult = run.mutation().unwrap();
        assert!(done.state() == MutationState::Verified);
        assert!(!done.is_nucleotide());
        assert!(done.target_coordinates() == vec![3_u64, 4_u64, 5_u64]);
        let edited: Sequence = run.edited().unwrap();
        assert!(edited.canonical_text() == "ATGTGGTGGAATCAGCAGCAGCAGCAGCAG");
        assert!(edited.length() == template().length());
    }

    #[test]
    fn an_unbound_residue_request_scoped_to_one_of_several_cdss_runs_the_edit() {
        // The gate test above stops at resolution; this one runs the slice. The
        // scope identified one of two annotations, `bind` supplied that
        // annotation's identifier to Milestone 2's intent, and the edit and the
        // verification are Milestone 2's against the same annotation the gate
        // resolved. The edit is the one the single-annotation case produces,
        // so a scope and a resolved single annotation bind the same target.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            unbound_residue_intent(),
        )
        .unwrap();
        let run: Q5WorkflowExecution = execute(
            request,
            vec![cds_of("cds-1", 30), cds_of("cds-2", 12)],
            edited_name(),
        )
        .unwrap();
        assert!(!run.needs_clarification());
        assert!(run.is_verified());
        let resolved: Cds = run.scope().target().unwrap();
        assert!(resolved.id() == cds_name("cds-1"));
        assert!(resolved.total_length() == 30);
        let done: MutationResult = run.mutation().unwrap();
        assert!(done.state() == MutationState::Verified);
        assert!(!done.is_nucleotide());
        assert!(done.target_coordinates() == vec![3_u64, 4_u64, 5_u64]);
        let edited: Sequence = run.edited().unwrap();
        assert!(edited.canonical_text() == "ATGTGGTGGAATCAGCAGCAGCAGCAGCAG");
    }

    #[test]
    fn an_infeasible_residue_against_one_annotation_resolves_and_is_refused_downstream() {
        // One annotation, and a residue number it cannot hold: the gate reads
        // annotation metadata and artifact identity, never sequence content,
        // so it resolves the single candidate without asking whether that
        // annotation could hold residue 42, and Milestone 1's translation
        // then refuses the residue against an annotation of 4 residues.
        // Whether resolution *ought* to exclude CDSs too short to hold the
        // requested residue is an open decision (`docs/q5-sdm.md` §3, §5 and
        // §10 item 2). This pins the single-candidate case only, where
        // narrowing would have nothing to remove; no test here offers several
        // annotations of which only one could hold the residue.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            unbound_residue(
                42,
                "A".to_string(),
                "G".to_string(),
                CodonSelectionPolicy::Default,
            ),
        )
        .unwrap();
        let run: Result<Q5WorkflowExecution, DomainError> =
            execute(request, vec![cds_of("cds-1", 12)], edited_name());
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::ResiduePositionOutsideCds(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        if let DomainError::ResiduePositionOutsideCds(problem) = error {
            assert!(problem.cds == "cds-1");
            assert!(problem.number == 42);
            assert!(problem.residue_count == 4);
        }
    }

    #[test]
    fn a_residue_number_milestone_one_rejects_is_refused_before_any_edit() {
        // The residue number is validated where a CDS identifier exists, at
        // bind time, by Milestone 1's own constructor, whose payload names
        // the CDS the request was resolved against.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            unbound_residue(
                0,
                "A".to_string(),
                "W".to_string(),
                CodonSelectionPolicy::Default,
            ),
        )
        .unwrap();
        let run: Result<Q5WorkflowExecution, DomainError> =
            execute(request, vec![cds_of("cds-1", 30)], edited_name());
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::InvalidResiduePosition(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        if let DomainError::InvalidResiduePosition(problem) = error {
            assert!(problem.cds == "cds-1");
            assert!(problem.number == 0);
        }
    }

    #[test]
    fn a_negative_residue_number_is_refused_at_bind_time_by_milestone_one_s_rule() {
        // Milestone 1's `ResiduePosition::create` rejects a number below 1, so
        // a negative number is refused where a CDS identifier exists, before
        // any edit, with the same variant and payload as residue 0. The
        // workflow layer adds no residue-number rule of its own.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            unbound_residue(
                -1,
                "A".to_string(),
                "W".to_string(),
                CodonSelectionPolicy::Default,
            ),
        )
        .unwrap();
        let run: Result<Q5WorkflowExecution, DomainError> =
            execute(request, vec![cds_of("cds-1", 30)], edited_name());
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::InvalidResiduePosition(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        if let DomainError::InvalidResiduePosition(problem) = error {
            assert!(problem.cds == "cds-1");
            assert!(problem.number == -1);
        }
    }

    #[test]
    fn residue_letters_are_validated_when_the_request_is_built() {
        // A stop is not a residue, and a lower-case letter is not one either,
        // so both are refused at construction through Milestone 2's own
        // parser: no residue rule is restated at the workflow layer.
        let stop: Result<ResidueIntentWithoutCds, DomainError> = ResidueIntentWithoutCds::create(
            2,
            "A".to_string(),
            "*".to_string(),
            CodonSelectionPolicy::Default,
        );
        assert!(stop.is_err());
        let stop_error: DomainError = stop.unwrap_err();
        assert!(matches!(stop_error, DomainError::InvalidMutationRequest(_)));
        assert!(stop_error.category() == ErrorCategory::MalformedInput);

        let lower: Result<ResidueIntentWithoutCds, DomainError> = ResidueIntentWithoutCds::create(
            2,
            "A".to_string(),
            "g".to_string(),
            CodonSelectionPolicy::Default,
        );
        assert!(lower.is_err());
        assert!(matches!(
            lower.unwrap_err(),
            DomainError::InvalidMutationRequest(_)
        ));
    }

    #[test]
    fn named_cds_claims_a_cds_only_when_the_request_named_one() {
        let unbound: Q5MutationIntent = unbound_residue(
            42,
            "A".to_string(),
            "G".to_string(),
            CodonSelectionPolicy::Default,
        );
        assert!(unbound.named_cds().is_none());
        assert!(unbound.source_reference().is_none());
        let named: Q5MutationIntent = residue_intent("cds-1");
        assert!(named.named_cds() == Some(cds_name("cds-1")));
        assert!(named.source_reference().is_none());
        let nucleotide: Q5MutationIntent = nucleotide_intent();
        assert!(nucleotide.named_cds().is_none());
    }

    #[test]
    fn a_validated_result_carries_a_target_and_a_clarification_result_does_not() {
        // The state invariants, for both residue shapes: `Validated` carries
        // exactly one resolved CDS and a clarification carries none, a
        // clarification text, and the candidates on offer.
        let annotations: Vec<Cds> = vec![cds_of("cds-1", 30), cds_of("cds-2", 12)];
        let decided: Q5WorkflowResult = validate_input_scope(
            Q5SdmRequest::create(
                template(),
                Some(cds_name("cds-1")),
                unbound_residue_intent(),
            )
            .unwrap(),
            annotations.clone(),
        )
        .unwrap();
        let ambiguous: Q5WorkflowResult = validate_input_scope(
            Q5SdmRequest::create(template(), None, unbound_residue_intent()).unwrap(),
            annotations.clone(),
        )
        .unwrap();
        assert!(decided.state().is_validated());
        assert!(decided.target().is_some());
        assert!(decided.clarifications().is_empty());
        assert!(ambiguous.needs_clarification());
        assert!(ambiguous.target().is_none());
        assert!(ambiguous.clarifications().len() == 1);
        assert!(ambiguous.candidates().len() == 2);
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
    fn an_unbound_residue_request_naming_an_unannotated_cds_needs_clarification() {
        // The scope is the only claim an unbound residue intent has, so an
        // unknown identifier is the same target-not-found case the nucleotide
        // form has, reached through the residue shape: a clarification naming
        // what the artifact *is* annotated with, not an error. Reporting
        // target-not-found as a clarification rather than a structured error
        // is the open decision in `docs/q5-sdm.md` §10 item 3, pinned here
        // for the unbound residue shape alongside the nucleotide form and the
        // named residue intent. That a single annotation can identify the
        // target with no claim at all is the separate decision in §10 item 4.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-9")),
            unbound_residue_intent(),
        )
        .unwrap();
        let result: Q5WorkflowResult =
            validate_input_scope(request, vec![cds_of("cds-1", 30), cds_of("cds-2", 12)]).unwrap();
        assert!(result.state() == Q5WorkflowState::NeedsClarification);
        assert!(result.target().is_none());
        assert!(result.clarifications()[0].contains("cds-9"));
        assert!(result.clarifications()[0].contains("'cds-1'"));
        assert!(result.candidates() == vec!["cds-1".to_string(), "cds-2".to_string()]);
    }

    #[test]
    fn a_named_residue_intent_naming_an_unannotated_cds_needs_clarification() {
        // The claim comes from the residue position rather than from the
        // request's scope, and the outcome is the same: the caller can resolve
        // it by naming an annotation that exists.
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, residue_intent("cds-9")).unwrap();
        let result: Q5WorkflowResult =
            validate_input_scope(request, vec![cds_of("cds-1", 30), cds_of("cds-2", 12)]).unwrap();
        assert!(result.state() == Q5WorkflowState::NeedsClarification);
        assert!(result.target().is_none());
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

    // The vertical slice of D015: gate, edit, independent verification,
    // structured result. Over the whole 30-nucleotide plus-strand CDS the
    // template translates to M A W N Q Q Q Q Q Q, so residue 2 is A, and the
    // named `Default` policy gives W as TGG: the edit writes T G G over the
    // codon GCT at reference coordinates 3, 4, 5.
    #[test]
    fn a_validated_residue_request_edits_the_template_and_verifies_the_edit() {
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, residue_intent("cds-1")).unwrap();
        let run: Q5WorkflowExecution =
            execute(request, vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(!run.needs_clarification());
        assert!(run.is_verified());
        let done: MutationResult = run.mutation().unwrap();
        assert!(done.state() == MutationState::Verified);
        assert!(!done.is_nucleotide());
        assert!(done.source() == template().reference());
        // The gate resolved the annotation the residue position named, so the
        // edit was applied to that annotation and to nothing else.
        let resolved: Cds = run.scope().target().unwrap();
        assert!(resolved.id() == cds_name("cds-1"));
        assert!(resolved.total_length() == 30);
    }

    #[test]
    fn the_run_never_reports_a_calculated_result_as_verified() {
        // The same edit without the verification pass is `Calculated`, and it
        // differs from the run's result only in state: D020's `calculated`
        // never stands for `verified`, and the run hands back the verified
        // one.
        let calculated: MutationResult = apply_residue(
            &template(),
            cds_of("cds-1", 30),
            edited_name(),
            residue_mutation_intent(
                "cds-1",
                "A".to_string(),
                "W".to_string(),
                CodonSelectionPolicy::Default,
            ),
        )
        .unwrap();
        assert!(calculated.state() == MutationState::Calculated);
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, residue_intent("cds-1")).unwrap();
        let run: Q5WorkflowExecution =
            execute(request, vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        let done: MutationResult = run.mutation().unwrap();
        assert!(done.state() == MutationState::Verified);
        assert!(done != calculated);
        assert!(run.is_verified());
    }

    #[test]
    fn a_clarification_run_performs_no_edit_and_produces_no_artifact() {
        // The request named no CDS and the record carries two, so D005
        // refuses to choose and the run stops at the gate.
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, nucleotide_intent()).unwrap();
        let annotations: Vec<Cds> = vec![cds_of("cds-1", 30), cds_of("cds-2", 12)];
        let run: Q5WorkflowExecution =
            execute(request, annotations.clone(), edited_name()).unwrap();
        assert!(run.needs_clarification());
        assert!(run.mutation().is_none());
        assert!(run.edited().is_none());
        assert!(run.mutation_state().is_none());
        assert!(!run.is_verified());
        assert!(run.scope().target().is_none());
        assert!(run.scope().candidates() == vec!["cds-1".to_string(), "cds-2".to_string()]);
        // The artifact is still identified, so the clarification says which
        // record it was about (D007).
        assert!(run.template_reference() == template().reference());
        // The gate's decision is carried as it produced it, so the run
        // restates none of it.
        let gate: Q5WorkflowResult = validate_input_scope(
            Q5SdmRequest::create(template(), None, nucleotide_intent()).unwrap(),
            annotations.clone(),
        )
        .unwrap();
        assert!(run.scope() == gate);
    }

    #[test]
    fn the_run_carries_the_scope_gate_and_the_request_unchanged() {
        // The slice reuses the gate rather than restating it, and the
        // interpreted request stays inspectable alongside the decision
        // (D004, D016).
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, residue_intent("cds-1")).unwrap();
        let annotations: Vec<Cds> = vec![cds_of("cds-1", 30)];
        let gate: Q5WorkflowResult =
            validate_input_scope(request.clone(), annotations.clone()).unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), annotations.clone(), edited_name()).unwrap();
        assert!(run.scope() == gate);
        assert!(run.scope().request() == request);
        assert!(run.needs_clarification() == gate.needs_clarification());
    }

    #[test]
    fn a_gate_failure_propagates_without_an_edit() {
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), Some(cds_name("cds-1")), nucleotide_intent()).unwrap();
        let run: Result<Q5WorkflowExecution, DomainError> =
            execute(request, Vec::new(), edited_name());
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::MissingCdsAttribute(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    #[test]
    fn a_wrong_expected_residue_stops_the_run_before_any_artifact() {
        // Residue 2 of the whole CDS is A, so asserting W is a well-formed
        // request that disagrees with the artifact: Milestone 2's failure,
        // reached through the run with its variant and category intact.
        let intent: Q5MutationIntent = Q5MutationIntent::Residue(residue_mutation_intent(
            "cds-1",
            "W".to_string(),
            "N".to_string(),
            CodonSelectionPolicy::Default,
        ));
        let request: Q5SdmRequest = Q5SdmRequest::create(template(), None, intent).unwrap();
        let run: Result<Q5WorkflowExecution, DomainError> =
            execute(request, vec![cds_of("cds-1", 30)], edited_name());
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::ExpectedResidueMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        if let DomainError::ExpectedResidueMismatch(problem) = error {
            assert!(problem.cds == "cds-1");
            assert!(problem.number == 2);
            assert!(problem.expected == "W");
            assert!(problem.actual == "A");
        }
    }

    #[test]
    fn an_unspecified_codon_policy_is_refused_by_the_run() {
        // D011: no codon-selection policy may be inferred silently, and the
        // run does not supply one on the caller's behalf. The scope resolved,
        // so this is Milestone 2's failure and not a clarification.
        let intent: Q5MutationIntent = Q5MutationIntent::Residue(residue_mutation_intent(
            "cds-1",
            "A".to_string(),
            "W".to_string(),
            CodonSelectionPolicy::Unspecified,
        ));
        let request: Q5SdmRequest = Q5SdmRequest::create(template(), None, intent).unwrap();
        let run: Result<Q5WorkflowExecution, DomainError> =
            execute(request, vec![cds_of("cds-1", 30)], edited_name());
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::UnspecifiedCodonPolicy(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    #[test]
    fn an_unspecified_codon_policy_on_the_unbound_form_is_refused_by_the_run() {
        // Same refusal as the named form: `bind` carries the policy through
        // unchanged, so resolving a target does not let the workflow choose a
        // codon policy on the caller's behalf, and the scope resolved so this
        // is Milestone 2's failure and not a clarification.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            unbound_residue(
                2,
                "A".to_string(),
                "W".to_string(),
                CodonSelectionPolicy::Unspecified,
            ),
        )
        .unwrap();
        let run: Result<Q5WorkflowExecution, DomainError> =
            execute(request, vec![cds_of("cds-1", 30)], edited_name());
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::UnspecifiedCodonPolicy(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    #[test]
    fn a_scoped_nucleotide_edit_inside_the_resolved_cds_is_applied_and_verified() {
        // Coordinate 3 lies inside a 12-nucleotide CDS, and the request
        // scoped that CDS, so Milestone 2's containment rule applies and is
        // satisfied: the run edits and verifies rather than refusing.
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), Some(cds_name("cds-1")), nucleotide_intent_at(3))
                .unwrap();
        let run: Q5WorkflowExecution =
            execute(request, vec![cds_of("cds-1", 12)], edited_name()).unwrap();
        assert!(!run.needs_clarification());
        assert!(run.is_verified());
        let resolved: Cds = run.scope().target().unwrap();
        assert!(resolved.id() == cds_name("cds-1"));
        assert!(resolved.total_length() == 12);
        let done: MutationResult = run.mutation().unwrap();
        assert!(done.state() == MutationState::Verified);
        assert!(done.is_nucleotide());
        assert!(done.source() == template().reference());
        assert!(done.target_coordinates() == vec![3_u64]);
        // The known answer: coordinate 3 holds G in the template, so the
        // edit writes C there and changes nothing else.
        let edited: Sequence = run.edited().unwrap();
        assert!(template().letter_at(3).unwrap() == 'G');
        assert!(edited.letter_at(3).unwrap() == 'C');
        assert!(edited.canonical_text() == "ATGCCTTGGAATCAGCAGCAGCAGCAGCAG");
        assert!(edited.length() == template().length());
    }

    #[test]
    fn a_scoped_nucleotide_edit_outside_the_resolved_cds_is_refused() {
        // Coordinate 20 lies outside a 12-nucleotide CDS, and the request
        // scoped that CDS, so Milestone 2's containment rule applies.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            nucleotide_intent_at(20),
        )
        .unwrap();
        let run: Result<Q5WorkflowExecution, DomainError> =
            execute(request, vec![cds_of("cds-1", 12)], edited_name());
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::TargetOutsideFeature(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn an_unscoped_nucleotide_edit_outside_the_resolved_cds_proceeds() {
        // The request scoped nothing, so the gate's resolution of the only
        // annotation says which CDS the request is about; it does not assert
        // that the DNA edit lies inside it. Milestone 2 keeps scoping a
        // parameter of the operation, not a property of the edit, and its
        // fixture `nuc-a-unscoped` pins an unscoped DNA edit as legal in the
        // mutation domain.
        //
        // Whether the workflow *ought* to impose that containment is open
        // (`docs/q5-sdm.md` §4, §10 item 1), and this test is named for what
        // it pins: Alternative 1, explicit-scope semantics, as expected
        // behavior. Coordinate 20 lies outside the 12-nucleotide CDS the gate
        // resolved, and the run edits and verifies rather than refusing. Under
        // Alternative 2 the gate's resolution would become a containment rule,
        // `nucleotide_scope` would hand that annotation to the operation, and
        // Milestone 2's `TargetOutsideFeature` would make this run an error,
        // so this test encodes one alternative of an unsettled decision and
        // would change if the product chose the other. It is recorded as impl,
        // not as a settled product rule.
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, nucleotide_intent_at(20)).unwrap();
        let run: Q5WorkflowExecution =
            execute(request, vec![cds_of("cds-1", 12)], edited_name()).unwrap();
        assert!(!run.needs_clarification());
        assert!(run.is_verified());
        let edited: Sequence = run.edited().unwrap();
        assert!(edited.name() == edited_name());
        assert!(edited.length() == template().length());
    }

    #[test]
    fn the_edited_artifact_is_new_and_the_source_keeps_its_identity() {
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, residue_intent("cds-1")).unwrap();
        let run: Q5WorkflowExecution =
            execute(request, vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        let edited: Sequence = run.edited().unwrap();
        let done: MutationResult = run.mutation().unwrap();
        // A substitution preserves length, so the source keeps its digest and
        // the edited artifact is a different artifact by name and digest
        // (D007).
        assert!(edited.length() == template().length());
        assert!(edited.content_hash() != template().content_hash());
        assert!(edited.name() == edited_name());
        assert!(done.source() == template().reference());
        assert!(done.reference() == edited.reference());
        assert!(run.template_reference() == template().reference());
    }

    #[test]
    fn the_run_returns_a_verified_result_under_the_caller_s_result_name() {
        // The display name of the edited artifact is the caller's, exactly as
        // Milestone 2's fixture table supplies a `result_id` per case, and
        // this layer names none of its own (D007, D020).
        let chosen: Identifier = Identifier::create("caller-chosen-result".to_string()).unwrap();
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, residue_intent("cds-1")).unwrap();
        let run: Q5WorkflowExecution =
            execute(request, vec![cds_of("cds-1", 30)], chosen.clone()).unwrap();
        assert!(run.is_verified());
        let done: MutationResult = run.mutation().unwrap();
        assert!(done.state() == MutationState::Verified);
        let edited: Sequence = run.edited().unwrap();
        assert!(edited.name() == chosen);
        assert!(edited.name() != template().name());
        assert!(edited.name() != edited_name());
        assert!(done.reference() == edited.reference());
        assert!(done.source() == template().reference());
    }

    // --- the variant-set API --------------------------------------------------
    //
    // The product brief's flagship exchange, `A42G and W87A as separate
    // variants`, needs an artifact long enough that residue 87 is a real
    // possibility. The letters are chosen so the residues are known from the
    // published standard genetic code rather than read off the
    // implementation: residues 1..41 and 43..86 are `ATG` (M), residue 42 is
    // `GCT` (A), and residue 87 is `TGG` (W), which is the CDS's last codon so
    // Milestone 1's internal-stop rule accepts it. That is 87 codons and 261
    // nucleotides, divisible by three, with no ambiguous codon. Residue 42
    // occupies reference coordinates 123, 124, 125 and residue 87 occupies
    // 258, 259, 260.
    fn flagship_text() -> String {
        let mut text: String = String::new();
        let mut index: usize = 0;
        while index < 41 {
            text += "ATG";
            index += 1;
        }
        text += "GCT";
        index = 0;
        while index < 44 {
            text += "ATG";
            index += 1;
        }
        text + "TGG"
    }

    // The flagship artifact shares `template()`'s display name, so a member
    // over one and a member over the other is the D007 case: same name,
    // different content digest. `other()` differs by name as well.
    fn flagship() -> Sequence {
        normalize(
            Identifier::create("q5-a".to_string()).unwrap(),
            flagship_text(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    // A whole-CDS annotation over the flagship artifact, so one annotation can
    // hold both residue 42 and residue 87.
    fn flagship_cds(name: &str, length: i64) -> Cds {
        let intervals: OrderedIntervals = vec![create(&flagship().reference(), 0, length).unwrap()];
        Cds::create(
            &flagship(),
            cds_name(name),
            intervals,
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap()
    }

    // The codon the composition tests write for each replacement residue they
    // use. These tests are about composition, not codon choice, so the codon
    // is supplied rather than left to the `Default` policy: G is `GGT`, A is
    // `GCT`, W is `TGG`, the codons the expected edits below were derived for.
    // (`Default` now picks the most used E. coli K-12 codon, `GGC` for G and
    // `GCG` for A; Milestone 2's tests pin that policy.)
    fn pinned_policy(replacement: &str) -> CodonSelectionPolicy {
        let codon: &str = match replacement {
            "G" => "GGT",
            "A" => "GCT",
            "W" => "TGG",
            other => panic!("no pinned codon for residue {}", other),
        };
        CodonSelectionPolicy::Supplied(Codon::create(codon.to_string()).unwrap())
    }

    // A residue substitution over a named CDS at an arbitrary residue number,
    // with the codon pinned by `pinned_policy`, so the flagship's residue 42
    // and residue 87 members can be written out.
    fn named_residue_at(
        cds: &str,
        number: i64,
        expected: &str,
        replacement: &str,
    ) -> Q5MutationIntent {
        named_residue_at_policy(
            cds,
            number,
            expected,
            replacement,
            pinned_policy(replacement),
        )
    }

    // The same intent with the codon policy supplied, so a test can ask for a
    // policy Milestone 2 refuses to choose for the caller.
    fn named_residue_at_policy(
        cds: &str,
        number: i64,
        expected: &str,
        replacement: &str,
        policy: CodonSelectionPolicy,
    ) -> Q5MutationIntent {
        let position: ResiduePosition = ResiduePosition::create(cds_name(cds), number).unwrap();
        Q5MutationIntent::Residue(
            ResidueMutationIntent::create(
                position,
                expected.to_string(),
                replacement.to_string(),
                policy.clone(),
            )
            .unwrap(),
        )
    }

    fn unbound_residue_at(number: i64, expected: &str, replacement: &str) -> Q5MutationIntent {
        Q5MutationIntent::ResidueWithoutCds(
            ResidueIntentWithoutCds::create(
                number,
                expected.to_string(),
                replacement.to_string(),
                pinned_policy(replacement),
            )
            .unwrap(),
        )
    }

    // One member of a flagship set: the request names the flagship artifact
    // and the CDS, and the caller supplies the name for that member's result.
    fn flagship_member(
        cds: &str,
        number: i64,
        expected: &str,
        replacement: &str,
        result: &str,
    ) -> Q5Variant {
        let request: Q5SdmRequest = Q5SdmRequest::create(
            flagship(),
            None,
            named_residue_at(cds, number, expected, replacement),
        )
        .unwrap();
        Q5Variant::create(request, Identifier::create(result.to_string()).unwrap())
    }

    // The same member but naming no CDS anywhere, so only the set's agreed
    // claim can resolve it.
    fn flagship_member_unbound(
        number: i64,
        expected: &str,
        replacement: &str,
        result: &str,
    ) -> Q5Variant {
        let request: Q5SdmRequest = Q5SdmRequest::create(
            flagship(),
            None,
            unbound_residue_at(number, expected, replacement),
        )
        .unwrap();
        Q5Variant::create(request, Identifier::create(result.to_string()).unwrap())
    }

    // The same member but claiming the CDS through the request's scope rather
    // than through the residue position, so the set's claim-agreement check
    // reads a claim of a different shape.
    fn flagship_member_scoped(
        cds: &str,
        number: i64,
        expected: &str,
        replacement: &str,
        result: &str,
    ) -> Q5Variant {
        let request: Q5SdmRequest = Q5SdmRequest::create(
            flagship(),
            Some(cds_name(cds)),
            unbound_residue_at(number, expected, replacement),
        )
        .unwrap();
        Q5Variant::create(request, Identifier::create(result.to_string()).unwrap())
    }

    // A member whose request addresses a different artifact, so the set's
    // template check has something to refuse. The request itself is well
    // formed: Milestone 3's own rule checks an intent's artifact only for a
    // DNA edit, and a residue member is refused by the set, not by its own
    // request.
    fn member_over(
        artifact: Sequence,
        cds: &str,
        number: i64,
        expected: &str,
        replacement: &str,
        result: &str,
    ) -> Q5Variant {
        let request: Q5SdmRequest = Q5SdmRequest::create(
            artifact.clone(),
            None,
            named_residue_at(cds, number, expected, replacement),
        )
        .unwrap();
        Q5Variant::create(request, Identifier::create(result.to_string()).unwrap())
    }

    // Letters of an artifact over a coordinate range, so a test can name the
    // codon a variant rewrote and the codon it left untouched.
    fn letters_over(sequence: &Sequence, start: usize, end: usize) -> String {
        let letters: Vec<char> = sequence.letters();
        let mut text: String = String::new();
        let mut index: usize = start;
        while index < end {
            text = format!("{}{}", text, letters[index]);
            index += 1;
        }
        text
    }

    fn a42g_result() -> Identifier {
        Identifier::create("res-a42g".to_string()).unwrap()
    }

    fn w87a_result() -> Identifier {
        Identifier::create("res-w87a".to_string()).unwrap()
    }

    fn m1w_result() -> Identifier {
        Identifier::create("res-m1w".to_string()).unwrap()
    }

    #[test]
    fn an_empty_variant_set_is_refused_at_construction() {
        // A set with nothing to run is not representable, so the refusal is a
        // construction error rather than a run that returns no results.
        let built: Result<Q5VariantSetRequest, DomainError> =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, Vec::new());
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        if let DomainError::InvalidMutationRequest(problem) = error {
            assert!(problem.request == "q5-variant-set");
        }
    }

    #[test]
    fn a_combined_construct_is_refused_before_any_member_resolves_or_edits() {
        // The flagship pair asked as one construct rather than as separate
        // variants. D011 makes these different operations, and composing them
        // needs overlap rules no milestone has defined, so the mode is refused
        // at construction: no target resolves, nothing is edited, and the
        // refusal is not quietly reinterpreted as the separate-variants run
        // the caller did not ask for.
        let members: Vec<Q5Variant> = vec![
            flagship_member("cds-1", 42, "A", "G", "res-a42g"),
            flagship_member("cds-1", 87, "W", "A", "res-w87a"),
        ];
        let built: Result<Q5VariantSetRequest, DomainError> =
            Q5VariantSetRequest::create(Q5Composition::CombinedConstruct, members.clone());
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::UnsupportedComposition(_)));
        assert!(error.category() == ErrorCategory::Unsupported);
        if let DomainError::UnsupportedComposition(problem) = error {
            assert!(problem.composition == "combined_construct");
            assert!(problem.variant_count == 2);
        }
    }

    #[test]
    fn a_one_member_combined_construct_is_refused_so_the_mode_is_not_counted() {
        // The composition mode is a parameter of `create`, never something
        // inferred from how many members were supplied: a set of one still
        // names the operation it means, so a combined construct of one member
        // is refused exactly as a pair is, and a one-member set is not a
        // single-request call with the single request's undecided rules.
        let members: Vec<Q5Variant> = vec![flagship_member("cds-1", 42, "A", "G", "res-a42g")];
        let built: Result<Q5VariantSetRequest, DomainError> =
            Q5VariantSetRequest::create(Q5Composition::CombinedConstruct, members.clone());
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::UnsupportedComposition(_)));
        assert!(error.category() == ErrorCategory::Unsupported);
        if let DomainError::UnsupportedComposition(problem) = error {
            assert!(problem.composition == "combined_construct");
            assert!(problem.variant_count == 1);
        }
    }

    #[test]
    fn separate_variants_are_independent_and_reported_in_request_order() {
        // The flagship exchange run as separate variants over one annotation.
        // Residue 42 is A (`GCT`) and residue 87 is W (`TGG`), so the first
        // member rewrites codon 42 to the `Default` policy's codon for G
        // (`GGT`) and the second rewrites codon 87 to the `Default` policy's
        // codon for A (`GCT`). Each member starts from the pristine flagship
        // artifact, so each edited artifact carries only its own change: codon
        // 87 is still `TGG` in the first and codon 42 is still `GCT` in the
        // second. Neither result is the other's edit applied twice.
        let members: Vec<Q5Variant> = vec![
            flagship_member("cds-1", 42, "A", "G", "res-a42g"),
            flagship_member("cds-1", 87, "W", "A", "res-w87a"),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        let run: Q5VariantSetExecution =
            execute_set(set.clone(), vec![flagship_cds("cds-1", 261)]).unwrap();

        assert!(!run.needs_clarification());
        assert!(run.is_verified());
        assert!(run.result_count() == 2);
        let target: Cds = run.target().unwrap();
        assert!(target.id() == cds_name("cds-1"));

        let results: Vec<Q5VariantResult> = run.results();
        assert!(results[0].identifier() == a42g_result());
        assert!(results[1].identifier() == w87a_result());
        let first: MutationResult = results[0].mutation();
        let second: MutationResult = results[1].mutation();
        assert!(first.state() == MutationState::Verified);
        assert!(second.state() == MutationState::Verified);
        // Each member was applied to the pristine template, not to a previous
        // member's edited artifact.
        assert!(first.source() == flagship().reference());
        assert!(second.source() == flagship().reference());

        let first_edited: Sequence = results[0].edited();
        let second_edited: Sequence = results[1].edited();
        assert!(first_edited.length() == flagship().length());
        assert!(second_edited.length() == flagship().length());
        assert!(letters_over(&first_edited, 123, 126) == "GGT");
        assert!(letters_over(&first_edited, 258, 261) == "TGG");
        assert!(letters_over(&second_edited, 123, 126) == "GCT");
        assert!(letters_over(&second_edited, 258, 261) == "GCT");
        assert!(letters_over(&flagship(), 123, 126) == "GCT");
        assert!(letters_over(&flagship(), 258, 261) == "TGG");
        // Two different artifacts, each named by its own member.
        assert!(first_edited.content_hash() != second_edited.content_hash());
        assert!(first_edited.content_hash() != flagship().content_hash());
        assert!(first.reference() != second.reference());
        assert!(first_edited.name() == a42g_result());
        assert!(second_edited.name() == w87a_result());

        // What the application understood stays inspectable (D004, D016).
        assert!(run.request() == set.clone());
        assert!(run.composition() == Q5Composition::SeparateVariants);
        assert!(run.template_reference() == flagship().reference());
    }

    #[test]
    fn a_one_member_separate_variant_set_reports_one_result() {
        // A set of one separate variant runs that one member: the mode, not
        // the member count, decides what the request means.
        let members: Vec<Q5Variant> = vec![flagship_member("cds-1", 42, "A", "G", "res-a42g")];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        let run: Q5VariantSetExecution =
            execute_set(set.clone(), vec![flagship_cds("cds-1", 261)]).unwrap();
        assert!(run.result_count() == 1);
        assert!(run.is_verified());
        assert!(run.results()[0].identifier() == a42g_result());
        assert!(letters_over(&run.results()[0].edited(), 123, 126) == "GGT");
        assert!(letters_over(&run.results()[0].edited(), 258, 261) == "TGG");
        assert!(run.request().variant_count() == 1);
        assert!(run.composition() == Q5Composition::SeparateVariants);
    }

    #[test]
    fn reversing_the_members_reverses_the_results_without_changing_an_edit() {
        // Request order is preserved, and independence holds in both orders:
        // each member's edited artifact carries only its own change whichever
        // member ran first, so the order decides only the order of reporting.
        let members: Vec<Q5Variant> = vec![
            flagship_member("cds-1", 87, "W", "A", "res-w87a"),
            flagship_member("cds-1", 42, "A", "G", "res-a42g"),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        let run: Q5VariantSetExecution =
            execute_set(set.clone(), vec![flagship_cds("cds-1", 261)]).unwrap();
        let results: Vec<Q5VariantResult> = run.results();
        assert!(results[0].identifier() == w87a_result());
        assert!(results[1].identifier() == a42g_result());
        assert!(run.is_verified());
        assert!(letters_over(&results[0].edited(), 123, 126) == "GCT");
        assert!(letters_over(&results[0].edited(), 258, 261) == "GCT");
        assert!(letters_over(&results[1].edited(), 123, 126) == "GGT");
        assert!(letters_over(&results[1].edited(), 258, 261) == "TGG");
    }

    #[test]
    fn members_over_different_template_artifacts_are_refused_at_construction() {
        // A set is one request about one record: every member must address the
        // same template artifact, checked with Milestone 1's identity rule and
        // its own error variant, so a set cannot mix two records.
        let members: Vec<Q5Variant> = vec![
            flagship_member("cds-1", 42, "A", "G", "res-a42g"),
            member_over(other(), "cds-1", 87, "W", "A", "res-w87a"),
        ];
        let built: Result<Q5VariantSetRequest, DomainError> =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone());
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        if let DomainError::SequenceReferenceMismatch(problem) = error {
            assert!(problem.expected_identifier == "q5-b");
            assert!(problem.actual_identifier == "q5-a");
        }
    }

    #[test]
    fn a_member_over_a_same_named_different_artifact_is_refused_by_digest() {
        // D007: artifacts are kept apart by name *and* content digest, so a
        // member over a different artifact that shares the display name is
        // refused rather than merged with the set's template. The payload
        // names the same identifier twice and two digests.
        let members: Vec<Q5Variant> = vec![
            flagship_member("cds-1", 42, "A", "G", "res-a42g"),
            member_over(template(), "cds-1", 87, "W", "A", "res-w87a"),
        ];
        let built: Result<Q5VariantSetRequest, DomainError> =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone());
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        if let DomainError::SequenceReferenceMismatch(problem) = error {
            assert!(problem.expected_identifier == "q5-a");
            assert!(problem.actual_identifier == "q5-a");
            assert!(problem.expected_digest != problem.actual_digest);
        }
    }

    #[test]
    fn members_that_claim_different_cdss_are_refused_at_construction() {
        // Two explicit claims name two annotations, so no set-level target
        // exists and the set is refused with Milestone 2's own variant rather
        // than by choosing one claim or running each member separately.
        let members: Vec<Q5Variant> = vec![
            flagship_member("cds-1", 42, "A", "G", "res-a42g"),
            flagship_member("cds-2", 87, "W", "A", "res-w87a"),
        ];
        let built: Result<Q5VariantSetRequest, DomainError> =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone());
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::ResiduePositionWrongCds(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        if let DomainError::ResiduePositionWrongCds(problem) = error {
            assert!(problem.expected == "cds-1");
            assert!(problem.actual == "cds-2");
        }
    }

    #[test]
    fn a_member_s_claim_from_its_scope_agrees_with_another_member_s_intent_claim() {
        // The claims a set compares come from either shape of a Q5 request:
        // one member claims the CDS through its scope, the other through its
        // residue position. The two agree, so the set is accepted and both
        // members are applied to the same annotation.
        let members: Vec<Q5Variant> = vec![
            flagship_member_scoped("cds-1", 42, "A", "G", "res-a42g"),
            flagship_member("cds-1", 87, "W", "A", "res-w87a"),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        assert!(set.target_cds_name().unwrap() == cds_name("cds-1"));
        let run: Q5VariantSetExecution =
            execute_set(set.clone(), vec![flagship_cds("cds-1", 261)]).unwrap();
        assert!(run.is_verified());
        assert!(run.result_count() == 2);
        assert!(run.target().unwrap().id() == cds_name("cds-1"));
        assert!(letters_over(&run.results()[0].edited(), 123, 126) == "GGT");
        assert!(letters_over(&run.results()[1].edited(), 258, 261) == "GCT");
    }

    #[test]
    fn an_unbound_member_takes_the_target_the_set_resolved() {
        // Two annotations and one member that claims nothing. Asked on its own
        // that member clarifies, because several annotations and no claim
        // identify no target; asked as a member of a set that does claim one
        // CDS, the set's agreed claim is the claim the gate is asked with, so
        // the member is applied to that annotation and edits rather than
        // clarifying for itself. The member's own request is not rewritten.
        let members: Vec<Q5Variant> = vec![
            flagship_member("cds-1", 42, "A", "G", "res-a42g"),
            flagship_member_unbound(87, "W", "A", "res-w87a"),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        assert!(set.target_cds_name().unwrap() == cds_name("cds-1"));
        let annotations: Vec<Cds> = vec![flagship_cds("cds-1", 261), flagship_cds("cds-2", 150)];
        let run: Q5VariantSetExecution = execute_set(set.clone(), annotations.clone()).unwrap();
        assert!(!run.needs_clarification());
        assert!(run.is_verified());
        assert!(run.result_count() == 2);
        assert!(run.target().unwrap().id() == cds_name("cds-1"));
        // The unbound member edited, and its result carries its own name.
        assert!(run.results()[1].identifier() == w87a_result());
        assert!(run.results()[1].is_verified());
        assert!(letters_over(&run.results()[1].edited(), 258, 261) == "GCT");
        assert!(letters_over(&run.results()[1].edited(), 123, 126) == "GCT");
    }

    #[test]
    fn a_set_that_needs_clarification_returns_no_results_and_edits_nothing() {
        // Neither member claims a CDS and the record carries two, so D005
        // refuses to choose for the set rather than for a member and D020
        // gives that ambiguity its own state: no member is edited, no
        // artifact is produced for any member, and the gate's clarification is
        // carried whole rather than restated per member.
        let members: Vec<Q5Variant> = vec![
            flagship_member_unbound(42, "A", "G", "res-a42g"),
            flagship_member_unbound(87, "W", "A", "res-w87a"),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        assert!(set.target_cds_name().is_none());
        let annotations: Vec<Cds> = vec![flagship_cds("cds-1", 261), flagship_cds("cds-2", 150)];
        let run: Q5VariantSetExecution = execute_set(set.clone(), annotations.clone()).unwrap();
        assert!(run.needs_clarification());
        assert!(run.scope().state() == Q5WorkflowState::NeedsClarification);
        assert!(run.scope().target().is_none());
        assert!(run.target().is_none());
        assert!(run.result_count() == 0);
        assert!(run.results().is_empty());
        assert!(run.edited_artifacts().is_empty());
        // A set that performed no edit is not verified (D020).
        assert!(!run.is_verified());
        assert!(run.scope().candidates() == vec!["cds-1".to_string(), "cds-2".to_string()]);
        // The artifact is still identified, so the clarification says which
        // record the set was about (D007).
        assert!(run.template_reference() == flagship().reference());
    }

    #[test]
    fn a_failing_member_ends_the_set_and_no_earlier_result_escapes() {
        // The flagship pair over an annotation of 50 codons: residue 42 is
        // inside it and residue 87 is not, so the second member fails with
        // Milestone 2's own error. Members run in request order and the set
        // returns that original `DomainError` — same variant, same category,
        // same payload — so the first member's edit is not reported as a
        // successful set and no partial result escapes.
        let members: Vec<Q5Variant> = vec![
            flagship_member("cds-1", 42, "A", "G", "res-a42g"),
            flagship_member("cds-1", 87, "W", "A", "res-w87a"),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        let run: Result<Q5VariantSetExecution, DomainError> =
            execute_set(set.clone(), vec![flagship_cds("cds-1", 150)]);
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::ResiduePositionOutsideCds(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        if let DomainError::ResiduePositionOutsideCds(problem) = error {
            assert!(problem.cds == "cds-1");
            assert!(problem.number == 87);
            assert!(problem.residue_count == 50);
        }
    }

    #[test]
    fn the_set_request_and_its_scope_decision_are_carried_into_the_execution() {
        // D004 and D016: what the application understood stays inspectable
        // beside what it decided, so the execution carries the set request
        // whole and the gate's `Q5WorkflowResult` exactly as `validate_input_scope`
        // produced it for the set's agreed claim. The run restates none of it.
        let members: Vec<Q5Variant> = vec![
            flagship_member("cds-1", 42, "A", "G", "res-a42g"),
            flagship_member("cds-1", 87, "W", "A", "res-w87a"),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        let annotations: Vec<Cds> = vec![flagship_cds("cds-1", 261)];
        let run: Q5VariantSetExecution = execute_set(set.clone(), annotations.clone()).unwrap();
        let gate: Q5WorkflowResult =
            validate_input_scope(set_gate_request(set.clone()).unwrap(), annotations.clone())
                .unwrap();
        assert!(run.scope() == gate.clone());
        assert!(run.request() == set.clone());
        assert!(gate.state() == Q5WorkflowState::Validated);
        assert!(gate.target().unwrap().id() == cds_name("cds-1"));
        // The gate was asked with the set's agreed claim, so it names the
        // artifact the set is about and the CDS every member was applied to.
        assert!(gate.request().template_reference() == flagship().reference());
        assert!(gate.request().target_cds_name().unwrap() == cds_name("cds-1"));
        assert!(run.scope().request().target_cds_name().unwrap() == cds_name("cds-1"));
    }

    // --- the set API against the long fixture, and the rules a member keeps --
    //
    // The long fixture is 41 `ATG` codons then `GCT` and `TGG`: 129
    // nucleotides, 43 residues, residue 42 alanine at coordinates 123-125,
    // residue 43 tryptophan at 126-128, residue 1 methionine at 0-2. `W`
    // is tryptophan, not a stop, so no residue substitution over this CDS
    // can produce one: the stops of the standard code are `TAA`, `TAG` and
    // `TGA`, and `default_codon` skips a `Stop` outcome when it resolves a
    // policy. An internal stop therefore reaches the set path only as a
    // property of the annotation a member is applied to, which is what the
    // fixture and test below pin.
    // Coordinates at which two artifacts differ, so a test can name the
    // letters a variant changed and prove it changed nothing else.
    fn differing_coordinates(first: &Sequence, second: &Sequence) -> Vec<usize> {
        let left: Vec<char> = first.letters();
        let right: Vec<char> = second.letters();
        let mut got: Vec<usize> = Vec::new();
        let count: usize = left.len();
        let mut index: usize = 0;
        while index < count {
            if left[index] != right[index] {
                got.push(index);
            }
            index += 1;
        }
        got
    }

    // A member over the long fixture, so the contract's residue 42 and
    // residue 1 pair can be written out.
    fn long_member(
        cds: &str,
        number: i64,
        expected: &str,
        replacement: &str,
        result: &str,
    ) -> Q5Variant {
        let request: Q5SdmRequest = Q5SdmRequest::create(
            long_template(),
            None,
            named_residue_at(cds, number, expected, replacement),
        )
        .unwrap();
        Q5Variant::create(request, Identifier::create(result.to_string()).unwrap())
    }

    fn long_member_unbound(
        number: i64,
        expected: &str,
        replacement: &str,
        result: &str,
    ) -> Q5Variant {
        let request: Q5SdmRequest = Q5SdmRequest::create(
            long_template(),
            None,
            unbound_residue_at(number, expected, replacement),
        )
        .unwrap();
        Q5Variant::create(request, Identifier::create(result.to_string()).unwrap())
    }

    // A member whose codon policy is the one Milestone 2 refuses to choose.
    fn long_member_unspecified_policy(cds: &str, number: i64, result: &str) -> Q5Variant {
        let request: Q5SdmRequest = Q5SdmRequest::create(
            long_template(),
            None,
            named_residue_at_policy(cds, number, "A", "W", CodonSelectionPolicy::Unspecified),
        )
        .unwrap();
        Q5Variant::create(request, Identifier::create(result.to_string()).unwrap())
    }

    // A DNA member that writes the letter already at that coordinate, so
    // Milestone 2's own intent constructor refuses it before any set exists.
    fn dna_no_op_request() -> Result<Q5SdmRequest, DomainError> {
        let intervals: OrderedIntervals = vec![create(&long_template().reference(), 3, 4).unwrap()];
        let intent: Result<NucleotideMutationIntent, DomainError> =
            NucleotideMutationIntent::create(
                long_template().reference(),
                intervals[0].clone(),
                "G".to_string(),
                "G".to_string(),
            );
        match intent {
            Ok(accepted) => Q5SdmRequest::create(
                long_template(),
                None,
                Q5MutationIntent::Nucleotide(accepted),
            ),
            Err(error) => Err(error),
        }
    }

    #[test]
    fn two_separate_variants_over_the_long_fixture_each_edit_only_their_own_codon() {
        // The contract's independent-variant case over the long fixture: A42G
        // and M1W as separate variants, the product brief's flagship pair
        // adapted to this fixture's length (residue 87 does not exist in 129
        // nucleotides, so the second member is the residue-1 member the
        // fixture can carry). The pinned codon for G is `GGT`, so residue 42's
        // `GCT` becomes `GGT`, which differs from the pristine template only at
        // coordinate 124; the contract names `GTT` for that edit, but `GTT` is
        // valine, so a glycine codon is used instead. The pinned codon for W
        // is `TGG`, so residue 1's `ATG` becomes
        // `TGG`, which differs only at coordinates 0 and 1, because the third
        // letter of both codons is `G`. Each member starts from the pristine
        // template, so each edited artifact differs from the other at exactly
        // the union of the two mutations.
        let members: Vec<Q5Variant> = vec![
            long_member("cds-1", 42, "A", "G", "res-a42g"),
            long_member("cds-1", 1, "M", "W", "res-m1w"),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        let run: Q5VariantSetExecution =
            execute_set(set.clone(), vec![long_cds_of("cds-1", 129)]).unwrap();

        assert!(run.is_verified());
        assert!(run.result_count() == 2);
        assert!(run.target().unwrap().id() == cds_name("cds-1"));

        let results: Vec<Q5VariantResult> = run.results();
        assert!(results[0].identifier() == a42g_result());
        assert!(results[1].identifier() == m1w_result());
        assert!(results[0].is_verified());
        assert!(results[1].is_verified());
        assert!(results[0].mutation().source() == long_template().reference());
        assert!(results[1].mutation().source() == long_template().reference());

        assert!(differing_coordinates(&long_template(), &results[0].edited()) == vec![124]);
        assert!(differing_coordinates(&long_template(), &results[1].edited()) == vec![0, 1]);
        assert!(
            differing_coordinates(&results[0].edited(), &results[1].edited()) == vec![0, 1, 124]
        );
        assert!(letters_over(&results[0].edited(), 123, 126) == "GGT");
        assert!(letters_over(&results[1].edited(), 0, 3) == "TGG");
        assert!(results[0].edited().length() == long_template().length());
        assert!(results[1].edited().length() == long_template().length());
    }

    // The long fixture with codon 2 replaced by `TAG`: still 129 nucleotides
    // and 43 codons, residue 1 still M and residue 42 still A, but a stop
    // that is not the last codon, so Milestone 1's translation refuses the
    // CDS it is applied to.
    fn stop_text() -> String {
        let mut text: String = "ATGTAG".to_string();
        let mut index: usize = 0;
        while index < 39 {
            text += "ATG";
            index += 1;
        }
        text + "GCT" + "TGG"
    }

    fn stop_template() -> Sequence {
        normalize(cds_name("q5-stop"), stop_text(), SequenceTopology::Linear).unwrap()
    }

    fn stop_cds_of(name: &str, length: i64) -> Cds {
        let intervals: OrderedIntervals =
            vec![create(&stop_template().reference(), 0, length).unwrap()];
        Cds::create(
            &stop_template(),
            cds_name(name),
            intervals,
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap()
    }

    // A member over the stop fixture: the same two residue requests the long
    // fixture carries, applied to an annotation Milestone 1 will refuse.
    fn stop_member(
        cds: &str,
        number: i64,
        expected: &str,
        replacement: &str,
        result: &str,
    ) -> Q5Variant {
        let request: Q5SdmRequest = Q5SdmRequest::create(
            stop_template(),
            None,
            named_residue_at(cds, number, expected, replacement),
        )
        .unwrap();
        Q5Variant::create(request, cds_name(result))
    }

    #[test]
    fn a_member_over_a_cds_milestone_one_refuses_ends_the_set_before_any_edit() {
        // The set path reuses Milestone 1's translation once per member, so a
        // template whose CDS has an internal stop is refused before any member
        // edits: codon 2 is `TAG`, a stop that is not the last codon, so the
        // CDS claims residues continue past it. The whole call returns that
        // original `InternalStopCodon`, naming the offending codon, and no
        // member's edit is reported as a partial success.
        let members: Vec<Q5Variant> = vec![
            stop_member("cds-1", 42, "A", "G", "res-a42g"),
            stop_member("cds-1", 1, "M", "G", "res-m1g"),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        let run: Result<Q5VariantSetExecution, DomainError> =
            execute_set(set.clone(), vec![stop_cds_of("cds-1", 129)]);
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::InternalStopCodon(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        if let DomainError::InternalStopCodon(problem) = error {
            assert!(problem.cds == "cds-1");
            assert!(problem.codon_number == 2);
            assert!(problem.codon == "TAG");
        }
    }

    #[test]
    fn a_later_member_with_a_wrong_expected_residue_fails_the_whole_set() {
        // D-C: the second member expects A at residue 43, which holds W, so
        // Milestone 2's own check refuses it and the whole call returns that
        // `ExpectedResidueMismatch`, naming the residue actually found. The
        // first member's verified edit is not returned as a partial success.
        let members: Vec<Q5Variant> = vec![
            long_member("cds-1", 42, "A", "G", "res-a42g"),
            long_member("cds-1", 43, "A", "G", "res-a43g"),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        let run: Result<Q5VariantSetExecution, DomainError> =
            execute_set(set.clone(), vec![long_cds_of("cds-1", 129)]);
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::ExpectedResidueMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        if let DomainError::ExpectedResidueMismatch(problem) = error {
            assert!(problem.cds == "cds-1");
            assert!(problem.number == 43);
            assert!(problem.expected == "A");
            assert!(problem.actual == "W");
        }
    }

    #[test]
    fn a_one_member_set_produces_the_same_result_as_the_single_request_path() {
        // D-A: one typed set request rather than a caller loop, but a member
        // must still mean the same thing. The same request run through the
        // old `execute` and through a one-member `SeparateVariants` set gives
        // the same coordinates, the same codon the policy selected, the same
        // edited text, and the same verified state.
        let single_request: Q5SdmRequest = Q5SdmRequest::create(
            long_template(),
            None,
            named_residue_at("cds-1", 42, "A", "G"),
        )
        .unwrap();
        let single: Q5WorkflowExecution = execute(
            single_request,
            vec![long_cds_of("cds-1", 129)],
            a42g_result(),
        )
        .unwrap();
        let set: Q5VariantSetRequest = Q5VariantSetRequest::create(
            Q5Composition::SeparateVariants,
            vec![long_member("cds-1", 42, "A", "G", "res-a42g")],
        )
        .unwrap();
        let run: Q5VariantSetExecution =
            execute_set(set.clone(), vec![long_cds_of("cds-1", 129)]).unwrap();

        let done: MutationResult = single.mutation().unwrap();
        let member: MutationResult = run.results()[0].mutation();
        assert!(member == done);
        assert!(member.target_coordinates() == done.target_coordinates());
        assert!(member.state() == done.state());
        assert!(member.edited().canonical_text() == done.edited().canonical_text());
        assert!(member.edited().canonical_text() == done.edited().canonical_text());
        // `MutationResult`'s public accessors stop at the artifact, so the codon
        // the policy selected is compared through the enum's own equality above,
        // which covers every field of the residue record including `selected
        // codon` and `original_codon`, rather than by a second accessor.
        assert!(run.results()[0].identifier() == a42g_result());
        assert!(run.target().unwrap().id() == single.scope().target().unwrap().id());
    }

    #[test]
    fn two_variants_sharing_a_result_name_are_refused() {
        // Owner decision, 2026-10-14 (`docs/q5-sdm.md` §10 item 6): result
        // identifiers in a set must be distinct. Refused at construction, before
        // any member resolves a target, with the same variant and category as an
        // empty set.
        let members: Vec<Q5Variant> = vec![
            long_member("cds-1", 42, "A", "G", "res-a42g"),
            long_member("cds-1", 43, "W", "G", "res-a42g"),
        ];
        let error: DomainError =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members).unwrap_err();
        assert!(error.category() == ErrorCategory::MalformedInput);
        match error {
            DomainError::InvalidMutationRequest(problem) => {
                assert!(problem.request == "q5-variant-set");
                assert!(problem.reason.contains("res-a42g"), "{}", problem.reason);
            }
            other => panic!("expected InvalidMutationRequest, got {:?}", other),
        }
    }

    #[test]
    fn result_names_are_compared_exactly() {
        // No case folding: D007 compares names exactly, and no source states a
        // folding rule. Two names differing only in case are distinct.
        let members: Vec<Q5Variant> = vec![
            long_member("cds-1", 42, "A", "G", "res-a42g"),
            long_member("cds-1", 43, "W", "G", "RES-A42G"),
        ];
        assert!(Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members).is_ok());
    }

    #[test]
    fn a_member_with_residue_zero_is_refused_by_milestone_one_at_bind_time() {
        // The set layer adds no residue-number rule: an unbound member carries
        // the number through to the annotation the set resolved, and Milestone
        // 1's `ResiduePosition::create` refuses 0 there, with its own variant
        // and payload naming that annotation.
        let members: Vec<Q5Variant> = vec![
            long_member("cds-1", 42, "A", "G", "res-a42g"),
            long_member_unbound(0, "A", "G", "res-zero"),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        let run: Result<Q5VariantSetExecution, DomainError> =
            execute_set(set.clone(), vec![long_cds_of("cds-1", 129)]);
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::InvalidResiduePosition(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        if let DomainError::InvalidResiduePosition(problem) = error {
            assert!(problem.cds == "cds-1");
            assert!(problem.number == 0);
        }
    }

    #[test]
    fn a_member_with_a_negative_residue_is_refused_by_milestone_one_at_bind_time() {
        // Same rule as residue 0, reached the same way: the number is refused
        // where a CDS identifier exists, by Milestone 1's constructor, and the
        // set returns that original error unchanged.
        let members: Vec<Q5Variant> = vec![
            long_member("cds-1", 42, "A", "G", "res-a42g"),
            long_member_unbound(-1, "A", "G", "res-negative"),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        let run: Result<Q5VariantSetExecution, DomainError> =
            execute_set(set.clone(), vec![long_cds_of("cds-1", 129)]);
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::InvalidResiduePosition(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        if let DomainError::InvalidResiduePosition(problem) = error {
            assert!(problem.cds == "cds-1");
            assert!(problem.number == -1);
        }
    }

    #[test]
    fn a_member_that_asks_for_an_unspecified_codon_policy_is_refused_by_milestone_two() {
        // Resolving a target for the set does not let the workflow choose a
        // codon policy on a member's behalf: the policy reaches Milestone 2's
        // own check and is refused with its own variant.
        let members: Vec<Q5Variant> = vec![
            long_member("cds-1", 42, "A", "G", "res-a42g"),
            long_member_unspecified_policy("cds-1", 42, "res-unspecified"),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        let run: Result<Q5VariantSetExecution, DomainError> =
            execute_set(set.clone(), vec![long_cds_of("cds-1", 129)]);
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::UnspecifiedCodonPolicy(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    #[test]
    fn a_member_whose_dna_edit_writes_no_change_is_refused_before_the_set_exists() {
        // A DNA-level no-op is refused where the member's request is built,
        // by Milestone 2's own intent constructor, so a `Q5Variant` never holds
        // it and the set inherits that refusal rather than adding its own.
        let built: Result<Q5SdmRequest, DomainError> = dna_no_op_request();
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        if let DomainError::InvalidMutationRequest(problem) = error {
            assert!(problem.request == "nucleotide");
        }
    }

    #[test]
    fn an_in_frame_deletion_carries_no_warning() {
        // Deleting `[9,12)` removes AAT, three coding bases of a whole 30-base
        // CDS, so the reading frame is kept and the run warns nothing. The
        // edited text is the template with those three bases cut out: plain
        // string slicing, planner 2026-10-15.
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, deletion_intent(9, 12, "AAT".to_string()))
                .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.mutation().unwrap().is_deletion());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGCAGCAGCAGCAGCAGCAG");
        assert!(run.warnings().is_empty());
    }

    #[test]
    fn a_one_base_deletion_warns_of_a_frameshift() {
        // One coding base removed from a whole 30-base CDS: 1 is not a
        // multiple of 3, so the reading frame shifts and the deletion lay
        // wholly inside the CDS.
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, deletion_intent(9, 10, "A".to_string()))
                .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::Frameshift {
                    cds: cds_name("cds-1"),
                    coding_bases_removed: 1,
                    wholly_inside: true,
                }
        );
    }

    #[test]
    fn a_deletion_across_the_cds_end_counts_only_its_coding_bases() {
        // A 12-base CDS and a deletion of `[10,14)`: positions 10 and 11 are
        // coding, 12 and 13 lie past the annotation, so 2 coding bases shift
        // the frame and the warning says the deletion extends outside the CDS.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            deletion_intent(10, 14, "ATCA".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 12)], edited_name()).unwrap();
        assert!(run.is_verified());
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::Frameshift {
                    cds: cds_name("cds-1"),
                    coding_bases_removed: 2,
                    wholly_inside: false,
                }
        );
        assert!(warnings[0].describe().contains("outside the CDS"));
    }

    #[test]
    fn a_deletion_outside_the_cds_proceeds_without_warning() {
        // §10 decision 1: an unscoped DNA edit outside the resolved CDS
        // proceeds, and no coding base was removed, so there is no frame to
        // shift and nothing to warn about.
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, deletion_intent(12, 15, "CAG".to_string()))
                .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 12)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.warnings().is_empty());
    }

    #[test]
    fn a_scoped_deletion_outside_its_cds_is_refused() {
        // The same deletion, but the request scoped the CDS it does not lie
        // inside, so Milestone 2's containment rule refuses it.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            deletion_intent(12, 15, "CAG".to_string()),
        )
        .unwrap();
        let run: Result<Q5WorkflowExecution, DomainError> =
            execute(request.clone(), vec![cds_of("cds-1", 12)], edited_name());
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::TargetOutsideFeature(_)));
    }

    #[test]
    fn only_bases_inside_the_cds_intervals_count() {
        // A circular artifact, so a CDS can cross the origin: `[21,30)` then
        // `[0,6)` is one feature split across it. Of the deleted `[5,10)` only
        // position 5 is coding, so 1 coding base shifts the frame and 6, 7, 8
        // and 9 lie outside both intervals.
        let circular: Sequence = normalize(
            Identifier::create("q5-c".to_string()).unwrap(),
            "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Circular,
        )
        .unwrap();
        let intervals: OrderedIntervals = vec![
            create(&circular.reference(), 21, 30).unwrap(),
            create(&circular.reference(), 0, 6).unwrap(),
        ];
        let annotation: Cds = Cds::create(
            &circular,
            cds_name("cds-1"),
            intervals,
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap();
        let intent: Q5MutationIntent =
            deletion_intent_on(circular.clone(), 5, 10, "TTGGA".to_string());
        let request: Q5SdmRequest = Q5SdmRequest::create(circular, None, intent).unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![annotation], edited_name()).unwrap();
        assert!(run.is_verified());
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::Frameshift {
                    cds: cds_name("cds-1"),
                    coding_bases_removed: 1,
                    wholly_inside: false,
                }
        );
    }

    #[test]
    fn a_deletion_intent_on_another_artifact_is_refused() {
        // The deletion intent addresses the decoy, so the request is refused
        // where it is built, by Milestone 1's identity rule and its own
        // category — the pattern the nucleotide form already pins.
        let built: Result<Q5SdmRequest, DomainError> = Q5SdmRequest::create(
            template(),
            None,
            deletion_intent_on(decoy(), 9, 12, "CAT".to_string()),
        );
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn each_set_member_carries_its_own_warnings() {
        // Two members over one whole 30-base CDS: the first deletes one
        // coding base and warns, the second deletes three and does not, so
        // the set reports each member's warnings rather than one for all.
        let members: Vec<Q5Variant> = vec![
            Q5Variant::create(
                Q5SdmRequest::create(template(), None, deletion_intent(9, 10, "A".to_string()))
                    .unwrap(),
                cds_name("m1"),
            ),
            Q5Variant::create(
                Q5SdmRequest::create(template(), None, deletion_intent(9, 12, "AAT".to_string()))
                    .unwrap(),
                cds_name("m2"),
            ),
        ];
        let set: Q5VariantSetRequest =
            Q5VariantSetRequest::create(Q5Composition::SeparateVariants, members.clone()).unwrap();
        let run: Q5VariantSetExecution =
            execute_set(set.clone(), vec![cds_of("cds-1", 30)]).unwrap();
        assert!(run.is_verified());
        assert!(run.result_count() == 2);
        assert!(run.results()[0].identifier() == cds_name("m1"));
        assert!(
            run.results()[0].warnings()
                == vec![Q5EditWarning::Frameshift {
                    cds: cds_name("cds-1"),
                    coding_bases_removed: 1,
                    wholly_inside: true,
                },]
        );
        assert!(run.results()[1].warnings().is_empty());
    }

    // A DNA-level insertion on `artifact`: the gap at `position` and the exact
    // bases on each side of it, as Milestone 4d-1's intent asserts them — the
    // pattern `deletion_intent_on` establishes. The flanks come from plain
    // string slicing of the template text (planner, 2026-10-15): at 9 they are
    // `G`/`A` and at 12 they are `T`/`C`.
    fn insertion_intent_on(
        artifact: Sequence,
        position: u64,
        before: String,
        after: String,
        inserted: String,
    ) -> Q5MutationIntent {
        Q5MutationIntent::Insertion(
            NucleotideInsertionIntent::create(
                artifact.reference(),
                create_point(&artifact.reference(), position as i64).unwrap(),
                before.clone(),
                after.clone(),
                inserted.clone(),
            )
            .unwrap(),
        )
    }

    fn insertion_intent(
        position: u64,
        before: String,
        after: String,
        inserted: String,
    ) -> Q5MutationIntent {
        insertion_intent_on(template(), position, before, after, inserted)
    }

    // A DNA-level replacement on `artifact`: the region it overwrites and the
    // exact text there, as Milestone 4e-2's intent asserts it — the pattern
    // `deletion_intent_on` establishes. The expected texts come from plain
    // string slicing of the template (planner, 2026-10-15): `[9, 12)` is `AAT`,
    // `[10, 14)` is `ATCA` and `[15, 18)` is `CAG`; on the decoy `[9, 12)` is
    // `CAT`.
    fn replacement_intent_on(
        artifact: Sequence,
        start: u64,
        end: u64,
        expected: String,
        replacement: String,
    ) -> Q5MutationIntent {
        let intervals: OrderedIntervals =
            vec![create(&artifact.reference(), start as i64, end as i64).unwrap()];
        Q5MutationIntent::Replacement(
            NucleotideReplacementIntent::create(
                artifact.reference(),
                intervals[0].clone(),
                expected,
                replacement,
            )
            .unwrap(),
        )
    }

    fn replacement_intent(
        start: u64,
        end: u64,
        expected: String,
        replacement: String,
    ) -> Q5MutationIntent {
        replacement_intent_on(template(), start, end, expected, replacement)
    }

    #[test]
    fn an_in_frame_insertion_has_no_warning() {
        // Three bases inserted at 9 into a whole 30-base CDS: 3 is a multiple of
        // 3, so the reading frame is kept and the run warns nothing. The edited
        // text is the template with `GGG` placed into the gap at 9.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            insertion_intent(9, "G".to_string(), "A".to_string(), "GGG".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.mutation().unwrap().is_insertion());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGGGGAATCAGCAGCAGCAGCAGCAG");
        assert!(run.warnings().is_empty());
    }

    #[test]
    fn a_one_base_insertion_warns_of_a_frameshift() {
        // One base inserted inside a whole 30-base CDS: 1 is not a multiple of
        // 3, so the reading frame shifts and the run warns while still editing.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            insertion_intent(9, "G".to_string(), "A".to_string(), "G".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGGAATCAGCAGCAGCAGCAGCAG");
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::InsertionFrameshift {
                    cds: cds_name("cds-1"),
                    bases_inserted: 1,
                }
        );
    }

    #[test]
    fn a_four_base_insertion_warns_too() {
        // Four bases inserted inside the same CDS: 4 is not a multiple of 3
        // either, and the warning counts the inserted bases.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            insertion_intent(9, "G".to_string(), "A".to_string(), "GGGG".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGGGGGAATCAGCAGCAGCAGCAGCAG");
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::InsertionFrameshift {
                    cds: cds_name("cds-1"),
                    bases_inserted: 4,
                }
        );
    }

    #[test]
    fn an_insertion_at_the_cds_edge_does_not_warn() {
        // The gap at 12 is the far edge of a 12-base CDS: no interval of it has
        // `start < 12 < end`, so the reading frame is left alone and nothing is
        // warned. The request scoped nothing, so the edit proceeds.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            insertion_intent(12, "T".to_string(), "C".to_string(), "G".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 12)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.warnings().is_empty());
    }

    #[test]
    fn a_scoped_insertion_at_the_cds_edge_is_refused() {
        // The same edge gap, but the request scoped the CDS it does not lie
        // inside, so Milestone 4d-1's containment rule refuses it.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            insertion_intent(12, "T".to_string(), "C".to_string(), "G".to_string()),
        )
        .unwrap();
        let run: Result<Q5WorkflowExecution, DomainError> =
            execute(request.clone(), vec![cds_of("cds-1", 12)], edited_name());
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::TargetOutsideFeature(_)));
    }

    #[test]
    fn an_insertion_intent_on_another_artifact_is_refused() {
        // The insertion intent addresses the decoy, so the request is refused
        // where it is built, by Milestone 1's identity rule and its own
        // category — the pattern the deletion form already pins. The flanks are
        // read off the decoy, so the intent is well formed on its own.
        let before: char = decoy().letter_at(8).unwrap();
        let after: char = decoy().letter_at(9).unwrap();
        let built: Result<Q5SdmRequest, DomainError> = Q5SdmRequest::create(
            template(),
            None,
            insertion_intent_on(
                decoy(),
                9,
                before.to_string(),
                after.to_string(),
                "G".to_string(),
            ),
        );
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn an_equal_length_replacement_carries_no_warning() {
        // Three bases overwritten by three inside a whole 30-base CDS: `m − w` is
        // 0, a multiple of 3, so the reading frame is kept and the run warns
        // nothing. The edited text is the template with `GGC` in place of the
        // `AAT` at `[9, 12)`: plain string slicing, planner 2026-10-15.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            replacement_intent(9, 12, "AAT".to_string(), "GGC".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.mutation().unwrap().is_replacement());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGGGCCAGCAGCAGCAGCAGCAG");
        assert!(run.warnings().is_empty());
    }

    #[test]
    fn a_shorter_replacement_warns_of_a_frameshift() {
        // Three coding bases overwritten by two inside a whole 30-base CDS: `m − w`
        // is −1, not a multiple of 3, so the frame shifts and the run warns while
        // still editing. Every overwritten base was coding, so the warning says
        // nothing about reaching outside the CDS.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            replacement_intent(9, 12, "AAT".to_string(), "GG".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGGGCAGCAGCAGCAGCAGCAG");
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::ReplacementFrameshift {
                    cds: cds_name("cds-1"),
                    bases_replaced: 3,
                    bases_written: 2,
                    wholly_inside: true,
                }
        );
        assert!(
            warnings[0].describe() == "replacing 3 bases with 2 in cds-1 shifts its reading frame"
        );
    }

    #[test]
    fn a_replacement_three_bases_longer_keeps_the_frame() {
        // Three coding bases overwritten by six: `m − w` is 3, a multiple of 3, so
        // the reading frame is kept and the run warns nothing.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            replacement_intent(9, 12, "AAT".to_string(), "GGGGGG".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGGGGGGGCAGCAGCAGCAGCAGCAG");
        assert!(run.warnings().is_empty());
    }

    #[test]
    fn a_longer_replacement_warns_too() {
        // Three coding bases overwritten by four: `m − w` is 1, so the frame shifts
        // the other way and the run warns again.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            replacement_intent(9, 12, "AAT".to_string(), "GGGG".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGGGGGCAGCAGCAGCAGCAGCAG");
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::ReplacementFrameshift {
                    cds: cds_name("cds-1"),
                    bases_replaced: 3,
                    bases_written: 4,
                    wholly_inside: true,
                }
        );
    }

    #[test]
    fn a_replacement_across_the_cds_edge_warns_and_says_so() {
        // A 12-base CDS and a replacement of `[10, 14)`: positions 10 and 11 are
        // coding, 12 and 13 lie past the annotation, so the region reaches
        // outside and the warning says so. `m − w` is −2, not a multiple of 3, and
        // the warning counts the whole 4-base region rather than the 2 coding
        // bases, the pattern the deletion form already pins.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            replacement_intent(10, 14, "ATCA".to_string(), "GG".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 12)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGAGGGCAGCAGCAGCAGCAG");
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::ReplacementFrameshift {
                    cds: cds_name("cds-1"),
                    bases_replaced: 4,
                    bases_written: 2,
                    wholly_inside: false,
                }
        );
        assert!(warnings[0]
            .describe()
            .ends_with("; the replacement also extends outside the CDS"));
    }

    #[test]
    fn a_scoped_replacement_across_the_cds_edge_is_refused() {
        // The same region, but the request scoped the CDS it does not lie wholly
        // inside, so Milestone 4e-2's containment rule refuses it.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            replacement_intent(10, 14, "ATCA".to_string(), "GG".to_string()),
        )
        .unwrap();
        let run: Result<Q5WorkflowExecution, DomainError> =
            execute(request.clone(), vec![cds_of("cds-1", 12)], edited_name());
        assert!(run.is_err());
        let error: DomainError = run.unwrap_err();
        assert!(matches!(error, DomainError::TargetOutsideFeature(_)));
    }

    #[test]
    fn a_replacement_outside_the_cds_does_not_warn() {
        // §10 decision 1 again: an unscoped DNA edit outside the resolved CDS
        // proceeds, and no region position is coding, so there is no frame to
        // shift and nothing to warn about even though `m − w` is −2.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            replacement_intent(15, 18, "CAG".to_string(), "G".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 12)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGAATCAGGCAGCAGCAGCAG");
        assert!(run.warnings().is_empty());
    }

    #[test]
    fn a_replacement_intent_on_another_artifact_is_refused() {
        // The replacement intent addresses the decoy, so the request is refused
        // where it is built, by Milestone 1's identity rule and its own category —
        // the pattern the insertion form already pins. `[9, 12)` is `CAT` on the
        // decoy, so the intent is well formed on its own.
        let built: Result<Q5SdmRequest, DomainError> = Q5SdmRequest::create(
            template(),
            None,
            replacement_intent_on(decoy(), 9, 12, "CAT".to_string(), "GG".to_string()),
        );
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    // The whole 30-base annotation on the minus strand: mut-a read backwards
    // translates to L L L L L L I P S H (`tests/fixtures/mutation/README.md`), so
    // it is a valid complete annotation. The frame rules below count positions
    // inside the CDS's intervals and never read their strand, so these tests pin
    // that rather than assume it.
    fn minus_cds_of(name: &str, length: i64) -> Cds {
        Cds::create(
            &template(),
            cds_name(name),
            whole(length),
            Strand::Minus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap()
    }

    // mut-c of the Milestone 2 fixture: circular, 24 bases, with the fixture's
    // `res-c-origin` annotation `cds-1` over [22, 24) then [0, 7), plus strand,
    // complete, reading TGG CTT GGA (W L G) across the origin.
    fn seq_c() -> Sequence {
        normalize(
            Identifier::create("seq-c".to_string()).unwrap(),
            "GCTTGGAACGAACGAACGAACATG".to_string(),
            SequenceTopology::Circular,
        )
        .unwrap()
    }

    fn cds_c() -> Cds {
        let intervals: OrderedIntervals = vec![
            create(&seq_c().reference(), 22, 24).unwrap(),
            create(&seq_c().reference(), 0, 7).unwrap(),
        ];
        Cds::create(
            &seq_c(),
            cds_name("cds-1"),
            intervals,
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap()
    }

    #[test]
    fn a_frameshift_deletion_in_a_minus_strand_cds_warns() {
        // The one-base deletion of `[9, 10)` (`A`), scoped to a reverse CDS over
        // [0, 30): one coding position is removed, 1 is not a multiple of 3, and
        // the region lies wholly inside the single interval, so the warning is the
        // one the plus-strand case already pins.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            deletion_intent(9, 10, "A".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution = execute(
            request.clone(),
            vec![minus_cds_of("cds-1", 30)],
            edited_name(),
        )
        .unwrap();
        assert!(run.is_verified());
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::Frameshift {
                    cds: cds_name("cds-1"),
                    coding_bases_removed: 1,
                    wholly_inside: true,
                }
        );
    }

    #[test]
    fn an_in_frame_deletion_in_a_minus_strand_cds_does_not_warn() {
        // The same deletion as `an_in_frame_deletion_carries_no_warning`, scoped to
        // a reverse CDS: `[9, 12)` is `AAT`, three coding positions, so the reading
        // frame is kept and the run warns nothing.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            deletion_intent(9, 12, "AAT".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution = execute(
            request.clone(),
            vec![minus_cds_of("cds-1", 30)],
            edited_name(),
        )
        .unwrap();
        assert!(run.is_verified());
        assert!(run.warnings().is_empty());
    }

    #[test]
    fn a_frameshift_insertion_in_a_minus_strand_cds_warns() {
        // One base inserted at the gap at 12, whose flanks are `T` and `C`: the gap
        // sits strictly inside [0, 30), so it is inside the CDS whatever its strand,
        // and 1 is not a multiple of 3.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            insertion_intent(12, "T".to_string(), "C".to_string(), "A".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution = execute(
            request.clone(),
            vec![minus_cds_of("cds-1", 30)],
            edited_name(),
        )
        .unwrap();
        assert!(run.is_verified());
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::InsertionFrameshift {
                    cds: cds_name("cds-1"),
                    bases_inserted: 1,
                }
        );
    }

    #[test]
    fn a_frameshift_replacement_in_a_minus_strand_cds_warns() {
        // Two coding positions of `[9, 11)` (`AA`) overwritten by one base: `m − w`
        // is 1, so the frame shifts, and every overwritten position was coding.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            replacement_intent(9, 11, "AA".to_string(), "G".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution = execute(
            request.clone(),
            vec![minus_cds_of("cds-1", 30)],
            edited_name(),
        )
        .unwrap();
        assert!(run.is_verified());
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::ReplacementFrameshift {
                    cds: cds_name("cds-1"),
                    bases_replaced: 2,
                    bases_written: 1,
                    wholly_inside: true,
                }
        );
    }

    #[test]
    fn a_frameshift_deletion_in_an_origin_wrapping_cds_warns() {
        // mut-c with its `res-c-origin` annotation: the deletion of `[2, 3)` (`T`)
        // lies in the post-origin interval [0, 7), so one coding position is
        // removed and the warning is the one a single interval gives. The CDS's
        // other interval, [22, 24), is not consulted for this region.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            seq_c(),
            Some(cds_name("cds-1")),
            deletion_intent_on(seq_c(), 2, 3, "T".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_c()], edited_name()).unwrap();
        assert!(run.is_verified());
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::Frameshift {
                    cds: cds_name("cds-1"),
                    coding_bases_removed: 1,
                    wholly_inside: true,
                }
        );
    }

    // D037: an in-frame DNA edit wholly inside the resolved CDS re-translates
    // that CDS on the edited artifact, and a stop that is not that CDS's last
    // codon is a warning the run carries while still applying the edit. The
    // letters and codon numbers below were derived by hand over the template
    // text `ATGGCTTGGAATCAGCAGCAGCAGCAGCAG` (planner, 2026-10-16): index 5 is
    // `T`, 6–8 are `TGG`, 9–11 are `AAT`, and a whole 30-base plus-strand CDS
    // reads M A W N Q Q Q Q Q Q with no terminal stop.
    fn nucleotide_substitution_at(
        position: u64,
        expected: String,
        replacement: String,
    ) -> Q5MutationIntent {
        let interval: OrderedIntervals = vec![create(
            &template().reference(),
            position as i64,
            (position + 1) as i64,
        )
        .unwrap()];
        Q5MutationIntent::Nucleotide(
            NucleotideMutationIntent::create(
                template().reference(),
                interval[0].clone(),
                expected,
                replacement,
            )
            .unwrap(),
        )
    }

    #[test]
    fn a_substitution_that_writes_a_stop_warns() {
        // Position 7 is the middle base of codon 3's `TGG`: writing `A` there
        // gives `TAG`. The edit changes no length, so the frame is kept, and
        // `[7, 8)` lies wholly inside [0, 30), so the CDS is re-translated:
        // ATG GCT TAG AAT CAG CAG CAG CAG CAG CAG puts a stop at codon 3 of
        // 10, before the last codon.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            nucleotide_substitution_at(7, "G".to_string(), "A".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTAGAATCAGCAGCAGCAGCAGCAG");
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::PrematureStop {
                    cds: cds_name("cds-1"),
                    codon_number: 3,
                }
        );
    }

    #[test]
    fn a_synonymous_substitution_does_not_warn() {
        // Position 5 is the last base of codon 2's `GCT`: writing `C` gives
        // `GCC`, still Thr. The edit is in-frame and wholly inside [0, 30), so
        // the CDS is re-translated, and ATG GCT GCC TGG AAT CAG CAG CAG CAG CAG
        // holds no stop at all: no warning.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            nucleotide_substitution_at(5, "T".to_string(), "C".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCCTGGAATCAGCAGCAGCAGCAGCAG");
        assert!(run.warnings().is_empty());
    }

    #[test]
    fn an_in_frame_insertion_of_a_stop_warns() {
        // `TAA` inserted into the gap at 9, whose flanks are `G` (index 8) and
        // `A` (index 9): 3 is a multiple of 3, so the frame is kept, and the
        // gap lies inside [0, 30). The edited CDS reads ATG GCT TGG TAA AAT
        // CAG CAG CAG CAG CAG CAG, so the stop sits at codon 4 of 11.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            insertion_intent(9, "G".to_string(), "A".to_string(), "TAA".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGTAAAATCAGCAGCAGCAGCAGCAG");
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::PrematureStop {
                    cds: cds_name("cds-1"),
                    codon_number: 4,
                }
        );
    }

    #[test]
    fn an_in_frame_insertion_without_a_stop_does_not_warn() {
        // `GCT` inserted into the same gap: 3 is a multiple of 3, so the frame
        // is kept and the CDS is re-translated, and ATG GCT TGG GCT AAT CAG
        // CAG CAG CAG CAG CAG holds no stop: no warning.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            insertion_intent(9, "G".to_string(), "A".to_string(), "GCT".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGGCTAATCAGCAGCAGCAGCAGCAG");
        assert!(run.warnings().is_empty());
    }

    #[test]
    fn an_equal_length_replacement_that_writes_a_stop_warns() {
        // `[9, 12)` is `AAT`, overwritten by `TGA`: `m − w` is 0, so the frame
        // is kept, and the region lies wholly inside [0, 30). The edited CDS
        // reads ATG GCT TGG TGA CAG CAG CAG CAG CAG CAG, so the stop is codon
        // 4 of 10.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            replacement_intent(9, 12, "AAT".to_string(), "TGA".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGTGACAGCAGCAGCAGCAGCAG");
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::PrematureStop {
                    cds: cds_name("cds-1"),
                    codon_number: 4,
                }
        );
    }

    #[test]
    fn an_in_frame_unequal_replacement_that_writes_a_stop_warns() {
        // `AAT` overwritten by six bases: `m − w` is 3, a multiple of 3, so the
        // frame is kept and the CDS is re-translated. The edited text is
        // ATGGCTTGGAATTAACAGCAGCAGCAGCAGCAG, which reads ATG GCT TGG AAT TAA
        // CAG CAG CAG CAG CAG CAG: the stop is codon 5 of 11.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            replacement_intent(9, 12, "AAT".to_string(), "AATTAA".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGAATTAACAGCAGCAGCAGCAGCAG");
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::PrematureStop {
                    cds: cds_name("cds-1"),
                    codon_number: 5,
                }
        );
    }

    #[test]
    fn a_frameshift_does_not_also_warn_of_a_stop() {
        // The one-base deletion of `[9, 10)` that `a_one_base_deletion_warns_of_a_frameshift`
        // already pins: `delta` is −1, not a multiple of 3, so the reading frame
        // shifts and D037 re-translates nothing. The run carries the frameshift
        // warning and nothing else.
        let request: Q5SdmRequest =
            Q5SdmRequest::create(template(), None, deletion_intent(9, 10, "A".to_string()))
                .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 30)], edited_name()).unwrap();
        assert!(run.is_verified());
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::Frameshift {
                    cds: cds_name("cds-1"),
                    coding_bases_removed: 1,
                    wholly_inside: true,
                }
        );
    }

    #[test]
    fn a_stop_on_a_minus_strand_cds_warns() {
        // The whole 30-base CDS on the minus strand reads L L L L L L I P S H,
        // and codon 7 is the reverse complement of reference [9, 12), which is
        // `AAT` → `ATT`. Overwriting that region with `TTA` changes no length
        // and lies wholly inside [0, 30), so the CDS is re-translated: codon 7
        // becomes revcomp(`TTA`) = `TAA`, a stop at codon 7 of 10.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            replacement_intent(9, 12, "AAT".to_string(), "TTA".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution = execute(
            request.clone(),
            vec![minus_cds_of("cds-1", 30)],
            edited_name(),
        )
        .unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGTTACAGCAGCAGCAGCAGCAG");
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::PrematureStop {
                    cds: cds_name("cds-1"),
                    codon_number: 7,
                }
        );
    }

    #[test]
    fn a_stop_in_an_origin_wrapping_cds_warns() {
        // mut-c's `cds-1` at [22, 24)+[0, 7) reads TGG CTT GGA across the origin.
        // `TAA` inserted into the gap at 1 — its flanks are `G` at index 0 and
        // `C` at index 1 — is 3 bases, so the frame is kept, and the gap lies
        // inside [0, 7). The edited artifact is 27 bases: the interval holding
        // the gap grows to [0, 10) and the interval at or past the edit's end
        // moves to [25, 27), so the CDS reads TGG TAA CTT GGA and the stop is
        // codon 2 of 4.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            seq_c(),
            Some(cds_name("cds-1")),
            insertion_intent_on(
                seq_c(),
                1,
                "G".to_string(),
                "C".to_string(),
                "TAA".to_string(),
            ),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_c()], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "GTAACTTGGAACGAACGAACGAACATG");
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::PrematureStop {
                    cds: cds_name("cds-1"),
                    codon_number: 2,
                }
        );
    }

    #[test]
    fn an_edit_only_partly_inside_the_cds_does_not_warn_of_a_stop() {
        // A plus-strand CDS over [0, 18) reads ATG GCT TGG AAT CAG CAG. The
        // unscoped deletion of [15, 21) removes `CAGCAG`: three of those bases
        // are coding, so the frame is kept and there is no frameshift, but the
        // region is not wholly inside one interval, so D037's rule does not
        // apply and the run warns nothing.
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            None,
            deletion_intent(15, 21, "CAGCAG".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 18)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "ATGGCTTGGAATCAGCAGCAGCAG");
        assert!(run.warnings().is_empty());
    }

    // D037's rule must not turn a valid edit into a failure: an in-frame edit
    // that removes a whole CDS leaves no CDS to re-translate, and the run still
    // applies the edit. Values derived by hand (planner, 2026-10-16): mut-a's
    // [0, 18) is `ATGGCTTGGAATCAGCAG` (ATG GCT TGG AAT CAG CAG, M A W N Q Q), so
    // deleting it leaves the last four codons, `CAGCAGCAGCAG`; 18 removed bases
    // are all coding and 18 is a multiple of 3, so there is no frameshift, and
    // the rebuilt CDS has no interval left at all.
    #[test]
    fn deleting_a_whole_cds_does_not_warn_or_fail() {
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template(),
            Some(cds_name("cds-1")),
            deletion_intent(0, 18, "ATGGCTTGGAATCAGCAG".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_of("cds-1", 18)], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "CAGCAGCAGCAG");
        assert!(run.warnings().is_empty());
    }

    // The same rule against a wrapping CDS whose first interval is deleted whole
    // and whose second is untouched: mut-c is `GCTTGGAACGAACGAACGAACATG` (24
    // bases, circular), and [21, 24) is `ATG` while [0, 6) is `GCTTGG`, so the
    // annotation [21, 24) + [0, 6) reads ATG GCT TGG (M A W). Deleting [21, 24)
    // is 3 bases, all coding and a multiple of 3, so no frameshift; the holding
    // interval becomes [21, 21) and is dropped, the interval at [0, 6) is
    // unchanged, and the 21-base edit is `GCTTGGAACGAACGAACGAAC`, whose [0, 6)
    // reads GCT TGG (A W): no stop, so no warning and no failure.
    fn cds_c_at_21() -> Cds {
        let intervals: OrderedIntervals = vec![
            create(&seq_c().reference(), 21, 24).unwrap(),
            create(&seq_c().reference(), 0, 6).unwrap(),
        ];
        Cds::create(
            &seq_c(),
            cds_name("cds-1"),
            intervals,
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap()
    }

    #[test]
    fn deleting_a_whole_interval_of_a_wrapping_cds_does_not_fail() {
        let request: Q5SdmRequest = Q5SdmRequest::create(
            seq_c(),
            Some(cds_name("cds-1")),
            deletion_intent_on(seq_c(), 21, 24, "ATG".to_string()),
        )
        .unwrap();
        let run: Q5WorkflowExecution =
            execute(request.clone(), vec![cds_c_at_21()], edited_name()).unwrap();
        assert!(run.is_verified());
        assert!(run.edited().unwrap().canonical_text() == "GCTTGGAACGAACGAACGAAC");
        assert!(run.warnings().is_empty());
    }
}
