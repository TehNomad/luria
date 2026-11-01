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

use serde::ser::SerializeMap;
use serde::Serialize;
use serde::Serializer;
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

use luria_core::cds::Cds;
use luria_core::error::{DesignObstacle, DomainError, ErrorCategory};
use luria_core::genbank_cds::{GenBankImport, ImportedCds, UnusedFeature};
use luria_core::identity::Identifier;
use luria_core::interval::OrderedIntervals;
use luria_core::mutation::MutationResult;
use luria_core::primer::{Primer, PrimerPair, PrimerPart, PrimerRole};
use luria_core::primer_design::{PrimerDesign, PrimerDesignRequest, UnresolvedInput};
use luria_core::primer_quality::PrimerWarning;
use luria_core::q5::{Q5EditWarning, Q5WorkflowResult};
use luria_core::q5_expand::Q5ExpansionWarning;
use luria_core::sequence::{Sequence, SequenceTopology};
use luria_core::strand::Strand;
use luria_core::tm::{BufferModel, MeltingTemperature};

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

/// The target CDS, as the result reports it (§3.1): its identifier, its
/// intervals as `[start, end]` pairs in D036's order, its strand, its
/// completeness, genetic code and numbering, and its total length.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CdsView {
    pub id: String,
    pub intervals: Vec<(u64, u64)>,
    pub strand: &'static str,
    pub completeness: String,
    pub genetic_code: String,
    pub numbering: String,
    pub length: u64,
}

/// The edited construct: its identifier, the digest of its content and its
/// length. The sequence text itself is not in the document (5g's
/// `--write-edited` writes it).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditedView {
    pub id: String,
    pub sha256: String,
    pub length: u64,
}

/// The mutation one member performed, keyed by its `kind`: the state it
/// reached, the recorded coordinates in D040's order, the text the core saw
/// and wrote, the codon and residues for a residue edit, and the edited
/// construct (§3.1).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum MutationView {
    Substitution {
        state: String,
        interval: (u64, u64),
        original: String,
        written: String,
        coordinates: Vec<u64>,
        edited: EditedView,
    },
    Residue {
        state: String,
        cds: String,
        residue: u64,
        original_residue: String,
        written_residue: String,
        original_codon: String,
        written_codon: String,
        codon_policy: String,
        codon_policy_source: Option<String>,
        coordinates: Vec<u64>,
        edited: EditedView,
    },
    Deletion {
        state: String,
        interval: (u64, u64),
        original: String,
        coordinates: Vec<u64>,
        edited: EditedView,
    },
    Insertion {
        state: String,
        point: u64,
        before: String,
        after: String,
        written: String,
        coordinates: Vec<u64>,
        edited: EditedView,
    },
    Replacement {
        state: String,
        interval: (u64, u64),
        original: String,
        written: String,
        coordinates: Vec<u64>,
        edited: EditedView,
    },
}

/// A warning about one member's edit, keyed by its `code` (§3.1).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(
    tag = "code",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum EditWarningView {
    Frameshift {
        cds: String,
        coding_bases_removed: u64,
        wholly_inside: bool,
        message: String,
    },
    InsertionFrameshift {
        cds: String,
        bases_inserted: u64,
        message: String,
    },
    ReplacementFrameshift {
        cds: String,
        bases_replaced: u64,
        bases_written: u64,
        wholly_inside: bool,
        message: String,
    },
    PrematureStop {
        cds: String,
        codon_number: u64,
        message: String,
    },
}

/// A warning about the operation as a whole, keyed by its `code` (§3.1): a
/// degenerate expansion that skipped a member, or a set that grew past its
/// threshold.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(
    tag = "code",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum OperationWarning {
    WildTypeSkipped {
        sequence: String,
        message: String,
    },
    ManyDesigns {
        count: usize,
        threshold: usize,
        message: String,
    },
}

/// One member's design (plan §3.5): the policy, style, algorithm and version
/// that produced it, the geometry the policy chose, both primers 5′→3′ with
/// their spans, tails, parts, annealing portions, Tms and guideline warnings,
/// the annealing temperature and balance, the pair's relationship and what the
/// design left unresolved. Every value is one core gave (D021, D028).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignView {
    pub policy: String,
    pub style: String,
    pub algorithm: AlgorithmRef,
    pub min_annealing_overlap: u64,
    pub placement: String,
    pub edit_span: (u64, u64),
    pub forward: PrimerView,
    pub reverse: PrimerView,
    pub annealing_temperature: f64,
    pub tm_balance_met: bool,
    pub pair: PairView,
    pub unresolved: Vec<UnresolvedView>,
}

/// The algorithm that ran the design, named with its version (D021, D028).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlgorithmRef {
    pub name: &'static str,
    pub version: &'static str,
}

/// One primer as it is to be typed into the kit, 5′→3′: its role, text, length,
/// the template interval it covers and the edited interval its text came from,
/// its 5′ tail when it has one, its parts, its annealing portion, its Tm and
/// whole-primer Tm, and its sequence guideline warnings (§3.5).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimerView {
    pub role: String,
    pub sequence: String,
    pub length: u64,
    pub span: (u64, u64),
    pub edited_span: Option<(u64, u64)>,
    pub tail: Option<TailView>,
    pub parts: Vec<PartView>,
    pub annealing: AnnealingView,
    pub tm: TmView,
    pub whole_primer_tm: Option<TmView>,
    pub warnings: Vec<PrimerWarningView>,
}

/// The 5′ tail a primer carries: its written text, and why the primer has one
/// where it has none (core's `tail_reason` states the absence, never the
/// presence).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TailView {
    pub sequence: String,
    pub reason: String,
}

/// One run of a primer: wild-type context that anneals, positions carrying the
/// mutation that do not, or a 5′ tail of written bases; the primer indices it
/// occupies and the template interval it sits at, absent for a tail (§3.5).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartView {
    pub role: String,
    pub annealing: bool,
    pub sequence: String,
    pub span: Option<(u64, u64)>,
    pub primer_start: u64,
    pub primer_end: u64,
}

/// The annealing portion of a primer: its text, its length, the overlap it
/// holds against the template beneath it and the template intervals it covers
/// (§3.5, I-1 and I-5).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnealingView {
    pub sequence: String,
    pub length: u64,
    pub overlap: u64,
    pub spans: Vec<(u64, u64)>,
}

/// A melting temperature: the value, the duplex it describes, why that is not
/// the basis the caller asked for when it is not, and the two terms it came
/// from. Floats are carried as core gave them, never rounded (§3.5).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TmView {
    pub celsius: f64,
    pub basis: String,
    pub substituted_because: Option<String>,
    pub enthalpy_kcal: f64,
    pub entropy_cal: f64,
}

/// The relationship between the two primers, derived by core from the two
/// spans and stated here rather than implied: the template positions between
/// the two annealing portions, the signed separation, where the 5′ ends meet,
/// whether that counts as abutting and the phrase core derived (§3.5, D031,
/// H5).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairView {
    pub deleted_width: u64,
    pub separation: i64,
    pub boundary: Option<u64>,
    pub abuts: bool,
    pub relationship: String,
}

/// An input the design contract names that this design did not use, and what
/// that means for the output (D028: the design states what it left unresolved).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnresolvedView {
    pub input: String,
    pub effect: String,
}

/// A sequence guideline warning about one primer, keyed by its `code`: core's
/// own measurement and core's own `describe` text, never a rephrasing of it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(
    tag = "code",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum PrimerWarningView {
    GcContentOutsideBand {
        gc_percent: f64,
        message: String,
    },
    SingleBaseRun {
        base: String,
        length: u64,
        start: u64,
        message: String,
    },
    DinucleotideRepeat {
        unit: String,
        repeats: u64,
        start: u64,
        message: String,
    },
    ThreePrimeGcRun {
        length: u64,
        message: String,
    },
    LengthOutsideGuideline {
        length: u64,
        message: String,
    },
    AboveOrderingLimit {
        length: u64,
        message: String,
    },
}

/// One member's result: its name, the mutation it performed, the warnings it
/// raised, and its design or the reason it had none (5f3 gives the last two)
/// (§3.1).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberView {
    pub name: String,
    pub mutation: MutationView,
    pub warnings: Vec<EditWarningView>,
    pub design: Option<DesignView>,
    pub design_failure: Option<ErrorView>,
}

/// The document as luria prints it: serde_json's pretty form plus one `\n`
/// (5d).
pub fn render(document: &ResultDocument) -> String {
    match to_string_pretty(document) {
        Ok(text) => text + "\n",
        Err(problem) => panic!("the result document always serializes: {}", problem),
    }
}

/// The strand, as the result writes it (D006): `+` or `-`.
fn strand_symbol(strand: Strand) -> &'static str {
    match strand {
        Strand::Plus => "+",
        Strand::Minus => "-",
    }
}

/// The target CDS, as the result reports it (§3.1): every value comes from a
/// `Cds` accessor, and the intervals keep D036's order as core gives them.
pub fn cds_view(cds: &Cds) -> CdsView {
    let intervals: OrderedIntervals = cds.intervals();
    CdsView {
        id: cds.id().value().to_string(),
        intervals: intervals
            .iter()
            .map(|interval| (interval.start(), interval.end()))
            .collect(),
        strand: strand_symbol(cds.strand()),
        completeness: cds.completeness().name(),
        genetic_code: cds.genetic_code().name(),
        numbering: cds.numbering().name(),
        length: cds.total_length(),
    }
}

/// The edited construct: its identifier, the digest of its content and its
/// length; the sequence text itself is not in the document (§3.1).
fn edited_view(sequence: &Sequence) -> EditedView {
    EditedView {
        id: sequence.name().value().to_string(),
        sha256: sequence.content_hash().hex_digest().to_string(),
        length: sequence.length(),
    }
}

/// One mutation result, as its member reports it (§3.1): the kind is the
/// operation core ran, and every value is one that result recorded. The
/// coordinates keep D040's order as `target_coordinates` gives them.
pub fn mutation_view(mutation: &MutationResult) -> MutationView {
    match mutation {
        MutationResult::Nucleotide(record) => MutationView::Substitution {
            state: record.state().name(),
            interval: (record.position().start(), record.position().end()),
            original: record.actual_original().to_string(),
            written: record.replacement().to_string(),
            coordinates: mutation.target_coordinates(),
            edited: edited_view(&mutation.edited()),
        },
        MutationResult::Residue(record) => MutationView::Residue {
            state: record.state().name(),
            cds: record.cds().value().to_string(),
            residue: record.position().number(),
            original_residue: record.actual_original().symbol(),
            written_residue: record.replacement().symbol(),
            original_codon: record.original_codon(),
            written_codon: record.selected_codon().text(),
            codon_policy: record.policy().name(),
            codon_policy_source: record.policy().source(),
            coordinates: mutation.target_coordinates(),
            edited: edited_view(&mutation.edited()),
        },
        MutationResult::Deletion(record) => MutationView::Deletion {
            state: record.state().name(),
            interval: (record.region().start(), record.region().end()),
            original: record.actual_deleted(),
            coordinates: mutation.target_coordinates(),
            edited: edited_view(&mutation.edited()),
        },
        MutationResult::Insertion(record) => MutationView::Insertion {
            state: record.state().name(),
            point: record.point().position(),
            before: record.actual_before().to_string(),
            after: record.actual_after().to_string(),
            written: record.inserted(),
            coordinates: mutation.target_coordinates(),
            edited: edited_view(&mutation.edited()),
        },
        MutationResult::Replacement(record) => MutationView::Replacement {
            state: record.state().name(),
            interval: (record.region().start(), record.region().end()),
            original: record.actual_original(),
            written: record.replacement(),
            coordinates: mutation.target_coordinates(),
            edited: edited_view(&mutation.edited()),
        },
    }
}

/// A warning about one member's edit, as the result reports it (§3.1): the
/// code, the values core counted, and core's own `describe` text.
pub fn edit_warning_view(warning: &Q5EditWarning) -> EditWarningView {
    match warning {
        Q5EditWarning::Frameshift {
            cds,
            coding_bases_removed,
            wholly_inside,
        } => EditWarningView::Frameshift {
            cds: cds.value().to_string(),
            coding_bases_removed: *coding_bases_removed,
            wholly_inside: *wholly_inside,
            message: warning.describe(),
        },
        Q5EditWarning::InsertionFrameshift {
            cds,
            bases_inserted,
        } => EditWarningView::InsertionFrameshift {
            cds: cds.value().to_string(),
            bases_inserted: *bases_inserted,
            message: warning.describe(),
        },
        Q5EditWarning::ReplacementFrameshift {
            cds,
            bases_replaced,
            bases_written,
            wholly_inside,
        } => EditWarningView::ReplacementFrameshift {
            cds: cds.value().to_string(),
            bases_replaced: *bases_replaced,
            bases_written: *bases_written,
            wholly_inside: *wholly_inside,
            message: warning.describe(),
        },
        Q5EditWarning::PrematureStop { cds, codon_number } => EditWarningView::PrematureStop {
            cds: cds.value().to_string(),
            codon_number: *codon_number,
            message: warning.describe(),
        },
    }
}

/// A warning about the operation as a whole: the code, the values core
/// counted, and core's own `describe` text (§3.1).
pub fn operation_warning_view(warning: &Q5ExpansionWarning) -> OperationWarning {
    match warning {
        Q5ExpansionWarning::WildTypeSkipped { sequence } => OperationWarning::WildTypeSkipped {
            sequence: sequence.clone(),
            message: warning.describe(),
        },
        Q5ExpansionWarning::ManyDesigns { count, threshold } => OperationWarning::ManyDesigns {
            count: *count,
            threshold: *threshold,
            message: warning.describe(),
        },
    }
}

/// One member: its name, its mutation, the warnings it raised, and no design
/// yet — 5f3 gives `design` and `designFailure` (§3.1).
pub fn member_view(
    name: &Identifier,
    mutation: &MutationResult,
    warnings: &[Q5EditWarning],
) -> MemberView {
    MemberView {
        name: name.value().to_string(),
        mutation: mutation_view(mutation),
        warnings: warnings.iter().map(edit_warning_view).collect(),
        design: None,
        design_failure: None,
    }
}

/// One member's design, as the result reports it (§3.5): every value comes from
/// a `PrimerDesign` accessor — the request it answers, the geometry, both primers,
/// the Tms, the balance and what was left unresolved — and nothing is computed
/// here.
pub fn design_view(design: &PrimerDesign) -> DesignView {
    let request: PrimerDesignRequest = design.request();
    DesignView {
        policy: request.policy().name(),
        style: request.style().name(),
        algorithm: algorithm_view(design),
        min_annealing_overlap: request.min_annealing_overlap(),
        placement: request.placement().name(),
        edit_span: design.edit_span(),
        forward: primer_view(design, PrimerRole::Forward),
        reverse: primer_view(design, PrimerRole::Reverse),
        annealing_temperature: design.annealing_temperature(),
        tm_balance_met: design.tm_balance_met(),
        pair: pair_view(design.pair()),
        unresolved: design.unresolved().iter().map(unresolved_view).collect(),
    }
}

/// The algorithm that ran this design, by name and version (D021, D028).
fn algorithm_view(design: &PrimerDesign) -> AlgorithmRef {
    AlgorithmRef {
        name: design.algorithm_name(),
        version: design.algorithm_version(),
    }
}

/// One of the design's two primers, as the result reports it (§3.5): the role
/// chooses which of the design's accessors give its Tm, its whole-primer Tm,
/// its warnings and the primer itself, so the reverse primer is never read
/// through the forward primer's accessors.
pub fn primer_view(design: &PrimerDesign, role: PrimerRole) -> PrimerView {
    let pair: PrimerPair = design.pair();
    let primer: Primer = match role {
        PrimerRole::Forward => pair.forward(),
        PrimerRole::Reverse => pair.reverse(),
    };
    let tm: MeltingTemperature = match role {
        PrimerRole::Forward => design.forward_tm(),
        PrimerRole::Reverse => design.reverse_tm(),
    };
    let whole_tm: Option<MeltingTemperature> = match role {
        PrimerRole::Forward => design.forward_whole_tm(),
        PrimerRole::Reverse => design.reverse_whole_tm(),
    };
    let warnings: Vec<PrimerWarning> = match role {
        PrimerRole::Forward => design.forward_warnings(),
        PrimerRole::Reverse => design.reverse_warnings(),
    };
    PrimerView {
        role: role.name(),
        sequence: primer.clone().sequence(),
        length: primer.clone().length(),
        span: (primer.clone().span().start(), primer.clone().span().end()),
        edited_span: primer
            .clone()
            .edited_span()
            .map(|interval| (interval.start(), interval.end())),
        tail: primer
            .clone()
            .tail()
            .map(|part| tail_view(part, primer.clone().tail_reason())),
        parts: primer.clone().parts().into_iter().map(part_view).collect(),
        annealing: annealing_view(primer),
        tm: tm_view(tm),
        whole_primer_tm: whole_tm.map(tm_view),
        warnings: warnings.iter().map(primer_warning_view).collect(),
    }
}

/// The pair's relationship to the edit, as core derived it from the two spans
/// (§3.5, D031, H5).
pub fn pair_view(pair: PrimerPair) -> PairView {
    PairView {
        deleted_width: pair.clone().deleted_width(),
        separation: pair.clone().separation(),
        boundary: pair.clone().boundary(),
        abuts: pair.clone().abuts(),
        relationship: pair.relationship(),
    }
}

/// An input the design left unresolved, and what that means for the output
/// (D028).
fn unresolved_view(input: &UnresolvedInput) -> UnresolvedView {
    UnresolvedView {
        input: input.input.clone(),
        effect: input.effect.clone(),
    }
}

/// The 5′ tail a primer carries: its written text and why it is there, which is
/// core's `tail_reason` — the reason the primer has no tail when it has none
/// (§3.5, D032, D033).
fn tail_view(part: PrimerPart, tail_reason: String) -> TailView {
    TailView {
        sequence: part.sequence(),
        reason: tail_reason,
    }
}

/// One run of a primer: its role, whether it anneals, its text, the template
/// interval it sits at and the primer indices it occupies (§3.5).
fn part_view(part: PrimerPart) -> PartView {
    PartView {
        role: part.clone().role().name(),
        annealing: part.clone().annealing(),
        sequence: part.clone().sequence(),
        span: part
            .clone()
            .span()
            .map(|interval| (interval.start(), interval.end())),
        primer_start: part.clone().primer_indices().0,
        primer_end: part.primer_indices().1,
    }
}

/// The annealing portion of a primer: its text, its length, the overlap it
/// holds against the template beneath it and the template intervals it covers
/// (§3.5).
fn annealing_view(primer: Primer) -> AnnealingView {
    AnnealingView {
        sequence: primer.clone().annealing_sequence(),
        length: primer.clone().annealing_length(),
        overlap: primer.clone().annealing_overlap(),
        spans: primer
            .annealing_spans()
            .iter()
            .map(|interval| (interval.start(), interval.end()))
            .collect(),
    }
}

/// A melting temperature: the value, the duplex it describes, why that is not
/// the basis the caller asked for when it is not, and the two terms it came
/// from. Nothing here is rounded or reformatted (§3.5).
fn tm_view(temperature: MeltingTemperature) -> TmView {
    TmView {
        celsius: temperature.clone().celsius(),
        basis: temperature.clone().basis().name(),
        substituted_because: temperature.clone().substituted_because(),
        enthalpy_kcal: temperature.clone().enthalpy_kcal(),
        entropy_cal: temperature.entropy_cal(),
    }
}

/// A sequence guideline warning about one primer: core's own measurement and
/// core's own `describe` text, and a run of one base given as its letter (§3.5).
pub fn primer_warning_view(warning: &PrimerWarning) -> PrimerWarningView {
    match warning {
        PrimerWarning::GcContentOutsideBand { gc_percent } => {
            PrimerWarningView::GcContentOutsideBand {
                gc_percent: *gc_percent,
                message: warning.describe(),
            }
        }
        PrimerWarning::SingleBaseRun {
            base,
            length,
            start,
        } => PrimerWarningView::SingleBaseRun {
            base: base.to_string(),
            length: *length,
            start: *start,
            message: warning.describe(),
        },
        PrimerWarning::DinucleotideRepeat {
            unit,
            repeats,
            start,
        } => PrimerWarningView::DinucleotideRepeat {
            unit: unit.clone(),
            repeats: *repeats,
            start: *start,
            message: warning.describe(),
        },
        PrimerWarning::ThreePrimeGcRun { length } => PrimerWarningView::ThreePrimeGcRun {
            length: *length,
            message: warning.describe(),
        },
        PrimerWarning::LengthOutsideGuideline { length } => {
            PrimerWarningView::LengthOutsideGuideline {
                length: *length,
                message: warning.describe(),
            }
        }
        PrimerWarning::AboveOrderingLimit { length } => PrimerWarningView::AboveOrderingLimit {
            length: *length,
            message: warning.describe(),
        },
    }
}

/// The member with its design: the design is attached and the design failure
/// stays absent, so a member states one or the other and never both (§3.5,
/// `docs/specs/q5-sdm.md` §12).
pub fn designed_member(member: MemberView, design: &PrimerDesign) -> MemberView {
    MemberView {
        name: member.name,
        mutation: member.mutation,
        warnings: member.warnings,
        design: Some(design_view(design)),
        design_failure: None,
    }
}

/// The member that could not be designed: its own error, at no path in the
/// request, and no design — the other members of a set are still designed
/// (§3.5, `docs/specs/q5-sdm.md` §12).
pub fn undesigned_member(member: MemberView, error: &DomainError) -> MemberView {
    MemberView {
        name: member.name,
        mutation: member.mutation,
        warnings: member.warnings,
        design: None,
        design_failure: Some(domain_error_view(error, None)),
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

/// One imported CDS, where it came from and where it sits (§3.2, C-12): the
/// intervals keep D036's order as core gives them, exactly as `cds_view` writes
/// a target's, so a `needs_clarification` document's candidates carry the
/// locations a consumer needs to name them.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedCdsView {
    pub id: String,
    pub intervals: Vec<(u64, u64)>,
    pub strand: &'static str,
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

/// One imported CDS and where it came from (§3.2, C-12).
fn imported_cds_view(entry: &ImportedCds) -> ImportedCdsView {
    let intervals: OrderedIntervals = entry.cds().intervals();
    ImportedCdsView {
        id: entry.cds().id().value().to_string(),
        intervals: intervals
            .iter()
            .map(|interval| (interval.start(), interval.end()))
            .collect(),
        strand: strand_symbol(entry.cds().strand()),
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

/// One value in an error's `details` (5f4 §3). Untagged: `Text` is a JSON
/// string, `Count` and `Signed` are JSON numbers, `Absent` is `null` for an
/// `Option` that holds nothing — the key is still present, so each code has one
/// fixed shape — and `Obstacles` is an array.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DetailValue {
    Text(String),
    Count(u64),
    Signed(i64),
    Absent,
    Obstacles(Vec<ObstacleView>),
}

/// An error's details: (key, value) pairs in the problem struct's field order,
/// serialized as a JSON object in that order (plan §3.5: keys in a fixed order).
/// The hand-written `impl Serialize` uses `serialize_map`; a `serde_json::Value`
/// or `Map` would sort its keys, which needs serde_json's `preserve_order`
/// feature (5d, §5).
#[derive(Clone, Debug, PartialEq)]
pub struct Details(pub Vec<(&'static str, DetailValue)>);

impl Serialize for Details {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in &self.0 {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

/// One reason a design did not fit (`DesignObstacle`), with core's own
/// `describe()` text as `message` (5f4 §3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum ObstacleView {
    FivePrimeAvailability {
        available: u64,
        needed: u64,
        message: String,
    },
    ThreePrimeAvailability {
        available: u64,
        needed: u64,
        message: String,
    },
    AmbiguousTemplateBase {
        position: u64,
        base: String,
        message: String,
    },
    FivePrimeTmFloorUnreachable {
        available: u64,
        reached_celsius: f64,
        message: String,
    },
    ThreePrimeTmFloorUnreachable {
        available: u64,
        reached_celsius: f64,
        message: String,
    },
}

/// An error: its code, the rule that refused it when one applies, the failure
/// class, one line of message, the JSON path of the request element it came
/// from and the payload fields as `details` (§3.4, 5f4).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorView {
    pub code: &'static str,
    pub rule: Option<&'static str>,
    pub category: &'static str,
    pub message: String,
    pub path: Option<String>,
    pub details: Details,
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
        | DomainError::GenBankTranslationMismatch(_)
        | DomainError::EmptySearchPattern
        | DomainError::InvalidSearchPatternCharacters(_)
        | DomainError::UnknownRecordFormat(_)
        | DomainError::RecordTooLong(_) => None,
    }
}

/// The payload fields of a domain error, as its `details` (5f4 §2, §3): one
/// entry per field of the variant's problem struct, in core's declaration
/// order, keyed by the field's name in camelCase. The six `rule` fields are
/// left out — the rule is already the error's own `rule` — as are the four
/// fields that hold nothing but interval text from core's `describe`, whose
/// text `message` already carries. `EmptyIdentifier` has no payload and the
/// errors that name no field carry one entry per payload field. There is no
/// `_` arm: a variant core gains is a compile error here (5f1).
pub fn error_details(error: &DomainError) -> Details {
    match error {
        DomainError::EmptyIdentifier => Details(vec![]),
        DomainError::InvalidIdentifierCharacters(problem) => Details(vec![
            ("identifier", DetailValue::Text(problem.identifier.clone())),
            ("reason", DetailValue::Text(problem.reason.clone())),
        ]),
        DomainError::EmptySequenceContent(problem) => Details(vec![(
            "sequence",
            DetailValue::Text(problem.sequence.clone()),
        )]),
        DomainError::InvalidSequenceCharacters(problem) => Details(vec![
            ("sequence", DetailValue::Text(problem.sequence.clone())),
            ("position", DetailValue::Count(problem.position)),
            ("characters", DetailValue::Text(problem.characters.clone())),
        ]),
        DomainError::NegativeCoordinate(problem) => Details(vec![
            ("sequence", DetailValue::Text(problem.sequence.clone())),
            ("start", DetailValue::Signed(problem.start)),
            ("end", DetailValue::Signed(problem.end)),
        ]),
        DomainError::ReversedInterval(problem) => Details(vec![
            ("sequence", DetailValue::Text(problem.sequence.clone())),
            ("start", DetailValue::Count(problem.start)),
            ("end", DetailValue::Count(problem.end)),
        ]),
        DomainError::EmptyInterval(problem) => Details(vec![
            ("sequence", DetailValue::Text(problem.sequence.clone())),
            ("position", DetailValue::Count(problem.position)),
        ]),
        DomainError::CoordinateOutOfBounds(problem) => Details(vec![
            ("sequence", DetailValue::Text(problem.sequence.clone())),
            ("start", DetailValue::Count(problem.start)),
            ("end", DetailValue::Count(problem.end)),
            ("length", DetailValue::Count(problem.length)),
        ]),
        DomainError::SequenceReferenceMismatch(problem) => Details(vec![
            (
                "expectedIdentifier",
                DetailValue::Text(problem.expected_identifier.clone()),
            ),
            (
                "actualIdentifier",
                DetailValue::Text(problem.actual_identifier.clone()),
            ),
            (
                "expectedDigest",
                DetailValue::Text(problem.expected_digest.clone()),
            ),
            (
                "actualDigest",
                DetailValue::Text(problem.actual_digest.clone()),
            ),
            (
                "expectedTopology",
                DetailValue::Text(problem.expected_topology.name()),
            ),
            (
                "actualTopology",
                DetailValue::Text(problem.actual_topology.name()),
            ),
        ]),
        DomainError::FeatureWithoutIntervals(problem) => Details(vec![
            ("feature", DetailValue::Text(problem.feature.clone())),
            ("sequence", DetailValue::Text(problem.sequence.clone())),
        ]),
        DomainError::IntervalsOutOfOrder(problem) => Details(vec![
            ("feature", DetailValue::Text(problem.feature.clone())),
            ("sequence", DetailValue::Text(problem.sequence.clone())),
        ]),
        DomainError::OverlappingIntervals(problem) => Details(vec![
            ("feature", DetailValue::Text(problem.feature.clone())),
            ("sequence", DetailValue::Text(problem.sequence.clone())),
            ("start", DetailValue::Count(problem.start)),
            ("end", DetailValue::Count(problem.end)),
        ]),
        DomainError::UnsupportedFeatureStructure(problem) => Details(vec![
            ("feature", DetailValue::Text(problem.feature.clone())),
            ("sequence", DetailValue::Text(problem.sequence.clone())),
            (
                "intervalCount",
                DetailValue::Count(problem.interval_count as u64),
            ),
            ("reason", DetailValue::Text(problem.reason.clone())),
        ]),
        DomainError::MissingCdsAttribute(problem) => Details(vec![
            ("cds", DetailValue::Text(problem.cds.clone())),
            ("attribute", DetailValue::Text(problem.attribute.clone())),
            (
                "value",
                match &problem.value {
                    Some(text) => DetailValue::Text(text.clone()),
                    None => DetailValue::Absent,
                },
            ),
        ]),
        DomainError::IncompleteCds(problem) => Details(vec![
            ("cds", DetailValue::Text(problem.cds.clone())),
            (
                "completeness",
                DetailValue::Text(problem.completeness.clone()),
            ),
        ]),
        DomainError::UnsupportedGeneticCode(problem) => Details(vec![
            ("cds", DetailValue::Text(problem.cds.clone())),
            (
                "geneticCode",
                DetailValue::Text(problem.genetic_code.clone()),
            ),
        ]),
        DomainError::CdsLengthNotDivisibleByThree(problem) => Details(vec![
            ("cds", DetailValue::Text(problem.cds.clone())),
            ("length", DetailValue::Count(problem.length)),
        ]),
        DomainError::AmbiguousCodon(problem) => Details(vec![
            ("cds", DetailValue::Text(problem.cds.clone())),
            (
                "codonNumber",
                DetailValue::Count(problem.codon_number as u64),
            ),
            ("codon", DetailValue::Text(problem.codon.clone())),
        ]),
        DomainError::InvalidCodon(problem) => Details(vec![
            ("cds", DetailValue::Text(problem.cds.clone())),
            (
                "codonNumber",
                DetailValue::Count(problem.codon_number as u64),
            ),
            ("codon", DetailValue::Text(problem.codon.clone())),
        ]),
        DomainError::InternalStopCodon(problem) => Details(vec![
            ("cds", DetailValue::Text(problem.cds.clone())),
            (
                "codonNumber",
                DetailValue::Count(problem.codon_number as u64),
            ),
            ("codon", DetailValue::Text(problem.codon.clone())),
        ]),
        DomainError::InvalidResiduePosition(problem) => Details(vec![
            ("cds", DetailValue::Text(problem.cds.clone())),
            ("number", DetailValue::Signed(problem.number)),
        ]),
        DomainError::ResiduePositionOutsideCds(problem) => Details(vec![
            ("cds", DetailValue::Text(problem.cds.clone())),
            ("number", DetailValue::Count(problem.number)),
            ("residueCount", DetailValue::Count(problem.residue_count)),
        ]),
        DomainError::ResiduePositionWrongCds(problem) => Details(vec![
            ("expected", DetailValue::Text(problem.expected.clone())),
            ("actual", DetailValue::Text(problem.actual.clone())),
        ]),
        DomainError::UnsupportedNumberingScheme(problem) => Details(vec![
            ("cds", DetailValue::Text(problem.cds.clone())),
            ("numbering", DetailValue::Text(problem.numbering.clone())),
        ]),
        DomainError::InvalidMutationRequest(problem) => Details(vec![
            ("request", DetailValue::Text(problem.request.clone())),
            ("reason", DetailValue::Text(problem.reason.clone())),
        ]),
        DomainError::UnspecifiedCodonPolicy(problem) => {
            Details(vec![("cds", DetailValue::Text(problem.cds.clone()))])
        }
        DomainError::UnsupportedMutationPolicy(problem) => {
            Details(vec![("policy", DetailValue::Text(problem.policy.clone()))])
        }
        DomainError::UnsupportedComposition(problem) => Details(vec![
            (
                "composition",
                DetailValue::Text(problem.composition.clone()),
            ),
            (
                "variantCount",
                DetailValue::Count(problem.variant_count as u64),
            ),
            ("reason", DetailValue::Text(problem.reason.clone())),
        ]),
        DomainError::ExpectedNucleotideMismatch(problem) => Details(vec![
            ("sequence", DetailValue::Text(problem.sequence.clone())),
            ("position", DetailValue::Count(problem.position)),
            ("expected", DetailValue::Text(problem.expected.clone())),
            ("actual", DetailValue::Text(problem.actual.clone())),
        ]),
        DomainError::ExpectedResidueMismatch(problem) => Details(vec![
            ("cds", DetailValue::Text(problem.cds.clone())),
            ("number", DetailValue::Count(problem.number)),
            ("expected", DetailValue::Text(problem.expected.clone())),
            ("actual", DetailValue::Text(problem.actual.clone())),
        ]),
        DomainError::TargetOutsideFeature(problem) => Details(vec![
            ("feature", DetailValue::Text(problem.feature.clone())),
            ("sequence", DetailValue::Text(problem.sequence.clone())),
            ("start", DetailValue::Count(problem.start)),
            ("end", DetailValue::Count(problem.end)),
        ]),
        DomainError::MutationVerificationFailure(problem) => Details(vec![
            ("result", DetailValue::Text(problem.result.clone())),
            ("reason", DetailValue::Text(problem.reason.clone())),
        ]),
        DomainError::InvalidPrimerCandidate(problem) => Details(vec![
            ("primer", DetailValue::Text(problem.primer.clone())),
            ("reason", DetailValue::Text(problem.reason.clone())),
        ]),
        DomainError::PrimerTextAtSpanMismatch(problem) => Details(vec![
            ("primer", DetailValue::Text(problem.primer.clone())),
            ("expected", DetailValue::Text(problem.expected.clone())),
            ("actual", DetailValue::Text(problem.actual.clone())),
        ]),
        DomainError::PrimerAnnealingNotTemplate(problem) => Details(vec![
            ("primer", DetailValue::Text(problem.primer.clone())),
            ("position", DetailValue::Count(problem.position)),
            ("expected", DetailValue::Text(problem.expected.clone())),
            ("actual", DetailValue::Text(problem.actual.clone())),
        ]),
        DomainError::PrimerPartition(problem) => Details(vec![
            ("primer", DetailValue::Text(problem.primer.clone())),
            ("reason", DetailValue::Text(problem.reason.clone())),
        ]),
        DomainError::AnnealingOverlapBelowBound(problem) => Details(vec![
            ("primer", DetailValue::Text(problem.primer.clone())),
            ("overlap", DetailValue::Count(problem.overlap)),
            ("bound", DetailValue::Count(problem.bound)),
        ]),
        DomainError::PrimerPairCoverage(_) => Details(vec![]),
        DomainError::PrimerOrientationConflict(problem) => Details(vec![
            ("forward", DetailValue::Text(problem.forward.clone())),
            ("reverse", DetailValue::Text(problem.reverse.clone())),
        ]),
        DomainError::UnsupportedPrimerPlacement(problem) => Details(vec![
            ("placement", DetailValue::Text(problem.placement.clone())),
            ("reason", DetailValue::Text(problem.reason.clone())),
        ]),
        DomainError::InvalidPrimerDesignRequest(problem) => Details(vec![
            ("field", DetailValue::Text(problem.field.clone())),
            ("reason", DetailValue::Text(problem.reason.clone())),
        ]),
        DomainError::UnsupportedPrimerDesignScope(problem) => Details(vec![
            ("policy", DetailValue::Text(problem.policy.clone())),
            ("reason", DetailValue::Text(problem.reason.clone())),
        ]),
        DomainError::NoPrimerDesignPossible(problem) => Details(vec![
            ("template", DetailValue::Text(problem.template.clone())),
            (
                "templateLength",
                DetailValue::Count(problem.template_length),
            ),
            ("smallestPair", DetailValue::Count(problem.smallest_pair)),
        ]),
        DomainError::NoPrimerDesignFits(problem) => Details(vec![
            ("template", DetailValue::Text(problem.template.clone())),
            ("editStart", DetailValue::Count(problem.edit_start)),
            ("editEnd", DetailValue::Count(problem.edit_end)),
            (
                "obstacles",
                DetailValue::Obstacles(
                    problem.obstacles.iter().map(design_obstacle_view).collect(),
                ),
            ),
        ]),
        DomainError::InvalidBufferModel(problem) => Details(vec![
            ("field", DetailValue::Text(problem.field.clone())),
            ("reason", DetailValue::Text(problem.reason.clone())),
        ]),
        DomainError::TmParametersUnavailable(problem) => Details(vec![
            ("function", DetailValue::Text(problem.function.clone())),
            ("aspect", DetailValue::Text(problem.aspect.clone())),
            ("reason", DetailValue::Text(problem.reason.clone())),
        ]),
        DomainError::DegenerateExpansionTooLarge(problem) => Details(vec![(
            "replacement",
            DetailValue::Text(problem.replacement.clone()),
        )]),
        DomainError::MalformedFasta(problem) => Details(vec![
            ("sequence", DetailValue::Text(problem.sequence.clone())),
            (
                "line",
                match problem.line {
                    Some(line) => DetailValue::Count(line),
                    None => DetailValue::Absent,
                },
            ),
        ]),
        DomainError::UnusableGenBankLocation(problem) => Details(vec![
            ("location", DetailValue::Text(problem.location.clone())),
            (
                "sequenceLength",
                DetailValue::Count(problem.sequence_length),
            ),
            (
                "cds",
                match &problem.cds {
                    Some(name) => DetailValue::Text(name.clone()),
                    None => DetailValue::Absent,
                },
            ),
            (
                "line",
                match problem.line {
                    Some(found) => DetailValue::Count(found),
                    None => DetailValue::Absent,
                },
            ),
        ]),
        DomainError::MalformedGenBank(problem) => Details(vec![
            ("sequence", DetailValue::Text(problem.sequence.clone())),
            (
                "line",
                match problem.line {
                    Some(line) => DetailValue::Count(line),
                    None => DetailValue::Absent,
                },
            ),
        ]),
        DomainError::UnusableGenBankCds(problem) => Details(vec![
            ("cds", DetailValue::Text(problem.cds.clone())),
            (
                "line",
                match problem.line {
                    Some(line) => DetailValue::Count(line),
                    None => DetailValue::Absent,
                },
            ),
            (
                "detail",
                match &problem.detail {
                    Some(text) => DetailValue::Text(text.clone()),
                    None => DetailValue::Absent,
                },
            ),
        ]),
        DomainError::GenBankTranslationMismatch(problem) => Details(vec![
            ("cds", DetailValue::Text(problem.cds.clone())),
            ("line", DetailValue::Count(problem.line)),
            ("residue", DetailValue::Count(problem.residue)),
            (
                "stated",
                match &problem.stated {
                    Some(text) => DetailValue::Text(text.clone()),
                    None => DetailValue::Absent,
                },
            ),
            (
                "translated",
                match &problem.translated {
                    Some(text) => DetailValue::Text(text.clone()),
                    None => DetailValue::Absent,
                },
            ),
        ]),
        // 8e: a pattern is something the caller wrote, so both refusals sit in
        // `MalformedInput` and take exit 4, `malformed_input`, like every other
        // input the caller wrote wrong. `EmptySearchPattern` names no field;
        // the other names its three, `position` already zero-based.
        DomainError::EmptySearchPattern => Details(vec![]),
        DomainError::InvalidSearchPatternCharacters(problem) => Details(vec![
            ("pattern", DetailValue::Text(problem.pattern.clone())),
            ("position", DetailValue::Count(problem.position)),
            ("characters", DetailValue::Text(problem.characters.clone())),
        ]),
        // 8b2: `open_record`'s two refusals (D049, D048). The format refusal is
        // about the input the caller supplied, so `MalformedInput`; the size
        // refusal is a limit luria does not go past, so `Unsupported`. Each
        // names its payload's fields, in core's declaration order.
        DomainError::UnknownRecordFormat(problem) => Details(vec![(
            "sequence",
            DetailValue::Text(problem.sequence.clone()),
        )]),
        DomainError::RecordTooLong(problem) => Details(vec![
            ("sequence", DetailValue::Text(problem.sequence.clone())),
            ("length", DetailValue::Count(problem.length)),
            ("limit", DetailValue::Count(problem.limit)),
        ]),
    }
}

/// One reason a design did not fit, as the result reports it: core's own
/// `describe` text as `message`, one entry per `DesignObstacle` field (§3).
fn design_obstacle_view(obstacle: &DesignObstacle) -> ObstacleView {
    match obstacle {
        DesignObstacle::FivePrimeAvailability { available, needed } => {
            ObstacleView::FivePrimeAvailability {
                available: *available,
                needed: *needed,
                message: obstacle.describe(),
            }
        }
        DesignObstacle::ThreePrimeAvailability { available, needed } => {
            ObstacleView::ThreePrimeAvailability {
                available: *available,
                needed: *needed,
                message: obstacle.describe(),
            }
        }
        DesignObstacle::AmbiguousTemplateBase { position, base } => {
            ObstacleView::AmbiguousTemplateBase {
                position: *position,
                base: base.clone(),
                message: obstacle.describe(),
            }
        }
        DesignObstacle::FivePrimeTmFloorUnreachable {
            available,
            reached_celsius,
        } => ObstacleView::FivePrimeTmFloorUnreachable {
            available: *available,
            reached_celsius: *reached_celsius,
            message: obstacle.describe(),
        },
        DesignObstacle::ThreePrimeTmFloorUnreachable {
            available,
            reached_celsius,
        } => ObstacleView::ThreePrimeTmFloorUnreachable {
            available: *available,
            reached_celsius: *reached_celsius,
            message: obstacle.describe(),
        },
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
        details: error_details(error),
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
                details: Details(vec![]),
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

/// The brief's fail-closed case (D020): the scope did not resolve, so nothing
/// was chosen and nothing was edited. The gate's clarifications and candidates
/// are the whole result: no target, no members, no error (§3.1).
pub fn clarification_document(prepared: &Prepared, gate: &Q5WorkflowResult) -> ResultDocument {
    ResultDocument {
        schema: RESULT_SCHEMA,
        state: "needs_clarification",
        experimental_validation: "not_performed",
        provenance: provenance(
            Some(prepared.request_sha256.clone()),
            Some(&prepared.document),
            Some(prepared),
        ),
        request: Some(request_echo(&prepared.document)),
        target: None,
        clarifications: gate.clarifications(),
        candidates: gate.candidates(),
        warnings: [].to_vec(),
        results: [].to_vec(),
        error: None,
    }
}

/// A run that edited every member (5g calls this): the resolved target, the
/// members in order and the operation's own warnings. Designs are 5f3's, so
/// every member's `design` and `designFailure` are still `null` (§3.1).
pub fn verified_document(
    prepared: &Prepared,
    target: Option<&Cds>,
    members: Vec<MemberView>,
    warnings: Vec<OperationWarning>,
) -> ResultDocument {
    ResultDocument {
        schema: RESULT_SCHEMA,
        state: "verified",
        experimental_validation: "not_performed",
        provenance: provenance(
            Some(prepared.request_sha256.clone()),
            Some(&prepared.document),
            Some(prepared),
        ),
        request: Some(request_echo(&prepared.document)),
        target: target.map(cds_view),
        clarifications: [].to_vec(),
        candidates: [].to_vec(),
        warnings,
        results: members,
        error: None,
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
            "{{\n  \"schema\": \"luria.q5-result/1\",\n  \"state\": \"failed\",\n  \"experimentalValidation\": \"not_performed\",\n  \"provenance\": {{\n    \"package\": {{\n      \"name\": \"luria-cli\",\n      \"version\": \"{}\"\n    }},\n    \"algorithms\": [\n{}    ],\n    \"requestSha256\": null,\n    \"operation\": null,\n    \"template\": null,\n    \"buffer\": null\n  }},\n  \"request\": null,\n  \"target\": null,\n  \"clarifications\": [],\n  \"candidates\": [],\n  \"warnings\": [],\n  \"results\": [],\n  \"error\": {{\n    \"code\": \"request_unreadable\",\n    \"rule\": null,\n    \"category\": \"malformed_input\",\n    \"message\": \"cannot read the request file: No such file or directory (os error 2)\",\n    \"path\": \"$\",\n    \"details\": {{}}\n  }}\n}}\n",
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
