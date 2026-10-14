//! Invariants and regressions for the mutation domain.
//!
//! Nothing here is derived from the implementation. The inputs are three
//! synthetic sequences and the expectations are worked out from the published
//! standard genetic code (`transl_table=1`, base order `T, C, A, G`, index
//! `16 * place(b1) + 4 * place(b2) + place(b3)`):
//!
//! * `mut-a` = `ATGGCTTGGAATCAGCAGCAGCAGCAGCAG`, codons `ATG GCT TGG AAT CAG
//!   CAG CAG CAG CAG CAG`, residues `M A W N Q Q Q Q Q Q`.
//! * `mut-b` = `ATTCCAAGCCATCAGCAGCAGCAGCAGCAG`, a different artifact that the
//!   fixtures give the same display name as, to test identity checks.
//! * `mut-c` = `GCTTGGAACGAACGAACGAACATG`, circular, 24 bases.
//!
//! The tests that pin coordinates and strand write-back name their codon
//! explicitly (`supplied:TTA` for `L`) so that the hand-derived edits do not
//! depend on the codon policy. `W` has a single codon, `TGG`, so `default`
//! and `supplied:TGG` write the same triplet.
//!
//! The `default` policy picks the highest-count codon in the *E. coli* K-12
//! (W3110) table of the Kazusa Codon Usage Database, species 316407: `L` is
//! `CTG` (count 72898, against 18894 for `TTA`), `A` is `GCG`
//! (46524, against 35252 for `GCC`), `W` is `TGG`, the only W codon.

use luria_core::cds::Cds;
use luria_core::cds::ResiduePosition;
use luria_core::error::DomainError;
use luria_core::error::ErrorCategory;
use luria_core::extraction::extract_cds;
use luria_core::genetic_code::Residue;
use luria_core::identity::Identifier;
use luria_core::identity::SequenceReference;
use luria_core::interval::create;
use luria_core::interval::OrderedIntervals;
use luria_core::mutation::apply_nucleotide;
use luria_core::mutation::apply_residue;
use luria_core::mutation::default_codon;
use luria_core::mutation::parse_codon_policy;
use luria_core::mutation::verify;
use luria_core::mutation::CodonSelectionPolicy;
use luria_core::mutation::MutationResult;
use luria_core::mutation::MutationState;
use luria_core::mutation::NucleotideMutationIntent;
use luria_core::mutation::ResidueMutationIntent;
use luria_core::sequence::normalize;
use luria_core::sequence::Sequence;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;
use luria_core::translation::translate;
use luria_core::translation::Translation;

fn text_a() -> String {
    "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".to_string()
}

fn text_b() -> String {
    "ATTCCAAGCCATCAGCAGCAGCAGCAGCAG".to_string()
}

fn text_c() -> String {
    "GCTTGGAACGAACGAACGAACATG".to_string()
}

fn name_id(value: String) -> Identifier {
    let made = Identifier::create(value.clone());
    if made.is_err() {
        panic!("fixture identifier {} is not usable", value);
    }
    made.unwrap()
}

fn artifact(name: String, text: String, circular: bool) -> Sequence {
    let topology: SequenceTopology = SequenceTopology::Linear;
    let chosen: SequenceTopology = if circular {
        SequenceTopology::Circular
    } else {
        topology
    };
    let made = normalize(name_id(name.clone()), text.clone(), chosen);
    if made.is_err() {
        panic!("fixture sequence {} cannot be normalized", name);
    }
    made.unwrap()
}

fn parse_coordinate(text: String) -> i64 {
    let parsed = text.clone().parse();
    if parsed.is_err() {
        panic!("coordinate {} is not an integer", text);
    }
    parsed.unwrap()
}

fn intervals(reference: SequenceReference, spec: String) -> OrderedIntervals {
    let mut out: OrderedIntervals = Vec::new();
    let parts: Vec<String> = spec.split(';').map(|x| x.to_string()).collect();
    for part in parts {
        let bounds: Vec<String> = part.split(':').map(|x| x.to_string()).collect();
        if bounds.len() != 2 {
            panic!("interval {} is not start:end", part);
        }
        let made = create(
            &reference,
            parse_coordinate(bounds[0].clone()),
            parse_coordinate(bounds[1].clone()),
        );
        if made.is_err() {
            panic!("interval {} is not usable", part);
        }
        out.push(made.unwrap());
    }
    out
}

fn cds_of(sequence: &Sequence, name: String, spec: String, strand: Strand) -> Cds {
    let made = Cds::from_optional(
        sequence,
        name_id(name.clone()),
        Some(intervals(sequence.reference(), spec.clone())),
        Some(strand),
        Some("complete".to_string()),
        Some("standard".to_string()),
        Some("from-cds-start".to_string()),
    );
    if made.is_err() {
        panic!("cannot build cds {} over {}", name, spec);
    }
    made.unwrap()
}

fn residue_position(cds: String, number: i64) -> ResiduePosition {
    let made = ResiduePosition::create(name_id(cds.clone()), number);
    if made.is_err() {
        panic!("residue {} of {} is not a position", number, cds);
    }
    made.unwrap()
}

fn nucleotide_intent(
    sequence: &Sequence,
    position: i64,
    expected: String,
    replacement: String,
) -> NucleotideMutationIntent {
    let made = NucleotideMutationIntent::create(
        sequence.reference(),
        create(&sequence.reference(), position, position + 1).unwrap(),
        expected.clone(),
        replacement.clone(),
    );
    if made.is_err() {
        panic!(
            "the nucleotide request {} is not representable",
            replacement
        );
    }
    made.unwrap()
}

fn residue_intent(
    cds: String,
    number: i64,
    expected: String,
    replacement: String,
    policy: String,
) -> ResidueMutationIntent {
    let parsed = parse_codon_policy(policy.clone());
    if parsed.is_err() {
        panic!("the policy {} is not one of the three", policy);
    }
    let made = ResidueMutationIntent::create(
        residue_position(cds.clone(), number),
        expected.clone(),
        replacement.clone(),
        parsed.unwrap(),
    );
    if made.is_err() {
        panic!("the residue request {} is not representable", replacement);
    }
    made.unwrap()
}

fn result_id(name: String) -> Identifier {
    name_id(name.clone())
}

/// The variant name of a domain error, read off the value. Classification
/// itself is only ever read from `DomainError::category()`.
fn variant_name(error: &DomainError) -> String {
    match error {
        DomainError::EmptyIdentifier => "EmptyIdentifier",
        DomainError::InvalidIdentifierCharacters(_) => "InvalidIdentifierCharacters",
        DomainError::EmptySequenceContent(_) => "EmptySequenceContent",
        DomainError::InvalidSequenceCharacters(_) => "InvalidSequenceCharacters",
        DomainError::NegativeCoordinate(_) => "NegativeCoordinate",
        DomainError::ReversedInterval(_) => "ReversedInterval",
        DomainError::EmptyInterval(_) => "EmptyInterval",
        DomainError::CoordinateOutOfBounds(_) => "CoordinateOutOfBounds",
        DomainError::SequenceReferenceMismatch(_) => "SequenceReferenceMismatch",
        DomainError::FeatureWithoutIntervals(_) => "FeatureWithoutIntervals",
        DomainError::IntervalsOutOfOrder(_) => "IntervalsOutOfOrder",
        DomainError::OverlappingIntervals(_) => "OverlappingIntervals",
        DomainError::UnsupportedFeatureStructure(_) => "UnsupportedFeatureStructure",
        DomainError::MissingCdsAttribute(_) => "MissingCdsAttribute",
        DomainError::IncompleteCds(_) => "IncompleteCds",
        DomainError::UnsupportedGeneticCode(_) => "UnsupportedGeneticCode",
        DomainError::CdsLengthNotDivisibleByThree(_) => "CdsLengthNotDivisibleByThree",
        DomainError::AmbiguousCodon(_) => "AmbiguousCodon",
        DomainError::InvalidCodon(_) => "InvalidCodon",
        DomainError::InternalStopCodon(_) => "InternalStopCodon",
        DomainError::InvalidResiduePosition(_) => "InvalidResiduePosition",
        DomainError::ResiduePositionOutsideCds(_) => "ResiduePositionOutsideCds",
        DomainError::ResiduePositionWrongCds(_) => "ResiduePositionWrongCds",
        DomainError::UnsupportedNumberingScheme(_) => "UnsupportedNumberingScheme",
        DomainError::InvalidMutationRequest(_) => "InvalidMutationRequest",
        DomainError::UnspecifiedCodonPolicy(_) => "UnspecifiedCodonPolicy",
        DomainError::UnsupportedMutationPolicy(_) => "UnsupportedMutationPolicy",
        DomainError::ExpectedNucleotideMismatch(_) => "ExpectedNucleotideMismatch",
        DomainError::ExpectedResidueMismatch(_) => "ExpectedResidueMismatch",
        DomainError::TargetOutsideFeature(_) => "TargetOutsideFeature",
        DomainError::MutationVerificationFailure(_) => "MutationVerificationFailure",
    }
    .to_string()
}

fn category_name(category: ErrorCategory) -> String {
    match category {
        ErrorCategory::MalformedInput => "MalformedInput",
        ErrorCategory::BiologicalValidation => "BiologicalValidation",
        ErrorCategory::Unsupported => "Unsupported",
    }
    .to_string()
}

/// Asserts the variant, and asserts the category the real classifier gives it.
fn expect_failure<T>(case: String, result: Result<T, DomainError>, want: String, category: String) {
    match result {
        Err(problem) => {
            let got: String = variant_name(&problem);
            assert!(
                got == want,
                "{}: expected error {}, the model gave {}",
                case,
                want,
                got
            );
            let got_category: String = category_name(problem.category());
            assert!(
                got_category == category,
                "{}: category() gives {} for {}, the invariant expects {}",
                case,
                got_category,
                want,
                category,
            );
        }
        Ok(_) => {
            panic!("{} expected an error but the operation succeeded", case);
        }
    }
}

/// Panics with context if an operation that should succeed failed.
fn report_failure<T>(case: String, result: &Result<T, DomainError>) {
    if let Err(problem) = result {
        panic!(
            "{}: unexpected error {} ({})",
            case,
            variant_name(problem),
            category_name(problem.category()),
        );
    }
}

fn residues_of(translation: &Translation) -> String {
    translation.residues_text()
}

fn written_coordinates(result: &MutationResult) -> Vec<u64> {
    result.target_coordinates()
}

fn diff_positions(before: &Sequence, after: &Sequence) -> Vec<u64> {
    let left: Vec<char> = before.letters();
    let right: Vec<char> = after.letters();
    let mut out: Vec<u64> = Vec::new();
    let count: usize = left.len();
    let mut index: usize = 0;
    while index < count {
        if left[index] != right[index] {
            out.push(index as u64);
        }
        index += 1;
    }
    out
}

#[test]
fn residue_positions_are_one_based_and_bounded() {
    let sequence: Sequence = artifact("mut-a".to_string(), text_a(), false);
    let annotation: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );

    // Residue 0 is not a position: the numbering is one-based.
    let zero = ResiduePosition::create(name_id("cds-1".to_string()), 0);
    expect_failure(
        "residue-zero".to_string(),
        zero,
        "InvalidResiduePosition".to_string(),
        "MalformedInput".to_string(),
    );

    // Residue 1 is the first codon of the strand-applied CDS.
    let first = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-res-1".to_string()),
        residue_intent(
            "cds-1".to_string(),
            1,
            "M".to_string(),
            "L".to_string(),
            "supplied:TTA".to_string(),
        ),
    );
    report_failure("residue-one".to_string(), &first);
    let made: MutationResult = first.unwrap();
    assert!(
        made.edited().canonical_text() == "TTAGCTTGGAATCAGCAGCAGCAGCAGCAG",
        "ATG replaced by the supplied L codon, TTA",
    );
    assert!(
        written_coordinates(&made) == vec![0, 1, 2],
        "the codon of residue 1 occupies reference coordinates 0,1,2",
    );

    // Residue N is the last residue of the CDS: there are ten.
    let last = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-res-10".to_string()),
        residue_intent(
            "cds-1".to_string(),
            10,
            "Q".to_string(),
            "L".to_string(),
            "supplied:TTA".to_string(),
        ),
    );
    report_failure("residue-n".to_string(), &last);
    let tenth: MutationResult = last.unwrap();
    assert!(
        tenth.edited().canonical_text() == "ATGGCTTGGAATCAGCAGCAGCAGCAGTTA",
        "the final CAG becomes TTA",
    );
    assert!(
        written_coordinates(&tenth) == vec![27, 28, 29],
        "the last codon of a 30-base CDS ends at 30",
    );

    // Residue N+1 does not exist in this CDS.
    let beyond = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-res-11".to_string()),
        residue_intent(
            "cds-1".to_string(),
            11,
            "Q".to_string(),
            "L".to_string(),
            "supplied:TTA".to_string(),
        ),
    );
    expect_failure(
        "residue-n-plus-one".to_string(),
        beyond,
        "ResiduePositionOutsideCds".to_string(),
        "BiologicalValidation".to_string(),
    );
}

#[test]
fn a_cds_or_target_that_names_another_artifact_is_refused() {
    let sequence: Sequence = artifact("mut-a".to_string(), text_a(), false);
    let decoy: Sequence = artifact("mut-a".to_string(), text_b(), false);

    // A CDS built against a different artifact that shares the display name.
    let foreign: Cds = cds_of(
        &decoy,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );
    let wrong = apply_residue(
        &sequence,
        foreign.clone(),
        result_id("mut-a-refused".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "A".to_string(),
            "W".to_string(),
            "default".to_string(),
        ),
    );
    expect_failure(
        "cds-names-another-artifact".to_string(),
        wrong,
        "SequenceReferenceMismatch".to_string(),
        "BiologicalValidation".to_string(),
    );

    // A nucleotide intent whose source names that other artifact.
    let intent = NucleotideMutationIntent::create(
        decoy.reference(),
        create(&decoy.reference(), 3, 4).unwrap(),
        "G".to_string(),
        "C".to_string(),
    )
    .unwrap();
    let applied = apply_nucleotide(
        &sequence,
        None,
        result_id("mut-a-refused".to_string()),
        intent,
    );
    expect_failure(
        "intent-names-another-artifact".to_string(),
        applied,
        "SequenceReferenceMismatch".to_string(),
        "BiologicalValidation".to_string(),
    );

    // A target coordinate naming a different artifact than the source is a
    // contradictory request, refused when the intent is built.
    let crossed = NucleotideMutationIntent::create(
        sequence.reference(),
        create(&decoy.reference(), 3, 4).unwrap(),
        "G".to_string(),
        "C".to_string(),
    );
    expect_failure(
        "target-names-another-artifact".to_string(),
        crossed,
        "InvalidMutationRequest".to_string(),
        "MalformedInput".to_string(),
    );

    // A residue position that names a different CDS.
    let other = apply_residue(
        &sequence,
        cds_of(
            &sequence,
            "cds-1".to_string(),
            "0:30".to_string(),
            Strand::Plus,
        ),
        result_id("mut-a-refused".to_string()),
        residue_intent(
            "cds-2".to_string(),
            2,
            "A".to_string(),
            "W".to_string(),
            "default".to_string(),
        ),
    );
    expect_failure(
        "position-names-another-cds".to_string(),
        other,
        "ResiduePositionWrongCds".to_string(),
        "BiologicalValidation".to_string(),
    );
}

#[test]
fn residue_numbering_follows_the_cds_strand() {
    let sequence: Sequence = artifact("mut-a".to_string(), text_a(), false);
    let plus: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );
    let minus: Cds = cds_of(
        &sequence,
        "cds-2".to_string(),
        "0:30".to_string(),
        Strand::Minus,
    );

    let plus_pass = translate(&sequence, &plus);
    report_failure("plus-translation".to_string(), &plus_pass);
    let plus_residues: Translation = plus_pass.unwrap();
    assert!(
        residues_of(&plus_residues) == "MAWNQQQQQQ",
        "the plus-strand codons read as MAWNQQQQQQ",
    );

    let minus_pass = translate(&sequence, &minus);
    report_failure("minus-translation".to_string(), &minus_pass);
    let minus_residues: Translation = minus_pass.unwrap();
    assert!(
        residues_of(&minus_residues) == "LLLLLLIPSH",
        "the reverse complement CTG CTG CTG CTG CTG CTG ATT CCA AGC CAT reads as LLLLLLIPSH",
    );

    // The same position number addresses different nucleotides on each strand.
    let seventh_plus = residue_position("cds-1".to_string(), 7);
    let seventh_minus = residue_position("cds-2".to_string(), 7);
    let q = plus_residues.residue_at(&seventh_plus);
    let i = minus_residues.residue_at(&seventh_minus);
    report_failure("seventh-plus".to_string(), &q);
    report_failure("seventh-minus".to_string(), &i);
    assert!(
        q.unwrap() != i.unwrap(),
        "residue 7 is Q on the plus strand and I on the minus strand",
    );
}

#[test]
fn a_minus_strand_codon_is_written_at_its_reference_coordinates() {
    let sequence: Sequence = artifact("mut-a".to_string(), text_a(), false);
    let annotation: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Minus,
    );

    // Residue 7 of the minus-strand CDS is the codon ATT, read from the far
    // end of the reference: reference coordinates 11,10,9. Replacing I by L
    // writes TTA, so the reference letters at 11,10,9 become A,G,T.
    let made = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-minus-7".to_string()),
        residue_intent(
            "cds-1".to_string(),
            7,
            "I".to_string(),
            "L".to_string(),
            "supplied:TTA".to_string(),
        ),
    );
    report_failure("minus-mutation".to_string(), &made);
    let result: MutationResult = made.unwrap();

    assert!(
        result.edited().canonical_text() == "ATGGCTTGGTAACAGCAGCAGCAGCAGCAG",
        "only reference coordinates 9 and 11 change: 9 becomes T, 11 becomes A",
    );
    assert!(
        written_coordinates(&result) == vec![9, 10, 11],
        "the recorded coordinates are the ascending reference coordinates of the codon",
    );

    let changed: Vec<u64> = diff_positions(&sequence, &result.edited());
    assert!(
        changed == vec![9, 11],
        "the written triplet TTA differs from the read triplet ATT at two of its three letters",
    );

    let checked = verify(&sequence, Some(annotation.clone()), result.clone());
    report_failure("minus-verification".to_string(), &checked);
    let verified: MutationResult = checked.unwrap();
    assert!(
        verified.state() == MutationState::Verified,
        "a verified result says so",
    );

    // Re-reading the same coordinates from the edited artifact gives the
    // requested triplet at the requested residue.
    let rebuilt: Cds = cds_of(
        &verified.edited(),
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Minus,
    );
    let re_read = extract_cds(&verified.edited(), &rebuilt);
    report_failure("minus-reread".to_string(), &re_read);
    assert!(
        re_read.unwrap() == "CTGCTGCTGCTGCTGCTGTTACCAAGCCAT",
        "the codon of residue 7 in the edited CDS text is TTA",
    );
}

#[test]
fn a_target_outside_the_scoped_cds_is_refused_before_any_edit() {
    let sequence: Sequence = artifact("mut-a".to_string(), text_a(), false);
    let scope: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "0:12".to_string(),
        Strand::Plus,
    );

    let outside = apply_nucleotide(
        &sequence,
        Some(scope.clone()),
        result_id("mut-a-outside".to_string()),
        nucleotide_intent(&sequence, 20, "G".to_string(), "T".to_string()),
    );
    expect_failure(
        "target-outside-scope".to_string(),
        outside,
        "TargetOutsideFeature".to_string(),
        "BiologicalValidation".to_string(),
    );

    // The same request with no scope is a legal DNA edit.
    let unscoped = apply_nucleotide(
        &sequence,
        None,
        result_id("mut-a-unscoped".to_string()),
        nucleotide_intent(&sequence, 20, "G".to_string(), "T".to_string()),
    );
    report_failure("same-request-unscoped".to_string(), &unscoped);
    assert!(
        unscoped.unwrap().edited().canonical_text() == "ATGGCTTGGAATCAGCAGCATCAGCAGCAG",
        "coordinate 20 holds G and becomes T",
    );

    // The boundary of the scope is half-open: 11 is inside, 12 is not.
    let inside = apply_nucleotide(
        &sequence,
        Some(scope.clone()),
        result_id("mut-a-inside".to_string()),
        nucleotide_intent(&sequence, 11, "T".to_string(), "C".to_string()),
    );
    report_failure("target-inside-scope".to_string(), &inside);
    let edge = apply_nucleotide(
        &sequence,
        Some(scope.clone()),
        result_id("mut-a-edge".to_string()),
        nucleotide_intent(&sequence, 12, "C".to_string(), "T".to_string()),
    );
    expect_failure(
        "target-at-scope-end".to_string(),
        edge,
        "TargetOutsideFeature".to_string(),
        "BiologicalValidation".to_string(),
    );

    // The strand of the scope is irrelevant to a DNA edit.
    let minus_scope: Cds = cds_of(
        &sequence,
        "cds-2".to_string(),
        "0:12".to_string(),
        Strand::Minus,
    );
    let minus_scoped = apply_nucleotide(
        &sequence,
        Some(minus_scope.clone()),
        result_id("mut-a-minus-scope".to_string()),
        nucleotide_intent(&sequence, 11, "T".to_string(), "C".to_string()),
    );
    report_failure("dna-edit-inside-a-minus-scope".to_string(), &minus_scoped);
    assert!(
        minus_scoped.unwrap().edited().canonical_text()
            == inside.unwrap().edited().canonical_text(),
        "scoping by a minus-strand CDS selects the same nucleotides",
    );
}

#[test]
fn an_expected_original_mismatch_names_the_value_that_is_actually_there() {
    let sequence: Sequence = artifact("mut-a".to_string(), text_a(), false);
    let annotation: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );

    // Coordinate 3 holds G, and the request claims T.
    let nucleotide = apply_nucleotide(
        &sequence,
        None,
        result_id("mut-a-mismatch".to_string()),
        nucleotide_intent(&sequence, 3, "T".to_string(), "C".to_string()),
    );
    expect_failure(
        "wrong-nucleotide".to_string(),
        nucleotide,
        "ExpectedNucleotideMismatch".to_string(),
        "BiologicalValidation".to_string(),
    );

    // Residue 2 is A, and the request claims Q.
    let residue = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-mismatch".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "Q".to_string(),
            "W".to_string(),
            "default".to_string(),
        ),
    );
    expect_failure(
        "wrong-residue".to_string(),
        residue,
        "ExpectedResidueMismatch".to_string(),
        "BiologicalValidation".to_string(),
    );

    // A residue the alphabet does not have cannot be represented at all.
    let stop = ResidueMutationIntent::create(
        residue_position("cds-1".to_string(), 2),
        "A".to_string(),
        "*".to_string(),
        CodonSelectionPolicy::Default,
    );
    expect_failure(
        "mutation-to-a-stop".to_string(),
        stop,
        "InvalidMutationRequest".to_string(),
        "MalformedInput".to_string(),
    );
}

#[test]
fn a_failed_mutation_leaves_the_source_artifact_unchanged() {
    let sequence: Sequence = artifact("mut-a".to_string(), text_a(), false);
    let annotation: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );

    let before_text: String = sequence.canonical_text();
    let before_digest: String = sequence.content_hash().hex_digest().to_string();

    let refused = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-refused".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "Q".to_string(),
            "W".to_string(),
            "default".to_string(),
        ),
    );
    expect_failure(
        "refused-request".to_string(),
        refused,
        "ExpectedResidueMismatch".to_string(),
        "BiologicalValidation".to_string(),
    );
    assert!(
        sequence.canonical_text() == before_text,
        "the source artifact text is untouched by a refused mutation",
    );
    assert!(
        before_digest == sequence.content_hash().hex_digest(),
        "the source digest is untouched by a refused mutation",
    );

    let out_of_range = apply_nucleotide(
        &sequence,
        None,
        result_id("mut-a-refused".to_string()),
        nucleotide_intent(&sequence, 30, "A".to_string(), "T".to_string()),
    );
    expect_failure(
        "coordinate-past-the-end".to_string(),
        out_of_range,
        "CoordinateOutOfBounds".to_string(),
        "BiologicalValidation".to_string(),
    );
    assert!(
        sequence.canonical_text() == before_text,
        "a coordinate error writes nothing",
    );

    // A successful mutation produces a new artifact and keeps the source.
    let made = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-res-2".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "A".to_string(),
            "W".to_string(),
            "default".to_string(),
        ),
    );
    report_failure("accepted-request".to_string(), &made);
    let result: MutationResult = made.unwrap();
    assert!(
        sequence.canonical_text() == before_text,
        "an applied mutation does not change the source either",
    );
    assert!(
        result.source() == sequence.reference(),
        "the result names the artifact it was computed from",
    );
    assert!(
        result.edited().canonical_text() == "ATGTGGTGGAATCAGCAGCAGCAGCAGCAG",
        "the new artifact holds the edit",
    );
    assert!(
        before_digest != result.edited().content_hash().hex_digest(),
        "the new artifact is a different artifact",
    );
}

#[test]
fn a_residue_mutation_changes_exactly_the_codon_it_targeted() {
    let sequence: Sequence = artifact("mut-a".to_string(), text_a(), false);
    let annotation: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );

    // GCT -> TGG differs at all three nucleotides.
    let all_three = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-res-2".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "A".to_string(),
            "W".to_string(),
            "default".to_string(),
        ),
    );
    report_failure("all-three-letters".to_string(), &all_three);
    let wide: MutationResult = all_three.unwrap();
    assert!(
        diff_positions(&sequence, &wide.edited()) == vec![3, 4, 5],
        "GCT becomes TGG at every letter",
    );

    // ATG -> TTA leaves the middle letter alone: the edit is confined to the
    // codon, it does not have to fill it.
    let middle = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-res-1".to_string()),
        residue_intent(
            "cds-1".to_string(),
            1,
            "M".to_string(),
            "L".to_string(),
            "supplied:TTA".to_string(),
        ),
    );
    report_failure("one-letter-shared".to_string(), &middle);
    let shared: MutationResult = middle.unwrap();
    assert!(
        diff_positions(&sequence, &shared.edited()) == vec![0, 2],
        "ATG and TTA share the T at coordinate 1",
    );

    for result in [wide.clone(), shared.clone()] {
        let checked = verify(&sequence, Some(annotation.clone()), result.clone());
        report_failure("confined-to-the-codon".to_string(), &checked);
        let verified: MutationResult = checked.unwrap();
        let written: Vec<u64> = written_coordinates(&verified);
        let changed: Vec<u64> = diff_positions(&sequence, &verified.edited());
        assert!(
            !changed.is_empty(),
            "an accepted mutation changes at least one nucleotide"
        );
        for position in changed {
            assert!(
                written.contains(&position),
                "a nucleotide outside the targeted codon changed at {}",
                position,
            );
        }
    }

    // The residues other than the targeted one are unchanged.
    let before = translate(&sequence, &annotation);
    report_failure("before-residues".to_string(), &before);
    assert!(
        residues_of(&before.unwrap()) == "MAWNQQQQQQ",
        "the source residues",
    );
    let edited: Sequence = shared.edited();
    let rebuilt: Cds = cds_of(
        &edited,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );
    let after = translate(&edited, &rebuilt);
    report_failure("after-residues".to_string(), &after);
    assert!(
        residues_of(&after.unwrap()) == "LAWNQQQQQQ",
        "only residue 1 changed, from M to L",
    );
}

#[test]
fn codon_policy_is_deterministic_and_never_guessed() {
    let sequence: Sequence = artifact("mut-a".to_string(), text_a(), false);
    let annotation: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );

    // The same request applied twice gives the same result.
    let once = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-res-2".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "A".to_string(),
            "W".to_string(),
            "default".to_string(),
        ),
    );
    let twice = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-res-2".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "A".to_string(),
            "W".to_string(),
            "default".to_string(),
        ),
    );
    report_failure("first-application".to_string(), &once);
    report_failure("second-application".to_string(), &twice);
    assert!(
        once.unwrap() == twice.unwrap(),
        "the same request produces the same result",
    );

    // The default is the highest-count codon in the E. coli K-12 table, so it
    // is fixed and checkable against the published counts.
    let l = default_codon(Residue::L);
    report_failure("default-l".to_string(), &l);
    assert!(
        l.unwrap().text() == "CTG",
        "CTG is the most used L codon in E. coli K-12, not the first in table order (TTA)",
    );
    let a = default_codon(Residue::A);
    report_failure("default-a".to_string(), &a);
    assert!(
        a.unwrap().text() == "GCG",
        "GCG is the most used A codon in E. coli K-12, not the first in table order (GCT)",
    );
    let w = default_codon(Residue::W);
    report_failure("default-w".to_string(), &w);
    assert!(
        w.unwrap().text() == "TGG",
        "W has one codon, so the default is that one",
    );

    // A request that omits the policy is not guessed.
    let missing = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-no-policy".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "A".to_string(),
            "W".to_string(),
            "unspecified".to_string(),
        ),
    );
    expect_failure(
        "no-policy".to_string(),
        missing,
        "UnspecifiedCodonPolicy".to_string(),
        "MalformedInput".to_string(),
    );

    // A supplied codon that does not encode the replacement residue is
    // contradictory: GCT is a codon for A, not for W.
    let crossed = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-supplied-crossed".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "A".to_string(),
            "W".to_string(),
            "supplied:GCT".to_string(),
        ),
    );
    expect_failure(
        "supplied-codon-disagrees".to_string(),
        crossed,
        "InvalidMutationRequest".to_string(),
        "MalformedInput".to_string(),
    );

    // A supplied codon that does encode the replacement is written exactly as
    // given. TGC is a codon for C, so residue 2 becomes C.
    let supplied = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-supplied".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "A".to_string(),
            "C".to_string(),
            "supplied:TGC".to_string(),
        ),
    );
    report_failure("supplied-codon".to_string(), &supplied);
    let chosen: MutationResult = supplied.unwrap();
    assert!(
        chosen.edited().canonical_text() == "ATGTGCTGGAATCAGCAGCAGCAGCAGCAG",
        "TGC is written at coordinates 3,4,5 and the residue reads as C",
    );
    match chosen.clone() {
        MutationResult::Residue(record) => {
            assert!(
                record.policy().name() == "supplied",
                "the record keeps the policy that chose the codon",
            );
            assert!(
                record.selected_codon().text() == "TGC",
                "the supplied codon is recorded exactly as it was given",
            );
            assert!(
                record.original_codon() == "GCT",
                "the record keeps the codon the policy replaced",
            );
        }
        MutationResult::Nucleotide(_record) => {
            panic!("a residue request produced a nucleotide result");
        }
    }

    // Policies outside the three are out of scope, and matching is exact.
    for policy in [
        "minimal-edit".to_string(),
        "DEFAULT".to_string(),
        "Supplied:TGG".to_string(),
    ] {
        let parsed = parse_codon_policy(policy.clone());
        expect_failure(
            format!("policy-{}", policy).to_string(),
            parsed,
            "UnsupportedMutationPolicy".to_string(),
            "Unsupported".to_string(),
        );
    }
}

#[test]
fn verification_requires_the_cds_the_result_names() {
    let sequence: Sequence = artifact("mut-a".to_string(), text_a(), false);
    let annotation: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );

    let made = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-res-2".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "A".to_string(),
            "W".to_string(),
            "default".to_string(),
        ),
    );
    report_failure("second-residue".to_string(), &made);
    let result: MutationResult = made.unwrap();

    // The record itself names the CDS, the policy, and the codon the policy
    // replaced, so a result can be read back without re-running the edit.
    match result.clone() {
        MutationResult::Residue(record) => {
            assert!(
                record.cds() == name_id("cds-1".to_string()),
                "the result names the CDS it was applied to",
            );
            assert!(
                record.policy() == CodonSelectionPolicy::Default,
                "the record keeps the policy that chose the codon",
            );
            assert!(
                record.original_codon() == "GCT",
                "the record keeps the codon the policy replaced",
            );
        }
        MutationResult::Nucleotide(_record) => {
            panic!("a residue request produced a nucleotide result");
        }
    }

    // The same coordinates under another identifier describe another
    // annotation. The result's residue claims are not claims about it, so
    // verification refuses it rather than re-checking the same edit under a
    // name the result never used.
    let renamed: Cds = cds_of(
        &sequence,
        "cds-9".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );
    expect_failure(
        "verify-under-another-cds-name".to_string(),
        verify(&sequence, Some(renamed.clone()), result.clone()),
        "MutationVerificationFailure".to_string(),
        "BiologicalValidation".to_string(),
    );

    let checked = verify(&sequence, Some(annotation.clone()), result.clone());
    report_failure("verify-under-the-named-cds".to_string(), &checked);
    assert!(
        checked.unwrap().state() == MutationState::Verified,
        "the CDS the result names is the CDS that verifies it",
    );
}

#[test]
fn verification_accepts_only_a_result_that_follows_from_its_source() {
    let sequence: Sequence = artifact("mut-a".to_string(), text_a(), false);
    let annotation: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );

    let made = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-res-10".to_string()),
        residue_intent(
            "cds-1".to_string(),
            10,
            "Q".to_string(),
            "L".to_string(),
            "supplied:TTA".to_string(),
        ),
    );
    report_failure("tenth-residue".to_string(), &made);
    let result: MutationResult = made.unwrap();
    assert!(
        result.state() == MutationState::Calculated,
        "a produced result is calculated, not verified",
    );

    let checked = verify(&sequence, Some(annotation.clone()), result.clone());
    report_failure("verify-tenth-residue".to_string(), &checked);
    let verified: MutationResult = checked.unwrap();
    assert!(
        verified.state() == MutationState::Verified,
        "verification is what sets the state",
    );
    assert!(
        verified != result,
        "a calculated result is never equal to its verified form",
    );

    // A residue result cannot be verified without the CDS it was applied to.
    let blind = verify(&sequence, None, result.clone());
    expect_failure(
        "verify-without-the-cds".to_string(),
        blind,
        "MutationVerificationFailure".to_string(),
        "BiologicalValidation".to_string(),
    );

    // Verifying against a CDS that does not have that residue fails.
    let narrow: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "0:12".to_string(),
        Strand::Plus,
    );
    let narrower = verify(&sequence, Some(narrow.clone()), result.clone());
    expect_failure(
        "verify-against-another-cds".to_string(),
        narrower,
        "MutationVerificationFailure".to_string(),
        "BiologicalValidation".to_string(),
    );

    // Verifying against a different artifact is a reference mismatch.
    let decoy: Sequence = artifact("mut-a".to_string(), text_b(), false);
    let wrong_source = verify(&decoy, Some(annotation.clone()), result.clone());
    expect_failure(
        "verify-another-artifact".to_string(),
        wrong_source,
        "SequenceReferenceMismatch".to_string(),
        "BiologicalValidation".to_string(),
    );

    // A nucleotide result verifies against its source alone.
    let nucleotide = apply_nucleotide(
        &sequence,
        None,
        result_id("mut-a-nuc-3".to_string()),
        nucleotide_intent(&sequence, 3, "G".to_string(), "C".to_string()),
    );
    report_failure("nucleotide-edit".to_string(), &nucleotide);
    let dna = verify(&sequence, None, nucleotide.unwrap());
    report_failure("verify-nucleotide-edit".to_string(), &dna);
    assert!(
        dna.unwrap().state() == MutationState::Verified,
        "a DNA substitution that matches its claims verifies",
    );
}

#[test]
fn an_origin_crossing_codon_is_recorded_as_two_intervals() {
    let sequence: Sequence = artifact("mut-c".to_string(), text_c(), true);
    let annotation: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "22:24;0:7".to_string(),
        Strand::Plus,
    );

    // The CDS text is TGG CTT GGA, so residue 1 is W, whose codon is the
    // reference coordinates 22,23,0: it straddles the origin.
    let made = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-c-res-1".to_string()),
        residue_intent(
            "cds-1".to_string(),
            1,
            "W".to_string(),
            "L".to_string(),
            "supplied:TTA".to_string(),
        ),
    );
    report_failure("origin-crossing-mutation".to_string(), &made);
    let result: MutationResult = made.unwrap();

    assert!(
        result.edited().canonical_text() == "ACTTGGAACGAACGAACGAACATT",
        "TTA written at 22,23,0 turns the trailing G into T and the leading G into A",
    );
    assert!(
        written_coordinates(&result) == vec![0, 22, 23],
        "the recorded coordinates are ascending reference coordinates, in two runs",
    );
    assert!(
        diff_positions(&sequence, &result.edited()) == vec![0, 23],
        "the first letter of TTA matches the T already at 22",
    );

    let checked = verify(&sequence, Some(annotation.clone()), result.clone());
    report_failure("verify-origin-crossing".to_string(), &checked);
    let verified: MutationResult = checked.unwrap();

    let rebuilt: Cds = cds_of(
        &verified.edited(),
        "cds-1".to_string(),
        "22:24;0:7".to_string(),
        Strand::Plus,
    );
    let re_read = extract_cds(&verified.edited(), &rebuilt);
    report_failure("reread-edited-cds".to_string(), &re_read);
    assert!(
        re_read.unwrap() == "TTACTTGGA",
        "re-extracting the same coordinates from the edited artifact gives TTA CTT GGA",
    );
}

#[test]
fn a_mutation_that_changes_nothing_is_refused() {
    let sequence: Sequence = artifact("mut-a".to_string(), text_a(), false);
    let annotation: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );

    let dna_noop: Result<NucleotideMutationIntent, DomainError> = NucleotideMutationIntent::create(
        sequence.reference(),
        create(&sequence.reference(), 3, 4).unwrap(),
        "G".to_string(),
        "G".to_string(),
    );
    expect_failure(
        "same-nucleotide".to_string(),
        dna_noop,
        "InvalidMutationRequest".to_string(),
        "MalformedInput".to_string(),
    );

    // Re-writing the same codon for the same residue changes nothing either.
    let same_codon = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-noop".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "A".to_string(),
            "A".to_string(),
            "supplied:GCT".to_string(),
        ),
    );
    expect_failure(
        "same-codon".to_string(),
        same_codon,
        "InvalidMutationRequest".to_string(),
        "MalformedInput".to_string(),
    );

    // A codon re-expression that does change the DNA is allowed, and the
    // residues stay the same. GCA is another codon for A.
    let reexpressed = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-reexpressed".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "A".to_string(),
            "A".to_string(),
            "supplied:GCA".to_string(),
        ),
    );
    report_failure("codon-reexpression".to_string(), &reexpressed);
    let result: MutationResult = reexpressed.unwrap();
    assert!(
        result.edited().canonical_text() == "ATGGCATGGAATCAGCAGCAGCAGCAGCAG",
        "GCT becomes GCA",
    );

    let edited: Sequence = result.edited();
    let rebuilt: Cds = cds_of(
        &edited,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );
    let after = translate(&edited, &rebuilt);
    report_failure("reexpressed-residues".to_string(), &after);
    assert!(
        residues_of(&after.unwrap()) == "MAWNQQQQQQ",
        "a codon re-expression keeps every residue",
    );

    let checked = verify(&sequence, Some(annotation.clone()), result.clone());
    report_failure("verify-reexpression".to_string(), &checked);
}

#[test]
fn mutation_failures_are_classified_by_meaning() {
    let sequence: Sequence = artifact("mut-a".to_string(), text_a(), false);
    let annotation: Cds = cds_of(
        &sequence,
        "cds-1".to_string(),
        "0:30".to_string(),
        Strand::Plus,
    );
    let scope: Cds = cds_of(
        &sequence,
        "cds-2".to_string(),
        "0:12".to_string(),
        Strand::Plus,
    );

    // A request that cannot be represented.
    let unrepresentable = NucleotideMutationIntent::create(
        sequence.reference(),
        create(&sequence.reference(), 3, 5).unwrap(),
        "G".to_string(),
        "C".to_string(),
    );
    expect_failure(
        "interval-wider-than-one".to_string(),
        unrepresentable,
        "InvalidMutationRequest".to_string(),
        "MalformedInput".to_string(),
    );

    // A required field is missing.
    let missing = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-x".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "A".to_string(),
            "W".to_string(),
            "unspecified".to_string(),
        ),
    );
    expect_failure(
        "no-policy".to_string(),
        missing,
        "UnspecifiedCodonPolicy".to_string(),
        "MalformedInput".to_string(),
    );

    // Well-formed but outside this milestone.
    let unknown = parse_codon_policy("minimal-edit".to_string());
    expect_failure(
        "unknown-policy".to_string(),
        unknown,
        "UnsupportedMutationPolicy".to_string(),
        "Unsupported".to_string(),
    );

    // Well-formed requests that disagree with the artifact.
    let nucleotide = apply_nucleotide(
        &sequence,
        None,
        result_id("mut-a-x".to_string()),
        nucleotide_intent(&sequence, 3, "T".to_string(), "C".to_string()),
    );
    expect_failure(
        "wrong-nucleotide".to_string(),
        nucleotide,
        "ExpectedNucleotideMismatch".to_string(),
        "BiologicalValidation".to_string(),
    );
    let residue = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-x".to_string()),
        residue_intent(
            "cds-1".to_string(),
            2,
            "Q".to_string(),
            "W".to_string(),
            "default".to_string(),
        ),
    );
    expect_failure(
        "wrong-residue".to_string(),
        residue,
        "ExpectedResidueMismatch".to_string(),
        "BiologicalValidation".to_string(),
    );
    let outside = apply_nucleotide(
        &sequence,
        Some(scope.clone()),
        result_id("mut-a-x".to_string()),
        nucleotide_intent(&sequence, 20, "G".to_string(), "T".to_string()),
    );
    expect_failure(
        "outside-scope".to_string(),
        outside,
        "TargetOutsideFeature".to_string(),
        "BiologicalValidation".to_string(),
    );

    // A claimed result that does not follow from its source.
    let made = apply_residue(
        &sequence,
        annotation.clone(),
        result_id("mut-a-x".to_string()),
        residue_intent(
            "cds-1".to_string(),
            10,
            "Q".to_string(),
            "L".to_string(),
            "supplied:TTA".to_string(),
        ),
    );
    report_failure("produce-a-result".to_string(), &made);
    let blind = verify(&sequence, None, made.unwrap());
    expect_failure(
        "unverifiable-result".to_string(),
        blind,
        "MutationVerificationFailure".to_string(),
        "BiologicalValidation".to_string(),
    );
}
