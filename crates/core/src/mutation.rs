//! Mutation: deterministic edits to a sequence artifact.
//!
//! A mutation is a typed intent that is checked against the artifact before
//! anything is written, and an edit that produces a *new* artifact rather
//! than changing the old one. Nothing here is applied in place: the source
//! artifact keeps its content and its digest, and the result names both.
//!
//! Two intents exist, and they are different types because they assert
//! different things:
//!
//! * `NucleotideMutationIntent` asserts a base at a reference coordinate.
//! * `ResidueMutationIntent` asserts a residue of a CDS and leaves the DNA
//!   edit to a `CodonSelectionPolicy`.
//!
//! Both carry the expected original state, and both are checked before the
//! edit. A mutation never applies to a different artifact, to a coordinate
//! outside the sequence or outside its scope, or to a base or residue that
//! disagrees with the request.
//!
//! Strand is never a parameter of a mutation. It is read from the `Cds`, as
//! extraction and translation already require, and residue numbering runs
//! over the strand-applied CDS text.
//!
//! Verification is a separate responsibility (D013). `verify` re-derives the
//! edit from the source artifact and the result's own claims; it does not
//! re-run the edit. A result is `Calculated` until that pass accepts it, and
//! a `Calculated` result is never equal to a `Verified` one.

use crate::cds::Cds;
use crate::cds::ResiduePosition;
use crate::codon_usage::ECOLI_K12_CODON_COUNTS;
use crate::codon_usage::ECOLI_K12_CODON_USAGE_SOURCE;
use crate::error::DomainError;
use crate::error::ExpectedNucleotideMismatchProblem;
use crate::error::ExpectedResidueMismatchProblem;
use crate::error::InvalidMutationRequestProblem;
use crate::error::MutationVerificationFailureProblem;
use crate::error::ResiduePositionRangeProblem;
use crate::error::TargetOutsideFeatureProblem;
use crate::error::UnspecifiedCodonPolicyProblem;
use crate::error::UnsupportedMutationPolicyProblem;
use crate::extraction::extract_cds;
use crate::genetic_code::lookup;
use crate::genetic_code::residue_for_symbol;
use crate::genetic_code::CodonOutcome;
use crate::genetic_code::Residue;
use crate::identity::Identifier;
use crate::identity::SequenceReference;
use crate::interval::create;
use crate::interval::require_sequence;
use crate::interval::NucleotideInterval;
use crate::interval::OrderedIntervals;
use crate::sequence::canonical_text;
use crate::sequence::complement_letter;
use crate::sequence::is_dna_base;
use crate::sequence::normalize;
use crate::sequence::Sequence;
use crate::strand::Strand;
use crate::translation::translate;
use crate::translation::Translation;

/// Identity of the code path that produced a mutation result (D021).
pub const MUTATION_ALGORITHM_NAME: &str = "luria-core-mutation";

/// Version of the code path that produced a mutation result (D021).
pub const MUTATION_ALGORITHM_VERSION: &str = "0.2.0";

/// A codon of the standard genetic code.
///
/// Constructible only when `genetic_code::lookup` returns the `Residue`
/// branch, so an ambiguous codon, an out-of-alphabet codon, and a stop codon
/// cannot exist as a `Codon` and cannot be handed to anything.
#[derive(Clone, Debug, PartialEq)]
pub struct Codon {
    text: String,
    residue: Residue,
}

impl Codon {
    pub fn create(text: String) -> Result<Codon, DomainError> {
        let outcome: CodonOutcome = lookup(text.clone());
        match outcome {
            CodonOutcome::Residue(value) => Ok(Codon {
                text: text.clone(),
                residue: value,
            }),
            CodonOutcome::Stop => {
                let problem = InvalidMutationRequestProblem {
                    request: text,
                    reason: "the codon is a stop codon, and a stop is not a residue".to_string(),
                };
                Err(DomainError::InvalidMutationRequest(problem))
            }
            CodonOutcome::Ambiguous => {
                let problem = InvalidMutationRequestProblem {
                    request: text,
                    reason: "the codon stands for more than one residue".to_string(),
                };
                Err(DomainError::InvalidMutationRequest(problem))
            }
            CodonOutcome::Invalid => {
                let problem = InvalidMutationRequestProblem {
                    request: text,
                    reason: "the codon is not three DNA letters".to_string(),
                };
                Err(DomainError::InvalidMutationRequest(problem))
            }
        }
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn residue(&self) -> Residue {
        self.residue.clone()
    }
}

/// How a residue mutation chooses the codon that writes the replacement
/// residue (D011). No branch is chosen silently: a request that omits the
/// policy is `Unspecified` and fails.
///
/// * `Default` must be named explicitly. It selects the codon of the
///   replacement residue with the highest count in the *E. coli* K-12 codon
///   usage table in `codon_usage` (Kazusa, W3110). Equal counts are broken by
///   standard table order; the current table has no such tie.
/// * `Supplied` writes exactly the codon the caller gave.
#[derive(Clone, Debug, PartialEq)]
pub enum CodonSelectionPolicy {
    Unspecified,
    Default,
    Supplied(Codon),
}

impl CodonSelectionPolicy {
    pub fn name(&self) -> String {
        match self {
            CodonSelectionPolicy::Unspecified => "unspecified".to_string(),
            CodonSelectionPolicy::Default => "default".to_string(),
            CodonSelectionPolicy::Supplied(_codon) => "supplied".to_string(),
        }
    }

    /// The data source a policy's choice depends on, for provenance (D021).
    /// Only `Default` consults a table.
    pub fn source(&self) -> Option<String> {
        match self {
            CodonSelectionPolicy::Default => Some(ECOLI_K12_CODON_USAGE_SOURCE.to_string()),
            CodonSelectionPolicy::Unspecified => None,
            CodonSelectionPolicy::Supplied(_codon) => None,
        }
    }
}

/// Boundary parse of a codon policy. Matching is exact and case-sensitive;
/// near matches are refused, not aliased.
pub fn parse_codon_policy(text: String) -> Result<CodonSelectionPolicy, DomainError> {
    if text == "unspecified" {
        return Ok(CodonSelectionPolicy::Unspecified);
    }
    if text == "default" {
        return Ok(CodonSelectionPolicy::Default);
    }
    if text.starts_with("supplied:") {
        let parts: Vec<String> = text.clone().split(':').map(|x| x.to_string()).collect();
        if parts.len() != 2 {
            let problem = InvalidMutationRequestProblem {
                request: text.clone(),
                reason: "a supplied codon is written supplied:XXX, with one codon".to_string(),
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }
        let codon: Codon = Codon::create(parts[1].clone())?;
        return Ok(CodonSelectionPolicy::Supplied(codon.clone()));
    }
    let problem = UnsupportedMutationPolicyProblem {
        policy: text.clone(),
    };
    Err(DomainError::UnsupportedMutationPolicy(problem))
}

/// A residue letter, parsed into the existing `Residue` enum.
///
/// A stop symbol is not a residue, so a mutation to `*` is refused here
/// rather than turned into a DNA edit.
pub fn parse_residue(text: String) -> Result<Residue, DomainError> {
    let letters: Vec<char> = text.chars().collect();
    if letters.len() != 1 {
        let problem = InvalidMutationRequestProblem {
            request: text,
            reason: "a residue is exactly one letter".to_string(),
        };
        return Err(DomainError::InvalidMutationRequest(problem));
    }
    let found: Option<Residue> = residue_for_symbol(letters[0]);
    if found.is_none() {
        let problem = InvalidMutationRequestProblem {
            request: text,
            reason: "the letter is not a residue of the standard genetic code".to_string(),
        };
        return Err(DomainError::InvalidMutationRequest(problem));
    }
    Ok(found.unwrap())
}

/// A DNA-level substitution: one nucleotide at a reference coordinate.
///
/// The target is a `NucleotideInterval` of length 1, which is the Milestone 1
/// coordinate abstraction and which already names its reference sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct NucleotideMutationIntent {
    source: SequenceReference,
    position: NucleotideInterval,
    expected_original: char,
    replacement: char,
}

impl NucleotideMutationIntent {
    /// Build a nucleotide substitution, rejecting a request that cannot be
    /// represented: a letter other than `A`, `C`, `G`, `T` (an ambiguity code
    /// such as `N` is not a base that can be written or asserted), an interval that
    /// selects more than one nucleotide, or an interval that names a
    /// different artifact from the one the request claims.
    pub fn create(
        source: SequenceReference,
        position: NucleotideInterval,
        expected_original: String,
        replacement: String,
    ) -> Result<NucleotideMutationIntent, DomainError> {
        let expected: char =
            single_dna_base("expected original".to_string(), expected_original.clone())?;
        let desired: char = single_dna_base("replacement".to_string(), replacement.clone())?;

        if position.reference() != source {
            let problem = InvalidMutationRequestProblem {
                request: "nucleotide".to_string(),
                reason: "the target coordinate names a sequence other than the source".to_string(),
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }
        if position.length() != 1 {
            let problem = InvalidMutationRequestProblem {
                request: "nucleotide".to_string(),
                reason: "a nucleotide substitution selects exactly one nucleotide".to_string(),
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }

        if expected == desired {
            let problem = InvalidMutationRequestProblem {
                request: "nucleotide".to_string(),
                reason: "the replacement is the nucleotide that is already there".to_string(),
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }

        Ok(NucleotideMutationIntent {
            source,
            position,
            expected_original: expected,
            replacement: desired,
        })
    }

    pub fn source(&self) -> SequenceReference {
        self.source.clone()
    }

    pub fn position(&self) -> NucleotideInterval {
        self.position.clone()
    }

    pub fn expected_original(&self) -> char {
        self.expected_original
    }

    pub fn replacement(&self) -> char {
        self.replacement
    }
}

/// An amino-acid substitution as in D011: a CDS residue, an expected
/// original residue, a replacement residue, and an explicit codon policy.
#[derive(Clone, Debug, PartialEq)]
pub struct ResidueMutationIntent {
    position: ResiduePosition,
    expected_original: Residue,
    replacement: Residue,
    policy: CodonSelectionPolicy,
}

impl ResidueMutationIntent {
    pub fn create(
        position: ResiduePosition,
        expected_original: String,
        replacement: String,
        policy: CodonSelectionPolicy,
    ) -> Result<ResidueMutationIntent, DomainError> {
        let expected: Residue = parse_residue(expected_original.clone())?;
        let desired: Residue = parse_residue(replacement.clone())?;
        Ok(ResidueMutationIntent {
            position,
            expected_original: expected,
            replacement: desired,
            policy,
        })
    }

    pub fn position(&self) -> ResiduePosition {
        self.position.clone()
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
}

/// The lifecycle state of a mutation result: the mutation-domain subset of
/// the D020 product lifecycle. A calculated result is never presented as
/// verified, and the two states are unequal values.
#[derive(Clone, Debug, PartialEq)]
pub enum MutationState {
    Calculated,
    Verified,
}

impl MutationState {
    pub fn name(&self) -> String {
        match self {
            MutationState::Calculated => "calculated".to_string(),
            MutationState::Verified => "verified".to_string(),
        }
    }

    pub fn is_verified(&self) -> bool {
        match self {
            MutationState::Verified => true,
            MutationState::Calculated => false,
        }
    }
}

/// The record of a nucleotide substitution: what was changed, where, and in
/// which artifact it happened.
#[derive(Clone, Debug, PartialEq)]
pub struct NucleotideMutation {
    source: SequenceReference,
    edited: Sequence,
    position: NucleotideInterval,
    expected_original: char,
    actual_original: char,
    replacement: char,
    state: MutationState,
}

/// The record of a residue substitution: the residue assertion, the DNA edit
/// that satisfied it, and where that edit landed on the reference.
#[derive(Clone, Debug, PartialEq)]
pub struct ResidueMutation {
    source: SequenceReference,
    edited: Sequence,
    cds: Identifier,
    position: ResiduePosition,
    expected_original: Residue,
    actual_original: Residue,
    replacement: Residue,
    original_codon: String,
    selected_codon: Codon,
    policy: CodonSelectionPolicy,
    coordinates: OrderedIntervals,
    state: MutationState,
}

/// A mutation result. The kind is the operation, not a label: a residue
/// record and a nucleotide record are different types.
#[derive(Clone, Debug, PartialEq)]
pub enum MutationResult {
    Nucleotide(NucleotideMutation),
    Residue(ResidueMutation),
}

impl MutationResult {
    /// The artifact this result was computed from, by identity.
    pub fn source(&self) -> SequenceReference {
        match self {
            MutationResult::Nucleotide(record) => record.source.clone(),
            MutationResult::Residue(record) => record.source.clone(),
        }
    }

    /// The new artifact the mutation produced. The source is unchanged.
    pub fn edited(&self) -> Sequence {
        match self {
            MutationResult::Nucleotide(record) => record.edited.clone(),
            MutationResult::Residue(record) => record.edited.clone(),
        }
    }

    /// Identity of the produced artifact.
    pub fn reference(&self) -> SequenceReference {
        self.edited().reference()
    }

    /// Reference coordinates the mutation wrote, ascending.
    pub fn target_coordinates(&self) -> Vec<u64> {
        match self {
            MutationResult::Nucleotide(record) => coordinates_of(record.position.clone()),
            MutationResult::Residue(record) => coordinates_positions(record.coordinates.clone()),
        }
    }

    pub fn state(&self) -> MutationState {
        match self {
            MutationResult::Nucleotide(record) => record.state.clone(),
            MutationResult::Residue(record) => record.state.clone(),
        }
    }

    pub fn is_nucleotide(&self) -> bool {
        match self {
            MutationResult::Nucleotide(_record) => true,
            MutationResult::Residue(_record) => false,
        }
    }

    /// Short label for error payloads.
    pub fn describe(&self) -> String {
        self.edited().reference().describe()
    }
}

impl NucleotideMutation {
    pub fn position(&self) -> NucleotideInterval {
        self.position.clone()
    }

    pub fn expected_original(&self) -> char {
        self.expected_original
    }

    pub fn actual_original(&self) -> char {
        self.actual_original
    }

    pub fn replacement(&self) -> char {
        self.replacement
    }

    pub fn state(&self) -> MutationState {
        self.state.clone()
    }
}

impl ResidueMutation {
    pub fn cds(&self) -> Identifier {
        self.cds.clone()
    }

    pub fn position(&self) -> ResiduePosition {
        self.position.clone()
    }

    pub fn expected_original(&self) -> Residue {
        self.expected_original.clone()
    }

    pub fn actual_original(&self) -> Residue {
        self.actual_original.clone()
    }

    pub fn replacement(&self) -> Residue {
        self.replacement.clone()
    }

    pub fn original_codon(&self) -> String {
        self.original_codon.clone()
    }

    pub fn selected_codon(&self) -> Codon {
        self.selected_codon.clone()
    }

    pub fn policy(&self) -> CodonSelectionPolicy {
        self.policy.clone()
    }

    pub fn coordinates(&self) -> OrderedIntervals {
        self.coordinates.clone()
    }

    pub fn state(&self) -> MutationState {
        self.state.clone()
    }
}

/// Reject a target coordinate that lies outside the intervals of the feature
/// or CDS it was scoped to.
pub fn require_within_feature(
    position: NucleotideInterval,
    feature: String,
    sequence: String,
    intervals: &[NucleotideInterval],
) -> Result<(), DomainError> {
    let start: u64 = position.start();
    let mut index: usize = 0;
    while index < intervals.len() {
        let interval: NucleotideInterval = intervals[index].clone();
        if start >= interval.start() && start < interval.end() {
            return Ok(());
        }
        index += 1;
    }
    let problem = TargetOutsideFeatureProblem {
        feature,
        sequence,
        start: position.start(),
        end: position.end(),
    };
    Err(DomainError::TargetOutsideFeature(problem))
}

/// Apply a nucleotide substitution.
///
/// `scope` optionally names a feature or CDS the coordinate must lie inside;
/// the strand of that scope is irrelevant to a DNA edit, so only its
/// intervals are consulted. Nothing is written until every check has passed,
/// and the source artifact is never modified.
pub fn apply_nucleotide(
    sequence: &Sequence,
    scope: Option<Cds>,
    result: Identifier,
    intent: NucleotideMutationIntent,
) -> Result<MutationResult, DomainError> {
    require_sequence(intent.source().clone(), sequence)?;
    require_sequence(intent.position().reference().clone(), sequence)?;
    intent.position().within(sequence)?;

    if let Some(cds) = scope.clone() {
        require_sequence(cds.reference().clone(), sequence)?;
        let label: String = cds.id().value().to_string();
        require_within_feature(
            intent.position().clone(),
            label,
            sequence.display_name(),
            cds.intervals().as_slice(),
        )?;
    }

    let position: u64 = intent.position().start();
    let actual: char = sequence.letter_at(position)?;
    if actual != intent.expected_original() {
        let problem = ExpectedNucleotideMismatchProblem {
            sequence: sequence.display_name(),
            position,
            expected: intent.expected_original().to_string(),
            actual: actual.to_string(),
        };
        return Err(DomainError::ExpectedNucleotideMismatch(problem));
    }

    let mut letters: Vec<char> = sequence.letters();
    letters[position as usize] = intent.replacement();
    let edited: Sequence = normalize(
        result.clone(),
        canonical_text(letters.clone()),
        sequence.topology(),
    )?;

    Ok(MutationResult::Nucleotide(NucleotideMutation {
        source: sequence.reference(),
        edited: edited.clone(),
        position: intent.position().clone(),
        expected_original: intent.expected_original(),
        actual_original: actual,
        replacement: intent.replacement(),
        state: MutationState::Calculated,
    }))
}

/// Apply a residue substitution to the CDS it names.
///
/// The CDS is translated first, so an incomplete CDS, an unsupported genetic
/// code or numbering scheme, a non-triplet length, and an ambiguous codon all
/// fail before any edit is considered. The expected residue is then checked
/// against the translation, the codon policy is resolved, and only then is
/// anything written.
pub fn apply_residue(
    sequence: &Sequence,
    cds: Cds,
    result: Identifier,
    intent: ResidueMutationIntent,
) -> Result<MutationResult, DomainError> {
    require_sequence(cds.reference().clone(), sequence)?;

    let translation: Translation = translate(sequence, &cds.clone())?;
    let actual: Residue = translation.residue_at(&intent.position())?;
    if actual != intent.expected_original() {
        let problem = ExpectedResidueMismatchProblem {
            cds: cds.id().value().to_string(),
            number: intent.position().number(),
            expected: intent.expected_original().symbol(),
            actual: actual.symbol(),
        };
        return Err(DomainError::ExpectedResidueMismatch(problem));
    }

    let label: String = cds.id().value().to_string();
    let selected: Codon = select_codon(
        label.clone(),
        intent.policy().clone(),
        intent.replacement().clone(),
    )?;

    let extracted: String = extract_cds(sequence, &cds.clone())?;
    let original_codon: String = codon_slice(extracted.clone(), intent.position().number(), label)?;
    if selected.text() == original_codon {
        let problem = InvalidMutationRequestProblem {
            request: selected.text(),
            reason: "the codon the policy chose is the codon already at that residue".to_string(),
        };
        return Err(DomainError::InvalidMutationRequest(problem));
    }

    let written: Vec<u64> = codon_positions(
        cds.intervals().clone(),
        cds.strand().clone(),
        intent.position().number(),
        extracted.len() as u64,
    )?;

    let mut letters: Vec<char> = sequence.letters();
    let codon_letters: Vec<char> = selected.text().chars().collect();
    let mut index: usize = 0;
    while index < written.len() {
        letters[written[index] as usize] =
            complement_or(codon_letters[index], cds.strand().clone());
        index += 1;
    }

    let edited: Sequence = normalize(
        result.clone(),
        canonical_text(letters.clone()),
        sequence.topology(),
    )?;

    Ok(MutationResult::Residue(ResidueMutation {
        source: sequence.reference(),
        edited: edited.clone(),
        cds: cds.id().clone(),
        position: intent.position().clone(),
        expected_original: intent.expected_original().clone(),
        actual_original: actual.clone(),
        replacement: intent.replacement().clone(),
        original_codon: original_codon.clone(),
        selected_codon: selected.clone(),
        policy: intent.policy().clone(),
        coordinates: intervals_for(written.clone(), edited.reference()),
        state: MutationState::Calculated,
    }))
}

/// Resolve a codon policy. Nothing is chosen when the policy is missing, and
/// a supplied codon that does not encode the replacement residue is a
/// contradictory request rather than something to interpret.
pub fn select_codon(
    cds: String,
    policy: CodonSelectionPolicy,
    replacement: Residue,
) -> Result<Codon, DomainError> {
    match policy {
        CodonSelectionPolicy::Unspecified => {
            let problem = UnspecifiedCodonPolicyProblem { cds: cds.clone() };
            Err(DomainError::UnspecifiedCodonPolicy(problem))
        }
        CodonSelectionPolicy::Default => default_codon(replacement.clone()),
        CodonSelectionPolicy::Supplied(codon) => {
            if codon.residue() != replacement {
                let problem = InvalidMutationRequestProblem {
                    request: codon.text().clone(),
                    reason: "the supplied codon does not encode the replacement residue"
                        .to_string(),
                };
                return Err(DomainError::InvalidMutationRequest(problem));
            }
            Ok(codon.clone())
        }
    }
}

/// The codon policy's `Default`: the codon of `residue` with the highest
/// count in the *E. coli* K-12 codon usage table (`codon_usage`). A codon
/// that ties the current best keeps the earlier one in standard table order,
/// so the choice is deterministic even if a future table has equal counts.
pub fn default_codon(residue: Residue) -> Result<Codon, DomainError> {
    let mut best: Option<(String, u64)> = None;
    for (codon, count) in ECOLI_K12_CODON_COUNTS.iter() {
        let encodes: bool = match lookup(codon.to_string()) {
            CodonOutcome::Residue(value) => value == residue,
            CodonOutcome::Stop => false,
            CodonOutcome::Ambiguous => false,
            CodonOutcome::Invalid => false,
        };
        if !encodes {
            continue;
        }
        let better: bool = match &best {
            None => true,
            Some((_text, best_count)) => *count > *best_count,
        };
        if better {
            best = Some((codon.to_string(), *count));
        }
    }

    match best {
        Some((codon, _count)) => Codon::create(codon),
        None => {
            // Every residue of the standard code has a codon in the table, so
            // this is unreachable; a residue that somehow has none is a bad
            // request.
            let problem = InvalidMutationRequestProblem {
                request: residue.symbol(),
                reason: "the residue has no codon in the codon usage table".to_string(),
            };
            Err(DomainError::InvalidMutationRequest(problem))
        }
    }
}

/// Independent verification of a mutation result (D013).
///
/// The edit is not re-run. The checks are: that the source artifact is the
/// one the result claims, by name, digest, and topology; that length is preserved, which
/// is the substitution invariant; and that the positions where source and
/// edited differ agree with the recorded target coordinates, exactly for a
/// nucleotide result and as a non-empty subset for a residue result, because
/// a replacement triplet may share letters with the triplet it replaces.
/// The result's own claims are then read back out of the two artifacts: the
/// nucleotide at a nucleotide target, and for a residue result the CDS the
/// result names, the residue sequences re-translated through Milestone 1's
/// extraction and translation, and the codon read at the requested residue.
/// Verification therefore accepts a residue result whose recorded codon
/// coordinates are a superset of the letters that actually changed, and it
/// does not re-derive the residue-to-coordinate mapping: that mapping is the
/// generator's responsibility, checked here by the residue and codon claims
/// rather than by a second copy of the mapping.
pub fn verify(
    source: &Sequence,
    cds: Option<Cds>,
    result: MutationResult,
) -> Result<MutationResult, DomainError> {
    require_sequence(result.source().clone(), source)?;

    let edited: Sequence = result.edited();
    let source_letters: Vec<char> = source.letters();
    let edited_letters: Vec<char> = edited.letters();

    if source_letters.len() != edited_letters.len() {
        let problem = MutationVerificationFailureProblem {
            result: result.describe(),
            reason: "a substitution preserves length".to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    let want: Vec<u64> = result.target_coordinates();
    let mut got: Vec<u64> = Vec::new();
    let count: usize = source_letters.len();
    let mut index: usize = 0;
    while index < count {
        if source_letters[index] != edited_letters[index] {
            got.push(index as u64);
        }
        index += 1;
    }
    match result.clone() {
        MutationResult::Nucleotide(record) => {
            if got != want {
                let problem = MutationVerificationFailureProblem {
                    result: result.describe(),
                    reason: "the edited artifact differs at ".to_string()
                        + &describe_positions(got.clone())
                        + ", the result claims "
                        + &describe_positions(want.clone()),
                };
                return Err(DomainError::MutationVerificationFailure(problem));
            }
            verify_nucleotide_claims(source, &edited.clone(), record)?;
        }
        MutationResult::Residue(record) => {
            // A codon substitution may share letters with the codon it
            // replaces, so the edit has to lie inside the recorded codon
            // rather than fill it. It must not be empty: a result that
            // changed nothing is not a mutation.
            if got.is_empty() {
                let problem = MutationVerificationFailureProblem {
                    result: result.describe(),
                    reason: "the edited artifact is identical to the source".to_string(),
                };
                return Err(DomainError::MutationVerificationFailure(problem));
            }
            for position in got {
                if !want.contains(&position) {
                    let problem = MutationVerificationFailureProblem {
                        result: result.describe(),
                        reason: "the edit wrote a nucleotide outside the recorded coordinates"
                            .to_string(),
                    };
                    return Err(DomainError::MutationVerificationFailure(problem));
                }
            }
            verify_residue_claims(source, cds, &edited.clone(), record)?;
        }
    }

    Ok(accepted(result.clone()))
}

/// The nucleotide claims of a result, checked against both artifacts.
fn verify_nucleotide_claims(
    source: &Sequence,
    edited: &Sequence,
    record: NucleotideMutation,
) -> Result<(), DomainError> {
    let position: u64 = record.position().start();
    let expected_source: bool = source.letter_at(position).unwrap() == record.expected_original();
    let expected_edited: bool = edited.letter_at(position).unwrap() == record.replacement();
    if !expected_source || !expected_edited {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the nucleotide claims do not match the artifacts".to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }
    Ok(())
}

/// The residue claims of a result, checked by re-translating both artifacts.
fn verify_residue_claims(
    source: &Sequence,
    cds: Option<Cds>,
    edited: &Sequence,
    record: ResidueMutation,
) -> Result<(), DomainError> {
    if cds.is_none() {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "a residue result must be verified against the CDS it was applied to"
                .to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    let annotation: Cds = cds.unwrap();

    // The result names the CDS it was applied to, so verification is only
    // independent in the residue claims: the annotation handed to it has to
    // be that same CDS, or the claims would be checked against an annotation
    // the result never made.
    if annotation.id() != record.cds() {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the CDS verified against is not the CDS the result names".to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    require_sequence(annotation.reference().clone(), source)?;

    let before: Translation = translate(source, &annotation.clone())?;
    let rebuilt: Cds = rebuild_cds(edited.clone(), annotation.clone())?;
    let after: Translation = translate(&edited.clone(), &rebuilt.clone())?;

    let number: u64 = record.position().number();
    if number < 1 || number > before.residue_count() as u64 || number > after.residue_count() as u64
    {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the requested residue does not exist in the CDS".to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    let before_residues: Vec<Residue> = before.residues();
    let after_residues: Vec<Residue> = after.residues();
    let offset: usize = number as usize - 1;

    if before_residues[offset] != record.expected_original() {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the source does not hold the expected original residue".to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }
    if after_residues[offset] != record.replacement() {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the edited CDS does not hold the replacement residue".to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    let mut index: usize = 0;
    while index < before_residues.len() {
        if index != offset && before_residues[index] != after_residues[index] {
            let problem = MutationVerificationFailureProblem {
                result: record.edited.reference().describe(),
                reason: "a residue other than the requested one changed".to_string(),
            };
            return Err(DomainError::MutationVerificationFailure(problem));
        }
        index += 1;
    }

    let edited_cds_text: String = extract_cds(&edited.clone(), &rebuilt.clone())?;
    let label: String = annotation.id().value().to_string();
    let codon: String = codon_slice(edited_cds_text.clone(), number, label)?;
    if codon != record.selected_codon().text() {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the codon read from the edited CDS is not the recorded codon".to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    Ok(())
}

/// The same result with state `Verified`. Only `verify` may produce this.
fn accepted(result: MutationResult) -> MutationResult {
    match result {
        MutationResult::Nucleotide(record) => {
            let mut copy: NucleotideMutation = record.clone();
            copy.state = MutationState::Verified;
            MutationResult::Nucleotide(copy)
        }
        MutationResult::Residue(record) => {
            let mut copy: ResidueMutation = record.clone();
            copy.state = MutationState::Verified;
            MutationResult::Residue(copy)
        }
    }
}

/// The same CDS coordinates, rebuilt against another artifact.
///
/// This is how verification re-reads an edited artifact: the annotation is
/// re-derived from its declared coordinates rather than carried over, because
/// a `Cds` names its reference sequence by identity.
pub fn rebuild_cds(edited: Sequence, annotation: Cds) -> Result<Cds, DomainError> {
    let reference: SequenceReference = edited.reference();
    let mut intervals: OrderedIntervals = Vec::new();
    for interval in annotation.intervals().iter() {
        let rebuilt: NucleotideInterval =
            create(&reference, interval.start() as i64, interval.end() as i64)?;
        intervals.push(rebuilt);
    }
    Cds::create(
        &edited.clone(),
        annotation.id().clone(),
        intervals.clone(),
        annotation.strand().clone(),
        annotation.completeness().clone(),
        annotation.genetic_code().name(),
        annotation.numbering().clone(),
    )
}

/// Reference coordinates of the codon at a residue, in the order the codon
/// is read on the CDS strand.
pub fn codon_positions(
    intervals: OrderedIntervals,
    strand: Strand,
    number: u64,
    total_length: u64,
) -> Result<Vec<u64>, DomainError> {
    let mut positions: Vec<u64> = Vec::new();
    let mut offset: usize = 0;
    let plus: bool = strand.clone().is_plus();
    while offset < 3 {
        let read: u64 = 3 * (number - 1) + offset as u64;
        let biological: u64 = if plus { read } else { total_length - 1 - read };
        let found: Option<u64> = reference_of(intervals.as_slice(), biological);
        if found.is_none() {
            let problem = InvalidMutationRequestProblem {
                request: "codon coordinates".to_string(),
                reason: "the residue does not select nucleotides that exist".to_string(),
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }
        positions.push(found.unwrap());
        offset += 1;
    }
    Ok(positions)
}

/// The reference coordinate of one biological offset.
fn reference_of(intervals: &[NucleotideInterval], biological: u64) -> Option<u64> {
    let mut seen: u64 = 0;
    for interval in intervals.iter() {
        let width: u64 = interval.length();
        if biological < seen + width {
            return Some(interval.start() + (biological - seen));
        }
        seen += width;
    }
    None
}

/// Group ascending coordinates into intervals, one per contiguous run.
///
/// A codon that straddles the origin of a circular reference becomes two
/// intervals; nothing is wrapped. The coordinates arrive in the order the
/// codon is read on the CDS strand, which is descending on a minus strand,
/// so they are put into ascending order first: the record is a set of
/// reference coordinates.
fn intervals_for(positions: Vec<u64>, reference: SequenceReference) -> OrderedIntervals {
    let positions: Vec<u64> = ascending(positions.clone());
    let mut out: OrderedIntervals = Vec::new();
    let mut index: usize = 0;
    while index < positions.len() {
        let start: u64 = positions[index];
        let mut last: usize = index;
        let mut next: usize = index + 1;
        while next < positions.len() && positions[next] == positions[last] + 1 {
            last = next;
            next += 1;
        }
        let interval: NucleotideInterval =
            create(&reference, start as i64, (positions[last] + 1) as i64).unwrap();
        out.push(interval);
        index = next;
    }
    out
}

/// Ascending copy of a list of coordinates. Insertion sort: the list is a
/// codon, three coordinates at most, and no ordering utility is imported.
fn ascending(positions: Vec<u64>) -> Vec<u64> {
    let mut out: Vec<u64> = positions.clone();
    let count: usize = out.len();
    let mut index: usize = 1;
    while index < count {
        let value: u64 = out[index];
        let mut hole: usize = index;
        while hole > 0 && out[hole - 1] > value {
            out[hole] = out[hole - 1];
            hole -= 1;
        }
        out[hole] = value;
        index += 1;
    }
    out
}

/// Ascending reference coordinates of an interval list.
fn coordinates_positions(intervals: OrderedIntervals) -> Vec<u64> {
    let mut out: Vec<u64> = Vec::new();
    for interval in intervals.iter() {
        let mut position: u64 = interval.start();
        while position < interval.end() {
            out.push(position);
            position += 1;
        }
    }
    out
}

/// Reference coordinates of one interval.
fn coordinates_of(interval: NucleotideInterval) -> Vec<u64> {
    coordinates_positions(vec![interval.clone()])
}

/// The codon read at a one-based residue of an extracted CDS text.
fn codon_slice(extracted: String, number: u64, cds: String) -> Result<String, DomainError> {
    let letters: Vec<char> = extracted.chars().collect();
    if number < 1 {
        let problem = ResiduePositionRangeProblem {
            cds: cds.clone(),
            number,
            residue_count: (letters.len() / 3) as u64,
        };
        return Err(DomainError::ResiduePositionOutsideCds(problem));
    }
    let start: usize = 3 * (number as usize - 1);
    if start + 3 > letters.len() {
        let problem = ResiduePositionRangeProblem {
            cds: cds.clone(),
            number,
            residue_count: (letters.len() / 3) as u64,
        };
        return Err(DomainError::ResiduePositionOutsideCds(problem));
    }
    Ok(text_of(vec![
        letters[start],
        letters[start + 1],
        letters[start + 2],
    ]))
}

/// Text of letters, for codons built out of the table.
fn text_of(letters: Vec<char>) -> String {
    canonical_text(letters.clone())
}

/// The letter to write at a reference coordinate for a codon letter read on
/// the CDS strand.
fn complement_or(letter: char, strand: Strand) -> char {
    if strand.is_plus() {
        return letter;
    }
    complement_letter(letter).unwrap()
}

/// One of `A`, `C`, `G`, `T`, or a request that is not representable.
///
/// IUPAC ambiguity codes are valid sequence letters but not substitution
/// targets or replacements: writing `N` is not a defined edit, and asserting
/// that a position holds `N` would not identify a base.
fn single_dna_base(field: String, text: String) -> Result<char, DomainError> {
    let letters: Vec<char> = text.chars().collect();
    if letters.len() != 1 {
        let problem = InvalidMutationRequestProblem {
            request: text,
            reason: field.to_string() + " must be exactly one letter",
        };
        return Err(DomainError::InvalidMutationRequest(problem));
    }
    if !is_dna_base(letters[0]) {
        let problem = InvalidMutationRequestProblem {
            request: text,
            reason: field.to_string() + " must be one of A, C, G, T",
        };
        return Err(DomainError::InvalidMutationRequest(problem));
    }
    Ok(letters[0])
}

/// Positions rendered for an error message.
fn describe_positions(positions: Vec<u64>) -> String {
    let mut text: String = String::new();
    let mut index: usize = 0;
    while index < positions.len() {
        if index > 0 {
            text += ",";
        }
        text += &positions[index].to_string();
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
    use crate::genetic_code::Residue;
    use crate::identity::Identifier;
    use crate::interval::create;
    use crate::interval::OrderedIntervals;
    use crate::sequence::normalize;
    use crate::sequence::Sequence;
    use crate::sequence::SequenceTopology;
    use crate::strand::Strand;

    // "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG" translates over a whole 30-nucleotide
    // CDS to M A W N Q Q Q Q Q Q, so residue 2 is A and residue 5 is Q.
    fn seq_a() -> Sequence {
        normalize(
            Identifier::create("seq-a".to_string()).unwrap(),
            "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    fn cds_name() -> Identifier {
        Identifier::create("cds-1".to_string()).unwrap()
    }

    fn result_name() -> Identifier {
        Identifier::create("mut-1".to_string()).unwrap()
    }

    fn whole(length: i64) -> OrderedIntervals {
        vec![create(&seq_a().reference(), 0, length).unwrap()]
    }

    fn cds_of(length: i64) -> Cds {
        Cds::create(
            &seq_a().clone(),
            cds_name(),
            whole(length),
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap()
    }

    fn at(start: i64, end: i64) -> NucleotideInterval {
        create(&seq_a().reference(), start, end).unwrap()
    }

    fn residue(number: i64) -> ResiduePosition {
        ResiduePosition::create(cds_name(), number).unwrap()
    }

    #[test]
    fn a_wrong_expected_nucleotide_names_the_letter_that_is_there() {
        let intent: NucleotideMutationIntent = NucleotideMutationIntent::create(
            seq_a().reference(),
            at(3, 4),
            "A".to_string(),
            "C".to_string(),
        )
        .unwrap();
        let error: DomainError =
            apply_nucleotide(&seq_a().clone(), None, result_name(), intent).unwrap_err();
        assert!(matches!(error, DomainError::ExpectedNucleotideMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    // Every IUPAC ambiguity code is a valid sequence letter, but none is a base
    // that a substitution may write or assert.
    #[test]
    fn a_substitution_writes_and_asserts_only_plain_bases() {
        for code in ["R", "Y", "S", "W", "K", "M", "B", "D", "H", "V", "N"] {
            let as_replacement = NucleotideMutationIntent::create(
                seq_a().reference(),
                at(3, 4),
                "G".to_string(),
                code.to_string(),
            );
            assert!(matches!(
                as_replacement.unwrap_err(),
                DomainError::InvalidMutationRequest(_)
            ));
            let as_original = NucleotideMutationIntent::create(
                seq_a().reference(),
                at(3, 4),
                code.to_string(),
                "C".to_string(),
            );
            let error: DomainError = as_original.unwrap_err();
            assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
            assert!(error.category() == ErrorCategory::MalformedInput);
        }
    }

    #[test]
    fn only_the_default_policy_names_a_codon_usage_source() {
        let source: Option<String> = CodonSelectionPolicy::Default.source();
        assert!(source == Some(ECOLI_K12_CODON_USAGE_SOURCE.to_string()));
        assert!(CodonSelectionPolicy::Unspecified.source().is_none());
        let supplied = CodonSelectionPolicy::Supplied(Codon::create("CTG".to_string()).unwrap());
        assert!(supplied.source().is_none());
    }

    #[test]
    fn a_wrong_expected_residue_names_the_residue_that_is_there() {
        let intent: ResidueMutationIntent = ResidueMutationIntent::create(
            residue(2),
            "M".to_string(),
            "W".to_string(),
            CodonSelectionPolicy::Default,
        )
        .unwrap();
        let error: DomainError =
            apply_residue(&seq_a().clone(), cds_of(30), result_name(), intent).unwrap_err();
        assert!(matches!(error, DomainError::ExpectedResidueMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn a_target_outside_the_scoped_cds_is_refused_before_any_edit() {
        let intent: NucleotideMutationIntent = NucleotideMutationIntent::create(
            seq_a().reference(),
            at(20, 21),
            "C".to_string(),
            "A".to_string(),
        )
        .unwrap();
        let error: DomainError = apply_nucleotide(
            &seq_a().clone(),
            Some(cds_of(18).clone()),
            result_name(),
            intent,
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::TargetOutsideFeature(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn a_missing_codon_policy_is_malformed_input() {
        let intent: ResidueMutationIntent = ResidueMutationIntent::create(
            residue(2),
            "A".to_string(),
            "W".to_string(),
            CodonSelectionPolicy::Unspecified,
        )
        .unwrap();
        let error: DomainError =
            apply_residue(&seq_a().clone(), cds_of(30), result_name(), intent).unwrap_err();
        assert!(matches!(error, DomainError::UnspecifiedCodonPolicy(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    #[test]
    fn an_unrecognised_codon_policy_is_out_of_scope() {
        let error: DomainError = parse_codon_policy("minimal-edit".to_string()).unwrap_err();
        assert!(matches!(error, DomainError::UnsupportedMutationPolicy(_)));
        assert!(error.category() == ErrorCategory::Unsupported);
    }

    #[test]
    fn an_unrepresentable_mutation_request_is_malformed_input() {
        let letter: Result<NucleotideMutationIntent, DomainError> =
            NucleotideMutationIntent::create(
                seq_a().reference(),
                at(3, 4),
                "G".to_string(),
                "Z".to_string(),
            );
        assert!(letter.is_err());
        let letter_error: DomainError = letter.unwrap_err();
        assert!(matches!(
            letter_error,
            DomainError::InvalidMutationRequest(_)
        ));

        // A stop is not a residue, so it cannot be a supplied codon.
        let stop: Result<Codon, DomainError> = Codon::create("TAA".to_string());
        assert!(stop.is_err());
        let stop_error: DomainError = stop.unwrap_err();
        assert!(matches!(stop_error, DomainError::InvalidMutationRequest(_)));

        // The request is well-formed but contradicts itself.
        let supplied: Codon = Codon::create("TGG".to_string()).unwrap();
        let contradictory: Result<Codon, DomainError> = select_codon(
            cds_name().value().to_string(),
            CodonSelectionPolicy::Supplied(supplied.clone()),
            Residue::Q,
        );
        assert!(contradictory.is_err());
        assert!(matches!(
            contradictory.unwrap_err(),
            DomainError::InvalidMutationRequest(_)
        ));
    }

    #[test]
    fn a_result_that_does_not_follow_from_its_source_fails_verification() {
        let intent: NucleotideMutationIntent = NucleotideMutationIntent::create(
            seq_a().reference(),
            at(3, 4),
            "G".to_string(),
            "C".to_string(),
        )
        .unwrap();
        let calculated: MutationResult =
            apply_nucleotide(&seq_a().clone(), None, result_name(), intent).unwrap();
        assert!(calculated.state() == MutationState::Calculated);

        // The same record, claiming a coordinate it did not write.
        let tampered: MutationResult = match calculated.clone() {
            MutationResult::Nucleotide(record) => {
                let mut copy: NucleotideMutation = record.clone();
                copy.position = at(5, 6);
                MutationResult::Nucleotide(copy)
            }
            MutationResult::Residue(_record) => calculated.clone(),
        };
        let error: DomainError = verify(&seq_a().clone(), None, tampered.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));

        let verified: MutationResult = verify(&seq_a().clone(), None, calculated.clone()).unwrap();
        assert!(verified.state() == MutationState::Verified);
        assert!(verified != calculated);
    }

    #[test]
    fn a_residue_result_is_verified_against_its_cds() {
        let intent: ResidueMutationIntent = ResidueMutationIntent::create(
            residue(2),
            "A".to_string(),
            "W".to_string(),
            CodonSelectionPolicy::Default,
        )
        .unwrap();
        let calculated: MutationResult =
            apply_residue(&seq_a().clone(), cds_of(30), result_name(), intent).unwrap();
        assert!(calculated.state() == MutationState::Calculated);

        let verified: MutationResult = verify(
            &seq_a().clone(),
            Some(cds_of(30).clone()),
            calculated.clone(),
        )
        .unwrap();
        assert!(verified.state() == MutationState::Verified);

        // Without the CDS the residue claims cannot be checked at all.
        let error: DomainError = verify(&seq_a().clone(), None, calculated.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
    }

    #[test]
    fn mutation_failures_are_not_collapsed_into_one_variant() {
        let missing_policy: DomainError = apply_residue(
            &seq_a().clone(),
            cds_of(30),
            result_name(),
            ResidueMutationIntent::create(
                residue(2),
                "A".to_string(),
                "W".to_string(),
                CodonSelectionPolicy::Unspecified,
            )
            .unwrap(),
        )
        .unwrap_err();
        let out_of_scope: DomainError = parse_codon_policy("minimal-edit".to_string()).unwrap_err();
        let disagreement: DomainError = apply_residue(
            &seq_a().clone(),
            cds_of(30),
            result_name(),
            ResidueMutationIntent::create(
                residue(2),
                "M".to_string(),
                "W".to_string(),
                CodonSelectionPolicy::Default,
            )
            .unwrap(),
        )
        .unwrap_err();

        assert!(missing_policy != out_of_scope);
        assert!(disagreement != missing_policy);
        assert!(disagreement != out_of_scope);
    }
}
