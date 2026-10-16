//! Mutation: deterministic edits to a sequence artifact.
//!
//! A mutation is a typed intent checked against the artifact before anything is
//! written, and an edit that produces a *new* artifact: the source keeps its
//! content and its digest, and the result names both.
//!
//! Five intents exist, each asserting something different, each with a
//! matching `MutationResult` variant and `apply_*` operation:
//!
//! * `NucleotideMutationIntent` asserts one base at a reference coordinate.
//! * `NucleotideDeletionIntent` asserts the exact text of a region (D031).
//! * `NucleotideInsertionIntent` asserts the flanking bases at a point (D032).
//! * `NucleotideReplacementIntent` asserts that region's text and writes a text
//!   of any length (D033).
//! * `ResidueMutationIntent` asserts a residue of a CDS and leaves the DNA
//!   edit to a `CodonSelectionPolicy` (D011).
//!
//! Every intent carries the expected original state, and every one is checked
//! before the edit: a mutation never applies to another artifact, to a
//! coordinate outside the sequence or outside its scope, or to a base or
//! residue that disagrees with the request. Strand is never a parameter of a
//! mutation: it is read from the `Cds`, and residue numbering runs over the
//! strand-applied CDS text.
//!
//! Verification is a separate responsibility (D013): `verify` re-derives the
//! edit from the source and the result's claims rather than re-running it, and
//! checks length per kind (owner decision 4.2 in
//! `docs/tasks/milestone-4-indels.md` §4.2). A result stays `Calculated`
//! until that pass accepts it (D020). Containment of a target inside its
//! scope is checked here, by `require_within_feature` and
//! `require_point_within_feature`; choosing that scope and the frameshift
//! warnings is `q5.rs`'s. Numbering, genetic-code and CDS rules belong to
//! `cds.rs` and `translation.rs`.

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
use crate::interval::InsertionPoint;
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
/// The target is a `NucleotideInterval` of length 1: the `interval` module's
/// coordinate abstraction, which already names its reference sequence.
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

/// A DNA-level deletion: a contiguous region of the reference, removed.
///
/// Like a substitution it asserts the original state (D011), but what it
/// asserts is the exact text of every base it removes, so a request that
/// disagrees with the artifact is refused before anything is written.
#[derive(Clone, Debug, PartialEq)]
pub struct NucleotideDeletionIntent {
    source: SequenceReference,
    region: NucleotideInterval,
    expected_deleted: String,
}

impl NucleotideDeletionIntent {
    /// Build a deletion, rejecting a request that cannot be represented: a
    /// region naming a different artifact from the one the request claims,
    /// a letter other than `A`, `C`, `G`, `T` (an ambiguity code is not a
    /// base that can be asserted as removed), or an expected text whose
    /// length does not match the number of nucleotides the region selects.
    pub fn create(
        source: SequenceReference,
        region: NucleotideInterval,
        expected_deleted: String,
    ) -> Result<NucleotideDeletionIntent, DomainError> {
        if region.reference() != source {
            let problem = InvalidMutationRequestProblem {
                request: "deletion".to_string(),
                reason: "the target coordinate names a sequence other than the source".to_string(),
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }

        let mut asserted: Vec<char> = Vec::new();
        for letter in expected_deleted.chars() {
            let base: char = single_dna_base("expected deleted".to_string(), letter.to_string())?;
            asserted.push(base);
        }
        if asserted.len() as u64 != region.length() {
            let problem = InvalidMutationRequestProblem {
                request: "deletion".to_string(),
                reason: "the expected text is ".to_string()
                    + &asserted.len().to_string()
                    + " letters; the region selects "
                    + &region.length().to_string()
                    + " nucleotides",
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }

        Ok(NucleotideDeletionIntent {
            source,
            region,
            expected_deleted: expected_deleted.clone(),
        })
    }

    pub fn source(&self) -> SequenceReference {
        self.source.clone()
    }

    pub fn region(&self) -> NucleotideInterval {
        self.region.clone()
    }

    pub fn expected_deleted(&self) -> String {
        self.expected_deleted.clone()
    }
}

/// A DNA-level insertion: bases placed into the gap between two nucleotides.
///
/// Like a substitution and a deletion it asserts the original state (D011),
/// but what an insertion asserts is the base on each side of the point: the
/// point itself selects nothing, so the two flanking bases are what identify
/// where the insertion sits. A request that disagrees with the artifact is
/// refused before anything is written.
#[derive(Clone, Debug, PartialEq)]
pub struct NucleotideInsertionIntent {
    source: SequenceReference,
    point: InsertionPoint,
    expected_before: char,
    expected_after: char,
    inserted: String,
}

impl NucleotideInsertionIntent {
    /// Build an insertion, rejecting a request that cannot be represented: a
    /// point naming a different artifact from the one the request claims, a
    /// flank other than `A`, `C`, `G`, `T` (an ambiguity code is not a base
    /// that can be asserted as sitting beside the point), an insertion of no
    /// bases at all, or an inserted text holding a letter other than `A`, `C`,
    /// `G`, `T`.
    pub fn create(
        source: SequenceReference,
        point: InsertionPoint,
        expected_before: String,
        expected_after: String,
        inserted: String,
    ) -> Result<NucleotideInsertionIntent, DomainError> {
        if point.reference() != source {
            let problem = InvalidMutationRequestProblem {
                request: "insertion".to_string(),
                reason: "the target coordinate names a sequence other than the source".to_string(),
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }

        let before: char = single_dna_base("expected before".to_string(), expected_before.clone())?;
        let after: char = single_dna_base("expected after".to_string(), expected_after.clone())?;

        if inserted.is_empty() {
            let problem = InvalidMutationRequestProblem {
                request: "insertion".to_string(),
                reason: "an insertion must insert at least one nucleotide".to_string(),
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }

        let mut added: Vec<char> = Vec::new();
        for letter in inserted.chars() {
            let base: char = single_dna_base("inserted".to_string(), letter.to_string())?;
            added.push(base);
        }

        Ok(NucleotideInsertionIntent {
            source,
            point,
            expected_before: before,
            expected_after: after,
            inserted: inserted.clone(),
        })
    }

    pub fn source(&self) -> SequenceReference {
        self.source.clone()
    }

    pub fn point(&self) -> InsertionPoint {
        self.point.clone()
    }

    pub fn expected_before(&self) -> char {
        self.expected_before
    }

    pub fn expected_after(&self) -> char {
        self.expected_after
    }

    pub fn inserted(&self) -> String {
        self.inserted.clone()
    }
}

/// A DNA-level replacement: a contiguous region of the reference rewritten as
/// a text of any length.
///
/// Like a substitution and a deletion it asserts the original state (D011),
/// but what it asserts is the exact text of every base it overwrites, and what
/// it writes need not have the width of what it removes. A request that
/// disagrees with the artifact is refused before anything is written, and a
/// request that writes back the text already in the region edits nothing.
#[derive(Clone, Debug, PartialEq)]
pub struct NucleotideReplacementIntent {
    source: SequenceReference,
    region: NucleotideInterval,
    expected_original: String,
    replacement: String,
}

impl NucleotideReplacementIntent {
    /// Build a replacement, rejecting a request that cannot be represented: a
    /// region naming a different artifact from the one the request claims, a
    /// letter other than `A`, `C`, `G`, `T` in either text (an ambiguity code
    /// is not a base that can be asserted or written), an expected text whose
    /// length does not match the number of nucleotides the region selects, a
    /// replacement of no bases at all, or a replacement that is the text
    /// already in the region.
    pub fn create(
        source: SequenceReference,
        region: NucleotideInterval,
        expected_original: String,
        replacement: String,
    ) -> Result<NucleotideReplacementIntent, DomainError> {
        if region.reference() != source {
            let problem = InvalidMutationRequestProblem {
                request: "replacement".to_string(),
                reason: "the target coordinate names a sequence other than the source".to_string(),
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }

        let mut asserted: Vec<char> = Vec::new();
        for letter in expected_original.chars() {
            let base: char = single_dna_base("expected original".to_string(), letter.to_string())?;
            asserted.push(base);
        }
        let mut written: Vec<char> = Vec::new();
        for letter in replacement.chars() {
            let base: char = single_dna_base("replacement".to_string(), letter.to_string())?;
            written.push(base);
        }

        if asserted.len() as u64 != region.length() {
            let problem = InvalidMutationRequestProblem {
                request: "replacement".to_string(),
                reason: "the expected text is ".to_string()
                    + &asserted.len().to_string()
                    + " letters; the region selects "
                    + &region.length().to_string()
                    + " nucleotides",
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }

        if written.is_empty() {
            let problem = InvalidMutationRequestProblem {
                request: "replacement".to_string(),
                reason: "a replacement must write at least one nucleotide".to_string(),
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }

        if replacement == expected_original {
            let problem = InvalidMutationRequestProblem {
                request: "replacement".to_string(),
                reason: "the replacement is the text that is already there".to_string(),
            };
            return Err(DomainError::InvalidMutationRequest(problem));
        }

        Ok(NucleotideReplacementIntent {
            source,
            region,
            expected_original: expected_original.clone(),
            replacement: replacement.clone(),
        })
    }

    pub fn source(&self) -> SequenceReference {
        self.source.clone()
    }

    pub fn region(&self) -> NucleotideInterval {
        self.region.clone()
    }

    pub fn expected_original(&self) -> String {
        self.expected_original.clone()
    }

    pub fn replacement(&self) -> String {
        self.replacement.clone()
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

/// The record of a deletion: the region removed, the bases the request
/// asserted, and the bases the artifact actually held.
#[derive(Clone, Debug, PartialEq)]
pub struct NucleotideDeletion {
    source: SequenceReference,
    edited: Sequence,
    region: NucleotideInterval,
    expected_deleted: String,
    actual_deleted: String,
    state: MutationState,
}

/// The record of an insertion: the point it sits at, the bases inserted, and
/// the flanking bases the artifact actually held.
#[derive(Clone, Debug, PartialEq)]
pub struct NucleotideInsertion {
    source: SequenceReference,
    edited: Sequence,
    point: InsertionPoint,
    expected_before: char,
    expected_after: char,
    actual_before: char,
    actual_after: char,
    inserted: String,
    state: MutationState,
}

/// The record of a replacement: the region it overwrote, the bases the request
/// asserted there, the bases the artifact actually held, and the text written
/// in their place.
#[derive(Clone, Debug, PartialEq)]
pub struct NucleotideReplacement {
    source: SequenceReference,
    edited: Sequence,
    region: NucleotideInterval,
    expected_original: String,
    actual_original: String,
    replacement: String,
    state: MutationState,
}

/// The record of a residue substitution: the residue assertion, the CDS
/// annotation it was applied to, the DNA edit that satisfied it, and where
/// that edit landed on the reference. `verify` is handed that annotation, not
/// another CDS that happens to share its name.
#[derive(Clone, Debug, PartialEq)]
pub struct ResidueMutation {
    source: SequenceReference,
    edited: Sequence,
    cds: Identifier,
    annotation: Cds,
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
    Deletion(NucleotideDeletion),
    Insertion(NucleotideInsertion),
    Replacement(NucleotideReplacement),
}

impl MutationResult {
    /// The artifact this result was computed from, by identity.
    pub fn source(&self) -> SequenceReference {
        match self {
            MutationResult::Nucleotide(record) => record.source.clone(),
            MutationResult::Residue(record) => record.source.clone(),
            MutationResult::Deletion(record) => record.source.clone(),
            MutationResult::Insertion(record) => record.source.clone(),
            MutationResult::Replacement(record) => record.source.clone(),
        }
    }

    /// The new artifact the mutation produced. The source is unchanged.
    pub fn edited(&self) -> Sequence {
        match self {
            MutationResult::Nucleotide(record) => record.edited.clone(),
            MutationResult::Residue(record) => record.edited.clone(),
            MutationResult::Deletion(record) => record.edited.clone(),
            MutationResult::Insertion(record) => record.edited.clone(),
            MutationResult::Replacement(record) => record.edited.clone(),
        }
    }

    /// Identity of the produced artifact.
    pub fn reference(&self) -> SequenceReference {
        self.edited().reference()
    }

    /// Reference coordinates the mutation wrote or removed, ascending.
    ///
    /// An insertion contributes none: it writes and removes no source
    /// coordinate, it puts bases into the gap between two of them.
    pub fn target_coordinates(&self) -> Vec<u64> {
        match self {
            MutationResult::Nucleotide(record) => coordinates_of(record.position.clone()),
            MutationResult::Residue(record) => coordinates_positions(record.coordinates.clone()),
            MutationResult::Deletion(record) => coordinates_of(record.region.clone()),
            MutationResult::Insertion(_record) => Vec::new(),
            MutationResult::Replacement(record) => coordinates_of(record.region.clone()),
        }
    }

    pub fn state(&self) -> MutationState {
        match self {
            MutationResult::Nucleotide(record) => record.state.clone(),
            MutationResult::Residue(record) => record.state.clone(),
            MutationResult::Deletion(record) => record.state.clone(),
            MutationResult::Insertion(record) => record.state.clone(),
            MutationResult::Replacement(record) => record.state.clone(),
        }
    }

    /// True only for a single-nucleotide substitution. A deletion, an insertion
    /// and a replacement are DNA edits too, each with its own length rule, and a
    /// residue substitution names a residue of a CDS rather than a position in
    /// the sequence.
    pub fn is_nucleotide(&self) -> bool {
        match self {
            MutationResult::Nucleotide(_record) => true,
            MutationResult::Residue(_record) => false,
            MutationResult::Deletion(_record) => false,
            MutationResult::Insertion(_record) => false,
            MutationResult::Replacement(_record) => false,
        }
    }

    /// True only for a DNA-level deletion.
    pub fn is_deletion(&self) -> bool {
        match self {
            MutationResult::Deletion(_record) => true,
            MutationResult::Nucleotide(_record) => false,
            MutationResult::Residue(_record) => false,
            MutationResult::Insertion(_record) => false,
            MutationResult::Replacement(_record) => false,
        }
    }

    /// True only for a DNA-level insertion.
    pub fn is_insertion(&self) -> bool {
        match self {
            MutationResult::Insertion(_record) => true,
            MutationResult::Nucleotide(_record) => false,
            MutationResult::Residue(_record) => false,
            MutationResult::Deletion(_record) => false,
            MutationResult::Replacement(_record) => false,
        }
    }

    /// True only for a DNA-level replacement.
    pub fn is_replacement(&self) -> bool {
        match self {
            MutationResult::Replacement(_record) => true,
            MutationResult::Nucleotide(_record) => false,
            MutationResult::Residue(_record) => false,
            MutationResult::Deletion(_record) => false,
            MutationResult::Insertion(_record) => false,
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

impl NucleotideDeletion {
    pub fn region(&self) -> NucleotideInterval {
        self.region.clone()
    }

    pub fn expected_deleted(&self) -> String {
        self.expected_deleted.clone()
    }

    pub fn actual_deleted(&self) -> String {
        self.actual_deleted.clone()
    }

    pub fn state(&self) -> MutationState {
        self.state.clone()
    }
}

impl NucleotideInsertion {
    pub fn point(&self) -> InsertionPoint {
        self.point.clone()
    }

    pub fn expected_before(&self) -> char {
        self.expected_before
    }

    pub fn expected_after(&self) -> char {
        self.expected_after
    }

    pub fn actual_before(&self) -> char {
        self.actual_before
    }

    pub fn actual_after(&self) -> char {
        self.actual_after
    }

    pub fn inserted(&self) -> String {
        self.inserted.clone()
    }

    pub fn state(&self) -> MutationState {
        self.state.clone()
    }
}

impl NucleotideReplacement {
    pub fn region(&self) -> NucleotideInterval {
        self.region.clone()
    }

    pub fn expected_original(&self) -> String {
        self.expected_original.clone()
    }

    pub fn actual_original(&self) -> String {
        self.actual_original.clone()
    }

    pub fn replacement(&self) -> String {
        self.replacement.clone()
    }

    pub fn state(&self) -> MutationState {
        self.state.clone()
    }
}

impl ResidueMutation {
    pub fn cds(&self) -> Identifier {
        self.cds.clone()
    }

    pub fn annotation(&self) -> Cds {
        self.annotation.clone()
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

/// Reject a target position that is not wholly inside one of the intervals of
/// the feature or CDS it was scoped to.
///
/// A position is a half-open span of coordinates, so the whole span has to lie
/// inside one interval rather than straddle the edge of two of them; for a
/// single nucleotide that is the same as checking its start.
fn require_within_feature(
    position: NucleotideInterval,
    feature: String,
    sequence: String,
    intervals: &[NucleotideInterval],
) -> Result<(), DomainError> {
    let mut index: usize = 0;
    while index < intervals.len() {
        let interval: NucleotideInterval = intervals[index].clone();
        if position.start() >= interval.start() && position.end() <= interval.end() {
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

/// Reject a point that does not sit strictly inside one of the intervals of
/// the feature or CDS it was scoped to.
///
/// A point selects no nucleotide, so the rule differs from `require_within_feature`, which
/// requires every nucleotide a position selects to lie inside one interval: an insertion
/// asserts and writes the bases on both sides of the point, so both of them have to lie inside
/// the same interval. A point exactly at a CDS's edge is outside it.
fn require_point_within_feature(
    point: InsertionPoint,
    feature: String,
    sequence: String,
    intervals: &[NucleotideInterval],
) -> Result<(), DomainError> {
    let position: u64 = point.position();
    let mut index: usize = 0;
    while index < intervals.len() {
        let interval: NucleotideInterval = intervals[index].clone();
        if position > interval.start() && position < interval.end() {
            return Ok(());
        }
        index += 1;
    }
    let problem = TargetOutsideFeatureProblem {
        feature,
        sequence,
        start: position,
        end: position,
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
        annotation: cds.clone(),
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

/// Apply a DNA-level deletion of a contiguous region.
///
/// The region is checked against the artifact and against its scope before
/// anything is written, and the bases it removes are asserted exactly, so a
/// request that disagrees with the artifact is refused rather than guessed.
/// Nothing is written until every check has passed, and the source artifact
/// is never modified: the edited artifact is a new one, one region shorter,
/// that keeps the source's topology.
pub fn apply_deletion(
    sequence: &Sequence,
    scope: Option<Cds>,
    result: Identifier,
    intent: NucleotideDeletionIntent,
) -> Result<MutationResult, DomainError> {
    require_sequence(intent.source().clone(), sequence)?;
    require_sequence(intent.region().reference().clone(), sequence)?;
    intent.region().within(sequence)?;

    if intent.region().length() == sequence.length() {
        let problem = InvalidMutationRequestProblem {
            request: "deletion".to_string(),
            reason: "a deletion may not remove the whole sequence".to_string(),
        };
        return Err(DomainError::InvalidMutationRequest(problem));
    }

    if let Some(cds) = scope.clone() {
        require_sequence(cds.reference().clone(), sequence)?;
        let label: String = cds.id().value().to_string();
        require_within_feature(
            intent.region().clone(),
            label,
            sequence.display_name(),
            cds.intervals().as_slice(),
        )?;
    }

    let start: u64 = intent.region().start();
    let end: u64 = intent.region().end();
    let letters: Vec<char> = sequence.letters();

    let mut removed: Vec<char> = Vec::new();
    let mut index: usize = start as usize;
    while index < end as usize {
        removed.push(letters[index]);
        index += 1;
    }

    let actual_deleted: String = canonical_text(removed.clone());
    if actual_deleted != intent.expected_deleted() {
        let problem = ExpectedNucleotideMismatchProblem {
            sequence: sequence.display_name(),
            position: start,
            expected: intent.expected_deleted().clone(),
            actual: actual_deleted.clone(),
        };
        return Err(DomainError::ExpectedNucleotideMismatch(problem));
    }

    let mut kept: Vec<char> = Vec::new();
    let count: usize = letters.len();
    let mut kept_index: usize = 0;
    while kept_index < count {
        let position: u64 = kept_index as u64;
        if position < start || position >= end {
            kept.push(letters[kept_index]);
        }
        kept_index += 1;
    }

    let edited: Sequence = normalize(
        result.clone(),
        canonical_text(kept.clone()),
        sequence.topology(),
    )?;

    Ok(MutationResult::Deletion(NucleotideDeletion {
        source: sequence.reference(),
        edited: edited.clone(),
        region: intent.region().clone(),
        expected_deleted: intent.expected_deleted().clone(),
        actual_deleted: actual_deleted.clone(),
        state: MutationState::Calculated,
    }))
}

/// Apply a DNA-level insertion of bases into the gap at a point.
///
/// The point is checked against the artifact and against its scope before
/// anything is written, and the bases on each side of it are asserted exactly,
/// so a request that disagrees with the artifact is refused rather than
/// guessed. A point at either end of the sequence has no base on one side of
/// it, which is what makes the insertion ambiguous there. Nothing is written
/// until every check has passed, and the source artifact is never modified:
/// the edited artifact is a new one, `inserted.len()` bases longer, that keeps
/// the source's topology.
pub fn apply_insertion(
    sequence: &Sequence,
    scope: Option<Cds>,
    result: Identifier,
    intent: NucleotideInsertionIntent,
) -> Result<MutationResult, DomainError> {
    require_sequence(intent.source().clone(), sequence)?;
    require_sequence(intent.point().reference().clone(), sequence)?;
    intent.point().within(sequence)?;

    let position: u64 = intent.point().position();
    if position == 0 || position == sequence.length() {
        let problem = InvalidMutationRequestProblem {
            request: "insertion".to_string(),
            reason: "an insertion needs a base on each side of the point, so the point may not sit at either end of the sequence".to_string(),
        };
        return Err(DomainError::InvalidMutationRequest(problem));
    }

    if let Some(cds) = scope.clone() {
        require_sequence(cds.reference().clone(), sequence)?;
        let label: String = cds.id().value().to_string();
        require_point_within_feature(
            intent.point().clone(),
            label,
            sequence.display_name(),
            cds.intervals().as_slice(),
        )?;
    }

    let letters: Vec<char> = sequence.letters();
    let before: char = letters[position as usize - 1];
    let after: char = letters[position as usize];

    if before != intent.expected_before() {
        let problem = ExpectedNucleotideMismatchProblem {
            sequence: sequence.display_name(),
            position: position - 1,
            expected: intent.expected_before().to_string(),
            actual: before.to_string(),
        };
        return Err(DomainError::ExpectedNucleotideMismatch(problem));
    }
    if after != intent.expected_after() {
        let problem = ExpectedNucleotideMismatchProblem {
            sequence: sequence.display_name(),
            position,
            expected: intent.expected_after().to_string(),
            actual: after.to_string(),
        };
        return Err(DomainError::ExpectedNucleotideMismatch(problem));
    }

    let inserted: Vec<char> = intent.inserted().clone().chars().collect();

    let mut kept: Vec<char> = Vec::new();
    let count: usize = letters.len();
    let mut index: usize = 0;
    while index < count {
        if index as u64 == position {
            kept.extend(inserted.clone());
        }
        kept.push(letters[index]);
        index += 1;
    }

    let edited: Sequence = normalize(
        result.clone(),
        canonical_text(kept.clone()),
        sequence.topology(),
    )?;

    Ok(MutationResult::Insertion(NucleotideInsertion {
        source: sequence.reference(),
        edited: edited.clone(),
        point: intent.point().clone(),
        expected_before: intent.expected_before(),
        expected_after: intent.expected_after(),
        actual_before: before,
        actual_after: after,
        inserted: intent.inserted().clone(),
        state: MutationState::Calculated,
    }))
}

/// Apply a DNA-level replacement of a contiguous region by a text of any
/// length.
///
/// The region is checked against the artifact and against its scope before
/// anything is written, and the bases it overwrites are asserted exactly, so
/// a request that disagrees with the artifact is refused rather than guessed.
/// The text written need not have the width of the region (D033), which is
/// what makes the edit a replacement rather than a substitution: the edited
/// artifact is a new one of `n − w + m` bases that keeps the source's topology,
/// and the source is never modified. A region that is the whole sequence is
/// refused; no source records a reason for that rule.
pub fn apply_replacement(
    sequence: &Sequence,
    scope: Option<Cds>,
    result: Identifier,
    intent: NucleotideReplacementIntent,
) -> Result<MutationResult, DomainError> {
    require_sequence(intent.source().clone(), sequence)?;
    require_sequence(intent.region().reference().clone(), sequence)?;
    intent.region().within(sequence)?;

    if intent.region().length() == sequence.length() {
        let problem = InvalidMutationRequestProblem {
            request: "replacement".to_string(),
            reason: "a replacement may not replace the whole sequence".to_string(),
        };
        return Err(DomainError::InvalidMutationRequest(problem));
    }

    if let Some(cds) = scope.clone() {
        require_sequence(cds.reference().clone(), sequence)?;
        let label: String = cds.id().value().to_string();
        require_within_feature(
            intent.region().clone(),
            label,
            sequence.display_name(),
            cds.intervals().as_slice(),
        )?;
    }

    let start: u64 = intent.region().start();
    let end: u64 = intent.region().end();
    let letters: Vec<char> = sequence.letters();

    let mut replaced: Vec<char> = Vec::new();
    let mut replaced_index: usize = start as usize;
    while replaced_index < end as usize {
        replaced.push(letters[replaced_index]);
        replaced_index += 1;
    }

    let actual_original: String = canonical_text(replaced.clone());
    if actual_original != intent.expected_original() {
        let problem = ExpectedNucleotideMismatchProblem {
            sequence: sequence.display_name(),
            position: start,
            expected: intent.expected_original().clone(),
            actual: actual_original.clone(),
        };
        return Err(DomainError::ExpectedNucleotideMismatch(problem));
    }

    let replacement_letters: Vec<char> = intent.replacement().clone().chars().collect();

    let mut kept: Vec<char> = Vec::new();
    let head: usize = start as usize;
    let mut index: usize = 0;
    while index < head {
        kept.push(letters[index]);
        index += 1;
    }
    kept.extend(replacement_letters.clone());
    let mut tail: usize = end as usize;
    while tail < letters.len() {
        kept.push(letters[tail]);
        tail += 1;
    }

    let edited: Sequence = normalize(
        result.clone(),
        canonical_text(kept.clone()),
        sequence.topology(),
    )?;

    Ok(MutationResult::Replacement(NucleotideReplacement {
        source: sequence.reference(),
        edited: edited.clone(),
        region: intent.region().clone(),
        expected_original: intent.expected_original().clone(),
        actual_original: actual_original.clone(),
        replacement: intent.replacement().clone(),
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

/// Independent verification of a mutation result (D013): the edit is not
/// re-run, and the result's claims are read back out of both artifacts. Every
/// kind first checks that the source is the artifact the result claims, by
/// name, digest and topology, and checks length for its kind (owner decision
/// 4.2 in `docs/tasks/milestone-4-indels.md` §4.2). A deletion, an insertion
/// and a replacement are re-derived by rebuilding the edited artifact from
/// the record; a substitution and a residue result are compared position by
/// position. What each kind then checks:
///
/// * substitution: unchanged length; the differing positions are exactly the
///   recorded coordinates; the nucleotide claims match both artifacts.
/// * residue: unchanged length; the differing positions are non-empty and lie
///   inside the recorded codon coordinates rather than filling them, since a
///   replacement codon may share letters with the one it replaces; the
///   annotation `verify` is handed equals the recorded one, compared as exact
///   `Cds` equality; the source's residue and codon at the requested residue;
///   the edited codon; and that no other residue changed.
/// * deletion: `n − w` bases; the edited artifact is the source with the
///   recorded region removed; the removed text matches the record.
/// * insertion: `n + k` bases; the edited artifact is the source with the
///   recorded bases at the recorded point; the source holds the recorded flanks.
/// * replacement: `n − w + m` bases; the edited artifact is the source with the
///   recorded region replaced by the recorded text; the source holds the
///   recorded original text there.
///
/// Only residue results are re-translated (`docs/q5-sdm.md` §8, open owner
/// question A5): a DNA-level edit inside a CDS is verified as a DNA edit, and
/// its effect on the translation is not checked. Verification does not re-derive
/// the residue-to-coordinate mapping: the generator owns it, and the residue
/// and codon claims check it instead of a second copy of the mapping.
pub fn verify(
    source: &Sequence,
    cds: Option<Cds>,
    result: MutationResult,
) -> Result<MutationResult, DomainError> {
    require_sequence(result.source().clone(), source)?;

    let edited: Sequence = result.edited();

    // A deletion changes the artifact's length, so it is re-derived here
    // rather than under the substitution checks below. The `cds` argument is
    // as irrelevant to it as it is to a nucleotide substitution.
    match result.clone() {
        MutationResult::Deletion(record) => {
            verify_deletion(source, &edited.clone(), record)?;
            return Ok(accepted(result.clone()));
        }
        // An insertion changes the artifact's length too, and it changes no
        // source coordinate, so it is re-derived here rather than under the
        // substitution checks below.
        MutationResult::Insertion(record) => {
            verify_insertion(source, &edited.clone(), record)?;
            return Ok(accepted(result.clone()));
        }
        // A replacement changes the artifact's length too, and it neither
        // removes a region nor writes into a gap, so it is re-derived here
        // rather than under the substitution checks below.
        MutationResult::Replacement(record) => {
            verify_replacement(source, &edited.clone(), record)?;
            return Ok(accepted(result.clone()));
        }
        MutationResult::Nucleotide(_record) => {}
        MutationResult::Residue(_record) => {}
    }

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
        MutationResult::Deletion(_record) => {
            unreachable!("a deletion is verified in its own branch above")
        }
        MutationResult::Insertion(_record) => {
            unreachable!("an insertion is verified in its own branch above")
        }
        MutationResult::Replacement(_record) => {
            unreachable!("a replacement is verified in its own branch above")
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

/// The deletion claims of a result, re-derived from both artifacts (D013).
///
/// The edit is not re-run: the source's own bases at the recorded region are
/// read back out, and the edited artifact has to be the source with exactly
/// that region removed and exactly `region.length()` bases fewer.
fn verify_deletion(
    source: &Sequence,
    edited: &Sequence,
    record: NucleotideDeletion,
) -> Result<(), DomainError> {
    let region: NucleotideInterval = record.region();
    let start: u64 = region.start();
    let end: u64 = region.end();
    let width: u64 = region.length();

    if edited.length() + width != source.length() {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "a deletion of ".to_string()
                + &width.to_string()
                + " bases shortens the artifact by exactly "
                + &width.to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    let source_letters: Vec<char> = source.letters();
    let edited_letters: Vec<char> = edited.letters();
    let mut kept: Vec<char> = Vec::new();
    let mut removed: Vec<char> = Vec::new();
    let count: usize = source_letters.len();
    let mut index: usize = 0;
    while index < count {
        let position: u64 = index as u64;
        if position < start || position >= end {
            kept.push(source_letters[index]);
        } else {
            removed.push(source_letters[index]);
        }
        index += 1;
    }

    if kept != edited_letters {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the edited artifact is not the source with ".to_string()
                + &start.to_string()
                + ".."
                + &end.to_string()
                + " removed",
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    let removed_text: String = canonical_text(removed.clone());
    if removed_text != record.expected_deleted() || removed_text != record.actual_deleted() {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the deletion claims do not match the source".to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    Ok(())
}

/// The insertion claims of a result, re-derived from both artifacts (D013).
///
/// The edit is not re-run: the edited artifact has to be the source with the
/// recorded bases sitting at the recorded point and exactly `inserted.len()`
/// bases more, and the source has to hold the flanking bases the record
/// claims on each side of the point. A point at either end of the sequence
/// has no base on one side, so its claims cannot be read back and fail here.
fn verify_insertion(
    source: &Sequence,
    edited: &Sequence,
    record: NucleotideInsertion,
) -> Result<(), DomainError> {
    let position: u64 = record.point().position();
    let inserted: Vec<char> = record.inserted().clone().chars().collect();
    let width: u64 = inserted.len() as u64;

    if edited.length() != source.length() + width {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "an insertion of ".to_string()
                + &width.to_string()
                + " bases lengthens the artifact by exactly "
                + &width.to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    let source_letters: Vec<char> = source.letters();
    let edited_letters: Vec<char> = edited.letters();

    let mut placed: Vec<char> = Vec::new();
    let count: usize = source_letters.len();
    let mut index: usize = 0;
    while index < count {
        let coordinate: u64 = index as u64;
        if coordinate == position {
            placed.extend(inserted.clone());
        }
        placed.push(source_letters[index]);
        index += 1;
    }
    if position >= source.length() {
        placed.extend(inserted.clone());
    }

    if placed != edited_letters {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the edited artifact is not the source with ".to_string()
                + &canonical_text(inserted.clone())
                + " inserted at "
                + &position.to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    let before_claim: bool = position >= 1
        && position <= source.length()
        && source.letter_at(position - 1).unwrap() == record.expected_before()
        && source.letter_at(position - 1).unwrap() == record.actual_before();
    let after_claim: bool = position < source.length()
        && source.letter_at(position).unwrap() == record.expected_after()
        && source.letter_at(position).unwrap() == record.actual_after();
    if !before_claim || !after_claim {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the insertion claims do not match the source".to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    Ok(())
}

/// The replacement claims of a result, re-derived from both artifacts (D013).
///
/// The edit is not re-run: the source's own bases at the recorded region are
/// read back out, and the edited artifact has to be the source with exactly
/// that region replaced by exactly the recorded text and holding exactly
/// `source.length() − region.length() + replacement.length()` bases.
fn verify_replacement(
    source: &Sequence,
    edited: &Sequence,
    record: NucleotideReplacement,
) -> Result<(), DomainError> {
    let region: NucleotideInterval = record.region();
    let start: u64 = region.start();
    let end: u64 = region.end();
    let width: u64 = region.length();
    let replacement: Vec<char> = record.replacement().clone().chars().collect();
    let written: u64 = replacement.len() as u64;

    // `source.length() - width + written` would underflow for a region wider
    // than the artifact, so the length rule is stated on both sides.
    if edited.length() + width != source.length() + written {
        let expected_length: u64 = source.length().saturating_sub(width) + written;
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "replacing ".to_string()
                + &width.to_string()
                + " bases with "
                + &written.to_string()
                + " gives "
                + &expected_length.to_string()
                + " bases; the artifact holds "
                + &edited.length().to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    let source_letters: Vec<char> = source.letters();
    let edited_letters: Vec<char> = edited.letters();

    let mut placed: Vec<char> = Vec::new();
    let count: usize = source_letters.len();
    let mut index: usize = 0;
    while index < count {
        let position: u64 = index as u64;
        if position == start {
            placed.extend(replacement.clone());
        }
        if position < start || position >= end {
            placed.push(source_letters[index]);
        }
        index += 1;
    }
    if start >= source.length() {
        placed.extend(replacement.clone());
    }

    if placed != edited_letters {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the edited artifact is not the source with ".to_string()
                + &start.to_string()
                + ".."
                + &end.to_string()
                + " replaced by "
                + &canonical_text(replacement.clone()),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    let mut claimed: Vec<char> = Vec::new();
    let mut claim_index: usize = 0;
    while claim_index < count {
        let position: u64 = claim_index as u64;
        if position >= start && position < end {
            claimed.push(source_letters[claim_index]);
        }
        claim_index += 1;
    }

    let claimed_text: String = canonical_text(claimed.clone());
    if claimed_text != record.expected_original() || claimed_text != record.actual_original() {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the replacement claims do not match the source".to_string(),
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

    // The result records the whole annotation it was applied to, so a
    // different CDS that shares that name is refused too: its intervals, its
    // strand and its genetic code are what the residue and codon claims are
    // read over.
    if annotation != record.annotation() {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the CDS verified against is not the annotation the result was applied to"
                .to_string(),
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
    if before_residues[offset] != record.actual_original() {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the source does not hold the recorded actual original residue".to_string(),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    let label: String = annotation.id().value().to_string();
    let source_cds_text: String = extract_cds(source, &annotation.clone())?;
    let source_codon: String = codon_slice(source_cds_text.clone(), number, label.clone())?;
    if source_codon != record.original_codon() {
        let problem = MutationVerificationFailureProblem {
            result: record.edited.reference().describe(),
            reason: "the codon read from the source CDS is not the recorded original codon"
                .to_string(),
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
        MutationResult::Deletion(record) => {
            let mut copy: NucleotideDeletion = record.clone();
            copy.state = MutationState::Verified;
            MutationResult::Deletion(copy)
        }
        MutationResult::Insertion(record) => {
            let mut copy: NucleotideInsertion = record.clone();
            copy.state = MutationState::Verified;
            MutationResult::Insertion(copy)
        }
        MutationResult::Replacement(record) => {
            let mut copy: NucleotideReplacement = record.clone();
            copy.state = MutationState::Verified;
            MutationResult::Replacement(copy)
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
fn codon_positions(
    intervals: OrderedIntervals,
    strand: Strand,
    number: u64,
    total_length: u64,
) -> Result<Vec<u64>, DomainError> {
    let mut positions: Vec<u64> = Vec::new();
    let mut offset: usize = 0;
    let plus: bool = strand.clone().is_plus();
    while offset < 3 {
        // Residues are numbered from 1, so `number - 1` underflows at residue 0 and
        // the offset can overflow the coordinate space; a minus strand reads the
        // codon back from the end of the CDS, so `total_length - 1 - read` underflows
        // for a residue past that end. A residue whose coordinates do not fit is
        // refused the same way as one outside the CDS.
        let read: Option<u64> = number
            .checked_sub(1)
            .and_then(|index| index.checked_mul(3))
            .and_then(|base| base.checked_add(offset as u64));
        let biological: Option<u64> = if plus {
            read
        } else {
            read.and_then(|read_offset| {
                total_length
                    .checked_sub(1)
                    .and_then(|last| last.checked_sub(read_offset))
            })
        };
        let found: Option<u64> =
            biological.and_then(|value| reference_of(intervals.as_slice(), value));
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
    use crate::interval::create_point;
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

    fn deletion_of(start: i64, end: i64, expected: String) -> MutationResult {
        apply_deletion(
            &seq_a().clone(),
            None,
            result_name(),
            NucleotideDeletionIntent::create(seq_a().reference(), at(start, end), expected)
                .unwrap(),
        )
        .unwrap()
    }

    fn deletion_record(result: MutationResult) -> NucleotideDeletion {
        match result {
            MutationResult::Deletion(record) => record,
            MutationResult::Nucleotide(_record) => {
                panic!("a deletion request produced a nucleotide result")
            }
            MutationResult::Residue(_record) => {
                panic!("a deletion request produced a residue result")
            }
            MutationResult::Insertion(_record) => {
                panic!("a deletion request produced an insertion result")
            }
            MutationResult::Replacement(_record) => {
                panic!("a deletion request produced a replacement result")
            }
        }
    }

    fn tampered_deletion(
        result: MutationResult,
        text: String,
        expected: String,
        actual: String,
    ) -> MutationResult {
        match result.clone() {
            MutationResult::Deletion(record) => {
                let mut copy: NucleotideDeletion = record.clone();
                copy.edited =
                    normalize(result_name(), text.clone(), SequenceTopology::Linear).unwrap();
                copy.expected_deleted = expected.clone();
                copy.actual_deleted = actual.clone();
                MutationResult::Deletion(copy)
            }
            MutationResult::Nucleotide(_record) => result.clone(),
            MutationResult::Residue(_record) => result.clone(),
            MutationResult::Insertion(_record) => result.clone(),
            MutationResult::Replacement(_record) => result.clone(),
        }
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
            MutationResult::Deletion(_record) => calculated.clone(),
            MutationResult::Insertion(_record) => calculated.clone(),
            MutationResult::Replacement(_record) => calculated.clone(),
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

    fn residue_record(result: MutationResult) -> ResidueMutation {
        match result.clone() {
            MutationResult::Residue(record) => record,
            MutationResult::Nucleotide(_record) => {
                panic!("a residue request produced a nucleotide result")
            }
            MutationResult::Deletion(_record) => {
                panic!("a residue request produced a deletion result")
            }
            MutationResult::Insertion(_record) => {
                panic!("a residue request produced an insertion result")
            }
            MutationResult::Replacement(_record) => {
                panic!("a residue request produced a replacement result")
            }
        }
    }

    fn tampered_original_codon(result: MutationResult, codon: String) -> MutationResult {
        let mut copy: ResidueMutation = residue_record(result.clone());
        copy.original_codon = codon.clone();
        MutationResult::Residue(copy)
    }

    fn tampered_actual_original(result: MutationResult, residue: Residue) -> MutationResult {
        let mut copy: ResidueMutation = residue_record(result.clone());
        copy.actual_original = residue.clone();
        MutationResult::Residue(copy)
    }

    #[test]
    fn a_residue_result_is_not_verified_against_a_same_named_annotation_with_other_coordinates() {
        // `cds_of(18)` selects ATG GCT TGG AAT, which translates to M A W N,
        // and `cds_of(21)` selects the same codons plus CAG, which translates
        // to M A W N Q: one identifier, two annotations, both translatable
        // over `seq-a`. The result was applied to the 18-base annotation, so
        // the 21-base one is not the annotation its claims are about.
        let intent: ResidueMutationIntent = ResidueMutationIntent::create(
            residue(2),
            "A".to_string(),
            "W".to_string(),
            CodonSelectionPolicy::Default,
        )
        .unwrap();
        let calculated: MutationResult =
            apply_residue(&seq_a().clone(), cds_of(18), result_name(), intent).unwrap();

        let error: DomainError =
            verify(&seq_a().clone(), Some(cds_of(21)), calculated.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
        match error {
            DomainError::MutationVerificationFailure(problem) => {
                assert!(
                    problem.reason.contains("not the annotation"),
                    "{}",
                    problem.reason
                );
            }
            other => panic!("expected MutationVerificationFailure, got {:?}", other),
        }
    }

    #[test]
    fn a_residue_result_with_a_tampered_original_codon_is_refused() {
        // The source holds GCT at residue 2; a record claiming another codon
        // there does not describe the artifact it was computed from.
        let intent: ResidueMutationIntent = ResidueMutationIntent::create(
            residue(2),
            "A".to_string(),
            "W".to_string(),
            CodonSelectionPolicy::Default,
        )
        .unwrap();
        let calculated: MutationResult =
            apply_residue(&seq_a().clone(), cds_of(30), result_name(), intent).unwrap();

        let tampered: MutationResult =
            tampered_original_codon(calculated.clone(), "GCA".to_string());
        let error: DomainError =
            verify(&seq_a().clone(), Some(cds_of(30)), tampered.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
        match error {
            DomainError::MutationVerificationFailure(problem) => {
                assert!(
                    problem.reason.contains("the source CDS"),
                    "{}",
                    problem.reason
                );
            }
            other => panic!("expected MutationVerificationFailure, got {:?}", other),
        }
    }

    #[test]
    fn a_residue_result_with_a_tampered_actual_original_is_refused() {
        // The source holds A at residue 2; a record claiming another residue
        // there does not describe the artifact it was computed from.
        let intent: ResidueMutationIntent = ResidueMutationIntent::create(
            residue(2),
            "A".to_string(),
            "W".to_string(),
            CodonSelectionPolicy::Default,
        )
        .unwrap();
        let calculated: MutationResult =
            apply_residue(&seq_a().clone(), cds_of(30), result_name(), intent).unwrap();

        let tampered: MutationResult = tampered_actual_original(calculated.clone(), Residue::M);
        let error: DomainError =
            verify(&seq_a().clone(), Some(cds_of(30)), tampered.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
        match error {
            DomainError::MutationVerificationFailure(problem) => {
                assert!(
                    problem.reason.contains("actual original residue"),
                    "{}",
                    problem.reason
                );
            }
            other => panic!("expected MutationVerificationFailure, got {:?}", other),
        }
    }

    #[test]
    fn a_residue_result_records_the_annotation_it_was_applied_to() {
        let annotation: Cds = cds_of(30);
        let intent: ResidueMutationIntent = ResidueMutationIntent::create(
            residue(2),
            "A".to_string(),
            "W".to_string(),
            CodonSelectionPolicy::Default,
        )
        .unwrap();
        let calculated: MutationResult =
            apply_residue(&seq_a().clone(), annotation.clone(), result_name(), intent).unwrap();

        let record: ResidueMutation = residue_record(calculated.clone());
        assert!(record.annotation() == annotation);
        assert!(record.cds() == annotation.id());
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

    // ---- deletions ---------------------------------------------------------

    #[test]
    fn a_deletion_cannot_target_another_sequence() {
        let other: Sequence = normalize(
            Identifier::create("seq-b".to_string()).unwrap(),
            "ATTCCAAGCCATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap();
        let refusal: Result<NucleotideDeletionIntent, DomainError> =
            NucleotideDeletionIntent::create(
                seq_a().reference(),
                create(&other.reference(), 3, 6).unwrap(),
                "GCT".to_string(),
            );
        assert!(matches!(
            refusal.unwrap_err(),
            DomainError::InvalidMutationRequest(_)
        ));
    }

    #[test]
    fn a_deletion_that_asserts_the_wrong_number_of_bases_is_refused() {
        // A length that disagrees with the region names both numbers.
        let mismatch: DomainError =
            NucleotideDeletionIntent::create(seq_a().reference(), at(3, 6), "GC".to_string())
                .unwrap_err();
        assert!(matches!(mismatch, DomainError::InvalidMutationRequest(_)));
        match mismatch {
            DomainError::InvalidMutationRequest(problem) => {
                assert!(problem.reason.contains("2"), "{}", problem.reason);
                assert!(problem.reason.contains("3"), "{}", problem.reason);
            }
            other => panic!("expected InvalidMutationRequest, got {:?}", other),
        }
    }

    #[test]
    fn a_deletion_asserts_only_the_bases_it_removes() {
        // An ambiguity code is a sequence letter, not a base that can be
        // asserted as removed.
        let ambiguity: DomainError =
            NucleotideDeletionIntent::create(seq_a().reference(), at(3, 6), "GCN".to_string())
                .unwrap_err();
        assert!(matches!(ambiguity, DomainError::InvalidMutationRequest(_)));
        match ambiguity {
            DomainError::InvalidMutationRequest(problem) => {
                assert!(problem.reason.contains("A, C, G, T"), "{}", problem.reason);
            }
            other => panic!("expected InvalidMutationRequest, got {:?}", other),
        }
    }

    #[test]
    fn a_deletion_removes_the_region_and_keeps_the_source() {
        let source: Sequence = seq_a();
        let intent: NucleotideDeletionIntent = NucleotideDeletionIntent::create(
            source.reference(),
            create(&source.reference(), 3, 6).unwrap(),
            "GCT".to_string(),
        )
        .unwrap();
        let made: MutationResult = apply_deletion(&source, None, result_name(), intent).unwrap();
        let record: NucleotideDeletion = deletion_record(made.clone());

        assert!(made.edited().canonical_text() == "ATGTGGAATCAGCAGCAGCAGCAGCAG");
        assert!(made.edited().length() == 27);
        assert!(made.edited().topology() == SequenceTopology::Linear);
        assert!(made.state() == MutationState::Calculated);
        assert!(made.target_coordinates() == vec![3, 4, 5]);
        assert!(made.is_deletion());
        assert!(!made.is_nucleotide());
        assert!(record.actual_deleted() == "GCT");

        // The source keeps its content, its length and its digest.
        assert!(source.length() == 30);
        assert!(source.canonical_text() == "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG");
        assert!(made.source() == source.reference());
    }

    #[test]
    fn a_deletion_at_either_end_of_the_sequence_is_applied() {
        let head: MutationResult = deletion_of(0, 3, "ATG".to_string());
        assert!(head.edited().canonical_text() == "GCTTGGAATCAGCAGCAGCAGCAGCAG");
        let tail: MutationResult = deletion_of(27, 30, "CAG".to_string());
        assert!(tail.edited().canonical_text() == "ATGGCTTGGAATCAGCAGCAGCAGCAG");
    }

    #[test]
    fn a_deletion_from_a_circular_sequence_stays_circular() {
        let circular: Sequence = normalize(
            Identifier::create("seq-a".to_string()).unwrap(),
            seq_a().canonical_text(),
            SequenceTopology::Circular,
        )
        .unwrap();
        let intent: NucleotideDeletionIntent = NucleotideDeletionIntent::create(
            circular.reference(),
            create(&circular.reference(), 3, 6).unwrap(),
            "GCT".to_string(),
        )
        .unwrap();
        let made: MutationResult = apply_deletion(&circular, None, result_name(), intent).unwrap();
        assert!(made.edited().topology() == SequenceTopology::Circular);
        assert!(made.edited().canonical_text() == "ATGTGGAATCAGCAGCAGCAGCAGCAG");
    }

    #[test]
    fn a_deletion_that_asserts_the_wrong_bases_names_the_bases_that_are_there() {
        let error: DomainError = apply_deletion(
            &seq_a().clone(),
            None,
            result_name(),
            NucleotideDeletionIntent::create(seq_a().reference(), at(3, 6), "GCA".to_string())
                .unwrap(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::ExpectedNucleotideMismatch(_)));
        match error {
            DomainError::ExpectedNucleotideMismatch(problem) => {
                assert!(problem.expected == "GCA");
                assert!(problem.actual == "GCT");
            }
            other => panic!("expected ExpectedNucleotideMismatch, got {:?}", other),
        }
    }

    #[test]
    fn a_deletion_may_not_remove_the_whole_sequence() {
        let error: DomainError = apply_deletion(
            &seq_a().clone(),
            None,
            result_name(),
            NucleotideDeletionIntent::create(
                seq_a().reference(),
                at(0, 30),
                seq_a().canonical_text(),
            )
            .unwrap(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
    }

    #[test]
    fn a_deletion_inside_its_scoped_cds_is_applied() {
        let made: MutationResult = apply_deletion(
            &seq_a().clone(),
            Some(cds_of(12).clone()),
            result_name(),
            NucleotideDeletionIntent::create(seq_a().reference(), at(3, 6), "GCT".to_string())
                .unwrap(),
        )
        .unwrap();
        assert!(made.state() == MutationState::Calculated);
        assert!(made.edited().canonical_text() == "ATGTGGAATCAGCAGCAGCAGCAGCAG");
    }

    #[test]
    fn a_deletion_that_straddles_the_scoped_cds_is_refused() {
        let error: DomainError = apply_deletion(
            &seq_a().clone(),
            Some(cds_of(12).clone()),
            result_name(),
            NucleotideDeletionIntent::create(seq_a().reference(), at(9, 15), "AATCAG".to_string())
                .unwrap(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::TargetOutsideFeature(_)));
    }

    #[test]
    fn a_deletion_result_is_verified_against_its_source() {
        let calculated: MutationResult = deletion_of(3, 6, "GCT".to_string());
        let verified: MutationResult = verify(&seq_a().clone(), None, calculated.clone()).unwrap();
        assert!(verified.state() == MutationState::Verified);
        assert!(verified != calculated);
        assert!(verified.edited().canonical_text() == "ATGTGGAATCAGCAGCAGCAGCAGCAG");
        assert!(verified.target_coordinates() == vec![3, 4, 5]);
        assert!(verified.is_deletion());
        let record: NucleotideDeletion = deletion_record(verified.clone());
        assert!(record.region() == at(3, 6));
        assert!(record.expected_deleted() == "GCT");
        assert!(record.actual_deleted() == "GCT");
    }

    #[test]
    fn verify_refuses_a_deletion_that_shortens_the_artifact_by_the_wrong_amount() {
        let calculated: MutationResult = deletion_of(3, 6, "GCT".to_string());
        // The same record, whose artifact lost only two bases.
        let tampered: MutationResult = tampered_deletion(
            calculated.clone(),
            "ATGTTGGAATCAGCAGCAGCAGCAGCAG".to_string(),
            "GCT".to_string(),
            "GCT".to_string(),
        );
        let error: DomainError = verify(&seq_a().clone(), None, tampered.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
        match error {
            DomainError::MutationVerificationFailure(problem) => {
                assert!(
                    problem
                        .reason
                        .contains("shortens the artifact by exactly 3"),
                    "{}",
                    problem.reason,
                );
            }
            other => panic!("expected MutationVerificationFailure, got {:?}", other),
        }
    }

    #[test]
    fn verify_refuses_an_edited_artifact_that_is_not_the_source_minus_the_region() {
        let calculated: MutationResult = deletion_of(3, 6, "GCT".to_string());
        // The right length, but the wrong bases were removed.
        let tampered: MutationResult = tampered_deletion(
            calculated.clone(),
            "ATGGGGAATCAGCAGCAGCAGCAGCAG".to_string(),
            "GCT".to_string(),
            "GCT".to_string(),
        );
        let error: DomainError = verify(&seq_a().clone(), None, tampered.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
    }

    #[test]
    fn verify_refuses_deletion_claims_that_disagree_with_the_source() {
        let calculated: MutationResult = deletion_of(3, 6, "GCT".to_string());
        // The artifacts agree, but the record claims other bases were removed.
        let tampered: MutationResult = tampered_deletion(
            calculated.clone(),
            calculated.edited().canonical_text(),
            "GCA".to_string(),
            "GCA".to_string(),
        );
        let error: DomainError = verify(&seq_a().clone(), None, tampered.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
    }

    // ---- insertions --------------------------------------------------------

    fn ins(position: i64, before: &str, after: &str, inserted: &str) -> NucleotideInsertionIntent {
        NucleotideInsertionIntent::create(
            seq_a().reference(),
            create_point(&seq_a().reference(), position).unwrap(),
            before.to_string(),
            after.to_string(),
            inserted.to_string(),
        )
        .unwrap()
    }

    fn insertion_record(result: MutationResult) -> NucleotideInsertion {
        match result {
            MutationResult::Insertion(record) => record,
            MutationResult::Nucleotide(_record) => {
                panic!("an insertion request produced a nucleotide result")
            }
            MutationResult::Residue(_record) => {
                panic!("an insertion request produced a residue result")
            }
            MutationResult::Deletion(_record) => {
                panic!("an insertion request produced a deletion result")
            }
            MutationResult::Replacement(_record) => {
                panic!("an insertion request produced a replacement result")
            }
        }
    }

    // The same record, claiming another edited artifact.
    fn tampered_insertion(result: MutationResult, text: String) -> MutationResult {
        match result.clone() {
            MutationResult::Insertion(record) => {
                let mut copy: NucleotideInsertion = record.clone();
                copy.edited =
                    normalize(result_name(), text.clone(), SequenceTopology::Linear).unwrap();
                MutationResult::Insertion(copy)
            }
            MutationResult::Nucleotide(_record) => result.clone(),
            MutationResult::Residue(_record) => result.clone(),
            MutationResult::Deletion(_record) => result.clone(),
            MutationResult::Replacement(_record) => result.clone(),
        }
    }

    // The same artifacts, claiming another base before the point.
    fn tampered_insertion_claims(result: MutationResult, claimed: char) -> MutationResult {
        match result.clone() {
            MutationResult::Insertion(record) => {
                let mut copy: NucleotideInsertion = record.clone();
                copy.expected_before = claimed;
                copy.actual_before = claimed;
                MutationResult::Insertion(copy)
            }
            MutationResult::Nucleotide(_record) => result.clone(),
            MutationResult::Residue(_record) => result.clone(),
            MutationResult::Deletion(_record) => result.clone(),
            MutationResult::Replacement(_record) => result.clone(),
        }
    }

    #[test]
    fn an_insertion_cannot_target_another_sequence() {
        let other: Sequence = normalize(
            Identifier::create("seq-b".to_string()).unwrap(),
            "ATTCCAAGCCATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap();
        let refusal: Result<NucleotideInsertionIntent, DomainError> =
            NucleotideInsertionIntent::create(
                seq_a().reference(),
                create_point(&other.reference(), 9).unwrap(),
                "G".to_string(),
                "A".to_string(),
                "GGG".to_string(),
            );
        assert!(matches!(
            refusal.unwrap_err(),
            DomainError::InvalidMutationRequest(_)
        ));
    }

    #[test]
    fn an_insertion_of_no_bases_is_refused() {
        let error: DomainError = NucleotideInsertionIntent::create(
            seq_a().reference(),
            create_point(&seq_a().reference(), 9).unwrap(),
            "G".to_string(),
            "A".to_string(),
            "".to_string(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
        match error {
            DomainError::InvalidMutationRequest(problem) => {
                assert!(
                    problem.reason.contains("at least one"),
                    "{}",
                    problem.reason
                );
            }
            other => panic!("expected InvalidMutationRequest, got {:?}", other),
        }
    }

    #[test]
    fn an_insertion_inserts_only_plain_bases() {
        let error: DomainError = NucleotideInsertionIntent::create(
            seq_a().reference(),
            create_point(&seq_a().reference(), 9).unwrap(),
            "G".to_string(),
            "A".to_string(),
            "GNG".to_string(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
        match error {
            DomainError::InvalidMutationRequest(problem) => {
                assert!(problem.reason.contains("A, C, G, T"), "{}", problem.reason);
            }
            other => panic!("expected InvalidMutationRequest, got {:?}", other),
        }
    }

    #[test]
    fn an_insertion_asserts_only_plain_bases_beside_the_point() {
        let error: DomainError = NucleotideInsertionIntent::create(
            seq_a().reference(),
            create_point(&seq_a().reference(), 9).unwrap(),
            "N".to_string(),
            "A".to_string(),
            "GGG".to_string(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
    }

    #[test]
    fn an_insertion_puts_its_bases_at_the_point_and_keeps_the_source() {
        let source: Sequence = seq_a();
        let made: MutationResult =
            apply_insertion(&source, None, result_name(), ins(9, "G", "A", "GGG")).unwrap();
        let record: NucleotideInsertion = insertion_record(made.clone());

        assert!(made.edited().canonical_text() == "ATGGCTTGGGGGAATCAGCAGCAGCAGCAGCAG");
        assert!(made.edited().length() == 33);
        assert!(made.edited().topology() == SequenceTopology::Linear);
        assert!(made.state() == MutationState::Calculated);
        assert!(made.is_insertion());
        assert!(!made.is_deletion());
        assert!(!made.is_nucleotide());
        assert!(made.target_coordinates().is_empty());
        assert!(record.inserted() == "GGG");
        assert!(record.actual_before() == 'G');
        assert!(record.actual_after() == 'A');

        // The source keeps its content, its length and its digest.
        assert!(source.length() == 30);
        assert!(source.canonical_text() == "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG");
        assert!(made.source() == source.reference());
    }

    #[test]
    fn an_insertion_near_either_end_of_the_sequence_is_applied() {
        let head: MutationResult =
            apply_insertion(&seq_a().clone(), None, result_name(), ins(1, "A", "T", "C")).unwrap();
        assert!(head.edited().canonical_text() == "ACTGGCTTGGAATCAGCAGCAGCAGCAGCAG");
        let tail: MutationResult = apply_insertion(
            &seq_a().clone(),
            None,
            result_name(),
            ins(29, "A", "G", "T"),
        )
        .unwrap();
        assert!(tail.edited().canonical_text() == "ATGGCTTGGAATCAGCAGCAGCAGCAGCATG");
    }

    #[test]
    fn an_insertion_at_either_end_of_the_sequence_is_refused() {
        for pair in [vec![0], vec![30]] {
            let error: DomainError = apply_insertion(
                &seq_a().clone(),
                None,
                result_name(),
                ins(pair[0], "A", "T", "C"),
            )
            .unwrap_err();
            assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
            match error {
                DomainError::InvalidMutationRequest(problem) => {
                    assert!(
                        problem.reason.contains("a base on each side"),
                        "{}",
                        problem.reason,
                    );
                }
                other => panic!("expected InvalidMutationRequest, got {:?}", other),
            }
        }
    }

    #[test]
    fn a_point_past_the_end_of_the_sequence_is_out_of_bounds() {
        let error: DomainError = apply_insertion(
            &seq_a().clone(),
            None,
            result_name(),
            ins(31, "A", "T", "C"),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::CoordinateOutOfBounds(_)));
    }

    #[test]
    fn a_wrong_flank_names_the_position_beside_the_point() {
        let error: DomainError = apply_insertion(
            &seq_a().clone(),
            None,
            result_name(),
            ins(9, "C", "A", "GGG"),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::ExpectedNucleotideMismatch(_)));
        match error {
            DomainError::ExpectedNucleotideMismatch(problem) => {
                assert!(problem.position == 8);
                assert!(problem.expected == "C");
                assert!(problem.actual == "G");
            }
            other => panic!("expected ExpectedNucleotideMismatch, got {:?}", other),
        }
    }

    #[test]
    fn an_insertion_from_a_circular_sequence_stays_circular() {
        let circular: Sequence = normalize(
            Identifier::create("seq-a".to_string()).unwrap(),
            seq_a().canonical_text(),
            SequenceTopology::Circular,
        )
        .unwrap();
        let intent: NucleotideInsertionIntent = NucleotideInsertionIntent::create(
            circular.reference(),
            create_point(&circular.reference(), 9).unwrap(),
            "G".to_string(),
            "A".to_string(),
            "GGG".to_string(),
        )
        .unwrap();
        let made: MutationResult = apply_insertion(&circular, None, result_name(), intent).unwrap();
        assert!(made.edited().topology() == SequenceTopology::Circular);
        assert!(made.edited().canonical_text() == "ATGGCTTGGGGGAATCAGCAGCAGCAGCAGCAG");
    }

    #[test]
    fn an_insertion_inside_its_scoped_cds_is_applied() {
        let made: MutationResult = apply_insertion(
            &seq_a().clone(),
            Some(cds_of(12).clone()),
            result_name(),
            ins(9, "G", "A", "GGG"),
        )
        .unwrap();
        assert!(made.state() == MutationState::Calculated);
        assert!(made.edited().canonical_text() == "ATGGCTTGGGGGAATCAGCAGCAGCAGCAGCAG");
    }

    #[test]
    fn an_insertion_at_the_scoped_cds_edge_is_refused() {
        let error: DomainError = apply_insertion(
            &seq_a().clone(),
            Some(cds_of(12).clone()),
            result_name(),
            ins(12, "T", "C", "GGG"),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::TargetOutsideFeature(_)));
    }

    #[test]
    fn an_insertion_result_is_verified_against_its_source() {
        let calculated: MutationResult = apply_insertion(
            &seq_a().clone(),
            None,
            result_name(),
            ins(9, "G", "A", "GGG"),
        )
        .unwrap();
        let verified: MutationResult = verify(&seq_a().clone(), None, calculated.clone()).unwrap();
        assert!(verified.state() == MutationState::Verified);
        assert!(verified != calculated);
        assert!(verified.edited().canonical_text() == "ATGGCTTGGGGGAATCAGCAGCAGCAGCAGCAG");
        assert!(verified.target_coordinates().is_empty());
        assert!(verified.is_insertion());
        let record: NucleotideInsertion = insertion_record(verified.clone());
        assert!(record.point() == create_point(&seq_a().reference(), 9).unwrap());
        assert!(record.inserted() == "GGG");
        assert!(record.expected_before() == 'G');
        assert!(record.actual_before() == 'G');
        assert!(record.expected_after() == 'A');
        assert!(record.actual_after() == 'A');
    }

    #[test]
    fn verify_refuses_an_insertion_that_lengthens_the_artifact_by_the_wrong_amount() {
        let calculated: MutationResult = apply_insertion(
            &seq_a().clone(),
            None,
            result_name(),
            ins(9, "G", "A", "GGG"),
        )
        .unwrap();
        // The same record, whose artifact gained only two bases.
        let tampered: MutationResult = tampered_insertion(
            calculated.clone(),
            "ATGGCTTGGGGGAATCAGCAGCAGCAGCAGCA".to_string(),
        );
        let error: DomainError = verify(&seq_a().clone(), None, tampered.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
        match error {
            DomainError::MutationVerificationFailure(problem) => {
                assert!(
                    problem
                        .reason
                        .contains("lengthens the artifact by exactly 3"),
                    "{}",
                    problem.reason,
                );
            }
            other => panic!("expected MutationVerificationFailure, got {:?}", other),
        }
    }

    #[test]
    fn verify_refuses_an_insertion_at_another_place() {
        let calculated: MutationResult = apply_insertion(
            &seq_a().clone(),
            None,
            result_name(),
            ins(9, "G", "A", "GGG"),
        )
        .unwrap();
        // The right length and the right bases, but inserted one gap later.
        let tampered: MutationResult = tampered_insertion(
            calculated.clone(),
            "ATGGCTTGGAGGGATCAGCAGCAGCAGCAGCAG".to_string(),
        );
        let error: DomainError = verify(&seq_a().clone(), None, tampered.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
    }

    #[test]
    fn verify_refuses_insertion_claims_that_disagree_with_the_source() {
        let calculated: MutationResult = apply_insertion(
            &seq_a().clone(),
            None,
            result_name(),
            ins(9, "G", "A", "GGG"),
        )
        .unwrap();
        // The artifacts agree, but the record claims another base sits before
        // the point.
        let tampered: MutationResult = tampered_insertion_claims(calculated.clone(), 'C');
        let error: DomainError = verify(&seq_a().clone(), None, tampered.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
        match error {
            DomainError::MutationVerificationFailure(problem) => {
                assert!(
                    problem.reason.contains("insertion claims"),
                    "{}",
                    problem.reason
                );
            }
            other => panic!("expected MutationVerificationFailure, got {:?}", other),
        }
    }

    // ---- replacements ----------------------------------------------------

    fn replacement_of(
        start: i64,
        end: i64,
        expected: String,
        replacement: String,
    ) -> MutationResult {
        apply_replacement(
            &seq_a().clone(),
            None,
            result_name(),
            NucleotideReplacementIntent::create(
                seq_a().reference(),
                at(start, end),
                expected.clone(),
                replacement.clone(),
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn replacement_record(result: MutationResult) -> NucleotideReplacement {
        match result {
            MutationResult::Replacement(record) => record,
            MutationResult::Nucleotide(_record) => {
                panic!("a replacement request produced a nucleotide result")
            }
            MutationResult::Residue(_record) => {
                panic!("a replacement request produced a residue result")
            }
            MutationResult::Deletion(_record) => {
                panic!("a replacement request produced a deletion result")
            }
            MutationResult::Insertion(_record) => {
                panic!("a replacement request produced an insertion result")
            }
        }
    }

    // The same record, claiming another edited artifact.
    fn tampered_replacement(result: MutationResult, text: String) -> MutationResult {
        match result.clone() {
            MutationResult::Replacement(record) => {
                let mut copy: NucleotideReplacement = record.clone();
                copy.edited =
                    normalize(result_name(), text.clone(), SequenceTopology::Linear).unwrap();
                MutationResult::Replacement(copy)
            }
            MutationResult::Nucleotide(_record) => result.clone(),
            MutationResult::Residue(_record) => result.clone(),
            MutationResult::Deletion(_record) => result.clone(),
            MutationResult::Insertion(_record) => result.clone(),
        }
    }

    // The same artifacts, claiming other bases in the region.
    fn tampered_replacement_claims(result: MutationResult, claimed: String) -> MutationResult {
        match result.clone() {
            MutationResult::Replacement(record) => {
                let mut copy: NucleotideReplacement = record.clone();
                copy.expected_original = claimed.clone();
                copy.actual_original = claimed.clone();
                MutationResult::Replacement(copy)
            }
            MutationResult::Nucleotide(_record) => result.clone(),
            MutationResult::Residue(_record) => result.clone(),
            MutationResult::Deletion(_record) => result.clone(),
            MutationResult::Insertion(_record) => result.clone(),
        }
    }

    #[test]
    fn a_replacement_cannot_target_another_sequence() {
        let other: Sequence = normalize(
            Identifier::create("seq-b".to_string()).unwrap(),
            "ATTCCAAGCCATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap();
        let refusal: Result<NucleotideReplacementIntent, DomainError> =
            NucleotideReplacementIntent::create(
                seq_a().reference(),
                create(&other.reference(), 9, 12).unwrap(),
                "AAT".to_string(),
                "GG".to_string(),
            );
        assert!(matches!(
            refusal.unwrap_err(),
            DomainError::InvalidMutationRequest(_)
        ));
    }

    #[test]
    fn a_replacement_that_asserts_the_wrong_number_of_bases_is_refused() {
        // A length that disagrees with the region names both numbers.
        let mismatch: DomainError = NucleotideReplacementIntent::create(
            seq_a().reference(),
            at(9, 12),
            "AA".to_string(),
            "GG".to_string(),
        )
        .unwrap_err();
        assert!(matches!(mismatch, DomainError::InvalidMutationRequest(_)));
        match mismatch {
            DomainError::InvalidMutationRequest(problem) => {
                assert!(problem.reason.contains("2"), "{}", problem.reason);
                assert!(problem.reason.contains("3"), "{}", problem.reason);
            }
            other => panic!("expected InvalidMutationRequest, got {:?}", other),
        }
    }

    #[test]
    fn a_replacement_asserts_and_writes_only_plain_bases() {
        // An ambiguity code is a sequence letter, not a base that can be
        // asserted as replaced or written in its place.
        let error: DomainError = NucleotideReplacementIntent::create(
            seq_a().reference(),
            at(9, 12),
            "AAT".to_string(),
            "GNG".to_string(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
        match error {
            DomainError::InvalidMutationRequest(problem) => {
                assert!(problem.reason.contains("A, C, G, T"), "{}", problem.reason);
            }
            other => panic!("expected InvalidMutationRequest, got {:?}", other),
        }
    }

    #[test]
    fn a_replacement_of_no_bases_is_refused() {
        let error: DomainError = NucleotideReplacementIntent::create(
            seq_a().reference(),
            at(9, 12),
            "AAT".to_string(),
            "".to_string(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
        match error {
            DomainError::InvalidMutationRequest(problem) => {
                assert!(
                    problem.reason.contains("at least one"),
                    "{}",
                    problem.reason
                );
            }
            other => panic!("expected InvalidMutationRequest, got {:?}", other),
        }
    }

    #[test]
    fn a_replacement_that_writes_back_the_text_already_there_is_refused() {
        let error: DomainError = NucleotideReplacementIntent::create(
            seq_a().reference(),
            at(9, 12),
            "AAT".to_string(),
            "AAT".to_string(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
        match error {
            DomainError::InvalidMutationRequest(problem) => {
                assert!(
                    problem.reason.contains("already there"),
                    "{}",
                    problem.reason
                );
            }
            other => panic!("expected InvalidMutationRequest, got {:?}", other),
        }
    }

    #[test]
    fn a_replacement_shorter_than_its_region_is_applied() {
        let source: Sequence = seq_a();
        let made: MutationResult = replacement_of(9, 12, "AAT".to_string(), "GG".to_string());
        let record: NucleotideReplacement = replacement_record(made.clone());

        assert!(made.edited().canonical_text() == "ATGGCTTGGGGCAGCAGCAGCAGCAGCAG");
        assert!(made.edited().length() == 29);
        assert!(made.edited().topology() == SequenceTopology::Linear);
        assert!(made.state() == MutationState::Calculated);
        assert!(made.is_replacement());
        assert!(!made.is_deletion());
        assert!(!made.is_insertion());
        assert!(!made.is_nucleotide());
        assert!(made.target_coordinates() == vec![9, 10, 11]);
        assert!(record.actual_original() == "AAT");

        // The source keeps its content, its length and its digest.
        assert!(source.length() == 30);
        assert!(source.canonical_text() == "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG");
        assert!(made.source() == source.reference());
    }

    #[test]
    fn a_replacement_longer_than_its_region_is_applied() {
        let made: MutationResult = replacement_of(9, 12, "AAT".to_string(), "GGGGG".to_string());
        assert!(made.edited().canonical_text() == "ATGGCTTGGGGGGGCAGCAGCAGCAGCAGCAG");
        assert!(made.edited().length() == 32);
    }

    #[test]
    fn a_replacement_of_the_same_width_is_applied() {
        let made: MutationResult = replacement_of(9, 12, "AAT".to_string(), "CCC".to_string());
        assert!(made.edited().canonical_text() == "ATGGCTTGGCCCCAGCAGCAGCAGCAGCAG");
        assert!(made.edited().length() == 30);
    }

    #[test]
    fn one_base_replaced_by_two_is_applied() {
        let made: MutationResult = replacement_of(9, 10, "A".to_string(), "GG".to_string());
        assert!(made.edited().canonical_text() == "ATGGCTTGGGGATCAGCAGCAGCAGCAGCAG");
        assert!(made.edited().length() == 31);
    }

    #[test]
    fn a_replacement_at_either_end_of_the_sequence_is_applied() {
        let head: MutationResult = replacement_of(0, 3, "ATG".to_string(), "C".to_string());
        assert!(head.edited().canonical_text() == "CGCTTGGAATCAGCAGCAGCAGCAGCAG");
        let tail: MutationResult = replacement_of(27, 30, "CAG".to_string(), "TT".to_string());
        assert!(tail.edited().canonical_text() == "ATGGCTTGGAATCAGCAGCAGCAGCAGTT");
    }

    #[test]
    fn a_replacement_may_not_replace_the_whole_sequence() {
        let error: DomainError = apply_replacement(
            &seq_a().clone(),
            None,
            result_name(),
            NucleotideReplacementIntent::create(
                seq_a().reference(),
                at(0, 30),
                seq_a().canonical_text(),
                "AAAA".to_string(),
            )
            .unwrap(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
        match error {
            DomainError::InvalidMutationRequest(problem) => {
                assert!(
                    problem.reason.contains("the whole sequence"),
                    "{}",
                    problem.reason
                );
            }
            other => panic!("expected InvalidMutationRequest, got {:?}", other),
        }
    }

    #[test]
    fn a_replacement_that_asserts_the_wrong_bases_names_the_bases_that_are_there() {
        let error: DomainError = apply_replacement(
            &seq_a().clone(),
            None,
            result_name(),
            NucleotideReplacementIntent::create(
                seq_a().reference(),
                at(9, 12),
                "AAC".to_string(),
                "GG".to_string(),
            )
            .unwrap(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::ExpectedNucleotideMismatch(_)));
        match error {
            DomainError::ExpectedNucleotideMismatch(problem) => {
                assert!(problem.expected == "AAC");
                assert!(problem.actual == "AAT");
            }
            other => panic!("expected ExpectedNucleotideMismatch, got {:?}", other),
        }
    }

    #[test]
    fn a_replacement_from_a_circular_sequence_stays_circular() {
        let circular: Sequence = normalize(
            Identifier::create("seq-a".to_string()).unwrap(),
            seq_a().canonical_text(),
            SequenceTopology::Circular,
        )
        .unwrap();
        let intent: NucleotideReplacementIntent = NucleotideReplacementIntent::create(
            circular.reference(),
            create(&circular.reference(), 9, 12).unwrap(),
            "AAT".to_string(),
            "GG".to_string(),
        )
        .unwrap();
        let made: MutationResult =
            apply_replacement(&circular, None, result_name(), intent).unwrap();
        assert!(made.edited().topology() == SequenceTopology::Circular);
        assert!(made.edited().canonical_text() == "ATGGCTTGGGGCAGCAGCAGCAGCAGCAG");
    }

    #[test]
    fn a_replacement_inside_its_scoped_cds_is_applied() {
        let made: MutationResult = apply_replacement(
            &seq_a().clone(),
            Some(cds_of(12).clone()),
            result_name(),
            NucleotideReplacementIntent::create(
                seq_a().reference(),
                at(9, 12),
                "AAT".to_string(),
                "GG".to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(made.state() == MutationState::Calculated);
        assert!(made.edited().canonical_text() == "ATGGCTTGGGGCAGCAGCAGCAGCAGCAG");
    }

    #[test]
    fn a_replacement_that_straddles_the_scoped_cds_is_refused() {
        let error: DomainError = apply_replacement(
            &seq_a().clone(),
            Some(cds_of(12).clone()),
            result_name(),
            NucleotideReplacementIntent::create(
                seq_a().reference(),
                at(10, 14),
                "ATCA".to_string(),
                "G".to_string(),
            )
            .unwrap(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::TargetOutsideFeature(_)));
    }

    #[test]
    fn a_replacement_result_is_verified_against_its_source() {
        let calculated: MutationResult = replacement_of(9, 12, "AAT".to_string(), "GG".to_string());
        let verified: MutationResult = verify(&seq_a().clone(), None, calculated.clone()).unwrap();
        assert!(verified.state() == MutationState::Verified);
        assert!(verified != calculated);
        assert!(verified.edited().canonical_text() == "ATGGCTTGGGGCAGCAGCAGCAGCAGCAG");
        assert!(verified.target_coordinates() == vec![9, 10, 11]);
        assert!(verified.is_replacement());
        let record: NucleotideReplacement = replacement_record(verified.clone());
        assert!(record.region() == at(9, 12));
        assert!(record.expected_original() == "AAT");
        assert!(record.actual_original() == "AAT");
        assert!(record.replacement() == "GG");
    }

    #[test]
    fn verify_refuses_a_replacement_that_changes_the_length_by_the_wrong_amount() {
        let calculated: MutationResult = replacement_of(9, 12, "AAT".to_string(), "GG".to_string());
        // The same record, whose artifact has the length an equal-width
        // replacement would leave.
        let tampered: MutationResult = tampered_replacement(
            calculated.clone(),
            "ATGGCTTGGCCCCAGCAGCAGCAGCAGCAG".to_string(),
        );
        let error: DomainError = verify(&seq_a().clone(), None, tampered.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
        match error {
            DomainError::MutationVerificationFailure(problem) => {
                assert!(
                    problem.reason.contains("replacing 3 bases with 2"),
                    "{}",
                    problem.reason,
                );
            }
            other => panic!("expected MutationVerificationFailure, got {:?}", other),
        }
    }

    #[test]
    fn verify_refuses_an_edited_artifact_that_is_not_the_source_with_the_region_replaced() {
        let calculated: MutationResult = replacement_of(9, 12, "AAT".to_string(), "GG".to_string());
        // The right length, but other bases were written.
        let tampered: MutationResult = tampered_replacement(
            calculated.clone(),
            "ATGGCTTGGTTCAGCAGCAGCAGCAGCAG".to_string(),
        );
        let error: DomainError = verify(&seq_a().clone(), None, tampered.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
    }

    #[test]
    fn verify_refuses_replacement_claims_that_disagree_with_the_source() {
        let calculated: MutationResult = replacement_of(9, 12, "AAT".to_string(), "GG".to_string());
        // The artifacts agree, but the record claims other bases were replaced.
        let tampered: MutationResult =
            tampered_replacement_claims(calculated.clone(), "AAC".to_string());
        let error: DomainError = verify(&seq_a().clone(), None, tampered.clone()).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
        match error {
            DomainError::MutationVerificationFailure(problem) => {
                assert!(
                    problem.reason.contains("replacement claims"),
                    "{}",
                    problem.reason,
                );
            }
            other => panic!("expected MutationVerificationFailure, got {:?}", other),
        }
    }

    #[test]
    fn a_residue_number_of_zero_selects_no_codon() {
        // Residues are numbered from 1, so residue 0 selects no codon: `number - 1`
        // would underflow.
        let error: DomainError = codon_positions(whole(30), Strand::Plus, 0, 30).unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
    }

    #[test]
    fn a_minus_strand_codon_past_the_cds_end_selects_no_codon() {
        // Residue 20 on a minus strand reads offsets 57, 58 and 59, so the
        // biological offset `30 - 1 - 57` would be negative: a 30-base CDS
        // holds 10 residues, and residue 20 lies past its end.
        let error: DomainError = codon_positions(whole(30), Strand::Minus, 20, 30).unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));

        // Residue u64::MAX / 2 starts at offset 3 * (number - 1), which does not
        // fit in a u64.
        let overflow: DomainError =
            codon_positions(whole(30), Strand::Plus, u64::MAX / 2, 30).unwrap_err();
        assert!(matches!(overflow, DomainError::InvalidMutationRequest(_)));
    }

    #[test]
    fn a_multi_base_interval_starting_inside_a_feature_but_ending_outside_is_refused() {
        let intervals: OrderedIntervals = whole(10);
        let error: DomainError = require_within_feature(
            at(8, 12),
            "cds-1".to_string(),
            "seq-a".to_string(),
            intervals.as_slice(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::TargetOutsideFeature(_)));
        match error {
            DomainError::TargetOutsideFeature(problem) => {
                assert!(problem.start == 8);
                assert!(problem.end == 12);
            }
            other => panic!("expected TargetOutsideFeature, got {:?}", other),
        }
        // The same start, ending on the interval's edge, lies wholly inside it.
        assert!(require_within_feature(
            at(8, 10),
            "cds-1".to_string(),
            "seq-a".to_string(),
            intervals.as_slice(),
        )
        .is_ok());
    }

    // mut-c of the Milestone 2 fixture: circular, 24 bases, and its `res-c-origin`
    // annotation `cds-1` over [22, 24) then [0, 7), plus strand, complete, reading
    // TGG CTT GGA (W L G) across the origin.
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
            cds_name(),
            intervals,
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap()
    }

    #[test]
    fn an_insertion_at_either_end_of_a_circular_sequence_is_refused() {
        // D032: "the request states the bases at p − 1 and p … So 1 ≤ p ≤ n − 1:
        // an insertion needs a base on both sides." That holds for every topology,
        // so a circular artifact refuses the gap at its origin and the gap after
        // its last base just as a linear one does. Both points fit the artifact —
        // `InsertionPoint::within` accepts 0 and n — so the refusal below is D032's
        // end rule, which `apply_insertion` reports as `InvalidMutationRequest`
        // (`MalformedInput`), and not an out-of-range coordinate. The flanks named
        // here are the bases a circular reading puts beside each gap, so the
        // intents are well formed on their own.
        let at_origin: NucleotideInsertionIntent = NucleotideInsertionIntent::create(
            seq_c().reference(),
            create_point(&seq_c().reference(), 0).unwrap(),
            "G".to_string(),
            "G".to_string(),
            "A".to_string(),
        )
        .unwrap();
        let error: DomainError =
            apply_insertion(&seq_c().clone(), None, result_name(), at_origin).unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);

        let past_last: NucleotideInsertionIntent = NucleotideInsertionIntent::create(
            seq_c().reference(),
            create_point(&seq_c().reference(), 24).unwrap(),
            "G".to_string(),
            "G".to_string(),
            "A".to_string(),
        )
        .unwrap();
        let error: DomainError =
            apply_insertion(&seq_c().clone(), None, result_name(), past_last).unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    #[test]
    fn a_deletion_inside_the_post_origin_interval_of_a_wrapping_cds_is_applied() {
        // [2, 5) is `TTG`, wholly inside the interval [0, 7) of the wrapping CDS, so
        // the scoped containment rule accepts it. The edited artifact is mut-c with
        // those three bases cut out, and it keeps the source's topology.
        let intent: NucleotideDeletionIntent = NucleotideDeletionIntent::create(
            seq_c().reference(),
            create(&seq_c().reference(), 2, 5).unwrap(),
            "TTG".to_string(),
        )
        .unwrap();
        let made: MutationResult =
            apply_deletion(&seq_c().clone(), Some(cds_c()), result_name(), intent).unwrap();
        assert!(made.edited().canonical_text() == "GCGAACGAACGAACGAACATG");
        assert!(made.edited().topology() == SequenceTopology::Circular);

        let verified: MutationResult = verify(&seq_c().clone(), Some(cds_c()), made).unwrap();
        assert!(verified.state() == MutationState::Verified);
    }

    #[test]
    fn a_deletion_straddling_the_start_of_the_pre_origin_interval_is_refused() {
        // [21, 23) is `AT`: 21 lies before [22, 24) and past [0, 7), so the region
        // lies wholly in neither interval of the wrapping CDS and the containment
        // rule refuses it before the bases are compared.
        let intent: NucleotideDeletionIntent = NucleotideDeletionIntent::create(
            seq_c().reference(),
            create(&seq_c().reference(), 21, 23).unwrap(),
            "AT".to_string(),
        )
        .unwrap();
        let error: DomainError =
            apply_deletion(&seq_c().clone(), Some(cds_c()), result_name(), intent).unwrap_err();
        assert!(matches!(error, DomainError::TargetOutsideFeature(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn an_insertion_inside_the_pre_origin_interval_of_a_wrapping_cds_is_applied() {
        // The gap at 23 sits strictly inside [22, 24), so both bases beside it are
        // coding: `T` at 22 and `G` at 23. `A` is written into that gap, and the
        // artifact keeps the source's topology.
        let intent: NucleotideInsertionIntent = NucleotideInsertionIntent::create(
            seq_c().reference(),
            create_point(&seq_c().reference(), 23).unwrap(),
            "T".to_string(),
            "G".to_string(),
            "A".to_string(),
        )
        .unwrap();
        let made: MutationResult =
            apply_insertion(&seq_c().clone(), Some(cds_c()), result_name(), intent).unwrap();
        assert!(made.edited().canonical_text() == "GCTTGGAACGAACGAACGAACATAG");

        let verified: MutationResult = verify(&seq_c().clone(), Some(cds_c()), made).unwrap();
        assert!(verified.state() == MutationState::Verified);
    }
}
