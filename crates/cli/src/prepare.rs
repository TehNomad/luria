//! prepare: one request file's path to everything `luria q5 design` needs
//! before it runs (milestone 5e2 §1).
//!
//! 5e1 turned request text into a `RequestDocument` without touching files or
//! core; this module takes the next step, so every value here - the template's
//! `Sequence` and its source metadata, the CDS annotations, the core request
//! for the operation and the `PrimerDesignRequest` - is built by a `luria-core`
//! constructor or reader, and core's own rules and errors apply (§1). Nothing
//! is executed: no `execute`, `execute_set` or design call, that is 5g (§1).
//!
//! # The rules this module applies (§2)
//!
//! - a template path is relative to the request file's own directory (plan
//!   §3.4), and an absolute path is refused with `request_invalid` so a result
//!   never echoes a machine-specific location (decided in 5e2);
//! - `scope` is optional: when it is absent the library's target resolution
//!   applies, so `Q5SdmRequest::create` gets `None` (plan §3.4);
//! - `combined_construct` is accepted by the parser and refused by the
//!   workflow, which here means `Q5VariantSetRequest::create` and its
//!   `UnsupportedComposition` (plan §3.4);
//! - FASTA topology and CDSs come from the request, a GenBank template's
//!   topology comes from its LOCUS line and its CDSs come from `import_cds`
//!   with the requested ids (plan §3.3, D041);
//! - the template's identifier is the request's `template.id`, never a file's
//!   header or LOCUS name (D007);
//! - two declared FASTA CDSs with the same id give `request_invalid`,
//!   matching GenBank's `RepeatedName` (decided in 5e2);
//! - `template.sha256` is compared with the sequence's content digest, the one
//!   `Sequence::content_hash` gives and provenance reports (decided in 5e2);
//! - provenance includes the request file's SHA-256, of its bytes (D021), and
//!   a failed result still carries it once the file has been read and the
//!   document once it has parsed, so 5f can echo the request.
//!
//! # The steps, in this order (§3.1)
//!
//! 1. read the request file's bytes with `std::fs::read`, and take their
//!    SHA-256 through `luria_core::identity::sha256_hex`;
//! 2. parse them with `crate::request::parse_request` (5e1);
//! 3. read the template file the request names, joined to the request's own
//!    directory;
//! 4. `luria_core::identity::Identifier::create` for `template.id`;
//! 5. `luria_core::fasta::read_fasta` or `luria_core::genbank::read_genbank`;
//! 6. compare `template.sha256` with `sequence::content_hash`;
//! 7. the annotations: `interval::create` and `Cds::from_optional` for each
//!    declared CDS, or `luria_core::genbank_cds::import_cds` for the ids the
//!    request names;
//! 8. `scope`, when the request gives one;
//! 9. the operation: one `q5::Q5SdmRequest` per member, then
//!    `q5::Q5Variant::create` and `q5::Q5VariantSetRequest::create`, or
//!    `q5_expand::expand_replacement`;
//! 10. the design: `luria_core::tm::BufferModel` and
//!     `luria_core::primer_design::PrimerDesignRequest::create`.
//!
//! The first failure is the one reported, and each names the JSON path of the
//! request element it came from: 5e1's own codes for the CLI's failures (plan
//! §3.6), and a `Domain` failure carrying core's refusal unchanged for
//! everything a core constructor or reader raised.

use crate::request::{
    parse_request, BufferSpec, CdsSpec, CodonSpec, CompositionSpec, DesignSpec, EditSpec,
    MemberSpec, OperationSpec, PlacementSpec, PolicySpec, RequestDocument, RequestError,
    RequestErrorCode, StrandSpec, StyleSpec, TemplateSpec, TopologySpec,
};

use luria_core::cds::{Cds, ResiduePosition};
use luria_core::error::DomainError;
use luria_core::fasta::{read_fasta, FastaRecord};
use luria_core::genbank::{read_genbank, GenBankRecord};
use luria_core::genbank_cds::{import_cds, GenBankImport};
use luria_core::identity::{sha256_hex, Identifier, SequenceReference};
use luria_core::interval;
use luria_core::interval::{InsertionPoint, NucleotideInterval};
use luria_core::mutation::{
    Codon, CodonSelectionPolicy, NucleotideDeletionIntent, NucleotideInsertionIntent,
    NucleotideMutationIntent, NucleotideReplacementIntent, ResidueMutationIntent,
};
use luria_core::primer::PrimerPlacement;
use luria_core::primer_design::{DesignPolicy, DesignStyle, PrimerDesignRequest};
use luria_core::q5::{
    Q5Composition, Q5MutationIntent, Q5SdmRequest, Q5Variant, Q5VariantSetRequest,
    ResidueIntentWithoutCds,
};
use luria_core::q5_expand::{expand_replacement, Q5Expansion};
use luria_core::sequence::{Sequence, SequenceTopology};
use luria_core::strand::Strand;
use luria_core::tm::BufferModel;
use std::fs::{read, read_to_string};
use std::path::Path;

/// Why a request could not be prepared (§3).
#[derive(Clone, Debug, PartialEq)]
pub enum RequestFailure {
    /// Raised by the CLI itself, with 5e1's codes (plan §3.6).
    Request(RequestError),
    /// Raised by a `luria-core` constructor or reader, at the JSON path of the
    /// request element the value was built from. Boxed so the failure stays
    /// small (clippy `result_large_err`).
    Domain {
        path: String,
        error: Box<DomainError>,
    },
}

/// Where the template came from: source metadata only, never an identifier
/// (D007).
#[derive(Clone, Debug, PartialEq)]
pub enum TemplateSource {
    Fasta {
        path: String,
        header: String,
    },
    GenBank {
        path: String,
        locus_name: String,
        accession: Option<String>,
        version: Option<String>,
        import: GenBankImport,
    },
}

/// The operation, as the core entry point that will run it takes it (§3).
#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    /// Boxed: a request carries its whole template (clippy
    /// `large_enum_variant`).
    Single {
        request: Box<Q5SdmRequest>,
        name: Identifier,
    },
    Set(Q5VariantSetRequest),
    Degenerate(Q5Expansion),
}

/// Everything `luria q5 design` needs before it runs (§3).
#[derive(Clone, Debug, PartialEq)]
pub struct Prepared {
    /// SHA-256 of the request file's bytes, lower-case hex (D021).
    pub request_sha256: String,
    pub document: RequestDocument,
    pub template: Sequence,
    pub source: TemplateSource,
    pub annotations: Vec<Cds>,
    pub operation: Operation,
    pub design: PrimerDesignRequest,
}

/// A failure, with whatever was known when it happened, so a failed result can
/// still echo the request (5f). `request_sha256` is `Some` once the file has
/// been read, and `document` once it has parsed.
#[derive(Clone, Debug, PartialEq)]
pub struct PrepareFailure {
    pub failure: RequestFailure,
    pub request_sha256: Option<String>,
    pub document: Option<Box<RequestDocument>>,
}

/// What steps 3 to 7 yield for either template form: the sequence, where it
/// came from and its annotations (§3.1).
struct Ready {
    sequence: Sequence,
    source: TemplateSource,
    annotations: Vec<Cds>,
}

/// The three words every template member gives: the identifier it states
/// (D007), the file it names, and the JSON path that file sits at (§3.1 steps
/// 3 and 4).
struct Named {
    id: String,
    path: String,
    location: String,
}

/// One named member: the identifier it states and the core request that
/// carries it (§3.1 step 9).
struct Member {
    name: Identifier,
    request: Q5SdmRequest,
}

/// A failure the CLI raises itself, at the JSON path it belongs to (plan §3.6).
fn refusal(code: RequestErrorCode, path: String, message: String) -> RequestError {
    RequestError {
        code,
        path,
        message,
    }
}

/// A refusal a `luria-core` constructor or reader raised, reported at the JSON
/// path of the request element the value was built from (§3.1).
fn domain(path: String, error: DomainError) -> RequestFailure {
    RequestFailure::Domain {
        path,
        error: Box::new(error),
    }
}

/// A failure, with everything the request file had yielded when it happened,
/// so a failed result can still echo the request (5f).
fn carried(
    request_sha256: String,
    document: RequestDocument,
    failure: RequestFailure,
) -> PrepareFailure {
    PrepareFailure {
        failure: failure.clone(),
        request_sha256: Some(request_sha256.clone()),
        document: Some(Box::new(document.clone())),
    }
}

/// The directory a request's template paths are relative to (plan §3.4): the
/// request file's own, or the current directory when the path names none.
fn directory_of(request_path: &Path) -> &Path {
    let here: &Path = match request_path.parent() {
        None => Path::new("."),
        Some(found) => found,
    };
    here
}

/// Step 1: the request file's bytes (§3.1). A file that cannot be read, or
/// bytes that are not UTF-8, is the CLI's own `request_unreadable` at `$`
/// (plan §3.6).
fn read_request(request_path: &Path) -> Result<Vec<u8>, RequestFailure> {
    match read(request_path) {
        Ok(found) => Ok(found),
        Err(cause) => Err(RequestFailure::Request(refusal(
            RequestErrorCode::Unreadable,
            "$".to_string(),
            format!("cannot read the request file: {}", cause),
        ))),
    }
}

/// Step 2: the document, from bytes that must be UTF-8 (§3.1). 5e1 reports the
/// document's shape, with its own path-located errors.
fn parse_request_text(contents: Vec<u8>) -> Result<RequestDocument, RequestFailure> {
    match String::from_utf8(contents) {
        Err(_problem) => Err(RequestFailure::Request(refusal(
            RequestErrorCode::Unreadable,
            "$".to_string(),
            "the request file is not UTF-8".to_string(),
        ))),
        Ok(text) => match parse_request(&text.clone()) {
            Ok(document) => Ok(document),
            Err(problem) => Err(RequestFailure::Request(problem.clone())),
        },
    }
}

/// Steps 1 and 2, wrapped with what was known when they failed: no digest
/// before the file was read, and no document before it parsed.
fn open(request_path: &Path) -> Result<(String, RequestDocument), PrepareFailure> {
    let contents: Result<Vec<u8>, RequestFailure> = read_request(request_path);
    match contents {
        Err(problem) => Err(PrepareFailure {
            failure: problem.clone(),
            request_sha256: None,
            document: None,
        }),
        Ok(found) => {
            // D021: the request file's own SHA-256, of its bytes.
            let request_sha256: String = sha256_hex(&found);
            let document: Result<RequestDocument, RequestFailure> = parse_request_text(found);
            match document {
                Err(problem) => Err(PrepareFailure {
                    failure: problem.clone(),
                    request_sha256: Some(request_sha256.clone()),
                    document: None,
                }),
                Ok(parsed) => Ok((request_sha256, parsed)),
            }
        }
    }
}

/// Steps 3 to 10, for a request whose bytes were read and parsed.
fn build(
    request_path: &Path,
    request_sha256: String,
    document: RequestDocument,
) -> Result<Prepared, RequestFailure> {
    let directory: &Path = directory_of(request_path);

    // Steps 3 to 7: the template, its source metadata and its annotations.
    let ready: Ready = template_of(directory, &document.template)?;

    // Step 8: the scope, when the request gives one (plan §3.4).
    let scope: Option<Identifier> = match &document.scope {
        Some(text) => Some(scope_of(text.clone())?),
        None => None,
    };

    // Step 9: the operation, as its core entry point takes it.
    let operation: Operation = operation_of(
        ready.sequence.clone(),
        scope.clone(),
        document.operation.clone(),
    )?;

    // Step 10: the design.
    let design: PrimerDesignRequest = design_of(document.design.clone())?;

    Ok(Prepared {
        request_sha256: request_sha256.clone(),
        document: document.clone(),
        template: ready.sequence.clone(),
        source: ready.source.clone(),
        annotations: ready.annotations.clone(),
        operation: operation.clone(),
        design: design.clone(),
    })
}

/// Steps 3 to 7 for either template form (§3.1).
fn template_of(directory: &Path, spec: &TemplateSpec) -> Result<Ready, RequestFailure> {
    let named: Named = match spec {
        TemplateSpec::Fasta { id, path, .. } => Named {
            id: id.clone(),
            path: path.clone(),
            location: "$.template.fasta".to_string(),
        },
        TemplateSpec::GenBank { id, path, .. } => Named {
            id: id.clone(),
            path: path.clone(),
            location: "$.template.genbank".to_string(),
        },
    };

    // Step 3: a template path is relative to the request's own directory
    // (plan §3.4), so an absolute path is refused rather than echoed (§2).
    if Path::new(&named.path.clone()).is_absolute() {
        return Err(RequestFailure::Request(refusal(
            RequestErrorCode::Invalid,
            named.location.clone(),
            "expected a path relative to the request file".to_string(),
        )));
    }

    let contents: Result<String, RequestFailure> =
        match read_to_string(directory.join(named.path.clone())) {
            Ok(found) => Ok(found),
            Err(cause) => Err(RequestFailure::Request(refusal(
                RequestErrorCode::TemplateUnreadable,
                named.location.clone(),
                format!(
                    "cannot read the template file '{}': {}",
                    named.path.clone(),
                    cause
                ),
            ))),
        };

    // Step 4: the sequence's identifier is the request's `template.id`, never
    // the file's header or LOCUS name (D007).
    let id: Identifier = Identifier::create(named.id.clone())
        .map_err(|problem| domain("$.template.id".to_string(), problem))?;
    let text: String = contents?;

    match spec {
        TemplateSpec::Fasta {
            topology,
            sha256,
            cds,
            id: _,
            path: _,
        } => {
            // Step 5: the FASTA record, whose topology the request states
            // (D041) and whose identifier came from step 4 (D007).
            let record: FastaRecord =
                read_fasta(id.clone(), text.clone(), topology_of(topology.clone()))
                    .map_err(|problem| domain(named.location.clone(), problem))?;
            let sequence: Sequence = record.sequence();

            // Step 6: `template.sha256` is the sequence's content digest.
            digest_of(sequence.clone(), sha256.clone())?;

            Ok(Ready {
                sequence: sequence.clone(),
                source: TemplateSource::Fasta {
                    path: named.path.clone(),
                    header: record.header(),
                },
                annotations: declared_of(sequence, cds.clone())?,
            })
        }
        TemplateSpec::GenBank {
            sha256,
            cds,
            id: _,
            path: _,
        } => {
            // Step 5: a GenBank template's topology comes from its LOCUS line
            // (D009, D041) and its identifier is still the request's (D007).
            let record: GenBankRecord = read_genbank(id.clone(), text.clone())
                .map_err(|problem| domain(named.location.clone(), problem))?;
            let sequence: Sequence = record.sequence();

            // Step 6.
            digest_of(sequence.clone(), sha256.clone())?;

            // Step 7: each requested id at `$.template.cds[i]`, then one
            // `import_cds` for all of them at `$.template.cds` (§3.1). The
            // annotations are the imported CDSs, in the order the request names
            // them.
            let names: Vec<Identifier> = requested_of(cds.clone())?;
            let imported: GenBankImport = import_cds(&record, &names)
                .map_err(|problem| domain("$.template.cds".to_string(), problem))?;
            Ok(Ready {
                sequence: sequence.clone(),
                source: TemplateSource::GenBank {
                    path: named.path.clone(),
                    locus_name: record.locus_name(),
                    accession: record.accession(),
                    version: record.version(),
                    import: imported.clone(),
                },
                annotations: imported.cds().iter().map(|entry| entry.cds()).collect(),
            })
        }
    }
}

/// Step 6: `template.sha256` is compared with the sequence's content digest,
/// the one `Sequence::content_hash` gives and provenance reports (§2).
fn digest_of(sequence: Sequence, given: Option<String>) -> Result<(), RequestFailure> {
    match given {
        Some(given_digest) => {
            let actual: String = sequence.content_hash().hex_digest().to_string();
            if actual != given_digest.clone() {
                return Err(RequestFailure::Request(refusal(
                    RequestErrorCode::TemplateDigestMismatch,
                    "$.template.sha256".to_string(),
                    format!(
                        "the template sequence's SHA-256 is {}, not {}",
                        actual, given_digest
                    ),
                )));
            }
            Ok(())
        }
        None => Ok(()),
    }
}

/// Step 7 for a FASTA template: each declared CDS, in the order the request
/// declares them (§3.1). An id that repeats one declared earlier is refused
/// with `request_invalid`, matching GenBank's `RepeatedName` (§2).
fn declared_of(sequence: Sequence, specs: Vec<CdsSpec>) -> Result<Vec<Cds>, RequestFailure> {
    let reference: SequenceReference = sequence.reference();
    let mut annotations: Vec<Cds> = Vec::new();
    for (index, declared_pair) in specs.iter().enumerate() {
        let declared: CdsSpec = declared_pair.clone();
        let position: String = format!("$.template.cds[{}]", index);

        // The id, at `$.template.cds[i].id`.
        let name: Identifier = Identifier::create(declared.id.clone())
            .map_err(|problem| domain(format!("{}.id", position), problem))?;
        if annotations.iter().any(|cds| cds.id() == name) {
            return Err(RequestFailure::Request(refusal(
                RequestErrorCode::Invalid,
                format!("{}.id", position),
                format!("CDS id '{}' is declared twice", declared.id),
            )));
        }

        // Each interval, at `$.template.cds[i].intervals[j]`.
        let intervals: Vec<NucleotideInterval> = declared_intervals(
            reference.clone(),
            declared.intervals.clone(),
            position.clone(),
        )?;

        // The CDS, at `$.template.cds[i]`, with what the request declares.
        let annotation: Cds = Cds::from_optional(
            &sequence.clone(),
            name.clone(),
            Some(intervals.clone()),
            Some(strand_of(declared.strand.clone())),
            Some(declared.completeness.clone()),
            Some(declared.genetic_code.clone()),
            Some(declared.numbering.clone()),
        )
        .map_err(|problem| domain(position.clone(), problem))?;
        annotations.push(annotation);
    }
    Ok(annotations)
}

/// One interval, built from the request element at `path` (§3.1).
fn region_of(
    reference: SequenceReference,
    pair: (i64, i64),
    path: String,
) -> Result<NucleotideInterval, RequestFailure> {
    match interval::create(&reference.clone(), pair.0, pair.1) {
        Ok(found) => Ok(found),
        Err(problem) => Err(domain(path.clone(), problem)),
    }
}

/// Step 7's intervals for one declared CDS, each at
/// `$.template.cds[i].intervals[j]` (§3.1).
fn declared_intervals(
    reference: SequenceReference,
    pairs: Vec<(i64, i64)>,
    position: String,
) -> Result<Vec<NucleotideInterval>, RequestFailure> {
    let mut found: Vec<NucleotideInterval> = Vec::new();
    for (index, pair) in pairs.iter().enumerate() {
        let area: NucleotideInterval = region_of(
            reference.clone(),
            *pair,
            format!("{}.intervals[{}]", position, index),
        )?;
        found.push(area);
    }
    Ok(found)
}

/// Step 7 for a GenBank template: the ids the request names, each validated at
/// `$.template.cds[i]` before `import_cds` sees them (§3.1).
fn requested_of(names: Vec<String>) -> Result<Vec<Identifier>, RequestFailure> {
    let mut found: Vec<Identifier> = Vec::new();
    for (index, text) in names.iter().enumerate() {
        let name: Identifier = Identifier::create(text.clone())
            .map_err(|problem| domain(format!("$.template.cds[{}]", index), problem))?;
        found.push(name);
    }
    Ok(found)
}

/// Step 8: the scope, when the request gives one. An absent scope states no
/// preference, which is what lets the library resolve the target (plan §3.4).
fn scope_of(text: String) -> Result<Identifier, RequestFailure> {
    match Identifier::create(text.clone()) {
        Ok(found) => Ok(found),
        Err(problem) => Err(domain("$.scope".to_string(), problem)),
    }
}

/// Step 9: the operation, built the way the core entry point that will run it
/// takes it (§3.1). `combined_construct` is refused here, by
/// `Q5VariantSetRequest::create` (§2, plan §3.4).
fn operation_of(
    template: Sequence,
    scope: Option<Identifier>,
    spec: OperationSpec,
) -> Result<Operation, RequestFailure> {
    match spec {
        OperationSpec::Edit(member) => {
            let built: Member = member_of(
                member.clone(),
                template.clone(),
                scope.clone(),
                "$.edit".to_string(),
            )?;
            Ok(Operation::Single {
                request: Box::new(built.request.clone()),
                name: built.name.clone(),
            })
        }
        OperationSpec::Variants {
            composition,
            members,
        } => {
            let mut variants: Vec<Q5Variant> = Vec::new();
            for (index, member) in members.iter().enumerate() {
                let at: String = format!("$.variants.members[{}]", index);
                let built: Member =
                    member_of(member.clone(), template.clone(), scope.clone(), at.clone())?;
                variants.push(Q5Variant::create(built.request.clone(), built.name.clone()));
            }
            let set: Q5VariantSetRequest =
                Q5VariantSetRequest::create(composition_of(composition.clone()), variants.clone())
                    .map_err(|problem| domain("$.variants".to_string(), problem))?;
            Ok(Operation::Set(set))
        }
        OperationSpec::Degenerate {
            name,
            interval: region,
            from,
            to,
        } => {
            let label: Identifier = Identifier::create(name.clone())
                .map_err(|problem| domain("$.degenerate.name".to_string(), problem))?;
            let area: NucleotideInterval = region_of(
                template.reference(),
                region,
                "$.degenerate.interval".to_string(),
            )?;
            let expansion: Q5Expansion = expand_replacement(
                template.clone(),
                scope.clone(),
                area.clone(),
                from,
                to,
                label.clone(),
            )
            .map_err(|problem| domain("$.degenerate".to_string(), problem))?;
            Ok(Operation::Degenerate(expansion))
        }
    }
}

/// One named member: its identifier, its intent and the core request that
/// carries it, each reported at the path the request wrote it (§3.1 step 9).
fn member_of(
    member: MemberSpec,
    template: Sequence,
    scope: Option<Identifier>,
    path: String,
) -> Result<Member, RequestFailure> {
    let reference: SequenceReference = template.reference();

    // `name`, at `P.name`.
    let name: Identifier = Identifier::create(member.name.clone())
        .map_err(|problem| domain(format!("{}.name", path), problem))?;

    // The intent, at `P.<kind>`.
    let intent: Q5MutationIntent = intent_of(member.edit.clone(), reference.clone(), path.clone())?;

    // The request, at `P`.
    let request: Q5SdmRequest =
        Q5SdmRequest::create(template.clone(), scope.clone(), intent.clone())
            .map_err(|problem| domain(path.clone(), problem))?;

    Ok(Member {
        name: name.clone(),
        request: request.clone(),
    })
}

/// The intent of one edit, built from the request element at `path` (§3.1
/// step 9). Every refusal names the intent's own path, `P.<kind>`.
fn intent_of(
    edit: EditSpec,
    reference: SequenceReference,
    path: String,
) -> Result<Q5MutationIntent, RequestFailure> {
    match edit {
        EditSpec::Residue {
            cds,
            number,
            from,
            to,
            codon,
        } => {
            let at: String = format!("{}.residue", path);
            let policy: CodonSelectionPolicy = codon_policy_of(codon.clone(), at.clone())?;
            match cds {
                Some(cds_name) => {
                    // The CDS the residue sits on is part of the position, so
                    // an identifier error for it is reported at the residue too
                    // (§3.1).
                    let claim: Identifier = Identifier::create(cds_name.clone())
                        .map_err(|problem| domain(at.clone(), problem))?;
                    let position: ResiduePosition = ResiduePosition::create(claim.clone(), number)
                        .map_err(|problem| domain(at.clone(), problem))?;
                    let intent: ResidueMutationIntent =
                        ResidueMutationIntent::create(position.clone(), from, to, policy.clone())
                            .map_err(|problem| domain(at.clone(), problem))?;
                    Ok(Q5MutationIntent::Residue(intent))
                }
                None => {
                    // The request has tied the residue number to no CDS, and
                    // inventing one is the library's target resolution (§2).
                    let intent: ResidueIntentWithoutCds =
                        ResidueIntentWithoutCds::create(number, from, to, policy.clone())
                            .map_err(|problem| domain(at, problem))?;
                    Ok(Q5MutationIntent::ResidueWithoutCds(intent))
                }
            }
        }
        EditSpec::Substitution {
            interval: region,
            from,
            to,
        } => {
            let at: String = format!("{}.substitution", path);
            let position: NucleotideInterval = region_of(reference.clone(), region, at.clone())?;
            let intent: NucleotideMutationIntent =
                NucleotideMutationIntent::create(reference.clone(), position.clone(), from, to)
                    .map_err(|problem| domain(at, problem))?;
            Ok(Q5MutationIntent::Nucleotide(intent))
        }
        EditSpec::Deletion {
            interval: region,
            from,
        } => {
            let at: String = format!("{}.deletion", path);
            let area: NucleotideInterval = region_of(reference.clone(), region, at.clone())?;
            let intent: NucleotideDeletionIntent =
                NucleotideDeletionIntent::create(reference.clone(), area.clone(), from)
                    .map_err(|problem| domain(at, problem))?;
            Ok(Q5MutationIntent::Deletion(intent))
        }
        EditSpec::Insertion {
            point,
            before,
            after,
            insert,
        } => {
            let at: String = format!("{}.insertion", path);
            let gap: InsertionPoint = interval::create_point(&reference.clone(), point)
                .map_err(|problem| domain(at.clone(), problem))?;
            let intent: NucleotideInsertionIntent = NucleotideInsertionIntent::create(
                reference.clone(),
                gap.clone(),
                before,
                after,
                insert,
            )
            .map_err(|problem| domain(at, problem))?;
            Ok(Q5MutationIntent::Insertion(intent))
        }
        EditSpec::Replacement {
            interval: region,
            from,
            to,
        } => {
            let at: String = format!("{}.replacement", path);
            let area: NucleotideInterval = region_of(reference.clone(), region, at.clone())?;
            let intent: NucleotideReplacementIntent =
                NucleotideReplacementIntent::create(reference.clone(), area.clone(), from, to)
                    .map_err(|problem| domain(at, problem))?;
            Ok(Q5MutationIntent::Replacement(intent))
        }
    }
}

/// The codon policy a residue edit states (§3.1 step 9): a supplied codon is a
/// `Codon` core validates, and the default is the table's own choice.
fn codon_policy_of(spec: CodonSpec, path: String) -> Result<CodonSelectionPolicy, RequestFailure> {
    match spec {
        CodonSpec::Supplied(text) => match Codon::create(text.clone()) {
            Ok(codon) => Ok(CodonSelectionPolicy::Supplied(codon)),
            Err(problem) => Err(domain(path.clone(), problem)),
        },
        CodonSpec::Default => Ok(CodonSelectionPolicy::Default),
    }
}

/// Step 10: the design. The buffer is Q5's own reaction rates, or the three
/// numbers the request states, and either way `BufferModel` validates it
/// before `PrimerDesignRequest::create` sees it (§3.1).
fn design_of(spec: DesignSpec) -> Result<PrimerDesignRequest, RequestFailure> {
    let buffer: BufferModel = match spec.buffer {
        BufferSpec::Q5Reaction => BufferModel::q5_reaction(),
        BufferSpec::Values {
            monovalent_millimolar,
            primer_nanomolar,
            dmso_percent,
        } => BufferModel::create(monovalent_millimolar, primer_nanomolar, dmso_percent)
            .map_err(|problem| domain("$.design.buffer".to_string(), problem))?,
    };

    let design: PrimerDesignRequest = PrimerDesignRequest::create(
        design_style_of(spec.style.clone()),
        design_policy_of(spec.policy.clone()),
        spec.min_annealing_overlap,
        placement_of(spec.placement.clone()),
        buffer,
    )
    .map_err(|problem| domain("$.design".to_string(), problem))?;
    Ok(design)
}

/// The topology the request states for a FASTA template (D041).
fn topology_of(spec: TopologySpec) -> SequenceTopology {
    match spec {
        TopologySpec::Linear => SequenceTopology::Linear,
        TopologySpec::Circular => SequenceTopology::Circular,
    }
}

/// The strand the request states for a declared CDS.
fn strand_of(spec: StrandSpec) -> Strand {
    match spec {
        StrandSpec::Plus => Strand::Plus,
        StrandSpec::Minus => Strand::Minus,
    }
}

/// The composition mode the request states (§3.5's two names).
fn composition_of(spec: CompositionSpec) -> Q5Composition {
    match spec {
        CompositionSpec::SeparateVariants => Q5Composition::SeparateVariants,
        CompositionSpec::CombinedConstruct => Q5Composition::CombinedConstruct,
    }
}

/// The design style the request states.
fn design_style_of(spec: StyleSpec) -> DesignStyle {
    match spec {
        StyleSpec::Q5Polymerase => DesignStyle::Q5Polymerase,
    }
}

/// The design policy the request states.
fn design_policy_of(spec: PolicySpec) -> DesignPolicy {
    match spec {
        PolicySpec::LuriaNative => DesignPolicy::LuriaNative,
    }
}

/// Where a placement may sit, as the request states it.
fn placement_of(spec: PlacementSpec) -> PrimerPlacement {
    match spec {
        PlacementSpec::Inline => PrimerPlacement::Inline,
        PlacementSpec::ConfineToTail => PrimerPlacement::ConfineToTail,
    }
}

/// Steps 1 to 10, in order (§3.1): the request file's path to everything the
/// workflow needs, or a failure that names the JSON path it came from and
/// carries what was known when it happened.
pub fn prepare(request_path: &Path) -> Result<Prepared, PrepareFailure> {
    let opened: Result<(String, RequestDocument), PrepareFailure> = open(request_path);
    match opened {
        Err(problem) => Err(problem),
        Ok((request_sha256, document)) => {
            let built: Result<Prepared, RequestFailure> =
                build(request_path, request_sha256.clone(), document.clone());
            match built {
                Ok(prepared) => Ok(prepared),
                Err(problem) => Err(carried(request_sha256.clone(), document.clone(), problem)),
            }
        }
    }
}
