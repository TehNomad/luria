//! result: the one JSON document `luria q5 design` prints, its provenance and
//! complete failed documents (milestone 5f1 §1).
//!
//! 5e1 turned request text into a `RequestDocument` and 5e2 prepared it into
//! core types; this module turns those types back into a document to print.
//! Every value here is a value the request stated or a `luria-core` accessor
//! gave: no coordinate, primer, sequence or experimental result is invented
//! (§1, D003). 5f2 fills the target, the members and the clarification
//! fields; 5f3 the designs; 5f4 the details of each error.
//!
//! # The rules this module enforces (§2)
//!
//! - one document for success, clarification and failure alike, with
//!   `schema`, `state` and `experimentalValidation` always present (plan §3.5,
//!   D020);
//! - the fixed shape: every top-level key is always present, `null` or `[]`
//!   when it does not apply, so a consumer never tests for a key's existence
//!   (decided in 5f1);
//! - `provenance` (D021): the package, the algorithms, the request file's
//!   SHA-256, the operation, the template's content digest, where it came from
//!   and the buffer; D041/G6 gives the buffer's numbers and the preset's name
//!   when one was used;
//! - the echoed `request` is itself a valid request: the request schema's own
//!   keys, in schema order, optional fields omitted as the input omitted them,
//!   and the buffer stated as the input stated it, so `parse_request` of the
//!   echo gives back the same `RequestDocument`; the preset's resolved numbers
//!   go in `provenance.buffer` instead;
//! - deterministic output: no timestamps, hostnames or absolute paths, keys in
//!   a fixed order (struct field order, 5d), floats in shortest round-trip
//!   form, and intervals as `[start, end]` pairs;
//! - every document is serde_json's pretty form plus one `\n`, built from
//!   `#[derive(Serialize)]` structs, never a `Value` or `json!` (5d);
//! - category names are `malformed_input`, `biological_validation` and
//!   `unsupported`; a CLI error is `malformed_input` with no rule, and only the
//!   six domain errors that carry a rule name one.

use serde::Serialize;
use serde_json::to_string_pretty;

use crate::prepare::{PrepareFailure, Prepared, RequestFailure, TemplateSource};
use crate::request::{
    BufferSpec, CdsSpec, CodonSpec, CompositionSpec, DesignSpec, EditSpec, MemberSpec,
    OperationSpec, PlacementSpec, PolicySpec, RequestDocument, RequestErrorCode, StrandSpec,
    StyleSpec, TemplateSpec, TopologySpec,
};
use crate::version::{algorithms, Algorithm};
use crate::REQUEST_SCHEMA;
use crate::RESULT_SCHEMA;

use luria_core::error::{DomainError, ErrorCategory};
use luria_core::genbank_cds::{GenBankImport, ImportedCds, UnusedFeature};
use luria_core::sequence::{Sequence, SequenceTopology};
use luria_core::tm::BufferModel;

/// The document `luria q5 design` prints (plan §3.5). Field order is the
/// document's key order (5d), and every key is always present: a consumer
/// never tests for a key's existence (decided in 5f1). `target`,
/// `clarifications`, `candidates`, `warnings` and `results` are filled in by
/// 5f2; only `None` and empty vectors appear in 5f1 (§3.1).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultDocument {
    pub schema: &'static str,
    pub state: &'static str,
    pub experimental_validation: &'static str,
    pub provenance: Provenance,
    pub request: Option<RequestEcho>,
    pub target: Option<CdsView>,
    pub clarifications: Vec<String>,
    pub candidates: Vec<String>,
    pub warnings: Vec<OperationWarning>,
    pub results: Vec<MemberView>,
    pub error: Option<ErrorView>,
}

/// The target CDS, filled in by 5f2 (§3.1).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CdsView {}

/// One edit or expansion warning, filled in by 5f2 (§3.1).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationWarning {}

/// One member's result, filled in by 5f2 (§3.1).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberView {}

/// The document as luria prints it: serde_json's pretty form plus one `\n`
/// (5d).
pub fn render(document: &ResultDocument) -> String {
    match to_string_pretty(document) {
        Ok(text) => text + "\n",
        Err(problem) => panic!("the result document always serializes: {}", problem),
    }
}

/// Where the result came from (D021): the package that ran, the algorithms it
/// can name, the request file's SHA-256, the operation, the template and the
/// buffer. `template` comes from a prepared request only (§3.2).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Provenance {
    pub package: PackageView,
    pub algorithms: Vec<Algorithm>,
    pub request_sha256: Option<String>,
    pub operation: Option<&'static str>,
    pub template: Option<TemplateView>,
    pub buffer: Option<BufferView>,
}

/// The package that ran: `luria-cli` and its version (D021).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageView {
    pub name: &'static str,
    pub version: &'static str,
}

/// The template: its identifier (D007), the digest of the sequence's content,
/// its length, its topology, where it came from and, for a GenBank record,
/// what it imported and what it did not (§3.2).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateView {
    pub id: String,
    pub sha256: String,
    pub length: u64,
    pub topology: String,
    pub source: SourceView,
    pub genbank: Option<GenBankView>,
}

/// Where the sequence came from: a FASTA record's path and header, or a
/// GenBank record's path and LOCUS name.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(
    tag = "format",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum SourceView {
    Fasta {
        path: String,
        header: String,
    },
    Genbank {
        path: String,
        locus_name: String,
        accession: Option<String>,
        version: Option<String>,
    },
}

/// A GenBank record: the CDSs the request imported, in the order it named
/// them, and every other feature, in file order (§3.2).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenBankView {
    pub imported_cds: Vec<ImportedCdsView>,
    pub unused_features: Vec<UnusedFeatureView>,
}

/// One imported CDS and where it came from (§3.2).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedCdsView {
    pub id: String,
    pub id_qualifier: &'static str,
    pub feature_index: usize,
    pub line: u64,
    pub transl_table: Option<String>,
}

/// One feature the request did not import (§3.2).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnusedFeatureView {
    pub index: usize,
    pub key: String,
    pub line: u64,
    pub id: Option<String>,
    pub refusal: Option<RefusalView>,
}

/// Why importing that feature would have refused it: its code, the rule that
/// refused it and the message (§3.2).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefusalView {
    pub code: &'static str,
    pub rule: Option<&'static str>,
    pub message: String,
}

/// The annealing buffer, resolved (D041, G6): the preset's name when one was
/// used, and the three numbers that were used, however they were reached.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferView {
    pub preset: Option<&'static str>,
    pub monovalent_millimolar: f64,
    pub primer_nanomolar: f64,
    pub dmso_percent: f64,
}

/// The package that ran.
fn package_view() -> PackageView {
    PackageView {
        name: "luria-cli",
        version: env!("CARGO_PKG_VERSION"),
    }
}

/// The template's identifier, which is the request's `template.id` and never
/// a file's header or LOCUS name (D007).
fn template_id_of(spec: &TemplateSpec) -> String {
    match spec {
        TemplateSpec::Fasta { id, .. } => id.clone(),
        TemplateSpec::GenBank { id, .. } => id.clone(),
    }
}

/// The topology as `luria-core` states it (D041).
fn topology_name(topology: SequenceTopology) -> String {
    topology.name()
}

/// Where the sequence came from, as `prepare` read it (§3.2).
fn source_view(source: &TemplateSource) -> SourceView {
    match source {
        TemplateSource::Fasta { path, header } => SourceView::Fasta {
            path: path.clone(),
            header: header.clone(),
        },
        TemplateSource::GenBank {
            path,
            locus_name,
            accession,
            version,
            ..
        } => SourceView::Genbank {
            path: path.clone(),
            locus_name: locus_name.clone(),
            accession: accession.clone(),
            version: version.clone(),
        },
    }
}

/// A GenBank source's import: the CDSs it imported, then every other feature
/// (§3.2). `None` is decided by the caller, which has no import for a FASTA.
fn genbank_view(import: &GenBankImport) -> GenBankView {
    let imported: Vec<ImportedCdsView> = import.cds().iter().map(imported_cds_view).collect();
    let unused: Vec<UnusedFeatureView> = import.unused().iter().map(unused_feature_view).collect();
    GenBankView {
        imported_cds: imported,
        unused_features: unused,
    }
}

/// One imported CDS and where it came from (§3.2).
fn imported_cds_view(entry: &ImportedCds) -> ImportedCdsView {
    ImportedCdsView {
        id: entry.cds().id().value().to_string(),
        id_qualifier: entry.id_qualifier().name(),
        feature_index: entry.index(),
        line: entry.line(),
        transl_table: entry.transl_table(),
    }
}

/// One feature the request did not import, and why importing it would
/// refuse it (§3.2).
fn unused_feature_view(entry: &UnusedFeature) -> UnusedFeatureView {
    UnusedFeatureView {
        index: entry.index(),
        key: entry.key(),
        line: entry.line(),
        id: entry.id(),
        refusal: entry.refusal().map(|problem| refusal_view(&problem)),
    }
}

/// A CDS refusal: its code, its rule and its message (§3.2).
fn refusal_view(error: &DomainError) -> RefusalView {
    RefusalView {
        code: error.code(),
        rule: rule_code(error),
        message: error.message(),
    }
}

/// The buffer, resolved (D041, G6): the preset `q5_reaction` gives Q5's own
/// reaction rates, and the object form gives the numbers the request stated.
fn buffer_view(spec: &BufferSpec) -> BufferView {
    match spec {
        BufferSpec::Q5Reaction => {
            let model: BufferModel = BufferModel::q5_reaction();
            BufferView {
                preset: Some("q5_reaction"),
                monovalent_millimolar: model.monovalent_millimolar(),
                primer_nanomolar: model.primer_nanomolar(),
                dmso_percent: model.dmso_percent(),
            }
        }
        BufferSpec::Values {
            monovalent_millimolar,
            primer_nanomolar,
            dmso_percent,
        } => BufferView {
            preset: None,
            monovalent_millimolar: *monovalent_millimolar,
            primer_nanomolar: *primer_nanomolar,
            dmso_percent: *dmso_percent,
        },
    }
}

/// The operation's name: `edit`, `variants` or `degenerate` (§3.2).
fn operation_name(spec: &OperationSpec) -> &'static str {
    match spec {
        OperationSpec::Edit(_) => "edit",
        OperationSpec::Variants { .. } => "variants",
        OperationSpec::Degenerate { .. } => "degenerate",
    }
}

/// Provenance for a result (D021, §3.2). `operation` and `buffer` come from
/// the request document, or from the prepared request's own document when
/// that is all there is; `template` comes from a prepared request only, and
/// `genbank` appears exactly for a GenBank source.
pub fn provenance(
    request_sha256: Option<String>,
    document: Option<&RequestDocument>,
    prepared: Option<&Prepared>,
) -> Provenance {
    let known: Option<&RequestDocument> = match document {
        Some(given) => Some(given),
        None => match prepared {
            Some(ready) => Some(&ready.document),
            None => None,
        },
    };
    Provenance {
        package: package_view(),
        algorithms: algorithms(),
        request_sha256: request_sha256.clone(),
        operation: match known {
            Some(given) => Some(operation_name(&given.operation)),
            None => None,
        },
        template: prepared.map(template_view),
        buffer: known.map(|given| buffer_view(&given.design.buffer)),
    }
}

/// The template's provenance, which only a prepared request has: its
/// identifier, the digest of the sequence's content, its length, its topology
/// and where it came from (§3.2).
fn template_view(ready: &Prepared) -> TemplateView {
    let sequence: Sequence = ready.template.clone();
    let topology: SequenceTopology = sequence.topology();
    TemplateView {
        id: template_id_of(&ready.document.template),
        sha256: sequence.content_hash().hex_digest().to_string(),
        length: sequence.length(),
        topology: topology_name(topology),
        source: source_view(&ready.source),
        genbank: match &ready.source {
            TemplateSource::GenBank { import, .. } => Some(genbank_view(import)),
            TemplateSource::Fasta { .. } => None,
        },
    }
}

/// The request as the result echoes it (§3.3): the request schema's own keys,
/// in schema order (5e1 §3.4), with the fixed values it must state, so
/// `parse_request` of the echo gives back the same `RequestDocument`. The
/// buffer is stated as the input stated it; the resolved numbers go in
/// `provenance.buffer` (§2).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestEcho {
    pub schema: &'static str,
    pub coordinates: &'static str,
    pub template: TemplateEcho,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(flatten)]
    pub operation: OperationEcho,
    pub design: DesignEcho,
}

/// The template, as the request writes it inside `template` (§3.3), in one of
/// its two forms: the `fasta` key says it is a FASTA request, the `genbank`
/// key says it is a GenBank one.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum TemplateEcho {
    Fasta(FastaTemplateEcho),
    GenBank(GenBankTemplateEcho),
}

/// A FASTA template, as the request writes it (§3.3): its identifier, the
/// file it names, its topology, the digest it states when it states one, and
/// what it declares about its CDS.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FastaTemplateEcho {
    pub id: String,
    pub fasta: String,
    pub topology: TopologyEcho,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    pub cds: Vec<CdsEcho>,
}

/// A GenBank template, as the request writes it (§3.3): its identifier, the
/// file it names, the digest it states when it states one, and the CDS it
/// names.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenBankTemplateEcho {
    pub id: String,
    pub genbank: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    pub cds: Vec<String>,
}

/// One declared CDS of a FASTA template, as the request declares it (§3.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CdsEcho {
    pub id: String,
    pub intervals: Vec<(i64, i64)>,
    pub strand: StrandEcho,
    pub completeness: String,
    pub genetic_code: String,
    pub numbering: String,
}

/// Which side of the sequence a CDS runs on, spelled as the request spells it
/// (§3.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum StrandEcho {
    #[serde(rename = "+")]
    Plus,
    #[serde(rename = "-")]
    Minus,
}

/// A FASTA sequence's topology, as the request states it (§3.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologyEcho {
    Linear,
    Circular,
}

/// The operation: exactly one of `edit`, `variants` and `degenerate` (§3.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum OperationEcho {
    Edit {
        #[serde(flatten)]
        member: MemberEcho,
    },
    Variants {
        composition: CompositionEcho,
        members: Vec<MemberEcho>,
    },
    Degenerate {
        name: String,
        interval: (i64, i64),
        from: String,
        to: String,
    },
}

/// Whether variants are built one at a time or as one construct (§3.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionEcho {
    SeparateVariants,
    CombinedConstruct,
}

/// One named edit: `name`, then the one key of its kind (§3.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberEcho {
    pub name: String,
    #[serde(flatten)]
    pub edit: EditEcho,
}

/// One edit, keyed by its kind (§3.3). A residue gives either `codon` or
/// `policy`, and `cds` only when the request gave one.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum EditEcho {
    Residue {
        #[serde(skip_serializing_if = "Option::is_none")]
        cds: Option<String>,
        number: i64,
        from: String,
        to: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        codon: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        policy: Option<&'static str>,
    },
    Substitution {
        interval: (i64, i64),
        from: String,
        to: String,
    },
    Deletion {
        interval: (i64, i64),
        from: String,
    },
    Insertion {
        point: i64,
        before: String,
        after: String,
        insert: String,
    },
    Replacement {
        interval: (i64, i64),
        from: String,
        to: String,
    },
}

/// The design: its planner, its style, the annealing bound, the placement and
/// the buffer, as the request states them (§3.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignEcho {
    pub policy: PolicyEcho,
    pub style: StyleEcho,
    pub min_annealing_overlap: i64,
    pub placement: PlacementEcho,
    pub buffer: BufferEcho,
}

/// The planner that runs the design, spelled as the request spells it (§3.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PolicyEcho {
    LuriaNative,
}

/// The style the planner designs in, spelled as the request spells it (§3.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleEcho {
    Q5Polymerase,
}

/// Where a placement may sit, as the request states it (§3.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementEcho {
    Inline,
    ConfineToTail,
}

/// The buffer as the request states it (§3.3): the preset `q5_reaction` or
/// the object of its three numbers. The resolved numbers are not echoed here;
/// they go in `provenance.buffer` (§2).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum BufferEcho {
    Preset(BufferPreset),
    Numbers(BufferValues),
}

/// The buffer preset the request names (§3.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BufferPreset {
    #[serde(rename = "q5_reaction")]
    Q5Reaction,
}

/// A buffer object: the three numbers the request states (§3.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferValues {
    pub monovalent_millimolar: f64,
    pub primer_nanomolar: f64,
    pub dmso_percent: f64,
}

/// The request as a valid request document (§3.3).
pub fn request_echo(document: &RequestDocument) -> RequestEcho {
    RequestEcho {
        schema: REQUEST_SCHEMA,
        coordinates: "zero-based-half-open",
        template: template_echo(&document.template),
        scope: document.scope.clone(),
        operation: operation_echo(&document.operation),
        design: design_echo(&document.design),
    }
}

/// The template, as the request wrote it (§3.3): its identifier, the file it
/// names, and what the request declared about it.
fn template_echo(spec: &TemplateSpec) -> TemplateEcho {
    match spec {
        TemplateSpec::Fasta {
            id,
            path,
            topology,
            sha256,
            cds,
        } => TemplateEcho::Fasta(FastaTemplateEcho {
            id: id.clone(),
            fasta: path.clone(),
            topology: topology_echo(topology),
            sha256: sha256.clone(),
            cds: cds.iter().map(cds_echo).collect(),
        }),
        TemplateSpec::GenBank {
            id,
            path,
            sha256,
            cds,
        } => TemplateEcho::GenBank(GenBankTemplateEcho {
            id: id.clone(),
            genbank: path.clone(),
            sha256: sha256.clone(),
            cds: cds.clone(),
        }),
    }
}

/// One declared CDS, as the request declared it (§3.3).
fn cds_echo(spec: &CdsSpec) -> CdsEcho {
    CdsEcho {
        id: spec.id.clone(),
        intervals: spec.intervals.clone(),
        strand: strand_echo(&spec.strand),
        completeness: spec.completeness.clone(),
        genetic_code: spec.genetic_code.clone(),
        numbering: spec.numbering.clone(),
    }
}

/// A CDS's strand, spelled as the request spells it (§3.3).
fn strand_echo(spec: &StrandSpec) -> StrandEcho {
    match spec {
        StrandSpec::Plus => StrandEcho::Plus,
        StrandSpec::Minus => StrandEcho::Minus,
    }
}

/// A FASTA sequence's topology, as the request states it (§3.3).
fn topology_echo(spec: &TopologySpec) -> TopologyEcho {
    match spec {
        TopologySpec::Linear => TopologyEcho::Linear,
        TopologySpec::Circular => TopologyEcho::Circular,
    }
}

/// The operation, as the request wrote it (§3.3): one edit, a set of variants
/// or a degenerate replacement.
fn operation_echo(spec: &OperationSpec) -> OperationEcho {
    match spec {
        OperationSpec::Edit(member) => OperationEcho::Edit {
            member: member_echo(member),
        },
        OperationSpec::Variants {
            composition,
            members,
        } => OperationEcho::Variants {
            composition: composition_echo(composition),
            members: members.iter().map(member_echo).collect(),
        },
        OperationSpec::Degenerate {
            name,
            interval,
            from,
            to,
        } => OperationEcho::Degenerate {
            name: name.clone(),
            interval: *interval,
            from: from.clone(),
            to: to.clone(),
        },
    }
}

/// The composition mode the request states (§3.3).
fn composition_echo(spec: &CompositionSpec) -> CompositionEcho {
    match spec {
        CompositionSpec::SeparateVariants => CompositionEcho::SeparateVariants,
        CompositionSpec::CombinedConstruct => CompositionEcho::CombinedConstruct,
    }
}

/// One named edit, as the request named it (§3.3): `name`, then the one key
/// of its kind.
fn member_echo(spec: &MemberSpec) -> MemberEcho {
    MemberEcho {
        name: spec.name.clone(),
        edit: edit_echo(&spec.edit),
    }
}

/// One edit, as the request wrote it (§3.3). A residue gives either a
/// supplied `codon` or `policy`, and `cds` only when the request gave one.
fn edit_echo(spec: &EditSpec) -> EditEcho {
    match spec {
        EditSpec::Residue {
            cds,
            number,
            from,
            to,
            codon,
        } => match codon {
            CodonSpec::Supplied(text) => EditEcho::Residue {
                cds: cds.clone(),
                number: *number,
                from: from.clone(),
                to: to.clone(),
                codon: Some(text.clone()),
                policy: None,
            },
            CodonSpec::Default => EditEcho::Residue {
                cds: cds.clone(),
                number: *number,
                from: from.clone(),
                to: to.clone(),
                codon: None,
                policy: Some("default"),
            },
        },
        EditSpec::Substitution { interval, from, to } => EditEcho::Substitution {
            interval: *interval,
            from: from.clone(),
            to: to.clone(),
        },
        EditSpec::Deletion { interval, from } => EditEcho::Deletion {
            interval: *interval,
            from: from.clone(),
        },
        EditSpec::Insertion {
            point,
            before,
            after,
            insert,
        } => EditEcho::Insertion {
            point: *point,
            before: before.clone(),
            after: after.clone(),
            insert: insert.clone(),
        },
        EditSpec::Replacement { interval, from, to } => EditEcho::Replacement {
            interval: *interval,
            from: from.clone(),
            to: to.clone(),
        },
    }
}

/// The design, as the request wrote it (§3.3).
fn design_echo(spec: &DesignSpec) -> DesignEcho {
    DesignEcho {
        policy: policy_echo(&spec.policy),
        style: style_echo(&spec.style),
        min_annealing_overlap: spec.min_annealing_overlap,
        placement: placement_echo(&spec.placement),
        buffer: buffer_echo(&spec.buffer),
    }
}

/// The planner the request states (§3.3).
fn policy_echo(spec: &PolicySpec) -> PolicyEcho {
    match spec {
        PolicySpec::LuriaNative => PolicyEcho::LuriaNative,
    }
}

/// The style the request states (§3.3).
fn style_echo(spec: &StyleSpec) -> StyleEcho {
    match spec {
        StyleSpec::Q5Polymerase => StyleEcho::Q5Polymerase,
    }
}

/// Where the request says a placement may sit (§3.3).
fn placement_echo(spec: &PlacementSpec) -> PlacementEcho {
    match spec {
        PlacementSpec::Inline => PlacementEcho::Inline,
        PlacementSpec::ConfineToTail => PlacementEcho::ConfineToTail,
    }
}

/// The buffer as the request states it (§3.3): the preset it names, or the
/// three numbers it states.
fn buffer_echo(spec: &BufferSpec) -> BufferEcho {
    match spec {
        BufferSpec::Q5Reaction => BufferEcho::Preset(BufferPreset::Q5Reaction),
        BufferSpec::Values {
            monovalent_millimolar,
            primer_nanomolar,
            dmso_percent,
        } => BufferEcho::Numbers(BufferValues {
            monovalent_millimolar: *monovalent_millimolar,
            primer_nanomolar: *primer_nanomolar,
            dmso_percent: *dmso_percent,
        }),
    }
}

/// An error: its code, the rule that refused it when one applies, the failure
/// class, one line of message and the JSON path of the request element it
/// came from (§3.4).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorView {
    pub code: &'static str,
    pub rule: Option<&'static str>,
    pub category: &'static str,
    pub message: String,
    pub path: Option<String>,
}

/// The failure class's name: `malformed_input`, `biological_validation` or
/// `unsupported` (§2).
pub fn category_name(category: ErrorCategory) -> &'static str {
    match category {
        ErrorCategory::MalformedInput => "malformed_input",
        ErrorCategory::BiologicalValidation => "biological_validation",
        ErrorCategory::Unsupported => "unsupported",
    }
}

/// The rule's code for the six variants that carry one, and `None` for every
/// other variant (§3.4).
pub fn rule_code(error: &DomainError) -> Option<&'static str> {
    match error {
        DomainError::InvalidMutationRequest(problem) => Some(problem.rule.code()),
        DomainError::InvalidPrimerCandidate(problem) => Some(problem.rule.code()),
        DomainError::MalformedFasta(problem) => Some(problem.rule.code()),
        DomainError::UnusableGenBankLocation(problem) => Some(problem.rule.code()),
        DomainError::MalformedGenBank(problem) => Some(problem.rule.code()),
        DomainError::UnusableGenBankCds(problem) => Some(problem.rule.code()),
        DomainError::EmptyIdentifier
        | DomainError::InvalidIdentifierCharacters(_)
        | DomainError::EmptySequenceContent(_)
        | DomainError::InvalidSequenceCharacters(_)
        | DomainError::NegativeCoordinate(_)
        | DomainError::ReversedInterval(_)
        | DomainError::EmptyInterval(_)
        | DomainError::CoordinateOutOfBounds(_)
        | DomainError::SequenceReferenceMismatch(_)
        | DomainError::FeatureWithoutIntervals(_)
        | DomainError::IntervalsOutOfOrder(_)
        | DomainError::OverlappingIntervals(_)
        | DomainError::UnsupportedFeatureStructure(_)
        | DomainError::MissingCdsAttribute(_)
        | DomainError::IncompleteCds(_)
        | DomainError::UnsupportedGeneticCode(_)
        | DomainError::CdsLengthNotDivisibleByThree(_)
        | DomainError::AmbiguousCodon(_)
        | DomainError::InvalidCodon(_)
        | DomainError::InternalStopCodon(_)
        | DomainError::InvalidResiduePosition(_)
        | DomainError::ResiduePositionOutsideCds(_)
        | DomainError::ResiduePositionWrongCds(_)
        | DomainError::UnsupportedNumberingScheme(_) => None,
        DomainError::UnspecifiedCodonPolicy(_)
        | DomainError::UnsupportedMutationPolicy(_)
        | DomainError::UnsupportedComposition(_)
        | DomainError::ExpectedNucleotideMismatch(_)
        | DomainError::ExpectedResidueMismatch(_)
        | DomainError::TargetOutsideFeature(_)
        | DomainError::MutationVerificationFailure(_) => None,
        DomainError::PrimerTextAtSpanMismatch(_)
        | DomainError::PrimerAnnealingNotTemplate(_)
        | DomainError::PrimerPartition(_)
        | DomainError::AnnealingOverlapBelowBound(_)
        | DomainError::PrimerPairCoverage(_)
        | DomainError::PrimerOrientationConflict(_)
        | DomainError::UnsupportedPrimerPlacement(_)
        | DomainError::InvalidPrimerDesignRequest(_)
        | DomainError::UnsupportedPrimerDesignScope(_)
        | DomainError::NoPrimerDesignPossible(_)
        | DomainError::NoPrimerDesignFits(_)
        | DomainError::InvalidBufferModel(_)
        | DomainError::TmParametersUnavailable(_)
        | DomainError::DegenerateExpansionTooLarge(_)
        | DomainError::GenBankTranslationMismatch(_) => None,
    }
}

/// A `luria-core` refusal, as the result reports it: its code, the rule that
/// refused it when one applies, its class, its message and the path the
/// caller gives for it (§3.4).
pub fn domain_error_view(error: &DomainError, path: Option<String>) -> ErrorView {
    ErrorView {
        code: error.code(),
        rule: rule_code(error),
        category: category_name(error.category()),
        message: error.message(),
        path: path.clone(),
    }
}

/// A request failure, as the result reports it (§3.4). A CLI error is
/// `malformed_input` with no rule; a domain failure keeps core's own code,
/// rule and class, at the path `prepare` named.
pub fn failure_view(failure: &RequestFailure) -> ErrorView {
    match failure {
        RequestFailure::Request(problem) => {
            let code: RequestErrorCode = problem.code;
            ErrorView {
                code: code.code(),
                rule: None,
                category: category_name(ErrorCategory::MalformedInput),
                message: problem.message.clone(),
                path: Some(problem.path.clone()),
            }
        }
        RequestFailure::Domain { path, error } => {
            domain_error_view(error.as_ref(), Some(path.clone()))
        }
    }
}

/// A request that could not be prepared, as the whole failed document
/// (§3.5). `request` is the echo of the request when it parsed, and the
/// provenance carries the digest of the file that was read.
pub fn failed_document(failure: &PrepareFailure) -> ResultDocument {
    let document: Option<&RequestDocument> = match &failure.document {
        Some(boxed) => Some(boxed.as_ref()),
        None => None,
    };
    let request: Option<RequestEcho> = document.map(request_echo);
    ResultDocument {
        schema: RESULT_SCHEMA,
        state: "failed",
        experimental_validation: "not_performed",
        provenance: provenance(failure.request_sha256.clone(), document, None),
        request,
        target: None,
        clarifications: [].to_vec(),
        candidates: [].to_vec(),
        warnings: [].to_vec(),
        results: [].to_vec(),
        error: Some(failure_view(&failure.failure)),
    }
}

/// A prepared request whose run failed (5g calls this): the whole failed
/// document, which keeps the template's provenance and echoes the request
/// it read (§3.5).
pub fn failed_after(prepared: &Prepared, error: ErrorView) -> ResultDocument {
    ResultDocument {
        schema: RESULT_SCHEMA,
        state: "failed",
        experimental_validation: "not_performed",
        provenance: provenance(
            Some(prepared.request_sha256.clone()),
            Some(&prepared.document),
            Some(prepared),
        ),
        request: Some(request_echo(&prepared.document)),
        target: None,
        clarifications: [].to_vec(),
        candidates: [].to_vec(),
        warnings: [].to_vec(),
        results: [].to_vec(),
        error: Some(error.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prepare::PrepareFailure;
    use crate::request::{parse_request, RequestError, RequestErrorCode};
    use luria_core::error::{InvalidMutationRequestProblem, MutationRequestRule};

    const SID1_REQUEST: &str = r#"{"schema":"luria.q5-request/1","coordinates":"zero-based-half-open","template":{"id":"sid-1","fasta":"sid-1_cDNA.fa","topology":"linear","cds":[{"id":"sid-1","intervals":[[0,3153]],"strand":"+","completeness":"complete","geneticCode":"standard","numbering":"from-cds-start"}]},"scope":"sid-1","edit":{"name":"D404N","residue":{"cds":"sid-1","number":404,"from":"D","to":"N","codon":"AAC"}},"design":{"policy":"luriaNative","style":"q5_polymerase","minAnnealingOverlap":15,"placement":"inline","buffer":"q5_reaction"}}"#;

    const SET_REQUEST: &str = r#"{"schema":"luria.q5-request/1","coordinates":"zero-based-half-open","template":{"id":"pbr322","genbank":"J01749.1.gb","sha256":"536042a7485bdabdf49c4d21a306213a098d77d19bdba57a1c6adc76ce2b8372","cds":["tet","bla"]},"variants":{"composition":"separate_variants","members":[{"name":"s1","substitution":{"interval":[99,100],"from":"T","to":"C"}},{"name":"d1","deletion":{"interval":[99,102],"from":"TGC"}},{"name":"i1","insertion":{"point":100,"before":"T","after":"G","insert":"GGG"}},{"name":"r1","replacement":{"interval":[99,102],"from":"TGC","to":"TT"}},{"name":"x1","residue":{"number":42,"from":"A","to":"G","policy":"default"}}]},"design":{"policy":"luriaNative","style":"q5_polymerase","minAnnealingOverlap":15,"placement":"confine_to_tail","buffer":{"monovalentMillimolar":50.0,"primerNanomolar":200.5,"dmsoPercent":0.0}}}"#;

    const DEGENERATE_REQUEST: &str = r#"{"schema":"luria.q5-request/1","coordinates":"zero-based-half-open","template":{"id":"sid-1","fasta":"sid-1_cDNA.fa","topology":"circular","cds":[]},"degenerate":{"name":"lib","interval":[1209,1212],"from":"GAT","to":"NNK"},"design":{"policy":"luriaNative","style":"q5_polymerase","minAnnealingOverlap":15,"placement":"inline","buffer":"q5_reaction"}}"#;

    fn algorithm_block(role: &str, name: &str, version: &str, last: bool) -> String {
        format!(
            "      {{\n        \"role\": \"{}\",\n        \"name\": \"{}\",\n        \"version\": \"{}\"\n      }}{}\n",
            role,
            name,
            version,
            if last { "" } else { "," }
        )
    }

    // The echo is the request itself, byte for byte when the input was
    // already in schema order and compact form, so it parses back to the
    // same document.
    #[test]
    fn the_echo_is_the_request_in_schema_order() {
        for text in [SID1_REQUEST, SET_REQUEST, DEGENERATE_REQUEST] {
            let document = parse_request(text).unwrap();
            let echo = serde_json::to_string(&request_echo(&document)).unwrap();
            assert_eq!(echo, text);
            assert_eq!(parse_request(&echo).unwrap(), document);
        }
    }

    #[test]
    fn the_buffer_is_resolved_in_provenance() {
        let preset = parse_request(SID1_REQUEST).unwrap();
        let view = provenance(None, Some(&preset), None);
        assert_eq!(view.operation, Some("edit"));
        assert_eq!(
            view.buffer,
            Some(BufferView {
                preset: Some("q5_reaction"),
                monovalent_millimolar: 150.0,
                primer_nanomolar: 500.0,
                dmso_percent: 0.0,
            })
        );
        let numbers = parse_request(SET_REQUEST).unwrap();
        let view = provenance(None, Some(&numbers), None);
        assert_eq!(view.operation, Some("variants"));
        assert_eq!(
            view.buffer,
            Some(BufferView {
                preset: None,
                monovalent_millimolar: 50.0,
                primer_nanomolar: 200.5,
                dmso_percent: 0.0,
            })
        );
        assert_eq!(view.template, None);
        let degenerate = parse_request(DEGENERATE_REQUEST).unwrap();
        assert_eq!(
            provenance(None, Some(&degenerate), None).operation,
            Some("degenerate")
        );
    }

    #[test]
    fn an_error_names_its_code_rule_category_and_path() {
        let error = DomainError::InvalidMutationRequest(InvalidMutationRequestProblem {
            request: "Z".to_string(),
            rule: MutationRequestRule::ResidueNotInStandardCode,
            reason: "the letter is not a residue of the standard genetic code".to_string(),
        });
        let view = domain_error_view(&error, Some("$.edit.residue".to_string()));
        assert_eq!(view.code, "invalid_mutation_request");
        assert_eq!(view.rule, Some("residue_not_in_standard_code"));
        assert_eq!(view.category, "malformed_input");
        assert_eq!(view.message, error.message());
        assert_eq!(view.path, Some("$.edit.residue".to_string()));
        assert_eq!(
            category_name(ErrorCategory::BiologicalValidation),
            "biological_validation"
        );
        assert_eq!(category_name(ErrorCategory::Unsupported), "unsupported");
        assert_eq!(rule_code(&DomainError::EmptyIdentifier), None);
    }

    // The whole failed document is pinned, so the fixed shape, the key order
    // and the layout are too.
    #[test]
    fn an_unreadable_request_gives_the_pinned_failed_document() {
        let failure = PrepareFailure {
            failure: RequestFailure::Request(RequestError {
                code: RequestErrorCode::Unreadable,
                path: "$".to_string(),
                message: "cannot read the request file: No such file or directory (os error 2)"
                    .to_string(),
            }),
            request_sha256: None,
            document: None,
        };
        let algorithms: Vec<crate::version::Algorithm> = crate::version::algorithms();
        let blocks: String = algorithms
            .iter()
            .enumerate()
            .map(|(index, algorithm)| {
                algorithm_block(
                    algorithm.role,
                    algorithm.name,
                    algorithm.version,
                    index + 1 == algorithms.len(),
                )
            })
            .collect();
        let expected: String = format!(
            "{{\n  \"schema\": \"luria.q5-result/1\",\n  \"state\": \"failed\",\n  \"experimentalValidation\": \"not_performed\",\n  \"provenance\": {{\n    \"package\": {{\n      \"name\": \"luria-cli\",\n      \"version\": \"{}\"\n    }},\n    \"algorithms\": [\n{}    ],\n    \"requestSha256\": null,\n    \"operation\": null,\n    \"template\": null,\n    \"buffer\": null\n  }},\n  \"request\": null,\n  \"target\": null,\n  \"clarifications\": [],\n  \"candidates\": [],\n  \"warnings\": [],\n  \"results\": [],\n  \"error\": {{\n    \"code\": \"request_unreadable\",\n    \"rule\": null,\n    \"category\": \"malformed_input\",\n    \"message\": \"cannot read the request file: No such file or directory (os error 2)\",\n    \"path\": \"$\"\n  }}\n}}\n",
            env!("CARGO_PKG_VERSION"),
            blocks
        );
        assert_eq!(render(&failed_document(&failure)), expected);
    }

    #[test]
    fn a_failure_after_parsing_echoes_the_request() {
        let document = parse_request(SID1_REQUEST).unwrap();
        let failure = PrepareFailure {
            failure: RequestFailure::Domain {
                path: "$.edit.residue".to_string(),
                error: Box::new(DomainError::EmptyIdentifier),
            },
            request_sha256: Some("ab".repeat(32)),
            document: Some(Box::new(document.clone())),
        };
        let failed = failed_document(&failure);
        assert_eq!(failed.state, "failed");
        assert_eq!(failed.experimental_validation, "not_performed");
        assert_eq!(failed.request, Some(request_echo(&document)));
        assert_eq!(failed.provenance.request_sha256, Some("ab".repeat(32)));
        assert_eq!(failed.provenance.operation, Some("edit"));
        assert_eq!(failed.provenance.template, None);
        let error = failed.error.unwrap();
        assert_eq!(error.code, "empty_identifier");
        assert_eq!(error.path, Some("$.edit.residue".to_string()));
    }
}
