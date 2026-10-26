//! Structured domain errors.
//!
//! Every failure produced by the core domain model is a `DomainError`.
//! Callers classify failures with `ErrorCategory` and with the concrete
//! enum variant. Message text exists for human reporting only; it must
//! never be the basis for programmatic classification.
//!
//! The three failure classes are kept deliberately distinct:
//!
//! * `MalformedInput`: the request could not be represented at all, for
//!   example a negative coordinate, an empty interval, or a character
//!   outside the declared alphabet.
//! * `BiologicalValidation`: a well-formed request that violates a domain
//!   invariant or the contract of the operation, for example coordinates
//!   outside the referenced sequence, a structure pointing at the wrong
//!   sequence, or an internal stop codon.
//! * `Unsupported`: a well-formed request outside the supported scope of
//!   this milestone, for example a compound feature, an incomplete CDS, or
//!   a genetic code other than the standard one.

use crate::sequence::SequenceTopology;

/// The failure classes the product must keep distinguishable.
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum ErrorCategory {
    MalformedInput,
    BiologicalValidation,
    Unsupported,
}

/// Payload for a rejected identifier.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidIdentifierProblem {
    pub identifier: String,
    pub reason: String,
}

/// Payload: a sequence record contains letters outside the declared alphabet.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidSequenceCharactersProblem {
    pub sequence: String,
    pub position: u64,
    pub characters: String,
}

/// Payload: a sequence record carries no bases at all.
#[derive(Clone, Debug, PartialEq)]
pub struct EmptySequenceProblem {
    pub sequence: String,
}

/// Payload: a DNA search pattern holds a letter the IUPAC DNA alphabet has no
/// such set for. `position` counts characters in `pattern`, which is the
/// pattern as the user wrote it, before case folding and the `U` -> `T` fold;
/// `characters` is the offending letter as written, because a refusal that
/// names only "invalid pattern" does not say what to fix.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidSearchPatternCharactersProblem {
    pub pattern: String,
    pub position: u64,
    pub characters: String,
}

/// Payload: signed coordinates that are negative.
#[derive(Clone, Debug, PartialEq)]
pub struct NegativeCoordinateProblem {
    pub sequence: String,
    pub start: i64,
    pub end: i64,
}

/// Payload: an interval whose end precedes its start.
#[derive(Clone, Debug, PartialEq)]
pub struct ReversedIntervalProblem {
    pub sequence: String,
    pub start: u64,
    pub end: u64,
}

/// Payload: an interval that contains no nucleotide positions.
#[derive(Clone, Debug, PartialEq)]
pub struct EmptyIntervalProblem {
    pub sequence: String,
    pub position: u64,
}

/// Payload: an interval that does not fit the referenced sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct CoordinateOutOfBoundsProblem {
    pub sequence: String,
    pub start: u64,
    pub end: u64,
    pub length: u64,
}

/// Payload: a structure references a sequence other than the one supplied.
///
/// The display identifier, the content digest, and the topology are reported
/// so the caller can see which of them disagree.
#[derive(Clone, Debug, PartialEq)]
pub struct SequenceReferenceMismatchProblem {
    pub expected_identifier: String,
    pub actual_identifier: String,
    pub expected_digest: String,
    pub actual_digest: String,
    pub expected_topology: SequenceTopology,
    pub actual_topology: SequenceTopology,
}

/// Payload: a codon contains something outside the DNA alphabet.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidCodonProblem {
    pub cds: String,
    pub codon_number: usize,
    pub codon: String,
}

/// Payload: a residue position that is not a positive one-based number.
#[derive(Clone, Debug, PartialEq)]
pub struct ResiduePositionProblem {
    pub cds: String,
    pub number: i64,
}

/// Payload: a residue position beyond the end of the translated CDS.
#[derive(Clone, Debug, PartialEq)]
pub struct ResiduePositionRangeProblem {
    pub cds: String,
    pub number: u64,
    pub residue_count: u64,
}

/// Payload: a residue position naming a CDS other than the one supplied.
#[derive(Clone, Debug, PartialEq)]
pub struct CdsMismatchProblem {
    pub expected: String,
    pub actual: String,
}

/// Payload: a residue numbering scheme this milestone refuses to apply.
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedNumberingProblem {
    pub cds: String,
    pub numbering: String,
}

/// Payload: a feature or CDS with no intervals.
#[derive(Clone, Debug, PartialEq)]
pub struct FeatureWithoutIntervalsProblem {
    pub feature: String,
    pub sequence: String,
}

/// Payload: an interval list that is not in biological order.
#[derive(Clone, Debug, PartialEq)]
pub struct IntervalsOutOfOrderProblem {
    pub feature: String,
    pub sequence: String,
}

/// Payload: an interval list whose members overlap.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlappingIntervalsProblem {
    pub feature: String,
    pub sequence: String,
    pub start: u64,
    pub end: u64,
}

/// Payload: an interval structure this milestone refuses to interpret.
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedFeatureStructureProblem {
    pub feature: String,
    pub sequence: String,
    pub interval_count: usize,
    pub reason: String,
}

/// Payload: a required CDS attribute was not supplied, or was supplied with
/// a value that is not one of the recognised values.
///
/// `value` is `None` when the attribute was absent and `Some(text)` when the
/// supplied text was not recognised.
#[derive(Clone, Debug, PartialEq)]
pub struct MissingCdsAttributeProblem {
    pub cds: String,
    pub attribute: String,
    pub value: Option<String>,
}

/// Payload: a CDS that is not explicitly complete.
#[derive(Clone, Debug, PartialEq)]
pub struct IncompleteCdsProblem {
    pub cds: String,
    pub completeness: String,
}

/// Payload: a genetic code outside the supported set.
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedGeneticCodeProblem {
    pub cds: String,
    pub genetic_code: String,
}

/// Payload: extracted CDS length is not a multiple of three.
#[derive(Clone, Debug, PartialEq)]
pub struct CdsLengthNotDivisibleByThreeProblem {
    pub cds: String,
    pub length: u64,
}

/// Payload: a codon that cannot be translated without guessing.
#[derive(Clone, Debug, PartialEq)]
pub struct AmbiguousCodonProblem {
    pub cds: String,
    pub codon_number: usize,
    pub codon: String,
}

/// Payload: a stop codon that is not the final codon of the CDS.
#[derive(Clone, Debug, PartialEq)]
pub struct InternalStopCodonProblem {
    pub cds: String,
    pub codon_number: usize,
    pub codon: String,
}

/// Which rule refused a mutation request. Stable: `code()` is part of the
/// CLI's output contract (D041), and renaming one needs approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MutationRequestRule {
    CodonIsStop,
    CodonIsAmbiguous,
    CodonNotThreeDnaLetters,
    SuppliedCodonSyntax,
    ResidueNotOneLetter,
    ResidueNotInStandardCode,
    TargetOnOtherSequence,
    SubstitutionNotOneNucleotide,
    ExpectedTextLengthMismatch,
    NothingWritten,
    NoChange,
    WholeSequence,
    InsertionAtSequenceEnd,
    SuppliedCodonWrongResidue,
    NoDefaultCodon,
    CodonOutsideSequence,
    BaseNotOneLetter,
    BaseNotDnaLetter,
    EmptyVariantSet,
    DuplicateVariantName,
}

impl MutationRequestRule {
    /// The rule's stable code: its name in snake_case.
    pub fn code(self) -> &'static str {
        use MutationRequestRule::*;
        match self {
            CodonIsStop => "codon_is_stop",
            CodonIsAmbiguous => "codon_is_ambiguous",
            CodonNotThreeDnaLetters => "codon_not_three_dna_letters",
            SuppliedCodonSyntax => "supplied_codon_syntax",
            ResidueNotOneLetter => "residue_not_one_letter",
            ResidueNotInStandardCode => "residue_not_in_standard_code",
            TargetOnOtherSequence => "target_on_other_sequence",
            SubstitutionNotOneNucleotide => "substitution_not_one_nucleotide",
            ExpectedTextLengthMismatch => "expected_text_length_mismatch",
            NothingWritten => "nothing_written",
            NoChange => "no_change",
            WholeSequence => "whole_sequence",
            InsertionAtSequenceEnd => "insertion_at_sequence_end",
            SuppliedCodonWrongResidue => "supplied_codon_wrong_residue",
            NoDefaultCodon => "no_default_codon",
            CodonOutsideSequence => "codon_outside_sequence",
            BaseNotOneLetter => "base_not_one_letter",
            BaseNotDnaLetter => "base_not_dna_letter",
            EmptyVariantSet => "empty_variant_set",
            DuplicateVariantName => "duplicate_variant_name",
        }
    }
}

/// Payload: a mutation request that cannot be represented.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidMutationRequestProblem {
    pub request: String,
    pub rule: MutationRequestRule,
    pub reason: String,
}

/// Payload: a residue mutation that did not say how to choose the codon.
#[derive(Clone, Debug, PartialEq)]
pub struct UnspecifiedCodonPolicyProblem {
    pub cds: String,
}

/// Payload: a codon-selection policy this milestone refuses to apply.
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedMutationPolicyProblem {
    pub policy: String,
}

/// Payload: a composition mode of a multi-variant request that this
/// milestone refuses to execute. `composition` names the mode the caller
/// asked for and `variant_count` how many members it carried, so the
/// refusal says which request it is about; `reason` states why the mode is
/// out of scope. Nothing about the members was executed.
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedCompositionProblem {
    pub composition: String,
    pub variant_count: usize,
    pub reason: String,
}

/// Payload: the nucleotide at the target is not the expected original.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectedNucleotideMismatchProblem {
    pub sequence: String,
    pub position: u64,
    pub expected: String,
    pub actual: String,
}

/// Payload: the translated residue is not the expected original residue.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectedResidueMismatchProblem {
    pub cds: String,
    pub number: u64,
    pub expected: String,
    pub actual: String,
}

/// Payload: the target coordinate lies outside the scoped feature or CDS.
#[derive(Clone, Debug, PartialEq)]
pub struct TargetOutsideFeatureProblem {
    pub feature: String,
    pub sequence: String,
    pub start: u64,
    pub end: u64,
}

/// Payload: a claimed mutation result does not follow from its source.
#[derive(Clone, Debug, PartialEq)]
pub struct MutationVerificationFailureProblem {
    pub result: String,
    pub reason: String,
}

/// Which rule refused a primer or pair. Stable: `code()` is part of the
/// CLI's output contract (D041), and renaming one needs approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimerCandidateRule {
    SpanOverlapsDeletion,
    SpanStraddlesInsertionPoint,
    SpanOverlapsReplacement,
    TextLengthNotSpanWidth,
    ForwardSpanDoesNotCoverReplacement,
    EditTakesNoTail,
    EmptyTail,
    RoleDoesNotMatchSlot,
    PrimersOnDifferentTemplates,
    PairNotOnEditTemplate,
    PairNotAbuttingEdit,
    MinimumOverlapBelowOne,
    TailsDoNotRebuild,
}

impl PrimerCandidateRule {
    /// The rule's stable code: its name in snake_case.
    pub fn code(self) -> &'static str {
        use PrimerCandidateRule::*;
        match self {
            SpanOverlapsDeletion => "span_overlaps_deletion",
            SpanStraddlesInsertionPoint => "span_straddles_insertion_point",
            SpanOverlapsReplacement => "span_overlaps_replacement",
            TextLengthNotSpanWidth => "text_length_not_span_width",
            ForwardSpanDoesNotCoverReplacement => "forward_span_does_not_cover_replacement",
            EditTakesNoTail => "edit_takes_no_tail",
            EmptyTail => "empty_tail",
            RoleDoesNotMatchSlot => "role_does_not_match_slot",
            PrimersOnDifferentTemplates => "primers_on_different_templates",
            PairNotOnEditTemplate => "pair_not_on_edit_template",
            PairNotAbuttingEdit => "pair_not_abutting_edit",
            MinimumOverlapBelowOne => "minimum_overlap_below_one",
            TailsDoNotRebuild => "tails_do_not_rebuild",
        }
    }
}

/// Payload: a proposed primer cannot become a primer record at all.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidPrimerCandidateProblem {
    pub primer: String,
    pub rule: PrimerCandidateRule,
    pub reason: String,
}

/// Payload: primer text is not the sequence the contract says sits at its span.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerTextAtSpanMismatchProblem {
    pub primer: String,
    pub span: String,
    pub expected: String,
    pub actual: String,
}

/// Payload: an annealing portion is not the wild-type template under its spans.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerAnnealingNotTemplateProblem {
    pub primer: String,
    pub position: u64,
    pub expected: String,
    pub actual: String,
}

/// Payload: a primer's parts do not partition its text.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerPartitionProblem {
    pub primer: String,
    pub reason: String,
}

/// Payload: a primer's annealing overlap is below the supplied bound.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnealingOverlapBelowBoundProblem {
    pub primer: String,
    pub overlap: u64,
    pub bound: u64,
}

/// Payload: a pair does not cover the changed region of its edit.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerPairCoverageProblem {
    pub pair: String,
    pub changed: String,
    pub uncovered: String,
}

/// Payload: the two primers do not state opposite orientations.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerOrientationConflictProblem {
    pub forward: String,
    pub reverse: String,
}

/// Payload: a primer placement the contract leaves open is requested.
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedPrimerPlacementProblem {
    pub placement: String,
    pub reason: String,
}

/// Payload: a primer-design request that cannot be acted on as given, such as
/// a `minAnnealingOverlap` below 1 or a mutation result that was never
/// verified.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidPrimerDesignRequestProblem {
    pub field: String,
    pub reason: String,
}

/// Payload: an edit or template outside the design policy's scope (D029:
/// a contiguous replacement region of at most 3 bases on a linear template).
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedPrimerDesignScopeProblem {
    pub policy: String,
    pub reason: String,
}

/// Payload: the template is shorter than the smallest pair the policy can
/// emit, so no placement of any abutting pair fits (D029).
#[derive(Clone, Debug, PartialEq)]
pub struct NoPrimerDesignPossibleProblem {
    pub template: String,
    pub template_length: u64,
    pub smallest_pair: u64,
}

/// One reason a design under D029/D030 does not fit the template.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignObstacle {
    /// Too few template bases 5′ of the edit for the forward 5′ flank plus
    /// the reverse primer.
    FivePrimeAvailability { available: u64, needed: u64 },
    /// Too few template bases 3′ of the edit for the forward 3′ flank.
    ThreePrimeAvailability { available: u64, needed: u64 },
    /// The template holds an ambiguity code where a primer base would be
    /// (H6: no degenerate symbol in a primer).
    AmbiguousTemplateBase { position: u64, base: String },
    /// The reverse primer ran out of template 5′ of the junction before its
    /// Tm reached the 55 °C floor (D030). `reached_celsius` is its Tm at the
    /// longest length available.
    FivePrimeTmFloorUnreachable {
        available: u64,
        reached_celsius: f64,
    },
    /// The forward primer ran out of template 3′ of the edit before its Tm
    /// reached the floor.
    ThreePrimeTmFloorUnreachable {
        available: u64,
        reached_celsius: f64,
    },
}

impl DesignObstacle {
    pub fn describe(&self) -> String {
        match self {
            DesignObstacle::FivePrimeAvailability { available, needed } => format!(
                "5' side: {} template bases before the edit, {} needed",
                available, needed
            ),
            DesignObstacle::ThreePrimeAvailability { available, needed } => format!(
                "3' side: {} template bases after the edit, {} needed",
                available, needed
            ),
            DesignObstacle::AmbiguousTemplateBase { position, base } => format!(
                "template position {} is the ambiguity code {}",
                position, base
            ),
            DesignObstacle::FivePrimeTmFloorUnreachable {
                available,
                reached_celsius,
            } => format!(
                "5' side: the reverse primer reaches only {:.2} °C using all {} bases available",
                reached_celsius, available
            ),
            DesignObstacle::ThreePrimeTmFloorUnreachable {
                available,
                reached_celsius,
            } => format!(
                "3' side: the forward primer reaches only {:.2} °C using all {} bases available",
                reached_celsius, available
            ),
        }
    }
}

/// Payload: a pair of the policy's starting size fits the template, but not
/// around this edit, or not at a Tm the policy accepts. Another geometry might
/// fit; this policy keeps the junction fixed and does not shift (D029, D030).
#[derive(Clone, Debug, PartialEq)]
pub struct NoPrimerDesignFitsProblem {
    pub template: String,
    pub edit_start: u64,
    pub edit_end: u64,
    pub obstacles: Vec<DesignObstacle>,
}

/// Payload: a buffer model that cannot be used, such as a concentration of
/// zero where the Tm equation takes a logarithm.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidBufferModelProblem {
    pub field: String,
    pub reason: String,
}

/// Payload: the named Tm function has no published parameters for this duplex.
/// The commonest cause is two adjacent mismatches, for which no nearest-
/// neighbour set exists (`tests/fixtures/tm/README.md`).
#[derive(Clone, Debug, PartialEq)]
pub struct TmParametersUnavailableProblem {
    pub function: String,
    pub aspect: String,
    pub reason: String,
}

/// Payload: a degenerate text denotes more concrete sequences than can be
/// counted (D033 sets no cap; this is representability, not policy).
#[derive(Clone, Debug, PartialEq)]
pub struct DegenerateExpansionTooLargeProblem {
    pub replacement: String,
}

/// Which rule refused a FASTA text. Stable: `code()` is part of the CLI's
/// output contract (D041), and renaming one needs approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FastaRule {
    NoRecord,
    TextBeforeHeader,
    SecondRecord,
}

impl FastaRule {
    /// The rule's stable code: its name in snake_case.
    pub fn code(self) -> &'static str {
        use FastaRule::*;
        match self {
            NoRecord => "no_record",
            TextBeforeHeader => "text_before_header",
            SecondRecord => "second_record",
        }
    }
}

/// Which rule refused a GenBank location. Stable: `code()` is part of the
/// CLI's output contract (D041), and renaming one needs approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenBankLocationRule {
    Syntax,
    PositionZero,
    RangeEndsBeforeStart,
    PastSequenceEnd,
    FuzzyEnd,
    BetweenBases,
    Order,
    RemoteReference,
    SinglePosition,
    OriginJoinOnLinear,
    UnsupportedJoin,
}

impl GenBankLocationRule {
    /// The rule's stable code: its name in snake_case.
    pub fn code(self) -> &'static str {
        use GenBankLocationRule::*;
        match self {
            Syntax => "syntax",
            PositionZero => "position_zero",
            RangeEndsBeforeStart => "range_ends_before_start",
            PastSequenceEnd => "past_sequence_end",
            FuzzyEnd => "fuzzy_end",
            BetweenBases => "between_bases",
            Order => "order",
            RemoteReference => "remote_reference",
            SinglePosition => "single_position",
            OriginJoinOnLinear => "origin_join_on_linear",
            UnsupportedJoin => "unsupported_join",
        }
    }

    /// The rule's failure class. `PastSequenceEnd` is `BiologicalValidation`
    /// because a well-formed location contradicts the sequence it was read
    /// against, as `CoordinateOutOfBounds` does; the rest of the rules
    /// either cannot be represented at all or are outside this milestone's
    /// scope (D009, D036).
    pub fn category(self) -> ErrorCategory {
        use GenBankLocationRule::*;
        match self {
            Syntax | PositionZero | RangeEndsBeforeStart => ErrorCategory::MalformedInput,
            PastSequenceEnd => ErrorCategory::BiologicalValidation,
            FuzzyEnd | BetweenBases | Order | RemoteReference | SinglePosition
            | OriginJoinOnLinear | UnsupportedJoin => ErrorCategory::Unsupported,
        }
    }
}

/// Which rule refused a GenBank CDS. Stable: `code()` is part of the CLI's
/// output contract (D041), and renaming one needs approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenBankCdsRule {
    NotFound,
    AmbiguousName,
    RepeatedName,
    NoId,
    RepeatedQualifier,
    Pseudo,
    CodonStart,
    TranslTable,
    AlternativeStart,
}

impl GenBankCdsRule {
    /// The rule's stable code: its name in snake_case.
    pub fn code(self) -> &'static str {
        use GenBankCdsRule::*;
        match self {
            NotFound => "not_found",
            AmbiguousName => "ambiguous_name",
            RepeatedName => "repeated_name",
            NoId => "no_id",
            RepeatedQualifier => "repeated_qualifier",
            Pseudo => "pseudo",
            CodonStart => "codon_start",
            TranslTable => "transl_table",
            AlternativeStart => "alternative_start",
        }
    }

    /// The rule's failure class. `NotFound` and `AmbiguousName` are
    /// `BiologicalValidation` because a well-formed request names a CDS the
    /// record does not hold exactly once, as `CoordinateOutOfBounds` does; a
    /// repeated request name or a repeated qualifier cannot be represented at
    /// all (D041 G1, G2, G5).
    pub fn category(self) -> ErrorCategory {
        use GenBankCdsRule::*;
        match self {
            NotFound | AmbiguousName => ErrorCategory::BiologicalValidation,
            RepeatedName | RepeatedQualifier => ErrorCategory::MalformedInput,
            NoId | Pseudo | CodonStart | TranslTable | AlternativeStart => {
                ErrorCategory::Unsupported
            }
        }
    }
}

/// Which rule refused a GenBank text. Stable: `code()` is part of the CLI's
/// output contract (D041), and renaming one needs approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenBankRule {
    NoLocus,
    LocusLength,
    TopologyMissing,
    TopologyConflict,
    RepeatedSection,
    LineLayout,
    QualifierSyntax,
    NoOrigin,
    OriginLine,
    OriginNumbering,
    LengthMismatch,
    MissingEnd,
    SecondRecord,
    TextAfterEnd,
}

impl GenBankRule {
    /// The rule's stable code: its name in snake_case.
    pub fn code(self) -> &'static str {
        use GenBankRule::*;
        match self {
            NoLocus => "no_locus",
            LocusLength => "locus_length",
            TopologyMissing => "topology_missing",
            TopologyConflict => "topology_conflict",
            RepeatedSection => "repeated_section",
            LineLayout => "line_layout",
            QualifierSyntax => "qualifier_syntax",
            NoOrigin => "no_origin",
            OriginLine => "origin_line",
            OriginNumbering => "origin_numbering",
            LengthMismatch => "length_mismatch",
            MissingEnd => "missing_end",
            SecondRecord => "second_record",
            TextAfterEnd => "text_after_end",
        }
    }
}

/// Payload: a FASTA text that is not exactly one record. `line` counts text
/// lines from 1, as an editor does; it is not a sequence coordinate. It is
/// `None` for `NoRecord`, which has no line to point at.
#[derive(Clone, Debug, PartialEq)]
pub struct MalformedFastaProblem {
    pub sequence: String,
    pub rule: FastaRule,
    pub line: Option<u64>,
}

/// Payload: a GenBank feature location luria cannot use. `location` is the
/// text as given; `sequence_length` is the length it was read against.
/// `cds` is the id text of the CDS feature whose location this is, and `line`
/// its key line in the GenBank text, counted from 1; both are `None` when the
/// location was read on its own, outside a record import.
#[derive(Clone, Debug, PartialEq)]
pub struct UnusableGenBankLocationProblem {
    pub location: String,
    pub rule: GenBankLocationRule,
    pub sequence_length: u64,
    pub cds: Option<String>,
    pub line: Option<u64>,
}

/// Payload: a GenBank text that is not one readable record. `line` counts
/// text lines from 1; it is not a sequence coordinate. It is `None` when
/// there is no line to point at (an empty text, or a text that ends early).
#[derive(Clone, Debug, PartialEq)]
pub struct MalformedGenBankProblem {
    pub sequence: String,
    pub rule: GenBankRule,
    pub line: Option<u64>,
}

/// Payload: a GenBank CDS luria cannot import. `cds` is the requested name
/// or the feature's id (empty for `NoId`). `line` is the feature's key line,
/// `None` for `NotFound` and `RepeatedName`. `detail` is the offending
/// value: the qualifier's value for `CodonStart` and `TranslTable` (`None`
/// for a value-less flag), the first codon for `AlternativeStart`, the
/// qualifier name for `RepeatedQualifier` and `Pseudo`; `None` otherwise.
#[derive(Clone, Debug, PartialEq)]
pub struct UnusableGenBankCdsProblem {
    pub cds: String,
    pub rule: GenBankCdsRule,
    pub line: Option<u64>,
    pub detail: Option<String>,
}

/// Payload: a CDS whose `/translation` differs from luria's translation
/// without the terminal stop (D041 G3). `residue` is the first differing
/// residue, counted from 1; `stated` and `translated` are the letters there,
/// `None` where one translation has already ended.
#[derive(Clone, Debug, PartialEq)]
pub struct GenBankTranslationMismatchProblem {
    pub cds: String,
    pub line: u64,
    pub residue: u64,
    pub stated: Option<String>,
    pub translated: Option<String>,
}

/// Payload: `open_record` could not tell what format a record is written in.
/// The format comes from the content, never the file name (D049): a GenBank
/// record's first non-blank line starts with `LOCUS` and a FASTA record's with
/// `>`; anything else, and a text with no non-blank line at all, is refused
/// here rather than guessed at (D005).
#[derive(Clone, Debug, PartialEq)]
pub struct UnknownRecordFormatProblem {
    pub sequence: String,
}

/// Payload: a record holds more bases than luria reads (D048, M8-14).
/// `length` is the sequence's own base count and `limit` is the ceiling named
/// in the message; both are exact, because a refusal that says only "too big"
/// does not tell the caller what to cut down to.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordTooLongProblem {
    pub sequence: String,
    pub length: u64,
    pub limit: u64,
}

/// A structured domain failure.
#[derive(Clone, Debug, PartialEq)]
pub enum DomainError {
    EmptyIdentifier,
    InvalidIdentifierCharacters(InvalidIdentifierProblem),
    EmptySequenceContent(EmptySequenceProblem),
    InvalidSequenceCharacters(InvalidSequenceCharactersProblem),
    /// Payload: a search pattern with nothing in it.
    EmptySearchPattern,
    /// Payload: a pattern letter outside the IUPAC DNA set after folding.
    InvalidSearchPatternCharacters(InvalidSearchPatternCharactersProblem),
    NegativeCoordinate(NegativeCoordinateProblem),
    ReversedInterval(ReversedIntervalProblem),
    EmptyInterval(EmptyIntervalProblem),
    CoordinateOutOfBounds(CoordinateOutOfBoundsProblem),
    SequenceReferenceMismatch(SequenceReferenceMismatchProblem),
    FeatureWithoutIntervals(FeatureWithoutIntervalsProblem),
    IntervalsOutOfOrder(IntervalsOutOfOrderProblem),
    OverlappingIntervals(OverlappingIntervalsProblem),
    UnsupportedFeatureStructure(UnsupportedFeatureStructureProblem),
    MissingCdsAttribute(MissingCdsAttributeProblem),
    IncompleteCds(IncompleteCdsProblem),
    UnsupportedGeneticCode(UnsupportedGeneticCodeProblem),
    CdsLengthNotDivisibleByThree(CdsLengthNotDivisibleByThreeProblem),
    AmbiguousCodon(AmbiguousCodonProblem),
    InvalidCodon(InvalidCodonProblem),
    InternalStopCodon(InternalStopCodonProblem),
    InvalidResiduePosition(ResiduePositionProblem),
    ResiduePositionOutsideCds(ResiduePositionRangeProblem),
    ResiduePositionWrongCds(CdsMismatchProblem),
    UnsupportedNumberingScheme(UnsupportedNumberingProblem),
    InvalidMutationRequest(InvalidMutationRequestProblem),
    UnspecifiedCodonPolicy(UnspecifiedCodonPolicyProblem),
    UnsupportedMutationPolicy(UnsupportedMutationPolicyProblem),
    UnsupportedComposition(UnsupportedCompositionProblem),
    ExpectedNucleotideMismatch(ExpectedNucleotideMismatchProblem),
    ExpectedResidueMismatch(ExpectedResidueMismatchProblem),
    TargetOutsideFeature(TargetOutsideFeatureProblem),
    MutationVerificationFailure(MutationVerificationFailureProblem),
    InvalidPrimerCandidate(InvalidPrimerCandidateProblem),
    PrimerTextAtSpanMismatch(PrimerTextAtSpanMismatchProblem),
    PrimerAnnealingNotTemplate(PrimerAnnealingNotTemplateProblem),
    PrimerPartition(PrimerPartitionProblem),
    AnnealingOverlapBelowBound(AnnealingOverlapBelowBoundProblem),
    PrimerPairCoverage(PrimerPairCoverageProblem),
    PrimerOrientationConflict(PrimerOrientationConflictProblem),
    UnsupportedPrimerPlacement(UnsupportedPrimerPlacementProblem),
    InvalidPrimerDesignRequest(InvalidPrimerDesignRequestProblem),
    UnsupportedPrimerDesignScope(UnsupportedPrimerDesignScopeProblem),
    NoPrimerDesignPossible(NoPrimerDesignPossibleProblem),
    NoPrimerDesignFits(NoPrimerDesignFitsProblem),
    InvalidBufferModel(InvalidBufferModelProblem),
    TmParametersUnavailable(TmParametersUnavailableProblem),
    DegenerateExpansionTooLarge(DegenerateExpansionTooLargeProblem),
    MalformedFasta(MalformedFastaProblem),
    UnusableGenBankLocation(UnusableGenBankLocationProblem),
    MalformedGenBank(MalformedGenBankProblem),
    UnusableGenBankCds(UnusableGenBankCdsProblem),
    GenBankTranslationMismatch(GenBankTranslationMismatchProblem),
    /// Payload: neither `LOCUS` nor `>` starts the record's first non-blank
    /// line (D049).
    UnknownRecordFormat(UnknownRecordFormatProblem),
    /// Payload: a record over `MAX_RECORD_BASES` (D048).
    RecordTooLong(RecordTooLongProblem),
}

impl ErrorCategory {
    pub fn is_malformed_input(self) -> bool {
        self == ErrorCategory::MalformedInput
    }

    pub fn is_biological_validation(self) -> bool {
        self == ErrorCategory::BiologicalValidation
    }

    pub fn is_unsupported(self) -> bool {
        self == ErrorCategory::Unsupported
    }
}

impl DomainError {
    /// The failure class. Callers branch on this, never on message text.
    pub fn category(&self) -> ErrorCategory {
        match self {
            DomainError::EmptyIdentifier
            | DomainError::InvalidIdentifierCharacters(_)
            | DomainError::EmptySequenceContent(_)
            | DomainError::InvalidSequenceCharacters(_)
            | DomainError::EmptySearchPattern
            | DomainError::InvalidSearchPatternCharacters(_)
            | DomainError::NegativeCoordinate(_)
            | DomainError::ReversedInterval(_)
            | DomainError::EmptyInterval(_)
            | DomainError::FeatureWithoutIntervals(_)
            | DomainError::IntervalsOutOfOrder(_)
            | DomainError::OverlappingIntervals(_)
            | DomainError::MissingCdsAttribute(_)
            | DomainError::InvalidResiduePosition(_)
            | DomainError::InvalidMutationRequest(_)
            | DomainError::UnspecifiedCodonPolicy(_) => ErrorCategory::MalformedInput,
            DomainError::InvalidPrimerCandidate(_)
            | DomainError::InvalidPrimerDesignRequest(_)
            | DomainError::InvalidBufferModel(_)
            | DomainError::MalformedFasta(_)
            | DomainError::MalformedGenBank(_)
            | DomainError::UnknownRecordFormat(_) => ErrorCategory::MalformedInput,
            DomainError::CoordinateOutOfBounds(_)
            | DomainError::SequenceReferenceMismatch(_)
            | DomainError::CdsLengthNotDivisibleByThree(_)
            | DomainError::AmbiguousCodon(_)
            | DomainError::InternalStopCodon(_)
            | DomainError::InvalidCodon(_)
            | DomainError::ResiduePositionOutsideCds(_)
            | DomainError::ResiduePositionWrongCds(_)
            | DomainError::ExpectedNucleotideMismatch(_)
            | DomainError::ExpectedResidueMismatch(_)
            | DomainError::TargetOutsideFeature(_)
            | DomainError::MutationVerificationFailure(_)
            | DomainError::PrimerTextAtSpanMismatch(_)
            | DomainError::PrimerAnnealingNotTemplate(_)
            | DomainError::PrimerPartition(_)
            | DomainError::AnnealingOverlapBelowBound(_)
            | DomainError::PrimerPairCoverage(_)
            | DomainError::PrimerOrientationConflict(_)
            | DomainError::NoPrimerDesignPossible(_)
            | DomainError::GenBankTranslationMismatch(_) => ErrorCategory::BiologicalValidation,
            DomainError::UnsupportedFeatureStructure(_)
            | DomainError::IncompleteCds(_)
            | DomainError::UnsupportedGeneticCode(_)
            | DomainError::UnsupportedNumberingScheme(_)
            | DomainError::UnsupportedMutationPolicy(_)
            | DomainError::UnsupportedComposition(_)
            | DomainError::UnsupportedPrimerPlacement(_)
            | DomainError::UnsupportedPrimerDesignScope(_)
            | DomainError::NoPrimerDesignFits(_)
            | DomainError::TmParametersUnavailable(_)
            | DomainError::DegenerateExpansionTooLarge(_)
            | DomainError::RecordTooLong(_) => ErrorCategory::Unsupported,
            // The variants whose category comes from their payload: the rule
            // that refused the location or the CDS decides, so `PastSequenceEnd`
            // is `BiologicalValidation` and the rest are `MalformedInput` or
            // `Unsupported`.
            DomainError::UnusableGenBankLocation(problem) => problem.rule.category(),
            DomainError::UnusableGenBankCds(problem) => problem.rule.category(),
        }
    }

    /// A stable machine-readable code: the variant name in snake_case.
    /// Codes are part of the CLI's output contract; renaming one needs approval.
    pub fn code(&self) -> &'static str {
        match self {
            DomainError::EmptyIdentifier => "empty_identifier",
            DomainError::InvalidIdentifierCharacters(_) => "invalid_identifier_characters",
            DomainError::EmptySequenceContent(_) => "empty_sequence_content",
            DomainError::InvalidSequenceCharacters(_) => "invalid_sequence_characters",
            DomainError::EmptySearchPattern => "empty_search_pattern",
            DomainError::InvalidSearchPatternCharacters(_) => "invalid_search_pattern_characters",
            DomainError::NegativeCoordinate(_) => "negative_coordinate",
            DomainError::ReversedInterval(_) => "reversed_interval",
            DomainError::EmptyInterval(_) => "empty_interval",
            DomainError::CoordinateOutOfBounds(_) => "coordinate_out_of_bounds",
            DomainError::SequenceReferenceMismatch(_) => "sequence_reference_mismatch",
            DomainError::FeatureWithoutIntervals(_) => "feature_without_intervals",
            DomainError::IntervalsOutOfOrder(_) => "intervals_out_of_order",
            DomainError::OverlappingIntervals(_) => "overlapping_intervals",
            DomainError::UnsupportedFeatureStructure(_) => "unsupported_feature_structure",
            DomainError::MissingCdsAttribute(_) => "missing_cds_attribute",
            DomainError::IncompleteCds(_) => "incomplete_cds",
            DomainError::UnsupportedGeneticCode(_) => "unsupported_genetic_code",
            DomainError::CdsLengthNotDivisibleByThree(_) => "cds_length_not_divisible_by_three",
            DomainError::AmbiguousCodon(_) => "ambiguous_codon",
            DomainError::InvalidCodon(_) => "invalid_codon",
            DomainError::InternalStopCodon(_) => "internal_stop_codon",
            DomainError::InvalidResiduePosition(_) => "invalid_residue_position",
            DomainError::ResiduePositionOutsideCds(_) => "residue_position_outside_cds",
            DomainError::ResiduePositionWrongCds(_) => "residue_position_wrong_cds",
            DomainError::UnsupportedNumberingScheme(_) => "unsupported_numbering_scheme",
            DomainError::InvalidMutationRequest(_) => "invalid_mutation_request",
            DomainError::UnspecifiedCodonPolicy(_) => "unspecified_codon_policy",
            DomainError::UnsupportedMutationPolicy(_) => "unsupported_mutation_policy",
            DomainError::UnsupportedComposition(_) => "unsupported_composition",
            DomainError::ExpectedNucleotideMismatch(_) => "expected_nucleotide_mismatch",
            DomainError::ExpectedResidueMismatch(_) => "expected_residue_mismatch",
            DomainError::TargetOutsideFeature(_) => "target_outside_feature",
            DomainError::MutationVerificationFailure(_) => "mutation_verification_failure",
            DomainError::InvalidPrimerCandidate(_) => "invalid_primer_candidate",
            DomainError::PrimerTextAtSpanMismatch(_) => "primer_text_at_span_mismatch",
            DomainError::PrimerAnnealingNotTemplate(_) => "primer_annealing_not_template",
            DomainError::PrimerPartition(_) => "primer_partition",
            DomainError::AnnealingOverlapBelowBound(_) => "annealing_overlap_below_bound",
            DomainError::PrimerPairCoverage(_) => "primer_pair_coverage",
            DomainError::PrimerOrientationConflict(_) => "primer_orientation_conflict",
            DomainError::UnsupportedPrimerPlacement(_) => "unsupported_primer_placement",
            DomainError::InvalidPrimerDesignRequest(_) => "invalid_primer_design_request",
            DomainError::UnsupportedPrimerDesignScope(_) => "unsupported_primer_design_scope",
            DomainError::NoPrimerDesignPossible(_) => "no_primer_design_possible",
            DomainError::NoPrimerDesignFits(_) => "no_primer_design_fits",
            DomainError::InvalidBufferModel(_) => "invalid_buffer_model",
            DomainError::TmParametersUnavailable(_) => "tm_parameters_unavailable",
            DomainError::DegenerateExpansionTooLarge(_) => "degenerate_expansion_too_large",
            DomainError::MalformedFasta(_) => "malformed_fasta",
            DomainError::UnusableGenBankLocation(_) => "unusable_genbank_location",
            DomainError::MalformedGenBank(_) => "malformed_genbank",
            DomainError::UnusableGenBankCds(_) => "unusable_genbank_cds",
            DomainError::GenBankTranslationMismatch(_) => "genbank_translation_mismatch",
            DomainError::UnknownRecordFormat(_) => "unknown_record_format",
            DomainError::RecordTooLong(_) => "record_too_long",
        }
    }

    /// Human-readable text. For reporting only.
    pub fn message(&self) -> String {
        match self {
            DomainError::EmptyIdentifier =>
                "an identifier was not supplied".to_string(),
            DomainError::InvalidIdentifierCharacters(problem) =>
                format!("identifier '{}' is not a single printable ASCII token without whitespace",
                    problem.identifier),
            DomainError::EmptySequenceContent(problem) =>
                format!("sequence record '{}' contains no bases", problem.sequence),
            DomainError::InvalidSequenceCharacters(problem) =>
                format!("sequence record '{}' contains unsupported characters '{}' at zero-based position {}",
                    problem.sequence, problem.characters, problem.position),
            DomainError::EmptySearchPattern =>
                "a search pattern was not supplied".to_string(),
            DomainError::InvalidSearchPatternCharacters(problem) =>
                format!("search pattern '{}' contains the unsupported pattern letter '{}' at zero-based position {}",
                    problem.pattern, problem.characters, problem.position),
            DomainError::NegativeCoordinate(problem) =>
                format!("interval [{}, {}) on '{}' uses negative coordinates",
                    problem.start, problem.end, problem.sequence),
            DomainError::ReversedInterval(problem) =>
                format!("interval [{}, {}) on '{}' ends before it starts",
                    problem.start, problem.end, problem.sequence),
            DomainError::EmptyInterval(problem) =>
                format!("interval [{}, {}) on '{}' selects no nucleotides",
                    problem.position, problem.position, problem.sequence),
            DomainError::CoordinateOutOfBounds(problem) =>
                format!("interval [{}, {}) exceeds the length {} of '{}'",
                    problem.start, problem.end, problem.length, problem.sequence),
            DomainError::SequenceReferenceMismatch(problem) =>
                format!("the structure references {} sequence '{}' (digest {}) but {} sequence '{}' (digest {}) was supplied",
                    problem.actual_topology.name(), problem.actual_identifier, problem.actual_digest,
                    problem.expected_topology.name(), problem.expected_identifier, problem.expected_digest),
            DomainError::InvalidResiduePosition(problem) =>
                format!("residue position {} of CDS '{}' is not a positive one-based number",
                    problem.number, problem.cds),
            DomainError::ResiduePositionOutsideCds(problem) =>
                format!("residue position {} of CDS '{}' is beyond its {} residues",
                    problem.number, problem.cds, problem.residue_count),
            DomainError::ResiduePositionWrongCds(problem) =>
                format!("the residue position names CDS '{}' but CDS '{}' was supplied",
                    problem.actual, problem.expected),
            DomainError::UnsupportedNumberingScheme(problem) =>
                format!("CDS '{}' requests numbering {}; only numbering from the CDS start is supported",
                    problem.cds, problem.numbering),
            DomainError::FeatureWithoutIntervals(problem) =>
                format!("feature '{}' on '{}' has no intervals",
                    problem.feature, problem.sequence),
            DomainError::IntervalsOutOfOrder(problem) =>
                format!("intervals of feature '{}' on '{}' are not in increasing biological order",
                    problem.feature, problem.sequence),
            DomainError::OverlappingIntervals(problem) =>
                format!("interval [{}, {}) of feature '{}' on '{}' overlaps the previous interval",
                    problem.start, problem.end, problem.feature, problem.sequence),
            DomainError::UnsupportedFeatureStructure(problem) =>
                format!("feature '{}' on '{}' has {} intervals and {}",
                    problem.feature, problem.sequence, problem.interval_count, problem.reason),
            DomainError::MissingCdsAttribute(problem) => match &problem.value {
                None => format!("CDS '{}' does not supply {}", problem.cds, problem.attribute),
                Some(value) => format!("CDS '{}' supplies {} '{}', which is not a recognised value",
                    problem.cds, problem.attribute, value),
            },
            DomainError::IncompleteCds(problem) =>
                format!("CDS '{}' is declared {}; only explicitly complete CDSs are supported",
                    problem.cds, problem.completeness),
            DomainError::UnsupportedGeneticCode(problem) =>
                format!("CDS '{}' requests genetic code '{}'; only the standard genetic code is supported",
                    problem.cds, problem.genetic_code),
            DomainError::CdsLengthNotDivisibleByThree(problem) =>
                format!("extracted CDS '{}' is {} nucleotides, which is not a multiple of three",
                    problem.cds, problem.length),
            DomainError::AmbiguousCodon(problem) =>
                format!("codon {} ('{}') of CDS '{}' is ambiguous; translation would require guessing",
                    problem.codon_number, problem.codon, problem.cds),
            DomainError::InvalidCodon(problem) =>
                format!("codon {} ('{}') of CDS '{}' is outside the DNA alphabet",
                    problem.codon_number, problem.codon, problem.cds),
            DomainError::InternalStopCodon(problem) =>
                format!("codon {} ('{}') of CDS '{}' is a stop codon but is not the final codon",
                    problem.codon_number, problem.codon, problem.cds),
            DomainError::InvalidMutationRequest(problem) =>
                format!("mutation request '{}' is not representable: {}", problem.request, problem.reason),
            DomainError::UnspecifiedCodonPolicy(problem) =>
                format!("the residue mutation on CDS '{}' does not say how to choose the replacement codon",
                    problem.cds),
            DomainError::UnsupportedMutationPolicy(problem) =>
                format!("codon policy '{}' is not supported; only default and supplied:XXX are supported",
                    problem.policy),
            DomainError::UnsupportedComposition(problem) =>
                format!("the request composes {} variants as {}; {}; no variant was executed",
                    problem.variant_count, problem.composition, problem.reason),
            DomainError::ExpectedNucleotideMismatch(problem) =>
                format!("the nucleotide at position {} of '{}' is {}, but the mutation expected {}",
                    problem.position, problem.sequence, problem.actual, problem.expected),
            DomainError::ExpectedResidueMismatch(problem) =>
                format!("residue {} of CDS '{}' is {}, but the mutation expected {}",
                    problem.number, problem.cds, problem.actual, problem.expected),
            DomainError::TargetOutsideFeature(problem) =>
                format!("interval [{}, {}) on '{}' lies outside feature '{}'",
                    problem.start, problem.end, problem.sequence, problem.feature),
            DomainError::MutationVerificationFailure(problem) =>
                format!("mutation result '{}' does not follow from its source: {}", problem.result, problem.reason),
            DomainError::InvalidPrimerCandidate(problem) =>
                format!("primer candidate '{}' cannot be represented: {}", problem.primer, problem.reason),
            DomainError::PrimerTextAtSpanMismatch(problem) =>
                format!("primer '{}' does not hold the sequence the contract puts at [{}): expected '{}', found '{}'",
                    problem.primer, problem.span, problem.expected, problem.actual),
            DomainError::PrimerAnnealingNotTemplate(problem) =>
                format!("primer '{}': its annealing portion at template position {} is '{}' where the template holds '{}'",
                    problem.primer, problem.position, problem.actual, problem.expected),
            DomainError::PrimerPartition(problem) =>
                format!("primer '{}': its parts do not partition its text: {}", problem.primer, problem.reason),
            DomainError::AnnealingOverlapBelowBound(problem) =>
                format!("primer '{}' anneals {} positions, below the supplied bound of {}",
                    problem.primer, problem.overlap, problem.bound),
            DomainError::PrimerPairCoverage(problem) =>
                format!("pair '{}' leaves the changed positions {} uncovered; it covers {}",
                    problem.pair, problem.changed, problem.uncovered),
            DomainError::PrimerOrientationConflict(problem) =>
                format!("primer pair states orientations {} and {}, which are not opposite", problem.forward, problem.reverse),
            DomainError::UnsupportedPrimerPlacement(problem) =>
                format!("primer placement {} is not supported: {}", problem.placement, problem.reason),
            DomainError::InvalidPrimerDesignRequest(problem) =>
                format!("the primer-design request's {} cannot be used: {}", problem.field, problem.reason),
            DomainError::UnsupportedPrimerDesignScope(problem) =>
                format!("design policy {} does not cover this request: {}", problem.policy, problem.reason),
            DomainError::NoPrimerDesignPossible(problem) =>
                format!("no primer pair fits '{}': it has {} bases and the smallest pair spans {}",
                    problem.template, problem.template_length, problem.smallest_pair),
            DomainError::InvalidBufferModel(problem) => format!(
                "the buffer model's {} cannot be used: {}",
                problem.field, problem.reason
            ),
            DomainError::TmParametersUnavailable(problem) => format!(
                "{} has no parameters for this duplex ({}): {}",
                problem.function, problem.aspect, problem.reason
            ),
            DomainError::NoPrimerDesignFits(problem) => {
                let reasons: Vec<String> =
                    problem.obstacles.iter().map(DesignObstacle::describe).collect();
                format!("no primer pair fits the edit [{}, {}) on '{}': {}",
                    problem.edit_start, problem.edit_end, problem.template, reasons.join("; "))
            }
            DomainError::DegenerateExpansionTooLarge(problem) =>
                format!("'{}' denotes more concrete sequences than can be counted",
                    problem.replacement),
            DomainError::MalformedFasta(problem) => {
                let line: String = match problem.line {
                    Some(number) => number.to_string(),
                    None => "?".to_string(),
                };
                match problem.rule {
                    FastaRule::NoRecord =>
                        format!("FASTA text for '{}' holds no record: it has no '>' header line",
                            problem.sequence),
                    FastaRule::TextBeforeHeader =>
                        format!("FASTA text for '{}' has text on line {} before its '>' header line",
                            problem.sequence, line),
                    FastaRule::SecondRecord =>
                        format!("FASTA text for '{}' starts a second record on line {}; only one record is read",
                            problem.sequence, line),
                }
            }
            DomainError::UnusableGenBankLocation(problem) => {
                let reason: String = match problem.rule {
                    GenBankLocationRule::Syntax =>
                        "is not a location luria can read".to_string(),
                    GenBankLocationRule::PositionZero =>
                        "uses position 0; GenBank positions count from 1".to_string(),
                    GenBankLocationRule::RangeEndsBeforeStart =>
                        "has a range that ends before it starts".to_string(),
                    GenBankLocationRule::PastSequenceEnd =>
                        format!("reaches past the end of the {}-base sequence",
                            problem.sequence_length),
                    GenBankLocationRule::FuzzyEnd =>
                        "has a fuzzy end (< or >), which marks an incomplete feature".to_string(),
                    GenBankLocationRule::BetweenBases =>
                        "names a site between two bases (^), not a range of bases".to_string(),
                    GenBankLocationRule::Order =>
                        "uses order(), whose parts have no stated connection".to_string(),
                    GenBankLocationRule::RemoteReference =>
                        "refers to another record".to_string(),
                    GenBankLocationRule::SinglePosition =>
                        "is a single base, not a range".to_string(),
                    GenBankLocationRule::OriginJoinOnLinear =>
                        "joins across the origin, but the sequence is linear".to_string(),
                    GenBankLocationRule::UnsupportedJoin =>
                        "is a join luria does not represent: only a join that crosses the origin of a circular sequence is read".to_string(),
                };
                format!("GenBank location '{}' {}", problem.location, reason)
            }
            DomainError::MalformedGenBank(problem) => {
                let reason: String = match problem.rule {
                    GenBankRule::NoLocus =>
                        "it does not start with a LOCUS line".to_string(),
                    GenBankRule::LocusLength =>
                        "the LOCUS line does not give a length in bp".to_string(),
                    GenBankRule::TopologyMissing =>
                        "the LOCUS line states neither linear nor circular".to_string(),
                    GenBankRule::TopologyConflict =>
                        "the LOCUS line states more than one topology".to_string(),
                    GenBankRule::RepeatedSection =>
                        "a section that appears once per record appears again".to_string(),
                    GenBankRule::LineLayout =>
                        "a line fits no part of the record layout".to_string(),
                    GenBankRule::QualifierSyntax =>
                        "a qualifier is not /name or /name=value, or its quoted value is not closed".to_string(),
                    GenBankRule::NoOrigin =>
                        "the record has no ORIGIN sequence".to_string(),
                    GenBankRule::OriginLine =>
                        "a line in ORIGIN is not a position number followed by bases".to_string(),
                    GenBankRule::OriginNumbering =>
                        "a line in ORIGIN is numbered other than one past the bases before it".to_string(),
                    GenBankRule::LengthMismatch =>
                        "the LOCUS line's length differs from the ORIGIN sequence's".to_string(),
                    GenBankRule::MissingEnd =>
                        "the record does not end with a // line".to_string(),
                    GenBankRule::SecondRecord =>
                        "a second record starts after //; only one record is read".to_string(),
                    GenBankRule::TextAfterEnd =>
                        "text follows the record's // line".to_string(),
                };
                let located: String = match problem.line {
                    Some(number) => format!(" (line {})", number),
                    None => "".to_string(),
                };
                format!("GenBank text for '{}': {}{}", problem.sequence, reason, located)
            }
            DomainError::UnusableGenBankCds(problem) => {
                let located: String = match problem.line {
                    Some(number) => format!(" (line {})", number),
                    None => "".to_string(),
                };
                let stated: String = match &problem.detail {
                    Some(text) => text.to_string(),
                    None => "(no value)".to_string(),
                };
                match problem.rule {
                    GenBankCdsRule::NotFound =>
                        format!("GenBank CDS '{}' is not the id of any CDS feature in the record{}",
                            problem.cds, located),
                    GenBankCdsRule::AmbiguousName =>
                        format!("GenBank CDS '{}' is the id of more than one CDS feature{}",
                            problem.cds, located),
                    GenBankCdsRule::RepeatedName =>
                        format!("GenBank CDS '{}' is named more than once in the request{}",
                            problem.cds, located),
                    GenBankCdsRule::NoId =>
                        format!("a GenBank CDS has none of /locus_tag, /gene or /label with a value, so it cannot be named{}",
                            located),
                    GenBankCdsRule::RepeatedQualifier =>
                        format!("GenBank CDS '{}' repeats /{}{}", problem.cds, stated, located),
                    GenBankCdsRule::Pseudo =>
                        format!("GenBank CDS '{}' is marked /{}, so it is not a coding sequence luria edits{}",
                            problem.cds, stated, located),
                    GenBankCdsRule::CodonStart =>
                        format!("GenBank CDS '{}' has /codon_start {}; only 1 is supported{}",
                            problem.cds, stated, located),
                    GenBankCdsRule::TranslTable =>
                        format!("GenBank CDS '{}' has /transl_table {}; only 1, and 11 with an ATG start, are supported{}",
                            problem.cds, stated, located),
                    GenBankCdsRule::AlternativeStart =>
                        format!("GenBank CDS '{}' starts with {}; luria imports a CDS only from an ATG start, under every /transl_table (D042){}",
                            problem.cds, stated, located),
                }
            }
            DomainError::GenBankTranslationMismatch(problem) => {
                let stated: String = match &problem.stated {
                    Some(letter) => letter.to_string(),
                    None => "nothing".to_string(),
                };
                let translated: String = match &problem.translated {
                    Some(letter) => letter.to_string(),
                    None => "nothing".to_string(),
                };
                format!("GenBank CDS '{}' states a /translation that differs from luria's at residue {}: the record has {}, luria has {} (line {})",
                    problem.cds, problem.residue, stated, translated, problem.line)
            }
            DomainError::UnknownRecordFormat(problem) =>
                format!("cannot tell what format record '{}' is written in: its first non-blank line starts with neither 'LOCUS' (GenBank) nor '>' (FASTA)",
                    problem.sequence),
            DomainError::RecordTooLong(problem) =>
                format!("record '{}' holds {} bases, over the limit of {} bases luria reads",
                    problem.sequence, problem.length, problem.limit),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_does_not_need_message_text() {
        let mismatch = DomainError::SequenceReferenceMismatch(SequenceReferenceMismatchProblem {
            expected_identifier: "seq-a".to_string(),
            actual_identifier: "seq-b".to_string(),
            expected_digest: "a83f6d82".to_string(),
            actual_digest: "748c00c0".to_string(),
            expected_topology: SequenceTopology::Linear,
            actual_topology: SequenceTopology::Linear,
        });
        assert!(mismatch.category() == ErrorCategory::BiologicalValidation);
        assert!(!mismatch.category().is_malformed_input());
        assert!(!mismatch.category().is_unsupported());
    }

    #[test]
    fn malformed_and_unsupported_are_distinct() {
        assert!(
            DomainError::EmptyInterval(EmptyIntervalProblem {
                sequence: "seq-a".to_string(),
                position: 6
            })
            .category()
                == ErrorCategory::MalformedInput
        );
        assert!(
            DomainError::IncompleteCds(IncompleteCdsProblem {
                cds: "cds-1".to_string(),
                completeness: "incomplete".to_string()
            })
            .category()
                == ErrorCategory::Unsupported
        );
        assert!(
            DomainError::InternalStopCodon(InternalStopCodonProblem {
                cds: "cds-1".to_string(),
                codon_number: 2,
                codon: "TAA".to_string()
            })
            .category()
                == ErrorCategory::BiologicalValidation
        );
    }

    #[test]
    fn every_category_is_reachable() {
        let categories = [
            DomainError::EmptyIdentifier.category(),
            DomainError::CoordinateOutOfBounds(CoordinateOutOfBoundsProblem {
                sequence: "seq-a".to_string(),
                start: 0,
                end: 31,
                length: 30,
            })
            .category(),
            DomainError::UnsupportedGeneticCode(UnsupportedGeneticCodeProblem {
                cds: "cds-1".to_string(),
                genetic_code: "iso1557".to_string(),
            })
            .category(),
            DomainError::UnsupportedNumberingScheme(UnsupportedNumberingProblem {
                cds: "cds-1".to_string(),
                numbering: "explicit-mapping".to_string(),
            })
            .category(),
            DomainError::InvalidResiduePosition(ResiduePositionProblem {
                cds: "cds-1".to_string(),
                number: 0,
            })
            .category(),
            DomainError::ResiduePositionOutsideCds(ResiduePositionRangeProblem {
                cds: "cds-1".to_string(),
                number: 9,
                residue_count: 5,
            })
            .category(),
            DomainError::ResiduePositionWrongCds(CdsMismatchProblem {
                expected: "cds-1".to_string(),
                actual: "cds-2".to_string(),
            })
            .category(),
            DomainError::InvalidCodon(InvalidCodonProblem {
                cds: "cds-1".to_string(),
                codon_number: 2,
                codon: "UAA".to_string(),
            })
            .category(),
        ];
        assert!(categories[0] == ErrorCategory::MalformedInput);
        assert!(categories[1] == ErrorCategory::BiologicalValidation);
        assert!(categories[2] == ErrorCategory::Unsupported);
        assert!(categories[3] == ErrorCategory::Unsupported);
        assert!(categories[4] == ErrorCategory::MalformedInput);
        assert!(categories[5] == ErrorCategory::BiologicalValidation);
        assert!(categories[6] == ErrorCategory::BiologicalValidation);
        assert!(categories[7] == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn name_and_content_mismatch_are_the_same_failure_class() {
        let name_differs =
            DomainError::SequenceReferenceMismatch(SequenceReferenceMismatchProblem {
                expected_identifier: "seq-a".to_string(),
                actual_identifier: "seq-b".to_string(),
                expected_digest: "a83f6d82".to_string(),
                actual_digest: "748c00c0".to_string(),
                expected_topology: SequenceTopology::Linear,
                actual_topology: SequenceTopology::Linear,
            });
        let content_differs =
            DomainError::SequenceReferenceMismatch(SequenceReferenceMismatchProblem {
                expected_identifier: "seq-a".to_string(),
                actual_identifier: "seq-a".to_string(),
                expected_digest: "a83f6d82".to_string(),
                actual_digest: "748c00c0".to_string(),
                expected_topology: SequenceTopology::Linear,
                actual_topology: SequenceTopology::Linear,
            });
        assert!(name_differs.category() == ErrorCategory::BiologicalValidation);
        assert!(content_differs.category() == ErrorCategory::BiologicalValidation);
        assert!(name_differs != content_differs);
    }

    // A missing codon policy is something the caller can supply; an
    // unrecognised one is out of scope. They must not share a category.
    #[test]
    fn mutation_failures_are_classified_by_meaning() {
        let missing_policy = DomainError::UnspecifiedCodonPolicy(UnspecifiedCodonPolicyProblem {
            cds: "cds-1".to_string(),
        });
        let out_of_scope =
            DomainError::UnsupportedMutationPolicy(UnsupportedMutationPolicyProblem {
                policy: "minimal-edit".to_string(),
            });
        assert!(missing_policy.category() == ErrorCategory::MalformedInput);
        assert!(out_of_scope.category() == ErrorCategory::Unsupported);

        // The request is well-formed; the artifact disagrees with it.
        let residue_mismatch =
            DomainError::ExpectedResidueMismatch(ExpectedResidueMismatchProblem {
                cds: "cds-1".to_string(),
                number: 42,
                expected: "N".to_string(),
                actual: "D".to_string(),
            });
        assert!(residue_mismatch.category() == ErrorCategory::BiologicalValidation);

        let nucleotide_mismatch =
            DomainError::ExpectedNucleotideMismatch(ExpectedNucleotideMismatchProblem {
                sequence: "seq-a".to_string(),
                position: 1209,
                expected: "G".to_string(),
                actual: "A".to_string(),
            });
        assert!(nucleotide_mismatch.category() == ErrorCategory::BiologicalValidation);

        let outside_scope = DomainError::TargetOutsideFeature(TargetOutsideFeatureProblem {
            feature: "cds-1".to_string(),
            sequence: "seq-a".to_string(),
            start: 12,
            end: 13,
        });
        assert!(outside_scope.category() == ErrorCategory::BiologicalValidation);

        let verification =
            DomainError::MutationVerificationFailure(MutationVerificationFailureProblem {
                result: "mut-1".to_string(),
                reason: "the edited artifact differs at 2 positions, not 1".to_string(),
            });
        assert!(verification.category() == ErrorCategory::BiologicalValidation);

        // A bad replacement letter is malformed input, not a world disagreement.
        let bad_request = DomainError::InvalidMutationRequest(InvalidMutationRequestProblem {
            request: "nucleotide".to_string(),
            rule: MutationRequestRule::BaseNotDnaLetter,
            reason: "the replacement is not a DNA letter".to_string(),
        });
        assert!(bad_request.category() == ErrorCategory::MalformedInput);
    }

    // The codes are the CLI's output contract (D041). This pins every one,
    // and checks that no two variants share a code.
    #[test]
    fn every_variant_has_its_pinned_code() {
        let s = || "x".to_string();
        let cases: Vec<(DomainError, &str)> = vec![
            (DomainError::EmptyIdentifier, "empty_identifier"),
            (
                DomainError::InvalidIdentifierCharacters(InvalidIdentifierProblem {
                    identifier: s(),
                    reason: s(),
                }),
                "invalid_identifier_characters",
            ),
            (
                DomainError::EmptySequenceContent(EmptySequenceProblem { sequence: s() }),
                "empty_sequence_content",
            ),
            (
                DomainError::InvalidSequenceCharacters(InvalidSequenceCharactersProblem {
                    sequence: s(),
                    position: 0,
                    characters: s(),
                }),
                "invalid_sequence_characters",
            ),
            (DomainError::EmptySearchPattern, "empty_search_pattern"),
            (
                DomainError::InvalidSearchPatternCharacters(
                    InvalidSearchPatternCharactersProblem {
                        pattern: s(),
                        position: 0,
                        characters: s(),
                    },
                ),
                "invalid_search_pattern_characters",
            ),
            (
                DomainError::NegativeCoordinate(NegativeCoordinateProblem {
                    sequence: s(),
                    start: -1,
                    end: 0,
                }),
                "negative_coordinate",
            ),
            (
                DomainError::ReversedInterval(ReversedIntervalProblem {
                    sequence: s(),
                    start: 2,
                    end: 1,
                }),
                "reversed_interval",
            ),
            (
                DomainError::EmptyInterval(EmptyIntervalProblem {
                    sequence: s(),
                    position: 0,
                }),
                "empty_interval",
            ),
            (
                DomainError::CoordinateOutOfBounds(CoordinateOutOfBoundsProblem {
                    sequence: s(),
                    start: 0,
                    end: 2,
                    length: 1,
                }),
                "coordinate_out_of_bounds",
            ),
            (
                DomainError::SequenceReferenceMismatch(SequenceReferenceMismatchProblem {
                    expected_identifier: s(),
                    actual_identifier: s(),
                    expected_digest: s(),
                    actual_digest: s(),
                    expected_topology: SequenceTopology::Linear,
                    actual_topology: SequenceTopology::Linear,
                }),
                "sequence_reference_mismatch",
            ),
            (
                DomainError::FeatureWithoutIntervals(FeatureWithoutIntervalsProblem {
                    feature: s(),
                    sequence: s(),
                }),
                "feature_without_intervals",
            ),
            (
                DomainError::IntervalsOutOfOrder(IntervalsOutOfOrderProblem {
                    feature: s(),
                    sequence: s(),
                }),
                "intervals_out_of_order",
            ),
            (
                DomainError::OverlappingIntervals(OverlappingIntervalsProblem {
                    feature: s(),
                    sequence: s(),
                    start: 0,
                    end: 1,
                }),
                "overlapping_intervals",
            ),
            (
                DomainError::UnsupportedFeatureStructure(UnsupportedFeatureStructureProblem {
                    feature: s(),
                    sequence: s(),
                    interval_count: 2,
                    reason: s(),
                }),
                "unsupported_feature_structure",
            ),
            (
                DomainError::MissingCdsAttribute(MissingCdsAttributeProblem {
                    cds: s(),
                    attribute: s(),
                    value: None,
                }),
                "missing_cds_attribute",
            ),
            (
                DomainError::IncompleteCds(IncompleteCdsProblem {
                    cds: s(),
                    completeness: s(),
                }),
                "incomplete_cds",
            ),
            (
                DomainError::UnsupportedGeneticCode(UnsupportedGeneticCodeProblem {
                    cds: s(),
                    genetic_code: s(),
                }),
                "unsupported_genetic_code",
            ),
            (
                DomainError::CdsLengthNotDivisibleByThree(CdsLengthNotDivisibleByThreeProblem {
                    cds: s(),
                    length: 4,
                }),
                "cds_length_not_divisible_by_three",
            ),
            (
                DomainError::AmbiguousCodon(AmbiguousCodonProblem {
                    cds: s(),
                    codon_number: 1,
                    codon: s(),
                }),
                "ambiguous_codon",
            ),
            (
                DomainError::InvalidCodon(InvalidCodonProblem {
                    cds: s(),
                    codon_number: 1,
                    codon: s(),
                }),
                "invalid_codon",
            ),
            (
                DomainError::InternalStopCodon(InternalStopCodonProblem {
                    cds: s(),
                    codon_number: 1,
                    codon: s(),
                }),
                "internal_stop_codon",
            ),
            (
                DomainError::InvalidResiduePosition(ResiduePositionProblem {
                    cds: s(),
                    number: 0,
                }),
                "invalid_residue_position",
            ),
            (
                DomainError::ResiduePositionOutsideCds(ResiduePositionRangeProblem {
                    cds: s(),
                    number: 9,
                    residue_count: 5,
                }),
                "residue_position_outside_cds",
            ),
            (
                DomainError::ResiduePositionWrongCds(CdsMismatchProblem {
                    expected: s(),
                    actual: s(),
                }),
                "residue_position_wrong_cds",
            ),
            (
                DomainError::UnsupportedNumberingScheme(UnsupportedNumberingProblem {
                    cds: s(),
                    numbering: s(),
                }),
                "unsupported_numbering_scheme",
            ),
            (
                DomainError::InvalidMutationRequest(InvalidMutationRequestProblem {
                    request: s(),
                    rule: MutationRequestRule::NoChange,
                    reason: s(),
                }),
                "invalid_mutation_request",
            ),
            (
                DomainError::UnspecifiedCodonPolicy(UnspecifiedCodonPolicyProblem { cds: s() }),
                "unspecified_codon_policy",
            ),
            (
                DomainError::UnsupportedMutationPolicy(UnsupportedMutationPolicyProblem {
                    policy: s(),
                }),
                "unsupported_mutation_policy",
            ),
            (
                DomainError::UnsupportedComposition(UnsupportedCompositionProblem {
                    composition: s(),
                    variant_count: 2,
                    reason: s(),
                }),
                "unsupported_composition",
            ),
            (
                DomainError::ExpectedNucleotideMismatch(ExpectedNucleotideMismatchProblem {
                    sequence: s(),
                    position: 0,
                    expected: s(),
                    actual: s(),
                }),
                "expected_nucleotide_mismatch",
            ),
            (
                DomainError::ExpectedResidueMismatch(ExpectedResidueMismatchProblem {
                    cds: s(),
                    number: 1,
                    expected: s(),
                    actual: s(),
                }),
                "expected_residue_mismatch",
            ),
            (
                DomainError::TargetOutsideFeature(TargetOutsideFeatureProblem {
                    feature: s(),
                    sequence: s(),
                    start: 0,
                    end: 1,
                }),
                "target_outside_feature",
            ),
            (
                DomainError::MutationVerificationFailure(MutationVerificationFailureProblem {
                    result: s(),
                    reason: s(),
                }),
                "mutation_verification_failure",
            ),
            (
                DomainError::InvalidPrimerCandidate(InvalidPrimerCandidateProblem {
                    primer: s(),
                    rule: PrimerCandidateRule::EmptyTail,
                    reason: s(),
                }),
                "invalid_primer_candidate",
            ),
            (
                DomainError::PrimerTextAtSpanMismatch(PrimerTextAtSpanMismatchProblem {
                    primer: s(),
                    span: s(),
                    expected: s(),
                    actual: s(),
                }),
                "primer_text_at_span_mismatch",
            ),
            (
                DomainError::PrimerAnnealingNotTemplate(PrimerAnnealingNotTemplateProblem {
                    primer: s(),
                    position: 0,
                    expected: s(),
                    actual: s(),
                }),
                "primer_annealing_not_template",
            ),
            (
                DomainError::PrimerPartition(PrimerPartitionProblem {
                    primer: s(),
                    reason: s(),
                }),
                "primer_partition",
            ),
            (
                DomainError::AnnealingOverlapBelowBound(AnnealingOverlapBelowBoundProblem {
                    primer: s(),
                    overlap: 1,
                    bound: 2,
                }),
                "annealing_overlap_below_bound",
            ),
            (
                DomainError::PrimerPairCoverage(PrimerPairCoverageProblem {
                    pair: s(),
                    changed: s(),
                    uncovered: s(),
                }),
                "primer_pair_coverage",
            ),
            (
                DomainError::PrimerOrientationConflict(PrimerOrientationConflictProblem {
                    forward: s(),
                    reverse: s(),
                }),
                "primer_orientation_conflict",
            ),
            (
                DomainError::UnsupportedPrimerPlacement(UnsupportedPrimerPlacementProblem {
                    placement: s(),
                    reason: s(),
                }),
                "unsupported_primer_placement",
            ),
            (
                DomainError::InvalidPrimerDesignRequest(InvalidPrimerDesignRequestProblem {
                    field: s(),
                    reason: s(),
                }),
                "invalid_primer_design_request",
            ),
            (
                DomainError::UnsupportedPrimerDesignScope(UnsupportedPrimerDesignScopeProblem {
                    policy: s(),
                    reason: s(),
                }),
                "unsupported_primer_design_scope",
            ),
            (
                DomainError::NoPrimerDesignPossible(NoPrimerDesignPossibleProblem {
                    template: s(),
                    template_length: 10,
                    smallest_pair: 20,
                }),
                "no_primer_design_possible",
            ),
            (
                DomainError::NoPrimerDesignFits(NoPrimerDesignFitsProblem {
                    template: s(),
                    edit_start: 0,
                    edit_end: 1,
                    obstacles: Vec::new(),
                }),
                "no_primer_design_fits",
            ),
            (
                DomainError::InvalidBufferModel(InvalidBufferModelProblem {
                    field: s(),
                    reason: s(),
                }),
                "invalid_buffer_model",
            ),
            (
                DomainError::TmParametersUnavailable(TmParametersUnavailableProblem {
                    function: s(),
                    aspect: s(),
                    reason: s(),
                }),
                "tm_parameters_unavailable",
            ),
            (
                DomainError::DegenerateExpansionTooLarge(DegenerateExpansionTooLargeProblem {
                    replacement: s(),
                }),
                "degenerate_expansion_too_large",
            ),
            (
                DomainError::MalformedFasta(MalformedFastaProblem {
                    sequence: s(),
                    rule: FastaRule::NoRecord,
                    line: None,
                }),
                "malformed_fasta",
            ),
            (
                DomainError::UnusableGenBankLocation(UnusableGenBankLocationProblem {
                    location: s(),
                    rule: GenBankLocationRule::Syntax,
                    sequence_length: 1,
                    cds: None,
                    line: None,
                }),
                "unusable_genbank_location",
            ),
            (
                DomainError::MalformedGenBank(MalformedGenBankProblem {
                    sequence: s(),
                    rule: GenBankRule::NoLocus,
                    line: None,
                }),
                "malformed_genbank",
            ),
            (
                DomainError::UnusableGenBankCds(UnusableGenBankCdsProblem {
                    cds: s(),
                    rule: GenBankCdsRule::NotFound,
                    line: None,
                    detail: None,
                }),
                "unusable_genbank_cds",
            ),
            (
                DomainError::GenBankTranslationMismatch(GenBankTranslationMismatchProblem {
                    cds: s(),
                    line: 1,
                    residue: 1,
                    stated: None,
                    translated: None,
                }),
                "genbank_translation_mismatch",
            ),
            (
                DomainError::UnknownRecordFormat(UnknownRecordFormatProblem { sequence: s() }),
                "unknown_record_format",
            ),
            (
                DomainError::RecordTooLong(RecordTooLongProblem {
                    sequence: s(),
                    length: 0,
                    limit: 0,
                }),
                "record_too_long",
            ),
        ];
        assert_eq!(cases.len(), 56);
        let mut seen = std::collections::BTreeSet::new();
        for (error, code) in &cases {
            assert_eq!(error.code(), *code, "{:?}", error);
            assert!(seen.insert(error.code()), "duplicate code {}", error.code());
            assert!(code
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));
        }
    }

    // The rule codes are the CLI's output contract (D041). This pins every
    // one, and checks that no two rules share a code.
    #[test]
    fn every_mutation_request_rule_has_its_pinned_code() {
        use MutationRequestRule::*;
        let cases: [(MutationRequestRule, &str); 20] = [
            (CodonIsStop, "codon_is_stop"),
            (CodonIsAmbiguous, "codon_is_ambiguous"),
            (CodonNotThreeDnaLetters, "codon_not_three_dna_letters"),
            (SuppliedCodonSyntax, "supplied_codon_syntax"),
            (ResidueNotOneLetter, "residue_not_one_letter"),
            (ResidueNotInStandardCode, "residue_not_in_standard_code"),
            (TargetOnOtherSequence, "target_on_other_sequence"),
            (
                SubstitutionNotOneNucleotide,
                "substitution_not_one_nucleotide",
            ),
            (ExpectedTextLengthMismatch, "expected_text_length_mismatch"),
            (NothingWritten, "nothing_written"),
            (NoChange, "no_change"),
            (WholeSequence, "whole_sequence"),
            (InsertionAtSequenceEnd, "insertion_at_sequence_end"),
            (SuppliedCodonWrongResidue, "supplied_codon_wrong_residue"),
            (NoDefaultCodon, "no_default_codon"),
            (CodonOutsideSequence, "codon_outside_sequence"),
            (BaseNotOneLetter, "base_not_one_letter"),
            (BaseNotDnaLetter, "base_not_dna_letter"),
            (EmptyVariantSet, "empty_variant_set"),
            (DuplicateVariantName, "duplicate_variant_name"),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (rule, code) in cases {
            assert_eq!(rule.code(), code, "{:?}", rule);
            assert!(seen.insert(code), "duplicate code {}", code);
        }
    }

    // The rule codes are the CLI's output contract (D041). This pins every
    // one, and checks that no two rules share a code.
    #[test]
    fn every_primer_candidate_rule_has_its_pinned_code() {
        use PrimerCandidateRule::*;
        let cases: [(PrimerCandidateRule, &str); 13] = [
            (SpanOverlapsDeletion, "span_overlaps_deletion"),
            (
                SpanStraddlesInsertionPoint,
                "span_straddles_insertion_point",
            ),
            (SpanOverlapsReplacement, "span_overlaps_replacement"),
            (TextLengthNotSpanWidth, "text_length_not_span_width"),
            (
                ForwardSpanDoesNotCoverReplacement,
                "forward_span_does_not_cover_replacement",
            ),
            (EditTakesNoTail, "edit_takes_no_tail"),
            (EmptyTail, "empty_tail"),
            (RoleDoesNotMatchSlot, "role_does_not_match_slot"),
            (
                PrimersOnDifferentTemplates,
                "primers_on_different_templates",
            ),
            (PairNotOnEditTemplate, "pair_not_on_edit_template"),
            (PairNotAbuttingEdit, "pair_not_abutting_edit"),
            (MinimumOverlapBelowOne, "minimum_overlap_below_one"),
            (TailsDoNotRebuild, "tails_do_not_rebuild"),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (rule, code) in cases {
            assert_eq!(rule.code(), code, "{:?}", rule);
            assert!(seen.insert(code), "duplicate code {}", code);
        }
    }

    #[test]
    fn every_fasta_rule_has_its_pinned_code() {
        use FastaRule::*;
        let cases: [(FastaRule, &str); 3] = [
            (NoRecord, "no_record"),
            (TextBeforeHeader, "text_before_header"),
            (SecondRecord, "second_record"),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (rule, code) in cases {
            assert_eq!(rule.code(), code, "{:?}", rule);
            assert!(seen.insert(code), "duplicate code {}", code);
        }
    }

    #[test]
    fn every_genbank_location_rule_has_its_pinned_code_and_category() {
        use ErrorCategory::*;
        use GenBankLocationRule::*;
        let cases: [(GenBankLocationRule, &str, ErrorCategory); 11] = [
            (Syntax, "syntax", MalformedInput),
            (PositionZero, "position_zero", MalformedInput),
            (
                RangeEndsBeforeStart,
                "range_ends_before_start",
                MalformedInput,
            ),
            (PastSequenceEnd, "past_sequence_end", BiologicalValidation),
            (FuzzyEnd, "fuzzy_end", Unsupported),
            (BetweenBases, "between_bases", Unsupported),
            (Order, "order", Unsupported),
            (RemoteReference, "remote_reference", Unsupported),
            (SinglePosition, "single_position", Unsupported),
            (OriginJoinOnLinear, "origin_join_on_linear", Unsupported),
            (UnsupportedJoin, "unsupported_join", Unsupported),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (rule, code, category) in cases {
            assert_eq!(rule.code(), code, "{:?}", rule);
            assert!(rule.category() == category, "{:?}", rule);
            assert!(seen.insert(code), "duplicate code {}", code);
        }
    }

    #[test]
    fn every_genbank_rule_has_its_pinned_code() {
        use GenBankRule::*;
        let cases: [(GenBankRule, &str); 14] = [
            (NoLocus, "no_locus"),
            (LocusLength, "locus_length"),
            (TopologyMissing, "topology_missing"),
            (TopologyConflict, "topology_conflict"),
            (RepeatedSection, "repeated_section"),
            (LineLayout, "line_layout"),
            (QualifierSyntax, "qualifier_syntax"),
            (NoOrigin, "no_origin"),
            (OriginLine, "origin_line"),
            (OriginNumbering, "origin_numbering"),
            (LengthMismatch, "length_mismatch"),
            (MissingEnd, "missing_end"),
            (SecondRecord, "second_record"),
            (TextAfterEnd, "text_after_end"),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (rule, code) in cases {
            assert_eq!(rule.code(), code, "{:?}", rule);
            assert!(seen.insert(code), "duplicate code {}", code);
        }
    }

    #[test]
    fn every_genbank_cds_rule_has_its_pinned_code_and_category() {
        use ErrorCategory::*;
        use GenBankCdsRule::*;
        let cases: [(GenBankCdsRule, &str, ErrorCategory); 9] = [
            (NotFound, "not_found", BiologicalValidation),
            (AmbiguousName, "ambiguous_name", BiologicalValidation),
            (RepeatedName, "repeated_name", MalformedInput),
            (NoId, "no_id", Unsupported),
            (RepeatedQualifier, "repeated_qualifier", MalformedInput),
            (Pseudo, "pseudo", Unsupported),
            (CodonStart, "codon_start", Unsupported),
            (TranslTable, "transl_table", Unsupported),
            (AlternativeStart, "alternative_start", Unsupported),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (rule, code, category) in cases {
            assert_eq!(rule.code(), code, "{:?}", rule);
            assert!(rule.category() == category, "{:?}", rule);
            assert!(seen.insert(code), "duplicate code {}", code);
        }
    }
}
