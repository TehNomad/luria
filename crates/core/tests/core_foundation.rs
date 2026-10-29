//! Integration tests driven by the shared fixtures in
//! `tests/fixtures/core-foundation`.
//!
//! Every expectation in the fixture table is re-derived here from the
//! sequence content, never copied from the Rust implementation. The driver
//! calls only the public domain API: `normalize`, `Feature::create`,
//! `Cds::from_optional`, `extract_feature`, `translate`, and
//! `DomainError::category`. It never hands a strand, an extracted text, or a
//! residue string to the operation under test; the fixture supplies inputs and
//! an expected outcome, and the model decides.

use luria_core::cds::Cds;
use luria_core::error::DomainError;
use luria_core::error::ErrorCategory;
use luria_core::extraction::extract_feature;
use luria_core::feature::Feature;
use luria_core::identity::Identifier;
use luria_core::identity::SequenceReference;
use luria_core::interval::create;
use luria_core::interval::NucleotideInterval;
use luria_core::interval::OrderedIntervals;
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
    interval_reference: String,
    intervals: String,
    strand: String,
    completeness: String,
    genetic_code: String,
    numbering: String,
    expected: String,
    category: String,
    note: String,
}

fn fixture_path(name: String) -> String {
    format!("{}/{}", "../../tests/fixtures/core-foundation", name)
}

fn read_fixture(name: String) -> String {
    let path: String = fixture_path(name);
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

fn parse_rows(text: String) -> Vec<FixtureRow> {
    let mut rows: Vec<FixtureRow> = Vec::new();
    let lines: Vec<String> = text.split('\n').map(|x| x.to_string()).collect();
    let mut header: bool = true;
    let mut index: usize = 0;
    while index < lines.len() {
        let line: String = lines[index].to_string();
        index += 1;
        if line.is_empty() {
            continue;
        }
        let fields: Vec<String> = line.split('\t').map(|x| x.to_string()).collect();
        if header {
            header = false;
            if fields.len() != 14 {
                panic!("fixture header has {} fields, expected 14", fields.len());
            }
            continue;
        }
        if fields.len() != 14 {
            panic!(
                "fixture row '{}' has {} fields, expected 14",
                fields[0],
                fields.len()
            );
        }
        let row: FixtureRow = FixtureRow {
            case_id: fields[0].to_string(),
            kind: fields[1].to_string(),
            sequence_file: fields[2].to_string(),
            topology: fields[3].to_string(),
            reference: fields[4].to_string(),
            interval_reference: fields[5].to_string(),
            intervals: fields[6].to_string(),
            strand: fields[7].to_string(),
            completeness: fields[8].to_string(),
            genetic_code: fields[9].to_string(),
            numbering: fields[10].to_string(),
            expected: fields[11].to_string(),
            category: fields[12].to_string(),
            note: fields[13].to_string(),
        };
        rows.push(row);
    }
    rows
}

fn fixture_rows() -> Vec<FixtureRow> {
    parse_rows(read_fixture("cases.tsv".to_string()))
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
    if row.strand == "+" {
        return Some(Strand::Plus);
    }
    if row.strand == "-" {
        return Some(Strand::Minus);
    }
    if row.strand == "none" {
        return None;
    }
    panic!("fixture strand {} is not '+', '-', or 'none'", row.strand)
}

/// Parses one coordinate token. A non-integer token is a fixture bug.
fn parse_coordinate(text: String) -> i64 {
    let parsed = text.parse();
    if parsed.is_err() {
        panic!("coordinate {} is not an integer", text);
    }
    parsed.unwrap()
}

/// Builds the interval list named by the fixture row, pointing at the given
/// sequence artifact.
fn parse_intervals(
    target: Option<SequenceReference>,
    spec: String,
    case_id: String,
) -> Result<OrderedIntervals, DomainError> {
    if spec == "none" {
        return Ok(Vec::new());
    }
    if target.is_none() {
        panic!("{} has intervals but no interval_reference", case_id);
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

/// The sequence artifact the intervals point at.
///
/// `same` means the row's own sequence. `identifier:fixture-file` means an
/// artifact named `identifier` whose content comes from `fixture-file`,
/// normalized with the row's topology: that is how a same-name/different-content
/// artifact is expressed. `none` means the row has no intervals.
fn resolve_interval_reference(row: &FixtureRow, sequence: &Sequence) -> Option<SequenceReference> {
    if row.interval_reference == "none" {
        return None;
    }
    if row.interval_reference == "same" {
        return Some(sequence.reference());
    }
    let parts: Vec<String> = row
        .interval_reference
        .split(':')
        .map(|x| x.to_string())
        .collect();
    if parts.len() != 2 {
        panic!(
            "interval_reference {} is not 'same' or 'identifier:fixture-file'",
            row.interval_reference
        );
    }
    let name_result = Identifier::create(parts[0].clone());
    report_failure(row.case_id.clone(), &name_result);
    let name: Identifier = name_result.unwrap();
    let other: Sequence =
        normalize(name, read_fixture(parts[1].clone()), topology_for(row)).unwrap();
    Some(other.reference())
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
        DomainError::UnusableGenBankCds(_) => "UnusableGenBankCds",
        DomainError::GenBankTranslationMismatch(_) => "GenBankTranslationMismatch",
        DomainError::EmptySearchPattern => "EmptySearchPattern",
        DomainError::InvalidSearchPatternCharacters(_) => "InvalidSearchPatternCharacters",
        DomainError::UnknownRecordFormat(_) => "UnknownRecordFormat",
        DomainError::RecordTooLong(_) => "RecordTooLong",
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

/// The fixture file named by the `interval_reference` column, `none` when the
/// row has no intervals or points at its own sequence.
fn interval_reference_file(row: &FixtureRow) -> String {
    if row.interval_reference == "none" || row.interval_reference == "same" {
        return "none".to_string();
    }
    let parts: Vec<String> = row
        .interval_reference
        .split(':')
        .map(|x| x.to_string())
        .collect();
    if parts.len() != 2 {
        panic!(
            "interval_reference {} is not 'same' or 'identifier:fixture-file'",
            row.interval_reference
        );
    }
    parts[1].to_string()
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

/// Checks that a case produced the declared error variant, and that the
/// variant is classified by the real `DomainError::category()` implementation.
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
        Ok(_) => {
            panic!("{} expected an error but the operation succeeded", case_id);
        }
    }
}

/// Reports an unexpected error, and does nothing when the case succeeded.
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
fn every_fixture_case_is_present_and_well_formed() {
    let rows: Vec<FixtureRow> = fixture_rows();
    assert!(rows.len() == 41);

    let mut seen: Vec<String> = Vec::new();
    for row in rows.iter() {
        assert!(
            row.kind == "sequence" || row.kind == "feature" || row.kind == "cds",
            "unknown fixture kind {}",
            row.kind,
        );
        for other in rows.iter() {
            if other.case_id == row.case_id && other.kind != row.kind {
                panic!("case_id {} is duplicated", row.case_id);
            }
        }
        seen.push(format!("{}/{}", row.case_id, row.kind));
    }
    assert!(seen.len() == rows.len());
}

#[test]
fn sequence_fixtures_match_their_documented_digests() {
    for row in fixture_rows().iter() {
        let case_id: String = row.case_id.to_string();
        let sequence_file: String = row.sequence_file.to_string();
        let reference: String = row.reference.to_string();
        let expected: String = row.expected.to_string();
        let category: String = row.category.to_string();
        let note: String = row.note.to_string();
        if row.kind != "sequence" {
            continue;
        }

        let raw: String = read_fixture(sequence_file);
        let identifier_result = Identifier::create(reference.clone());
        if expected == "error:InvalidIdentifierCharacters" {
            expect_error(case_id.clone(), identifier_result, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &identifier_result);
        let identifier: Identifier = identifier_result.unwrap();

        let created = normalize(identifier.clone(), raw, topology_for(row));
        if expected.starts_with("error:") {
            expect_error(case_id.clone(), created, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &created);
        let sequence: Sequence = created.unwrap();

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
fn feature_fixtures_match_their_documented_text_and_errors() {
    for row in fixture_rows().iter() {
        let case_id: String = row.case_id.to_string();
        let sequence_file: String = row.sequence_file.to_string();
        let reference: String = row.reference.to_string();
        let intervals_spec: String = row.intervals.to_string();
        let expected: String = row.expected.to_string();
        let category: String = row.category.to_string();
        let note: String = row.note.to_string();
        if row.kind != "feature" {
            continue;
        }

        let raw: String = read_fixture(sequence_file);
        let identifier_result = Identifier::create(reference.clone());
        report_failure(case_id.clone(), &identifier_result);
        let identifier: Identifier = identifier_result.unwrap();

        let sequence_result = normalize(identifier.clone(), raw, topology_for(row));
        report_failure(case_id.clone(), &sequence_result);
        let sequence: Sequence = sequence_result.unwrap();

        let target: Option<SequenceReference> = resolve_interval_reference(row, &sequence);
        let intervals_result = parse_intervals(target, intervals_spec, case_id.clone());
        if expected.starts_with("error:") && intervals_result.is_err() {
            expect_error(case_id.clone(), intervals_result, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &intervals_result);
        let intervals: OrderedIntervals = intervals_result.unwrap();

        let feature_result = Feature::create(
            &sequence,
            identifier,
            "fixture".to_string(),
            intervals,
            strand_for(row).unwrap(),
        );
        if expected.starts_with("error:") && feature_result.is_err() {
            expect_error(case_id.clone(), feature_result, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &feature_result);
        let feature: Feature = feature_result.unwrap();

        // The strand comes from the Feature, never from this call site.
        let text_result = extract_feature(&sequence, &feature);
        report_failure(case_id.clone(), &text_result);
        let text: String = text_result.unwrap();
        let want: String = expected.replace("extracted:", "");
        assert!(
            text == want,
            "{}: extracted mismatch, got {}, want {}",
            case_id,
            text,
            want,
        );
    }
}

#[test]
fn cds_fixtures_match_their_documented_residues_and_errors() {
    for row in fixture_rows().iter() {
        let case_id: String = row.case_id.to_string();
        let sequence_file: String = row.sequence_file.to_string();
        let reference: String = row.reference.to_string();
        let intervals_spec: String = row.intervals.to_string();
        let completeness_text: String = row.completeness.to_string();
        let genetic_code_text: String = row.genetic_code.to_string();
        let numbering_text: String = row.numbering.to_string();
        let expected: String = row.expected.to_string();
        let category: String = row.category.to_string();
        let note: String = row.note.to_string();
        if row.kind != "cds" {
            continue;
        }

        let raw: String = read_fixture(sequence_file);
        let identifier_result = Identifier::create(reference.clone());
        report_failure(case_id.clone(), &identifier_result);
        let identifier: Identifier = identifier_result.unwrap();

        let sequence_result = normalize(identifier.clone(), raw, topology_for(row));
        report_failure(case_id.clone(), &sequence_result);
        let sequence: Sequence = sequence_result.unwrap();

        let target: Option<SequenceReference> = resolve_interval_reference(row, &sequence);
        let intervals_result = parse_intervals(target, intervals_spec, case_id.clone());
        if expected.starts_with("error:") && intervals_result.is_err() {
            expect_error(case_id.clone(), intervals_result, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &intervals_result);
        let intervals: OrderedIntervals = intervals_result.unwrap();

        // Every CDS row goes through the same optional-attribute constructor,
        // so an absent attribute is the fixture's doing, not a special case.
        let created = Cds::from_optional(
            &sequence,
            identifier,
            Some(intervals),
            strand_for(row),
            Some(completeness_text),
            Some(genetic_code_text),
            Some(numbering_text),
        );
        if expected.starts_with("error:") && created.is_err() {
            expect_error(case_id.clone(), created, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &created);
        let cds: Cds = created.unwrap();

        let result = translate(&sequence, &cds);
        if expected.starts_with("error:") {
            expect_error(case_id.clone(), result, expected, category, note);
            continue;
        }
        report_failure(case_id.clone(), &result);
        let translation: Translation = result.unwrap();

        let documented: String = expected.replace("residues:", "");
        let has_stop: bool = documented.ends_with("+stop");
        let want: String = documented.replace("+stop", "");
        let got: String = translation.residues_text();
        assert!(
            got == want,
            "{}: residue mismatch, got {}, want {}",
            case_id,
            got,
            want,
        );
        if has_stop {
            assert!(
                translation.terminal_stop(),
                "{}: terminal stop flag mismatch",
                case_id,
            );
        } else {
            assert!(
                !translation.terminal_stop(),
                "{}: terminal stop flag mismatch",
                case_id,
            );
        }
    }
}

#[test]
fn every_error_case_names_a_category() {
    for row in fixture_rows().iter() {
        let case_id: String = row.case_id.to_string();
        let expected: String = row.expected.to_string();
        let category: String = row.category.to_string();
        if !expected.starts_with("error:") {
            assert!(
                category == "none",
                "{}: success case declares a category",
                case_id,
            );
            continue;
        }
        assert!(
            category != "none",
            "{}: error case has no category",
            case_id,
        );
        // The declared category must name an `ErrorCategory`; the category the
        // model actually assigns is checked in `expect_error`, which calls
        // `DomainError::category()`.
        let declared: ErrorCategory = declared_category_to_category(category.clone());
        assert!(
            category_name(declared) == category,
            "{}: declared category {} is not an ErrorCategory",
            case_id,
            category,
        );
    }
}

#[test]
fn fixture_rows_use_only_documented_kinds() {
    for row in fixture_rows().iter() {
        let case_id: String = row.case_id.to_string();
        if row.kind == "sequence" {
            assert!(
                row.intervals == "none"
                    && row.interval_reference == "none"
                    && row.strand == "none"
                    && row.completeness == "none"
                    && row.genetic_code == "none"
                    && row.numbering == "none",
                "{}: a sequence case supplies domain attributes",
                case_id,
            );
            assert!(
                row.expected.starts_with("hash:") || row.expected.starts_with("error:"),
                "{}: a sequence case expects {}",
                case_id,
                row.expected,
            );
        } else if row.kind == "feature" {
            assert!(
                row.strand == "+" || row.strand == "-",
                "{}: a feature case must supply a strand",
                case_id,
            );
            assert!(
                row.completeness == "none" && row.genetic_code == "none" && row.numbering == "none",
                "{}: a feature case supplies CDS attributes",
                case_id,
            );
            assert!(
                row.expected.starts_with("extracted:") || row.expected.starts_with("error:"),
                "{}: a feature case expects {}",
                case_id,
                row.expected,
            );
        } else if row.kind == "cds" {
            assert!(
                row.completeness != "none" && row.genetic_code != "none" && row.numbering != "none",
                "{}: a CDS case is missing an attribute column",
                case_id,
            );
            assert!(
                row.expected.starts_with("residues:") || row.expected.starts_with("error:"),
                "{}: a CDS case expects {}",
                case_id,
                row.expected,
            );
        } else {
            panic!("unknown fixture kind {}", row.kind);
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
fn fixture_sequence_files_are_named_by_the_cases_that_use_them() {
    let rows: Vec<FixtureRow> = fixture_rows();
    let names: Vec<String> =
        "seq-a.txt seq-b.txt seq-c.txt seq-d.txt seq-e.txt seq-f.txt seq-g.txt seq-h.txt seq-i.txt"
            .split(' ')
            .map(|x| x.to_string())
            .collect();
    for name in names.iter() {
        let suffix: String = name.to_string();
        let used: bool = rows
            .iter()
            .any(|row| row.sequence_file == suffix || interval_reference_file(row) == suffix);
        assert!(
            used,
            "fixture sequence file {} is not used by any case",
            name,
        );
        read_fixture(name.clone());
    }
}
