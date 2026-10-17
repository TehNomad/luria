//! Integration tests driven by the fixtures in `tests/fixtures/mutation`.
//!
//! Every expectation in the fixture table is re-derived from the sequence
//! content by hand (standard genetic code, base order `T, C, A, G`) and
//! written here as text; none of it is copied from the Rust implementation.
//! The driver calls only the public domain API: `normalize`, `Cds::from_optional`,
//! `NucleotideMutationIntent::create`, `ResidueMutationIntent::create`,
//! `apply_nucleotide`, `apply_residue`, `verify`, `rebuild_cds`, `translate`,
//! and `DomainError::category`. It never tells the model which codon to use,
//! which coordinates to write, or which residue the result should claim: the
//! fixture supplies inputs and an expected outcome, and the model decides.

use luria_core::cds::Cds;
use luria_core::cds::ResiduePosition;
use luria_core::error::DomainError;
use luria_core::error::ErrorCategory;
use luria_core::genetic_code::Residue;
use luria_core::identity::Identifier;
use luria_core::identity::SequenceReference;
use luria_core::interval::create;
use luria_core::interval::NucleotideInterval;
use luria_core::interval::OrderedIntervals;
use luria_core::mutation::apply_nucleotide;
use luria_core::mutation::apply_residue;
use luria_core::mutation::parse_codon_policy;
use luria_core::mutation::rebuild_cds;
use luria_core::mutation::verify;
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

struct FixtureRow {
    case_id: String,
    kind: String,
    sequence_file: String,
    topology: String,
    reference: String,
    cds_id: String,
    cds_intervals: String,
    cds_reference: String,
    cds_strand: String,
    completeness: String,
    genetic_code: String,
    numbering: String,
    target: String,
    expected_original: String,
    replacement: String,
    policy: String,
    result_id: String,
    expected: String,
    category: String,
    verify_with: String,
    note: String,
}

fn fixture_rows() -> Vec<FixtureRow> {
    let path: String = "../../tests/fixtures/mutation/cases.tsv".to_string();
    let handle = std::fs::File::open(path.clone());
    if handle.is_err() {
        panic!("cannot open fixture {}", path);
    }
    let text = std::io::read_to_string(handle.unwrap());
    if text.is_err() {
        panic!("cannot read fixture {}", path);
    }
    parse_rows(text.unwrap())
}

fn parse_rows(text: String) -> Vec<FixtureRow> {
    let mut rows: Vec<FixtureRow> = Vec::new();
    let lines: Vec<String> = text.split('\n').map(|x| x.to_string()).collect();
    let mut header: bool = true;
    for line in lines.iter() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<String> = line.split('\t').map(|x| x.to_string()).collect();
        if header {
            header = false;
            if fields.len() != 21 {
                panic!("fixture header has {} fields, expected 21", fields.len());
            }
            continue;
        }
        if fields.len() != 21 {
            panic!(
                "fixture row '{}' has {} fields, expected 21",
                fields[0],
                fields.len()
            );
        }
        rows.push(FixtureRow {
            case_id: fields[0].to_string(),
            kind: fields[1].to_string(),
            sequence_file: fields[2].to_string(),
            topology: fields[3].to_string(),
            reference: fields[4].to_string(),
            cds_id: fields[5].to_string(),
            cds_intervals: fields[6].to_string(),
            cds_reference: fields[7].to_string(),
            cds_strand: fields[8].to_string(),
            completeness: fields[9].to_string(),
            genetic_code: fields[10].to_string(),
            numbering: fields[11].to_string(),
            target: fields[12].to_string(),
            expected_original: fields[13].to_string(),
            replacement: fields[14].to_string(),
            policy: fields[15].to_string(),
            result_id: fields[16].to_string(),
            expected: fields[17].to_string(),
            category: fields[18].to_string(),
            verify_with: fields[19].to_string(),
            note: fields[20].to_string(),
        });
    }
    rows
}

fn fixture_file(name: String) -> String {
    format!("{}/{}", "../../tests/fixtures/mutation", name)
}

fn read_fixture(name: String) -> String {
    let path: String = fixture_file(name);
    let handle = std::fs::File::open(path.clone());
    if handle.is_err() {
        panic!("cannot open fixture {}", path);
    }
    let text = std::io::read_to_string(handle.unwrap());
    if text.is_err() {
        panic!("cannot read fixture {}", path);
    }
    text.unwrap()
}

fn topology_for(row: &FixtureRow) -> SequenceTopology {
    if row.topology == "circular" {
        return SequenceTopology::Circular;
    }
    if row.topology == "linear" {
        return SequenceTopology::Linear;
    }
    panic!("unknown topology {}", row.topology)
}

fn strand_for(row: &FixtureRow) -> Option<Strand> {
    if row.cds_strand == "+" {
        return Some(Strand::Plus);
    }
    if row.cds_strand == "-" {
        return Some(Strand::Minus);
    }
    if row.cds_strand == "none" {
        return None;
    }
    panic!(
        "fixture strand {} is not '+', '-', or 'none'",
        row.cds_strand
    )
}

fn declared_category_to_category(category: String) -> ErrorCategory {
    if category == "MalformedInput" {
        return ErrorCategory::MalformedInput;
    }
    if category == "BiologicalValidation" {
        return ErrorCategory::BiologicalValidation;
    }
    if category == "Unsupported" {
        return ErrorCategory::Unsupported;
    }
    panic!("unknown category {}", category)
}

/// The domain-error variant name, derived from the value itself.
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
        DomainError::UnsupportedComposition(_) => "UnsupportedComposition",
        DomainError::ExpectedNucleotideMismatch(_) => "ExpectedNucleotideMismatch",
        DomainError::ExpectedResidueMismatch(_) => "ExpectedResidueMismatch",
        DomainError::TargetOutsideFeature(_) => "TargetOutsideFeature",
        DomainError::MutationVerificationFailure(_) => "MutationVerificationFailure",
        DomainError::InvalidPrimerCandidate(_) => "InvalidPrimerCandidate",
        DomainError::PrimerTextAtSpanMismatch(_) => "PrimerTextAtSpanMismatch",
        DomainError::PrimerAnnealingNotTemplate(_) => "PrimerAnnealingNotTemplate",
        DomainError::PrimerPartition(_) => "PrimerPartition",
        DomainError::AnnealingOverlapBelowBound(_) => "AnnealingOverlapBelowBound",
        DomainError::PrimerPairCoverage(_) => "PrimerPairCoverage",
        DomainError::PrimerOrientationConflict(_) => "PrimerOrientationConflict",
        DomainError::UnsupportedPrimerPlacement(_) => "UnsupportedPrimerPlacement",
        DomainError::InvalidPrimerDesignRequest(_) => "InvalidPrimerDesignRequest",
        DomainError::UnsupportedPrimerDesignScope(_) => "UnsupportedPrimerDesignScope",
        DomainError::NoPrimerDesignPossible(_) => "NoPrimerDesignPossible",
        DomainError::NoPrimerDesignFits(_) => "NoPrimerDesignFits",
        DomainError::InvalidBufferModel(_) => "InvalidBufferModel",
        DomainError::TmParametersUnavailable(_) => "TmParametersUnavailable",
        DomainError::DegenerateExpansionTooLarge(_) => "DegenerateExpansionTooLarge",
        DomainError::MalformedFasta(_) => "MalformedFasta",
        DomainError::UnusableGenBankLocation(_) => "UnusableGenBankLocation",
        DomainError::MalformedGenBank(_) => "MalformedGenBank",
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

fn parse_coordinate(text: String) -> i64 {
    let parsed = text.parse();
    if parsed.is_err() {
        panic!("coordinate {} is not an integer", text);
    }
    parsed.unwrap()
}

fn parse_interval_spec(
    target: Option<SequenceReference>,
    spec: String,
    case_id: String,
) -> Result<OrderedIntervals, DomainError> {
    if spec == "none" {
        return Ok(Vec::new());
    }
    if target.is_none() {
        panic!("{} has intervals but no reference", case_id);
    }
    let reference: SequenceReference = target.unwrap();

    let mut intervals: OrderedIntervals = Vec::new();
    let parts: Vec<String> = spec.split(';').map(|x| x.to_string()).collect();
    for part in parts {
        let bounds: Vec<String> = part.split(':').map(|x| x.to_string()).collect();
        if bounds.len() != 2 {
            panic!("interval {} is not start:end", part);
        }
        let interval: NucleotideInterval = create(
            &reference,
            parse_coordinate(bounds[0].to_string()),
            parse_coordinate(bounds[1].to_string()),
        )?;
        intervals.push(interval);
    }
    Ok(intervals)
}

/// The artifact a set of intervals or a target coordinate names.
///
/// `same` is the row's own sequence. `identifier:fixture-file` is an artifact
/// named `identifier` whose content comes from another fixture file: that is
/// how an artifact that shares a display name but not its content is
/// expressed.
fn artifact_reference(row: &FixtureRow, sequence: &Sequence, spec: String) -> SequenceReference {
    if spec == "same" {
        return sequence.reference();
    }
    let parts: Vec<String> = spec.split(':').map(|x| x.to_string()).collect();
    if parts.len() != 2 {
        panic!(
            "reference spec {} is not 'same' or 'identifier:fixture-file'",
            spec
        );
    }
    let name_result = Identifier::create(parts[0].clone());
    report_failure(row.case_id.clone(), &name_result);
    let name: Identifier = name_result.unwrap();
    let other: Sequence =
        normalize(name, read_fixture(parts[1].clone()), topology_for(row)).unwrap();
    other.reference()
}

/// The sequence artifact named by the row.
fn sequence_of(row: &FixtureRow) -> Sequence {
    let name_result = Identifier::create(row.reference.clone());
    report_failure(row.case_id.clone(), &name_result);
    let name: Identifier = name_result.unwrap();
    let created = normalize(
        name,
        read_fixture(row.sequence_file.clone()),
        topology_for(row),
    );
    report_failure(row.case_id.clone(), &created);
    created.unwrap()
}

/// The CDS named by the row, or `none` for an unscoped request.
fn scope_of(row: &FixtureRow, sequence: &Sequence) -> Result<Option<Cds>, DomainError> {
    if row.cds_id == "none" {
        return Ok(None);
    }
    let name_result = Identifier::create(row.cds_id.clone());
    if let Err(problem) = name_result {
        return Err(problem);
    }
    let name: Identifier = name_result.unwrap();

    let reference: SequenceReference = artifact_reference(row, sequence, row.cds_reference.clone());
    let intervals_result = parse_interval_spec(
        Some(reference),
        row.cds_intervals.clone(),
        row.case_id.clone(),
    );
    if let Err(problem) = intervals_result {
        return Err(problem);
    }
    let intervals: OrderedIntervals = intervals_result.unwrap();

    some_cds(Cds::from_optional(
        sequence,
        name,
        Some(intervals),
        strand_for(row),
        Some(row.completeness.clone()),
        Some(row.genetic_code.clone()),
        Some(row.numbering.clone()),
    ))
}

/// Turns a `Result<Cds, DomainError>` into a `Result<Option<Cds>, DomainError>`.
fn some_cds(result: Result<Cds, DomainError>) -> Result<Option<Cds>, DomainError> {
    match result {
        Ok(cds) => Ok(Some(cds)),
        Err(problem) => Err(problem),
    }
}

/// The nucleotide coordinate a row targets, as a `NucleotideInterval`.
///
/// A target of `3` is one coordinate; `3:5` is deliberately too wide and is
/// rejected by the intent constructor. A target of `3#other.txt` names the
/// artifact normalized from another fixture file.
fn target_interval(
    row: &FixtureRow,
    sequence: &Sequence,
) -> Result<NucleotideInterval, DomainError> {
    let parts: Vec<String> = row.target.split('#').map(|x| x.to_string()).collect();
    if parts.len() > 2 {
        panic!(
            "target {} is not coordinate or coordinate#fixture-file",
            row.target
        );
    }
    let reference: SequenceReference = if parts.len() == 2 {
        artifact_reference(row, sequence, format!("{}:{}", row.reference, parts[1]))
    } else {
        sequence.reference()
    };

    let bounds: Vec<String> = parts[0].split(':').map(|x| x.to_string()).collect();
    if bounds.len() == 1 {
        let start: i64 = parse_coordinate(bounds[0].to_string());
        return create(&reference, start, start + 1);
    }
    if bounds.len() == 2 {
        return create(
            &reference,
            parse_coordinate(bounds[0].to_string()),
            parse_coordinate(bounds[1].to_string()),
        );
    }
    panic!("target {} is not a coordinate or start:end", row.target)
}

/// The residue position a row targets.
///
/// A target of `2` names the row's CDS. A target of `2@cds-2` names another
/// CDS identifier, which is how a position that belongs to a different CDS
/// is expressed.
fn residue_position_of(row: &FixtureRow) -> Result<ResiduePosition, DomainError> {
    let parts: Vec<String> = row.target.split('@').map(|x| x.to_string()).collect();
    if parts.len() > 2 {
        panic!("target {} is not number or number@cds-id", row.target);
    }
    let name = Identifier::create(if parts.len() == 2 {
        parts[1].clone()
    } else {
        row.cds_id.clone()
    });
    report_failure(row.case_id.clone(), &name);
    ResiduePosition::create(name.unwrap(), parse_coordinate(parts[0].clone()))
}

/// One expectation, split into its labelled parts.
struct Expectation {
    edited: String,
    residues: String,
    has_stop: bool,
    codon: String,
    coords: Vec<u64>,
}

fn parse_expectation(spec: String) -> Expectation {
    let mut want: Expectation = Expectation {
        edited: "".to_string(),
        residues: "".to_string(),
        has_stop: false,
        codon: "".to_string(),
        coords: Vec::new(),
    };
    let parts: Vec<String> = spec.split('+').map(|x| x.to_string()).collect();
    for part in parts.iter() {
        if part.starts_with("edited:") {
            want.edited = part.replace("edited:", "").to_string();
        } else if part.starts_with("residues:") {
            let text: String = part.replace("residues:", "").to_string();
            want.has_stop = text.ends_with("*");
            want.residues = text.replace("*", "").to_string();
        } else if part.starts_with("codon:") {
            want.codon = part.replace("codon:", "").to_string();
        } else if part.starts_with("coords:") {
            let list: Vec<String> = part
                .replace("coords:", "")
                .split(',')
                .map(|x| x.to_string())
                .collect();
            for item in list.iter() {
                want.coords.push(parse_coordinate(item.clone()) as u64);
            }
        } else {
            panic!("expectation part {} is not labelled", part);
        }
    }
    want
}

fn expect_error<T>(
    case_id: String,
    result: Result<T, DomainError>,
    expected: String,
    category: String,
    note: String,
) {
    match result {
        Err(problem) => {
            let want_variant: String = expected.replace("error:", "");
            let got_variant: String = variant_name(&problem);
            if got_variant != want_variant {
                panic!(
                    "{}: expected error:{}, the domain model gave error:{} ({})",
                    case_id, want_variant, got_variant, note,
                );
            }
            let want: ErrorCategory = declared_category_to_category(category.clone());
            let got: ErrorCategory = problem.category();
            if got != want {
                panic!(
                    "{}: DomainError::category() gives {} for {}, the fixture declares {}",
                    case_id,
                    category_name(got),
                    got_variant,
                    category,
                );
            }
        }
        Ok(_value) => {
            panic!("{} expected an error but the operation succeeded", case_id);
        }
    }
}

fn report_failure<T>(case_id: String, result: &Result<T, DomainError>) {
    if let Err(problem) = result {
        panic!(
            "{}: unexpected error {} ({})",
            case_id,
            variant_name(problem),
            category_name(problem.category()),
        );
    }
}

#[test]
fn every_mutation_fixture_case_is_present_and_well_formed() {
    let rows: Vec<FixtureRow> = fixture_rows();
    assert!(rows.len() == 47);

    for row in rows.iter() {
        assert!(
            row.kind == "sequence" || row.kind == "nucleotide" || row.kind == "residue",
            "unknown fixture kind {}",
            row.kind,
        );
        if row.expected.starts_with("error:") {
            assert!(
                row.category != "none",
                "{}: an error case declares no category",
                row.case_id,
            );
            assert!(
                row.verify_with == "none",
                "{}: an error case is not verified",
                row.case_id,
            );
        } else {
            assert!(
                row.category == "none",
                "{}: a success case declares a category",
                row.case_id,
            );
        }
    }
}

#[test]
fn fixture_rows_have_unique_case_ids() {
    let rows: Vec<FixtureRow> = fixture_rows();
    let mut index: usize = 0;
    while index < rows.len() {
        let mut other: usize = index + 1;
        while other < rows.len() {
            assert!(
                rows[index].case_id != rows[other].case_id,
                "case_id {} appears more than once",
                rows[index].case_id,
            );
            other += 1;
        }
        index += 1;
    }
}

#[test]
fn fixture_sequence_files_are_used_and_readable() {
    let rows: Vec<FixtureRow> = fixture_rows();
    let names: Vec<String> = "mut-a.txt mut-b.txt mut-c.txt mut-d.txt mut-e.txt"
        .split(' ')
        .map(|x| x.to_string())
        .collect();
    for name in names.iter() {
        let used: bool = rows.iter().any(|row| row.sequence_file == *name);
        assert!(
            used,
            "fixture sequence file {} is not used by any case",
            name
        );
        read_fixture(name.clone());
    }
}

#[test]
fn sequence_fixtures_match_their_documented_digests() {
    for row in fixture_rows().iter() {
        let case_id: String = row.case_id.to_string();
        let expected: String = row.expected.to_string();
        let category: String = row.category.to_string();
        let note: String = row.note.to_string();
        if row.kind != "sequence" {
            continue;
        }

        let sequence_result = normalize(
            Identifier::create(row.reference.clone()).unwrap(),
            read_fixture(row.sequence_file.clone()),
            topology_for(row),
        );
        if expected.starts_with("error:") {
            expect_error(case_id.clone(), sequence_result, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &sequence_result);
        let sequence: Sequence = sequence_result.unwrap();

        let want: String = expected.replace("hash:", "");
        let digest: String = sequence.content_hash().hex_digest().to_string();
        assert!(
            digest == want,
            "{}: digest mismatch, got {}, want {}",
            case_id,
            digest,
            want,
        );
    }
}

#[test]
fn nucleotide_fixture_cases_match_their_documented_edits() {
    for row in fixture_rows().iter() {
        let case_id: String = row.case_id.to_string();
        let expected: String = row.expected.to_string();
        let category: String = row.category.to_string();
        let note: String = row.note.to_string();
        if row.kind != "nucleotide" {
            continue;
        }

        let sequence: Sequence = sequence_of(row);
        let scope_result = scope_of(row, &sequence);
        if expected.starts_with("error:") && scope_result.is_err() {
            expect_error(case_id.clone(), scope_result, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &scope_result);
        let scope: Option<Cds> = scope_result.unwrap();
        let interval_result = target_interval(row, &sequence);
        if expected.starts_with("error:") && interval_result.is_err() {
            expect_error(case_id.clone(), interval_result, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &interval_result);
        let interval: NucleotideInterval = interval_result.unwrap();

        let intent = NucleotideMutationIntent::create(
            sequence.reference(),
            interval.clone(),
            row.expected_original.clone(),
            row.replacement.clone(),
        );
        if expected.starts_with("error:") && intent.is_err() {
            expect_error(case_id.clone(), intent, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &intent);
        let request: NucleotideMutationIntent = intent.unwrap();

        let result = apply_nucleotide(
            &sequence,
            scope.clone(),
            Identifier::create(row.result_id.clone()).unwrap(),
            request.clone(),
        );
        if expected.starts_with("error:") {
            expect_error(case_id.clone(), result, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &result);
        let made: MutationResult = result.unwrap();

        let want: Expectation = parse_expectation(expected.clone());
        assert!(
            made.is_nucleotide(),
            "{}: a nucleotide request produced a residue result",
            case_id
        );
        assert!(
            made.edited().canonical_text() == want.edited,
            "{}: edited mismatch, got {}, want {}",
            case_id,
            made.edited().canonical_text(),
            want.edited,
        );
        assert!(
            made.edited().length() == sequence.length(),
            "{}: a substitution must preserve length",
            case_id,
        );
        assert!(
            made.source() == sequence.reference(),
            "{}: the result names a different source artifact",
            case_id,
        );
        assert!(
            made.target_coordinates() == want.coords,
            "{}: coordinate mismatch, got {:?}, want {:?}",
            case_id,
            made.target_coordinates(),
            want.coords,
        );
        assert!(
            made.state() == MutationState::Calculated,
            "{}: a calculated result is not a verified one",
            case_id,
        );

        if !want.residues.is_empty() {
            check_residues(
                case_id.clone(),
                &sequence,
                scope.clone(),
                made.edited(),
                &want,
            );
        }
        check_verification(
            case_id.clone(),
            &sequence,
            scope,
            made,
            row.verify_with.clone(),
        );
    }
}

#[test]
fn residue_fixture_cases_match_their_documented_edits() {
    for row in fixture_rows().iter() {
        let case_id: String = row.case_id.to_string();
        let expected: String = row.expected.to_string();
        let category: String = row.category.to_string();
        let note: String = row.note.to_string();
        if row.kind != "residue" {
            continue;
        }

        let sequence: Sequence = sequence_of(row);
        let scope_result = scope_of(row, &sequence);
        if expected.starts_with("error:") && scope_result.is_err() {
            expect_error(case_id.clone(), scope_result, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &scope_result);
        let scope: Option<Cds> = scope_result.unwrap();
        if scope.is_none() {
            panic!("{}: a residue case must name a CDS", case_id);
        }
        let annotation: Cds = scope.unwrap();
        let position_result = residue_position_of(row);
        if expected.starts_with("error:") && position_result.is_err() {
            expect_error(case_id.clone(), position_result, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &position_result);
        let position: ResiduePosition = position_result.unwrap();

        let policy_result = parse_codon_policy(row.policy.clone());
        if expected.starts_with("error:") && policy_result.is_err() {
            expect_error(case_id.clone(), policy_result, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &policy_result);
        let policy = policy_result.unwrap();

        let intent = ResidueMutationIntent::create(
            position.clone(),
            row.expected_original.clone(),
            row.replacement.clone(),
            policy.clone(),
        );
        if expected.starts_with("error:") && intent.is_err() {
            expect_error(case_id.clone(), intent, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &intent);
        let request: ResidueMutationIntent = intent.unwrap();

        let result = apply_residue(
            &sequence,
            annotation.clone(),
            Identifier::create(row.result_id.clone()).unwrap(),
            request.clone(),
        );
        if expected.starts_with("error:") {
            expect_error(case_id.clone(), result, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &result);
        let made: MutationResult = result.unwrap();

        let want: Expectation = parse_expectation(expected.clone());
        assert!(
            !made.is_nucleotide(),
            "{}: a residue request produced a nucleotide result",
            case_id,
        );
        assert!(
            made.edited().canonical_text() == want.edited,
            "{}: edited mismatch, got {}, want {}",
            case_id,
            made.edited().canonical_text(),
            want.edited,
        );
        assert!(
            made.edited().length() == sequence.length(),
            "{}: a substitution must preserve length",
            case_id,
        );
        assert!(
            made.source() == sequence.reference(),
            "{}: the result names a different source artifact",
            case_id,
        );
        assert!(
            made.target_coordinates() == want.coords,
            "{}: coordinate mismatch, got {:?}, want {:?}",
            case_id,
            made.target_coordinates(),
            want.coords,
        );
        assert!(
            made.state() == MutationState::Calculated,
            "{}: a calculated result is not a verified one",
            case_id,
        );

        match made.clone() {
            MutationResult::Residue(record) => {
                assert!(
                    record.selected_codon().text() == want.codon,
                    "{}: codon mismatch, got {}, want {}",
                    case_id,
                    record.selected_codon().text(),
                    want.codon,
                );
                assert!(
                    record.expected_original()
                        == parse_residue_symbol(row.expected_original.clone()),
                    "{}: the result claims a different expected residue",
                    case_id,
                );
                assert!(
                    record.replacement() == parse_residue_symbol(row.replacement.clone()),
                    "{}: the result claims a different replacement residue",
                    case_id,
                );
            }
            MutationResult::Nucleotide(_record) => {
                panic!("{}: a residue case produced a nucleotide result", case_id);
            }
            MutationResult::Deletion(_record) => {
                panic!("{}: a residue case produced a deletion result", case_id);
            }
            MutationResult::Insertion(_record) => {
                panic!("{}: a residue case produced an insertion result", case_id);
            }
            MutationResult::Replacement(_record) => {
                panic!("{}: a residue case produced a replacement result", case_id);
            }
        }

        check_residues(
            case_id.clone(),
            &sequence,
            Some(annotation.clone()),
            made.edited(),
            &want,
        );
        check_verification(
            case_id.clone(),
            &sequence,
            Some(annotation.clone()),
            made,
            row.verify_with.clone(),
        );
    }
}

/// Re-reads the edited artifact under the same CDS coordinates and compares
/// the residues, including whether the CDS still ends with a stop codon.
fn check_residues(
    case_id: String,
    source: &Sequence,
    scope: Option<Cds>,
    edited: Sequence,
    want: &Expectation,
) {
    if want.residues.is_empty() {
        return;
    }
    if scope.is_none() {
        panic!("{} asks for residues but names no CDS", case_id);
    }
    let annotation: Cds = scope.unwrap();

    let rebuilt_result = rebuild_cds(edited.clone(), annotation.clone());
    report_failure(case_id.clone(), &rebuilt_result);
    let rebuilt: Cds = rebuilt_result.unwrap();

    let after_result = translate(&edited.clone(), &rebuilt.clone());
    if want.has_stop {
        // A documented stop is still a residue string plus a flag; a failure
        // here is a real difference and must be reported.
        report_failure(case_id.clone(), &after_result);
    } else {
        if let Err(problem) = after_result {
            panic!(
                "{}: the edited CDS cannot be read: {} ({})",
                case_id,
                variant_name(&problem),
                category_name(problem.category()),
            );
        }
    }
    let after: Translation = after_result.unwrap();

    let got: String = after.residues_text();
    assert!(
        got == want.residues,
        "{}: residue mismatch, got {}, want {}",
        case_id,
        got,
        want.residues,
    );
    assert!(
        after.terminal_stop() == want.has_stop,
        "{}: terminal stop flag mismatch",
        case_id,
    );
    assert!(
        source.length() == edited.length(),
        "{}: a substitution must preserve length",
        case_id,
    );
}

/// Runs the verification step the fixture asks for.
fn check_verification(
    case_id: String,
    source: &Sequence,
    scope: Option<Cds>,
    result: MutationResult,
    verify_with: String,
) {
    if verify_with == "none" {
        assert!(
            result.state() == MutationState::Calculated,
            "{}: an unverified result must still be Calculated",
            case_id,
        );
        return;
    }
    if verify_with == "same" {
        let checked = verify(source, scope.clone(), result.clone());
        report_failure(case_id.clone(), &checked);
        let verified: MutationResult = checked.unwrap();
        assert!(
            verified.state() == MutationState::Verified,
            "{}: a verified result must be Verified",
            case_id,
        );
        assert!(
            verified.edited().canonical_text() == result.edited().canonical_text(),
            "{}: verification must not change the artifact",
            case_id,
        );
        return;
    }
    if verify_with.starts_with("narrower:") {
        // Verification against a CDS that does not cover the recorded target
        // must fail: the result does not follow from that annotation.
        let spec: String = verify_with.replace("narrower:", "").to_string();
        let annotation: Cds = scope.unwrap();
        let reference: SequenceReference = source.reference();
        let intervals_result = parse_interval_spec(Some(reference), spec, case_id.clone());
        report_failure(case_id.clone(), &intervals_result);
        let intervals: OrderedIntervals = intervals_result.unwrap();

        let narrower_result = Cds::from_optional(
            source,
            annotation.id().clone(),
            Some(intervals),
            Some(annotation.strand().clone()),
            Some(annotation.completeness().name()),
            Some(annotation.genetic_code().name()),
            Some(annotation.numbering().name()),
        );
        report_failure(case_id.clone(), &narrower_result);
        let narrower: Cds = narrower_result.unwrap();

        let checked = verify(source, Some(narrower.clone()), result.clone());
        if let Err(problem) = checked {
            let name: String = variant_name(&problem);
            assert!(
                name == "MutationVerificationFailure",
                "{}: verification against a narrower CDS gave {} instead of MutationVerificationFailure",
                case_id,
                name,
            );
            assert!(
                problem.category() == ErrorCategory::BiologicalValidation,
                "{}: the fixture declares a narrower CDS as BiologicalValidation, category() gives {}",
                case_id,
                category_name(problem.category()),
            );
            return;
        }
        panic!(
            "{}: verification against a narrower CDS accepted the result",
            case_id
        );
    }
    panic!("unknown verify_with {}", verify_with);
}

/// A residue letter, parsed into the existing `Residue` enum.
fn parse_residue_symbol(text: String) -> Residue {
    let letters: Vec<char> = text.chars().collect();
    if letters.len() != 1 {
        panic!("residue {} is not one letter", text);
    }
    match letters[0] {
        'A' => Residue::A,
        'R' => Residue::R,
        'N' => Residue::N,
        'D' => Residue::D,
        'C' => Residue::C,
        'Q' => Residue::Q,
        'E' => Residue::E,
        'G' => Residue::G,
        'H' => Residue::H,
        'I' => Residue::I,
        'L' => Residue::L,
        'T' => Residue::T,
        'F' => Residue::F,
        'P' => Residue::P,
        'S' => Residue::S,
        'W' => Residue::W,
        'V' => Residue::V,
        'K' => Residue::K,
        'M' => Residue::M,
        'Y' => Residue::Y,
        _ => panic!("{} is not a residue of the standard genetic code", text),
    }
}
