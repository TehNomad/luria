//! Primer design under the Luria-native policy (D029, amended by D030).
//!
//! This is the step `primer.rs` deliberately leaves out: from a verified
//! `MutationResult` to a primer pair. `primer.rs` decides whether a proposed
//! primer is one; this module proposes the pair the policy defines and then
//! hands it to `construct_primer`, `assemble_pair` and
//! `require_design_constraints`, so every hard requirement of
//! `docs/q5-primer-design.md` §7 is checked on the output by the same code that
//! checks any other primer.
//!
//! # The policy
//!
//! For a contiguous replacement region `[s, e)`, `L = e − s`, `1 ≤ L ≤ 3`, on a
//! linear template of length `n`, with `b = minAnnealingOverlap`:
//!
//! ```text
//! B  = s − 10            the junction; the forward primer's 5′ flank is 10, fixed
//! v0 = max(10, b − 10)   the forward primer's 3′ flank, to start with
//! r0 = max(b, 15)        the reverse primer's length, to start with
//!
//! forward  covers [B, e + v), text from the edited artifact
//! reverse  covers [B − r, B), reverse complement of the template
//! ```
//!
//! The primers abut at `B` and the junction never moves: both grow only at
//! their 3′ ends (D030). Each primer grows one base at a time until its Tm is
//! at least `TM_FLOOR`, then toward a G or C last base while its Tm is below
//! `GC_CEILING`, returning to the floor length if it reaches none — forward
//! first, then reverse. If the two Tms then differ by more than
//! `TM_BALANCE`, the colder primer grows until they are within it, then toward
//! a G/C last base while it stays at or below the other's Tm, keeping that
//! length only if it ends in G/C; otherwise it keeps the balanced length.
//! Balance and the G/C end are preferences: running out of template leaves
//! them unmet and the design is still emitted, recording it. The floor is a
//! requirement.
//!
//! The forward Tm is mismatch-aware against the wild-type site where published
//! parameters cover the mismatch pattern, and matched where two mismatches are
//! adjacent (`tm::melting_temperature_or_matched`); the reverse primer carries
//! no mismatch and uses the matched Tm. The annealing temperature is the lower
//! Tm plus `TA_OFFSET`, capped at `TA_CAP`. D030 records the source of every
//! threshold.
//!
//! D033 designs a `MutationResult::Replacement` of `[s, e)` by `m` bases on one
//! of these two geometries, by `t = max(w, m)` with `w = e − s`: at or below
//! `MAX_INLINE_REPLACEMENT` it is the inline geometry above, whose forward primer
//! reads the edited artifact and is therefore `m − w` longer or shorter than the
//! template span it names (and whose Tm is matched when `m ≠ w`, since no
//! position-by-position alignment exists then); above it the replacement is split
//! onto the two 5′ tails exactly as D032's insertion is, the point replaced by the
//! region's two edges: the forward primer anneals at `[e, e + f)` and the reverse
//! at `[s − r, s)`.
//!
//! # Failure, by cause (D029, D030)
//!
//! * `NoPrimerDesignPossible` (`BiologicalValidation`): `L + 10 + v0 + r0 > n`,
//!   so no abutting pair of the starting size fits anywhere on the template;
//! * `NoPrimerDesignFits` (`Unsupported`): a pair of that size fits the
//!   template but not around this edit, an ambiguity code sits under the
//!   starting pair (H6), or a primer runs out of template before reaching the
//!   Tm floor. Each obstacle is named; the junction is never shifted;
//! * refused before the search: `InvalidPrimerDesignRequest`
//!   (`MalformedInput`) for a bound below 1 or an unverified result;
//!   `UnsupportedPrimerDesignScope` or `UnsupportedPrimerPlacement`
//!   (`Unsupported`) for a request outside the policy.

use crate::error::DesignObstacle;
use crate::error::DomainError;
use crate::error::InvalidPrimerDesignRequestProblem;
use crate::error::NoPrimerDesignFitsProblem;
use crate::error::NoPrimerDesignPossibleProblem;
use crate::error::UnsupportedPrimerDesignScopeProblem;
use crate::error::UnsupportedPrimerPlacementProblem;
use crate::identity::SequenceReference;
use crate::interval::create;
use crate::interval::require_sequence;
use crate::interval::NucleotideInterval;
use crate::mutation::MutationResult;
use crate::primer::assemble_pair;
use crate::primer::construct_primer;
use crate::primer::construct_tailed_primer;
use crate::primer::require_design_constraints;
use crate::primer::Primer;
use crate::primer::PrimerPair;
use crate::primer::PrimerPlacement;
use crate::primer::PrimerRole;
use crate::primer_quality::inspect;
use crate::primer_quality::PrimerWarning;
use crate::sequence::canonical_text;
use crate::sequence::is_dna_base;
use crate::sequence::reverse_complement;
use crate::sequence::Sequence;
use crate::sequence::SequenceTopology;
use crate::tm::melting_temperature;
use crate::tm::melting_temperature_or_matched;
use crate::tm::BufferModel;
use crate::tm::MeltingTemperature;

/// Identity of this design code path (D021).
pub const NATIVE_DESIGN_ALGORITHM_NAME: &str = "luria-core-primer-design-native";

/// Version of this design code path (D021). 0.2.0 is D030's Tm-driven sizing;
/// 0.3.0 adds D031's deletion design, 0.4.0 adds D032's insertion design and
/// 0.5.0 adds D033's replacement design, each with the earlier output unchanged.
pub const NATIVE_DESIGN_ALGORITHM_VERSION: &str = "0.5.0";

/// D030: the forward primer's 5′ flank, fixed, and the shortest 3′ flank.
pub const MIN_FLANK: u64 = 10;

/// D029: the shortest reverse primer.
pub const MIN_REVERSE: u64 = 15;

/// D029: the longest replacement region in the native scope (a codon).
pub const MAX_REPLACEMENT_REGION: u64 = 3;

/// D032: the longest insertion a single 5′ tail carries. At or below this the
/// whole insertion goes onto the forward primer's tail and the reverse primer
/// gets none; above it the insertion is split between the two primers. The
/// threshold is the reference tool's help text: "If 6 or fewer nucleotides are
/// to be inserted, they will be placed at the 5′ end of one primer. If more
/// than 6 … they will be split between the two primers to keep the primer
/// lengths and Tm's similar" (`docs/research/nbc_help_v284.txt`).
pub const MAX_WHOLE_TAIL: u64 = 6;

/// D033: the largest replacement that stays inline, on `t = max(w, m)`: the
/// wider of the region `[s, e)` and the text written in its place. At or below
/// this the replacement is designed by the inline geometry above, with the
/// region in place of the substituted bases; above it the replacement is split
/// onto the two 5′ tails, as D032's insertion is at the same threshold.
pub const MAX_INLINE_REPLACEMENT: u64 = 6;

/// D030: the lowest Tm a primer may have, °C.
pub const TM_FLOOR: f64 = 55.0;

/// D030: extension toward a G/C last base stops once a primer's Tm reaches
/// this, °C.
pub const GC_CEILING: f64 = 63.0;

/// D030: the largest preferred difference between the two primers' Tms, °C.
pub const TM_BALANCE: f64 = 5.0;

/// D030: annealing temperature is the lower Tm plus this, °C (Q5 "Tm+3").
pub const TA_OFFSET: f64 = 3.0;

/// D030: the highest annealing temperature reported, °C (top of the Q5 SDM
/// protocol's range).
pub const TA_CAP: f64 = 72.0;

/// The design style (§2.2). Only Q5 is in scope (D014).
#[derive(Clone, Debug, PartialEq)]
pub enum DesignStyle {
    Q5Polymerase,
}

impl DesignStyle {
    pub fn name(&self) -> String {
        match self {
            DesignStyle::Q5Polymerase => "q5_polymerase".to_string(),
        }
    }
}

/// The design policy (D028). Required, never defaulted. Only the native policy
/// is implemented; a compatibility target is not representable yet.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignPolicy {
    LuriaNative,
}

impl DesignPolicy {
    pub fn name(&self) -> String {
        match self {
            DesignPolicy::LuriaNative => "luriaNative".to_string(),
        }
    }
}

/// A primer-design request (§2.2), restricted to the fields the native policy
/// can act on. The buffer is required, because the same reference tool uses
/// different salt and primer concentrations at different call sites and a
/// default would be a decision the caller did not make. `minTm` and
/// `maxAnnealingOverlap` are not accepted; the design reports them as
/// unresolved.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerDesignRequest {
    style: DesignStyle,
    policy: DesignPolicy,
    min_annealing_overlap: u64,
    placement: PrimerPlacement,
    buffer: BufferModel,
}

impl PrimerDesignRequest {
    /// `min_annealing_overlap` is signed so that a negative value is reported
    /// rather than wrapped; it must be at least 1 (§2.2).
    pub fn create(
        style: DesignStyle,
        policy: DesignPolicy,
        min_annealing_overlap: i64,
        placement: PrimerPlacement,
        buffer: BufferModel,
    ) -> Result<PrimerDesignRequest, DomainError> {
        if min_annealing_overlap < 1 {
            let problem = InvalidPrimerDesignRequestProblem {
                field: "minAnnealingOverlap".to_string(),
                reason: format!(
                    "{} is not a bound; §2.2 requires an integer of at least 1",
                    min_annealing_overlap
                ),
            };
            return Err(DomainError::InvalidPrimerDesignRequest(problem));
        }
        Ok(PrimerDesignRequest {
            style,
            policy,
            min_annealing_overlap: min_annealing_overlap as u64,
            placement,
            buffer,
        })
    }

    pub fn style(&self) -> DesignStyle {
        self.style.clone()
    }

    pub fn policy(&self) -> DesignPolicy {
        self.policy.clone()
    }

    pub fn min_annealing_overlap(&self) -> u64 {
        self.min_annealing_overlap
    }

    pub fn placement(&self) -> PrimerPlacement {
        self.placement.clone()
    }

    pub fn buffer(&self) -> BufferModel {
        self.buffer
    }
}

/// An input the contract names that this design did not use, and what that
/// means for the output (§10).
#[derive(Clone, Debug, PartialEq)]
pub struct UnresolvedInput {
    pub input: String,
    pub effect: String,
}

/// A primer design (§3): the pair, the request it answers, the template it was
/// built against, the geometry and Tms the policy chose, and what was left
/// unresolved.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerDesign {
    request: PrimerDesignRequest,
    built_against: SequenceReference,
    edited: SequenceReference,
    edit_start: u64,
    edit_end: u64,
    five_prime_flank: u64,
    three_prime_flank: u64,
    reverse_length: u64,
    pair: PrimerPair,
    forward_tm: MeltingTemperature,
    reverse_tm: MeltingTemperature,
    forward_whole_tm: Option<MeltingTemperature>,
    reverse_whole_tm: Option<MeltingTemperature>,
    annealing_temperature: f64,
    tm_balance_met: bool,
    forward_warnings: Vec<PrimerWarning>,
    reverse_warnings: Vec<PrimerWarning>,
    unresolved: Vec<UnresolvedInput>,
}

impl PrimerDesign {
    pub fn request(&self) -> PrimerDesignRequest {
        self.request.clone()
    }

    /// The pristine template, by identity (§3 `builtAgainst`, I-6).
    pub fn built_against(&self) -> SequenceReference {
        self.built_against.clone()
    }

    /// The edited artifact the forward primer's text comes from.
    pub fn edited(&self) -> SequenceReference {
        self.edited.clone()
    }

    /// The edited region `[s, e)` on the template: the replacement region for
    /// a substitution, the removed region for a deletion (D031), and the point
    /// twice, `(p, p)`, for an insertion (D032), which occupies no template
    /// base of its own.
    pub fn edit_span(&self) -> (u64, u64) {
        (self.edit_start, self.edit_end)
    }

    /// The forward primer's unchanged bases 5′ of the edit. `MIN_FLANK` for a
    /// substitution (D030); 0 for a deletion (D031), whose forward primer's 5′
    /// end sits at the far edge of the removed region, and 0 for an insertion
    /// (D032), whose forward primer's 5′ end is its tail.
    pub fn five_prime_flank(&self) -> u64 {
        self.five_prime_flank
    }

    /// For a substitution, the forward primer's unchanged bases 3′ of the
    /// replacement region. For a deletion, the forward primer's whole length:
    /// it has no 5′ flank, so its length is its annealing total. For an
    /// insertion (D032), the forward primer's **annealing** length: its 5′
    /// tail carries inserted bases and anneals nowhere, so it is not counted.
    pub fn three_prime_flank(&self) -> u64 {
        self.three_prime_flank
    }

    /// The forward primer's annealing total, both flanks.
    pub fn forward_annealing_total(&self) -> u64 {
        self.five_prime_flank + self.three_prime_flank
    }

    /// The reverse primer's length, which for a substitution and a deletion is
    /// also its whole length. For an insertion (D032) it is the reverse
    /// primer's **annealing** length: a 5′ tail, when the insertion was split,
    /// is not counted here.
    pub fn reverse_length(&self) -> u64 {
        self.reverse_length
    }

    pub fn pair(&self) -> PrimerPair {
        self.pair.clone()
    }

    /// The number of template positions between the pair's two annealing
    /// portions: the deleted region for a deletion, the replaced region for a
    /// tail-path replacement, and 0 otherwise (D031, D033).
    pub fn deleted_width(&self) -> u64 {
        self.pair.clone().deleted_width()
    }

    /// The forward primer's Tm, with the basis it was computed on: mismatch-
    /// aware against the wild-type site for a substitution (D030), matched for
    /// a deletion and for an insertion, whose annealing portion is wild-type
    /// template (D031, D032). It is always the annealing portion's Tm, never
    /// the whole primer's.
    pub fn forward_tm(&self) -> MeltingTemperature {
        self.forward_tm.clone()
    }

    /// The reverse primer's Tm (matched basis), the annealing portion's.
    pub fn reverse_tm(&self) -> MeltingTemperature {
        self.reverse_tm.clone()
    }

    /// D032: the forward primer's whole-primer Tm, tail included, on the
    /// matched basis. It is reported because it is what a user's own calculator
    /// shows for the text they were given; it is never used for sizing, for
    /// balance or for the annealing temperature, which stay on the annealing
    /// portions. `None` for a substitution and a deletion, whose primers carry
    /// no tail and whose whole text is therefore the annealing portion already
    /// reported by `forward_tm`.
    pub fn forward_whole_tm(&self) -> Option<MeltingTemperature> {
        self.forward_whole_tm.clone()
    }

    /// D032: the reverse primer's whole-primer Tm, tail included, as
    /// `forward_whole_tm` reports it, and `None` for the same reason.
    pub fn reverse_whole_tm(&self) -> Option<MeltingTemperature> {
        self.reverse_whole_tm.clone()
    }

    /// The recommended annealing temperature, °C (D030).
    pub fn annealing_temperature(&self) -> f64 {
        self.annealing_temperature
    }

    /// Whether the two Tms are within `TM_BALANCE`. False only when the
    /// template ran out before the colder primer could be grown enough.
    pub fn tm_balance_met(&self) -> bool {
        self.tm_balance_met
    }

    /// Sequence-only guideline warnings on the forward primer. A warning never
    /// stops a design: these are published rules of thumb, not biological
    /// requirements (owner, 2026-10-14;
    /// `tests/fixtures/primer-quality/README.md`).
    pub fn forward_warnings(&self) -> Vec<PrimerWarning> {
        self.forward_warnings.clone()
    }

    pub fn reverse_warnings(&self) -> Vec<PrimerWarning> {
        self.reverse_warnings.clone()
    }

    /// Every warning, labelled with the primer it is about, forward first.
    pub fn warnings(&self) -> Vec<(PrimerRole, PrimerWarning)> {
        let mut out: Vec<(PrimerRole, PrimerWarning)> = Vec::new();
        for warning in self.forward_warnings.iter() {
            out.push((PrimerRole::Forward, warning.clone()));
        }
        for warning in self.reverse_warnings.iter() {
            out.push((PrimerRole::Reverse, warning.clone()));
        }
        out
    }

    pub fn unresolved(&self) -> Vec<UnresolvedInput> {
        self.unresolved.clone()
    }

    pub fn algorithm_name(&self) -> &'static str {
        NATIVE_DESIGN_ALGORITHM_NAME
    }

    pub fn algorithm_version(&self) -> &'static str {
        NATIVE_DESIGN_ALGORITHM_VERSION
    }
}

/// Design a primer pair for a verified mutation result under the request's
/// policy. A failure emits no pair (D010).
pub fn design_primers(
    template: &Sequence,
    result: MutationResult,
    request: PrimerDesignRequest,
) -> Result<PrimerDesign, DomainError> {
    require_sequence(result.source(), template)?;

    // §2.1: a design consumes a verified result; it does not re-verify.
    if !result.state().is_verified() {
        let problem = InvalidPrimerDesignRequestProblem {
            field: "mutation result".to_string(),
            reason: format!(
                "{} is {}; the design layer takes a verified result",
                result.describe(),
                result.state().name()
            ),
        };
        return Err(DomainError::InvalidPrimerDesignRequest(problem));
    }

    if !request.placement().is_supported() {
        let problem = UnsupportedPrimerPlacementProblem {
            placement: request.placement().name(),
            reason: "D029's native scope is inline placement".to_string(),
        };
        return Err(DomainError::UnsupportedPrimerPlacement(problem));
    }

    let policy: String = request.policy().name();
    if template.topology() != SequenceTopology::Linear {
        return Err(out_of_scope(
            &policy,
            "the template is circular; D029's native scope is a linear template",
        ));
    }

    // D031: a deletion is designed by its own path, before its removed
    // coordinates are read as a replacement region.
    if result.is_deletion() {
        return design_deletion(template, result, request);
    }

    // D032: an insertion is designed by its own path, before its point
    // coordinates — which name no region at all — are read as one.
    if result.is_insertion() {
        return design_insertion(template, result, request);
    }

    // D033: a replacement is designed here, before its target coordinates are
    // read as a replacement region: D029's `MAX_REPLACEMENT_REGION` bounds the
    // substitution scope, not D033's, whose path is decided by `t = max(w, m)`.
    if result.is_replacement() {
        let (region_start, region_end, text): (u64, u64, String) = match &result {
            MutationResult::Replacement(record) => (
                record.region().start(),
                record.region().end(),
                record.replacement(),
            ),
            MutationResult::Nucleotide(_)
            | MutationResult::Residue(_)
            | MutationResult::Deletion(_)
            | MutationResult::Insertion(_) => {
                return Err(out_of_scope(
                    &policy,
                    "the result is not a replacement: design_primers read it as one",
                ))
            }
        };
        let width: u64 = region_end - region_start;
        let written: u64 = text.chars().count() as u64;
        if width.max(written) <= MAX_INLINE_REPLACEMENT {
            return design_inline(template, result, request, region_start, region_end, written);
        }
        return design_tailed(
            template,
            result,
            request,
            region_start,
            region_end,
            text,
            true,
        );
    }

    let (start, end) = replacement_region(&policy, result.target_coordinates())?;
    design_inline(template, result, request, start, end, end - start)
}

/// D030's inline pair, which D033 reuses for a replacement whose region width
/// `w` and replacement length `m` both stay at or below `MAX_INLINE_REPLACEMENT`:
/// the region sits in place of the substituted bases, so the forward primer
/// reads the edited artifact and its text is `m − w` longer or shorter than the
/// template span `[s − 10, e + v)` it names. Everything else — the junction at
/// `start − MIN_FLANK`, the availability and ambiguity checks, the spans and
/// the reported edit — stays in template coordinates, as in D030.
fn design_inline(
    template: &Sequence,
    result: MutationResult,
    request: PrimerDesignRequest,
    start: u64,
    end: u64,
    written: u64,
) -> Result<PrimerDesign, DomainError> {
    let length: u64 = template.length();
    let region: u64 = end - start;
    let bound: u64 = request.min_annealing_overlap();
    let first_flank: u64 = bound.saturating_sub(MIN_FLANK).max(MIN_FLANK);
    let first_reverse: u64 = bound.max(MIN_REVERSE);

    // Saturating: a bound no template can hold saturates the sum to `u64::MAX`
    // rather than wrapping it to a value the check below would accept.
    let smallest_pair: u64 = region
        .saturating_add(MIN_FLANK)
        .saturating_add(first_flank)
        .saturating_add(first_reverse);
    if smallest_pair > length {
        let problem = NoPrimerDesignPossibleProblem {
            template: template.display_name(),
            template_length: length,
            smallest_pair,
        };
        return Err(DomainError::NoPrimerDesignPossible(problem));
    }

    let fits = |obstacles: Vec<DesignObstacle>| -> DomainError {
        DomainError::NoPrimerDesignFits(NoPrimerDesignFitsProblem {
            template: template.display_name(),
            edit_start: start,
            edit_end: end,
            obstacles,
        })
    };

    let mut obstacles: Vec<DesignObstacle> = Vec::new();
    if start < MIN_FLANK + first_reverse {
        obstacles.push(DesignObstacle::FivePrimeAvailability {
            available: start,
            needed: MIN_FLANK + first_reverse,
        });
    }
    if length - end < first_flank {
        obstacles.push(DesignObstacle::ThreePrimeAvailability {
            available: length - end,
            needed: first_flank,
        });
    }
    if !obstacles.is_empty() {
        return Err(fits(obstacles));
    }

    let boundary: u64 = start - MIN_FLANK;
    let wild_type: Vec<char> = template.letters();

    // H6: no degenerate symbol in a primer. Checked over the starting pair;
    // beyond it an ambiguity code only ends availability on that side.
    for position in boundary - first_reverse..end + first_flank {
        let base: char = wild_type[position as usize];
        if !is_dna_base(base) {
            obstacles.push(DesignObstacle::AmbiguousTemplateBase {
                position,
                base: base.to_string(),
            });
        }
    }
    if !obstacles.is_empty() {
        return Err(fits(obstacles));
    }

    let mut three_prime_available: u64 = 0;
    while end + three_prime_available < length
        && is_dna_base(wild_type[(end + three_prime_available) as usize])
    {
        three_prime_available += 1;
    }
    let mut five_prime_available: u64 = 0;
    while five_prime_available < boundary
        && is_dna_base(wild_type[(boundary - five_prime_available - 1) as usize])
    {
        five_prime_available += 1;
    }

    let sizer: Sizer = Sizer {
        wild_type: wild_type.clone(),
        forward_letters: result.edited().letters(),
        forward_start: boundary,
        // D033: the forward primer's 3′ end is `written` bases past the region's
        // start on the edited artifact, not `end` on the template; the two
        // coincide when the replacement is as long as the region.
        forward_anchor: start + written,
        reverse_end: boundary,
        // D033: the mismatch-aware basis needs a primer and a site of the same
        // length, position by position, which only exists when the replacement
        // is as long as the region; otherwise the forward Tm is matched.
        forward_mismatched: written == end - start,
        buffer: request.buffer(),
    };

    // D030's sizing, shared with the deletion path (D031): forward floor and
    // G/C end, then reverse, then balance.
    let (flank, reverse): (u64, u64) = size_pair(
        &sizer,
        first_flank,
        first_reverse,
        three_prime_available,
        five_prime_available,
        &fits,
    )?;

    let forward_tm: MeltingTemperature = sizer.forward_tm(flank)?;
    let reverse_tm: MeltingTemperature = sizer.reverse_tm(reverse)?;
    let tm_balance_met: bool = (forward_tm.celsius() - reverse_tm.celsius()).abs() <= TM_BALANCE;
    let annealing_temperature: f64 =
        (forward_tm.celsius().min(reverse_tm.celsius()) + TA_OFFSET).min(TA_CAP);

    let forward_span: NucleotideInterval = span(template, boundary, end + flank)?;
    let reverse_span: NucleotideInterval = span(template, boundary - reverse, boundary)?;
    let forward_primer: Primer = construct_primer(
        template,
        result.clone(),
        PrimerRole::Forward,
        request.placement(),
        canonical_text(sizer.forward(flank)),
        forward_span,
    )?;
    let reverse_primer: Primer = construct_primer(
        template,
        result.clone(),
        PrimerRole::Reverse,
        request.placement(),
        canonical_text(sizer.reverse(reverse)?),
        reverse_span,
    )?;
    let pair: PrimerPair = assemble_pair(forward_primer, reverse_primer, result.clone())?;
    require_design_constraints(pair.clone(), bound)?;

    Ok(PrimerDesign {
        request,
        built_against: template.reference(),
        edited: result.reference(),
        edit_start: start,
        edit_end: end,
        five_prime_flank: MIN_FLANK,
        three_prime_flank: flank,
        reverse_length: reverse,
        pair,
        forward_tm,
        reverse_tm,
        forward_whole_tm: None,
        reverse_whole_tm: None,
        annealing_temperature,
        tm_balance_met,
        forward_warnings: inspect(&sizer.forward(flank)),
        reverse_warnings: inspect(&sizer.reverse(reverse)?),
        unresolved: native_unresolved(),
    })
}

/// D031: the pair a deletion needs. Both primers are pure template, so both
/// Tms are on the matched basis, and the sizing is D030's with the whole
/// forward length in place of the forward 3′ flank: the forward primer starts
/// at `e`, the reverse at `s`, and neither has a 5′ flank.
fn design_deletion(
    template: &Sequence,
    result: MutationResult,
    request: PrimerDesignRequest,
) -> Result<PrimerDesign, DomainError> {
    let policy: String = request.policy().name();
    let coordinates: Vec<u64> = result.target_coordinates();
    if coordinates.is_empty() {
        return Err(out_of_scope(
            &policy,
            "the deletion claims no removed position",
        ));
    }
    let start: u64 = coordinates[0];
    let end: u64 = coordinates[coordinates.len() - 1] + 1;
    let length: u64 = template.length();
    let bound: u64 = request.min_annealing_overlap();
    let first: u64 = bound.max(MIN_REVERSE);
    let width: u64 = end - start;

    // Saturating: a bound no template can hold saturates the sum to `u64::MAX`
    // rather than wrapping it to a value the check below would accept.
    let smallest_pair: u64 = width.saturating_add(first.saturating_mul(2));
    if smallest_pair > length {
        let problem = NoPrimerDesignPossibleProblem {
            template: template.display_name(),
            template_length: length,
            smallest_pair,
        };
        return Err(DomainError::NoPrimerDesignPossible(problem));
    }

    let fits = |obstacles: Vec<DesignObstacle>| -> DomainError {
        DomainError::NoPrimerDesignFits(NoPrimerDesignFitsProblem {
            template: template.display_name(),
            edit_start: start,
            edit_end: end,
            obstacles,
        })
    };

    let mut obstacles: Vec<DesignObstacle> = Vec::new();
    if start < first {
        obstacles.push(DesignObstacle::FivePrimeAvailability {
            available: start,
            needed: first,
        });
    }
    if length - end < first {
        obstacles.push(DesignObstacle::ThreePrimeAvailability {
            available: length - end,
            needed: first,
        });
    }
    if !obstacles.is_empty() {
        return Err(fits(obstacles));
    }

    let wild_type: Vec<char> = template.letters();

    // H6: no degenerate symbol under the starting pair, the reverse primer's
    // span first and then the forward's, each ascending.
    for position in start - first..start {
        let base: char = wild_type[position as usize];
        if !is_dna_base(base) {
            obstacles.push(DesignObstacle::AmbiguousTemplateBase {
                position,
                base: base.to_string(),
            });
        }
    }
    for position in end..end + first {
        let base: char = wild_type[position as usize];
        if !is_dna_base(base) {
            obstacles.push(DesignObstacle::AmbiguousTemplateBase {
                position,
                base: base.to_string(),
            });
        }
    }
    if !obstacles.is_empty() {
        return Err(fits(obstacles));
    }

    // Availability: the ACGT bases each side can still take, counted from the
    // edit outward and stopped by an ambiguity code or the template's end.
    let mut forward_available: u64 = 0;
    while end + forward_available < length
        && is_dna_base(wild_type[(end + forward_available) as usize])
    {
        forward_available += 1;
    }
    let mut reverse_available: u64 = 0;
    while reverse_available < start
        && is_dna_base(wild_type[(start - reverse_available - 1) as usize])
    {
        reverse_available += 1;
    }

    let sizer: Sizer = Sizer {
        wild_type: wild_type.clone(),
        forward_letters: wild_type,
        forward_start: end,
        forward_anchor: end,
        reverse_end: start,
        forward_mismatched: false,
        buffer: request.buffer(),
    };

    let (forward, reverse): (u64, u64) = size_pair(
        &sizer,
        first,
        first,
        forward_available,
        reverse_available,
        &fits,
    )?;

    let forward_tm: MeltingTemperature = sizer.forward_tm(forward)?;
    let reverse_tm: MeltingTemperature = sizer.reverse_tm(reverse)?;
    let tm_balance_met: bool = (forward_tm.celsius() - reverse_tm.celsius()).abs() <= TM_BALANCE;
    let annealing_temperature: f64 =
        (forward_tm.celsius().min(reverse_tm.celsius()) + TA_OFFSET).min(TA_CAP);

    let forward_span: NucleotideInterval = span(template, end, end + forward)?;
    let reverse_span: NucleotideInterval = span(template, start - reverse, start)?;
    let forward_primer: Primer = construct_primer(
        template,
        result.clone(),
        PrimerRole::Forward,
        request.placement(),
        canonical_text(sizer.forward(forward)),
        forward_span,
    )?;
    let reverse_primer: Primer = construct_primer(
        template,
        result.clone(),
        PrimerRole::Reverse,
        request.placement(),
        canonical_text(sizer.reverse(reverse)?),
        reverse_span,
    )?;
    let pair: PrimerPair = assemble_pair(forward_primer, reverse_primer, result.clone())?;
    require_design_constraints(pair.clone(), bound)?;

    Ok(PrimerDesign {
        request,
        built_against: template.reference(),
        edited: result.reference(),
        edit_start: start,
        edit_end: end,
        five_prime_flank: 0,
        three_prime_flank: forward,
        reverse_length: reverse,
        pair,
        forward_tm,
        reverse_tm,
        forward_whole_tm: None,
        reverse_whole_tm: None,
        annealing_temperature,
        tm_balance_met,
        forward_warnings: inspect(&sizer.forward(forward)),
        reverse_warnings: inspect(&sizer.reverse(reverse)?),
        unresolved: native_unresolved(),
    })
}

/// D032: the pair an insertion needs. An insertion writes no template position,
/// so its point names no region: both edges of the region it hands the tails
/// path are the point, and the pair's footprint stays 0. The inserted bases go
/// into the two 5′ tails, whole on the forward primer at or below `MAX_WHOLE_TAIL`
/// and split above it.
fn design_insertion(
    template: &Sequence,
    result: MutationResult,
    request: PrimerDesignRequest,
) -> Result<PrimerDesign, DomainError> {
    let policy: String = request.policy().name();

    // D032: the point and the inserted text, read from the result itself.
    // Its `target_coordinates()` are empty, which is what makes an insertion
    // unreadable as a replacement region.
    let (point, inserted): (u64, String) = match &result {
        MutationResult::Insertion(record) => (record.point().position(), record.inserted()),
        MutationResult::Nucleotide(_)
        | MutationResult::Residue(_)
        | MutationResult::Deletion(_)
        | MutationResult::Replacement(_) => {
            return Err(out_of_scope(
                &policy,
                "the result is not an insertion: design_insertion is the insertion path",
            ))
        }
    };

    // An insertion writes no template position, so both edges of its region are
    // the point and it adds nothing to the pair's footprint; D033's replacement
    // reaches the same tails geometry with `s` and `e` apart.
    let width: u64 = inserted.chars().count() as u64;
    design_tailed(
        template,
        result,
        request,
        point,
        point,
        inserted,
        width > MAX_WHOLE_TAIL,
    )
}

/// D032's tails path, which D033 reuses for a replacement too wide to sit inline:
/// the insertion's point splits into the two edges `[s, e)` of the edited region,
/// so the forward primer anneals to `[e, e + f)` and the reverse to the reverse
/// complement of `[s − r, s)`. The written bases go into the two 5′ tails rather
/// than into either annealing portion: whole on the forward primer when `split`
/// is false, otherwise `text[m/2..]` forward and the reverse complement of
/// `text[..m/2]` in reverse, so a single written base leaves the reverse primer
/// untailed. Sizing runs on the annealing portions alone, both Tms matched, and
/// the whole primer's Tm is reported but never used. With `start == end == point`
/// this is D032's insertion, whose region is empty.
fn design_tailed(
    template: &Sequence,
    result: MutationResult,
    request: PrimerDesignRequest,
    start: u64,
    end: u64,
    text: String,
    split: bool,
) -> Result<PrimerDesign, DomainError> {
    let length: u64 = template.length();
    let bound: u64 = request.min_annealing_overlap();
    let first: u64 = bound.max(MIN_REVERSE);
    let text_letters: Vec<char> = text.chars().collect();
    let width: u64 = text_letters.len() as u64;

    // The written bases stand in place of the region `[start, end)`, which is
    // empty for an insertion, so the smallest pair is the region plus its two
    // annealing portions.
    // Saturating: a bound no template can hold saturates the sum to `u64::MAX`
    // rather than wrapping it to a value the check below would accept.
    let smallest_pair: u64 = (end - start).saturating_add(first.saturating_mul(2));
    if smallest_pair > length {
        let problem = NoPrimerDesignPossibleProblem {
            template: template.display_name(),
            template_length: length,
            smallest_pair,
        };
        return Err(DomainError::NoPrimerDesignPossible(problem));
    }

    let fits = |obstacles: Vec<DesignObstacle>| -> DomainError {
        DomainError::NoPrimerDesignFits(NoPrimerDesignFitsProblem {
            template: template.display_name(),
            edit_start: start,
            edit_end: end,
            obstacles,
        })
    };

    // Availability, counted outward from the two edges: the reverse primer needs
    // `first` bases before `s` and the forward `first` after `e`.
    let mut obstacles: Vec<DesignObstacle> = Vec::new();
    if start < first {
        obstacles.push(DesignObstacle::FivePrimeAvailability {
            available: start,
            needed: first,
        });
    }
    if length - end < first {
        obstacles.push(DesignObstacle::ThreePrimeAvailability {
            available: length - end,
            needed: first,
        });
    }
    if !obstacles.is_empty() {
        return Err(fits(obstacles));
    }

    let wild_type: Vec<char> = template.letters();

    // H6: no degenerate symbol under the starting pair, the reverse primer's
    // annealing portion first and then the forward's, each ascending.
    for position in start - first..start {
        let base: char = wild_type[position as usize];
        if !is_dna_base(base) {
            obstacles.push(DesignObstacle::AmbiguousTemplateBase {
                position,
                base: base.to_string(),
            });
        }
    }
    for position in end..end + first {
        let base: char = wild_type[position as usize];
        if !is_dna_base(base) {
            obstacles.push(DesignObstacle::AmbiguousTemplateBase {
                position,
                base: base.to_string(),
            });
        }
    }
    if !obstacles.is_empty() {
        return Err(fits(obstacles));
    }

    // Availability: the ACGT bases each side can still take, counted outward
    // from the two edges and stopped by an ambiguity code or the template's end.
    let mut forward_available: u64 = 0;
    while end + forward_available < length
        && is_dna_base(wild_type[(end + forward_available) as usize])
    {
        forward_available += 1;
    }
    let mut reverse_available: u64 = 0;
    while reverse_available < start
        && is_dna_base(wild_type[(start - reverse_available - 1) as usize])
    {
        reverse_available += 1;
    }

    // D030's sizing, shared with the deletion path (D031): both primers read
    // wild-type template, so both Tms are matched, and the junction sits at the
    // two edges of the region.
    let sizer: Sizer = Sizer {
        wild_type: wild_type.clone(),
        forward_letters: wild_type,
        forward_start: end,
        forward_anchor: end,
        reverse_end: start,
        forward_mismatched: false,
        buffer: request.buffer(),
    };

    let (forward, reverse): (u64, u64) = size_pair(
        &sizer,
        first,
        first,
        forward_available,
        reverse_available,
        &fits,
    )?;

    let forward_tm: MeltingTemperature = sizer.forward_tm(forward)?;
    let reverse_tm: MeltingTemperature = sizer.reverse_tm(reverse)?;
    let tm_balance_met: bool = (forward_tm.celsius() - reverse_tm.celsius()).abs() <= TM_BALANCE;
    let annealing_temperature: f64 =
        (forward_tm.celsius().min(reverse_tm.celsius()) + TA_OFFSET).min(TA_CAP);

    // D032/D033's tails: at or below `MAX_WHOLE_TAIL` the whole written text is
    // the forward primer's tail and the reverse primer gets none; above it — and
    // always for D033's replacement — the text is split, the forward tail taking
    // the last `m − m/2` bases and the reverse tail the reverse complement of the
    // first `m/2`, so a single written base leaves the reverse primer untailed.
    let half: u64 = width / 2;
    let forward_tail: Vec<char> = if !split {
        text_letters.clone()
    } else {
        text_letters[half as usize..].to_vec()
    };
    let reverse_tail: Vec<char> = if !split {
        Vec::new()
    } else {
        reverse_complement(text_letters[..half as usize].to_vec())?
    };

    let forward_span: NucleotideInterval = span(template, end, end + forward)?;
    let reverse_span: NucleotideInterval = span(template, start - reverse, start)?;

    // The written text has at least one base, so the forward tail never is empty.
    let forward_primer: Primer = construct_tailed_primer(
        template,
        result.clone(),
        PrimerRole::Forward,
        canonical_text(forward_tail),
        canonical_text(sizer.forward(forward)),
        forward_span,
    )?;

    // A reverse primer with no tail is an untailed primer of the edit, which
    // `construct_tailed_primer` refuses rather than defaulting: an insertion of
    // at most `MAX_WHOLE_TAIL` bases, and D033's replacement of a single base.
    let reverse_text: String = canonical_text(sizer.reverse(reverse)?);
    let reverse_primer: Primer = if reverse_tail.is_empty() {
        construct_primer(
            template,
            result.clone(),
            PrimerRole::Reverse,
            request.placement(),
            reverse_text,
            reverse_span,
        )
    } else {
        construct_tailed_primer(
            template,
            result.clone(),
            PrimerRole::Reverse,
            canonical_text(reverse_tail),
            reverse_text,
            reverse_span,
        )
    }?;

    let pair: PrimerPair = assemble_pair(forward_primer, reverse_primer, result.clone())?;
    require_design_constraints(pair.clone(), bound)?;

    // D032: the whole primer's Tm, tail included. Reported because it is what
    // a user's own calculator shows for the text they were given; it is never
    // used, because sizing, balance and the annealing temperature stay on the
    // annealing portions alone.
    let forward_whole: Vec<char> = pair.clone().forward().sequence().chars().collect();
    let reverse_whole: Vec<char> = pair.clone().reverse().sequence().chars().collect();
    let forward_whole_tm: MeltingTemperature =
        melting_temperature(&forward_whole, &forward_whole, &request.buffer())?;
    let reverse_whole_tm: MeltingTemperature =
        melting_temperature(&reverse_whole, &reverse_whole, &request.buffer())?;

    Ok(PrimerDesign {
        request,
        built_against: template.reference(),
        edited: result.reference(),
        edit_start: start,
        edit_end: end,
        five_prime_flank: 0,
        three_prime_flank: forward,
        reverse_length: reverse,
        pair,
        forward_tm,
        reverse_tm,
        forward_whole_tm: Some(forward_whole_tm),
        reverse_whole_tm: Some(reverse_whole_tm),
        annealing_temperature,
        tm_balance_met,
        // The guidelines are about the oligo as ordered, so they are inspected
        // on the whole primer, tail included.
        forward_warnings: inspect(&forward_whole),
        reverse_warnings: inspect(&reverse_whole),
        unresolved: native_unresolved(),
    })
}

/// D030's sizing, unchanged: the forward primer reaches the floor, then grows
/// toward a G/C last base below the ceiling; the reverse primer does the same;
/// then the colder one grows toward a 5 °C balance. `forward_available` and
/// `reverse_available` are the ACGT bases each side can still take. The pair
/// returned is (forward length argument, reverse length): a 3′ flank for a
/// substitution, the whole forward primer for a deletion.
fn size_pair(
    sizer: &Sizer,
    first_forward: u64,
    first_reverse: u64,
    forward_available: u64,
    reverse_available: u64,
    fits: &dyn Fn(Vec<DesignObstacle>) -> DomainError,
) -> Result<(u64, u64), DomainError> {
    // Forward: the floor, then a G/C last base below the ceiling.
    let mut forward: u64 = first_forward;
    loop {
        let tm: f64 = sizer.forward_tm(forward)?.celsius();
        if tm >= TM_FLOOR {
            break;
        }
        if forward >= forward_available {
            return Err(fits(vec![DesignObstacle::ThreePrimeTmFloorUnreachable {
                available: forward_available,
                reached_celsius: tm,
            }]));
        }
        forward += 1;
    }
    let floor_forward: u64 = forward;
    while sizer.forward_tm(forward)?.celsius() < GC_CEILING
        && forward < forward_available
        && !ends_gc(&sizer.forward(forward))
    {
        forward += 1;
    }
    if !ends_gc(&sizer.forward(forward)) {
        forward = floor_forward;
    }

    // Reverse: the same, growing away from the junction.
    let mut reverse: u64 = first_reverse;
    loop {
        let tm: f64 = sizer.reverse_tm(reverse)?.celsius();
        if tm >= TM_FLOOR {
            break;
        }
        if reverse >= reverse_available {
            return Err(fits(vec![DesignObstacle::FivePrimeTmFloorUnreachable {
                available: reverse_available,
                reached_celsius: tm,
            }]));
        }
        reverse += 1;
    }
    let floor_reverse: u64 = reverse;
    while sizer.reverse_tm(reverse)?.celsius() < GC_CEILING
        && reverse < reverse_available
        && !ends_gc(&sizer.reverse(reverse)?)
    {
        reverse += 1;
    }
    if !ends_gc(&sizer.reverse(reverse)?) {
        reverse = floor_reverse;
    }

    // Balance: grow the colder primer.
    let forward_celsius: f64 = sizer.forward_tm(forward)?.celsius();
    let reverse_celsius: f64 = sizer.reverse_tm(reverse)?.celsius();
    if forward_celsius - reverse_celsius > TM_BALANCE {
        while forward_celsius - sizer.reverse_tm(reverse)?.celsius() > TM_BALANCE
            && reverse < reverse_available
        {
            reverse += 1;
        }
        let balanced: u64 = reverse;
        let mut trial: u64 = reverse;
        if !ends_gc(&sizer.reverse(trial)?) {
            while trial < reverse_available
                && sizer.reverse_tm(trial)?.celsius() < forward_celsius
                && !ends_gc(&sizer.reverse(trial)?)
            {
                trial += 1;
            }
            reverse = if ends_gc(&sizer.reverse(trial)?)
                && sizer.reverse_tm(trial)?.celsius() <= forward_celsius
            {
                trial
            } else {
                balanced
            };
        }
    } else if reverse_celsius - forward_celsius > TM_BALANCE {
        while reverse_celsius - sizer.forward_tm(forward)?.celsius() > TM_BALANCE
            && forward < forward_available
        {
            forward += 1;
        }
        let balanced: u64 = forward;
        let mut trial: u64 = forward;
        if !ends_gc(&sizer.forward(trial)) {
            while trial < forward_available
                && sizer.forward_tm(trial)?.celsius() < reverse_celsius
                && !ends_gc(&sizer.forward(trial))
            {
                trial += 1;
            }
            forward = if ends_gc(&sizer.forward(trial))
                && sizer.forward_tm(trial)?.celsius() <= reverse_celsius
            {
                trial
            } else {
                balanced
            };
        }
    }

    Ok((forward, reverse))
}

/// The primer texts and Tms at a candidate length, around a fixed junction.
/// The forward letters are the edited artifact's for a substitution, which is
/// why its Tm is mismatch-aware, and the wild type's for a deletion, whose Tm
/// is matched. `forward(n)` counts from `forward_anchor`, so the length
/// argument is a 3′ flank for a substitution and the whole forward primer for
/// a deletion.
struct Sizer {
    wild_type: Vec<char>,
    forward_letters: Vec<char>,
    forward_start: u64,
    forward_anchor: u64,
    reverse_end: u64,
    forward_mismatched: bool,
    buffer: BufferModel,
}

impl Sizer {
    /// The forward primer at a candidate length.
    fn forward(&self, length: u64) -> Vec<char> {
        self.forward_letters[self.forward_start as usize..(self.forward_anchor + length) as usize]
            .to_vec()
    }

    /// The template the forward primer anneals over, same span.
    fn forward_site(&self, length: u64) -> Vec<char> {
        self.wild_type[self.forward_start as usize..(self.forward_anchor + length) as usize]
            .to_vec()
    }

    /// The reverse primer of `length` bases, 5′→3′.
    fn reverse(&self, length: u64) -> Result<Vec<char>, DomainError> {
        reverse_complement(
            self.wild_type[(self.reverse_end - length) as usize..self.reverse_end as usize]
                .to_vec(),
        )
    }

    fn forward_tm(&self, length: u64) -> Result<MeltingTemperature, DomainError> {
        let primer: Vec<char> = self.forward(length);
        if self.forward_mismatched {
            return melting_temperature_or_matched(
                &primer,
                &self.forward_site(length),
                &self.buffer,
            );
        }
        melting_temperature(&primer, &primer, &self.buffer)
    }

    fn reverse_tm(&self, length: u64) -> Result<MeltingTemperature, DomainError> {
        let primer: Vec<char> = self.reverse(length)?;
        melting_temperature(&primer, &primer, &self.buffer)
    }
}

fn ends_gc(primer: &[char]) -> bool {
    matches!(primer.last(), Some('G') | Some('C'))
}

/// The replacement region a result claims, which the native policy requires to
/// be one contiguous interval of at most `MAX_REPLACEMENT_REGION` bases.
fn replacement_region(policy: &str, coordinates: Vec<u64>) -> Result<(u64, u64), DomainError> {
    if coordinates.is_empty() {
        return Err(out_of_scope(
            policy,
            "the result claims no changed position",
        ));
    }
    let start: u64 = coordinates[0];
    for (offset, position) in coordinates.iter().enumerate() {
        if *position != start + offset as u64 {
            return Err(out_of_scope(
                policy,
                "the changed positions are not one contiguous interval",
            ));
        }
    }
    let region: u64 = coordinates.len() as u64;
    if region > MAX_REPLACEMENT_REGION {
        return Err(out_of_scope(
            policy,
            &format!(
                "the replacement region is {} bases; the native scope is at most {}",
                region, MAX_REPLACEMENT_REGION
            ),
        ));
    }
    Ok((start, start + region))
}

fn out_of_scope(policy: &str, reason: &str) -> DomainError {
    DomainError::UnsupportedPrimerDesignScope(UnsupportedPrimerDesignScopeProblem {
        policy: policy.to_string(),
        reason: reason.to_string(),
    })
}

fn span(template: &Sequence, start: u64, end: u64) -> Result<NucleotideInterval, DomainError> {
    create(&template.reference(), start as i64, end as i64)
}

/// §10: the contract inputs the native policy does not take, and their effect.
fn native_unresolved() -> Vec<UnresolvedInput> {
    vec![
        UnresolvedInput {
            input: "minTm".to_string(),
            effect: "not accepted: luriaNative applies its fixed 55 °C floor (D030)".to_string(),
        },
        UnresolvedInput {
            input: "maxAnnealingOverlap".to_string(),
            effect: "not accepted (owner, 2026-10-14): no upper bound was applied; the chosen lengths are reported"
                .to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cds::Cds;
    use crate::cds::Completeness;
    use crate::cds::Numbering;
    use crate::cds::ResiduePosition;
    use crate::error::ErrorCategory;
    use crate::identity::Identifier;
    use crate::interval::create_point;
    use crate::mutation::apply_deletion;
    use crate::mutation::apply_insertion;
    use crate::mutation::apply_nucleotide;
    use crate::mutation::apply_replacement;
    use crate::mutation::apply_residue;
    use crate::mutation::verify;
    use crate::mutation::Codon;
    use crate::mutation::CodonSelectionPolicy;
    use crate::mutation::MutationState;
    use crate::mutation::NucleotideDeletionIntent;
    use crate::mutation::NucleotideInsertionIntent;
    use crate::mutation::NucleotideMutationIntent;
    use crate::mutation::NucleotideReplacementIntent;
    use crate::mutation::ResidueMutationIntent;
    use crate::primer::PrimerOrientation;
    use crate::primer_quality::PrimerWarning;
    use crate::sequence::normalize;
    use crate::strand::Strand;
    use crate::tm::TmBasis;

    // Every expected span, length, Tm and failure below was produced by
    // `tests/fixtures/q5-primer/reference/native-design.py`, which restates
    // D030 in plain Python and computes Tm with Biopython; it shares no code
    // with this crate. Tms are compared to the two decimals that script
    // prints.
    //
    //   0         1         2         3         4         5
    //   0123456789012345678901234567890123456789012345678901234567890
    //   ATGGCTTGGAATCAGCAGCAGTTTAAACCCGGGATCGATCGTACGTACGGCATGCATGCA
    const TEMPLATE: &str = "ATGGCTTGGAATCAGCAGCAGTTTAAACCCGGGATCGATCGTACGTACGGCATGCATGCA";

    // Synthetic templates built from a GC-rich, an AT-rich and a mixed 20-mer,
    // chosen so that between them every branch of D030 is exercised.
    const GC: &str = "GCCGAGCGGCTCGCGGACGC";
    const AT: &str = "ATTAAATATTTAAATTATAT";
    const MIX: &str = "ATGGCTTGGAATCAGCAGCA";

    fn balanced() -> String {
        [MIX, GC, MIX, GC, MIX].concat()
    }

    fn at_upstream() -> String {
        [AT, AT, MIX, GC, GC].concat()
    }

    fn at_downstream() -> String {
        [GC, GC, MIX, AT, AT].concat()
    }

    fn template_of(text: &str, topology: SequenceTopology) -> Sequence {
        normalize(
            Identifier::create("pd-a".to_string()).unwrap(),
            text.to_string(),
            topology,
        )
        .unwrap()
    }

    fn template() -> Sequence {
        template_of(TEMPLATE, SequenceTopology::Linear)
    }

    fn linear(text: &str) -> Sequence {
        template_of(text, SequenceTopology::Linear)
    }

    fn request(bound: i64) -> PrimerDesignRequest {
        PrimerDesignRequest::create(
            DesignStyle::Q5Polymerase,
            DesignPolicy::LuriaNative,
            bound,
            PrimerPlacement::Inline,
            BufferModel::q5_reaction(),
        )
        .unwrap()
    }

    fn calculated_nucleotide(
        sequence: &Sequence,
        position: i64,
        replacement: &str,
    ) -> MutationResult {
        let original: String = sequence.letters()[position as usize].to_string();
        let intent: NucleotideMutationIntent = NucleotideMutationIntent::create(
            sequence.reference(),
            create(&sequence.reference(), position, position + 1).unwrap(),
            original,
            replacement.to_string(),
        )
        .unwrap();
        apply_nucleotide(
            sequence,
            None,
            Identifier::create("pd-edit".to_string()).unwrap(),
            intent,
        )
        .unwrap()
    }

    fn nucleotide(sequence: &Sequence, position: i64, replacement: &str) -> MutationResult {
        let made: MutationResult = calculated_nucleotide(sequence, position, replacement);
        verify(sequence, None, made).unwrap()
    }

    // A residue edit on a one-codon CDS over [start, start + 3).
    fn codon_edit(
        sequence: &Sequence,
        start: i64,
        strand: Strand,
        expected: &str,
        replacement: &str,
        codon: &str,
    ) -> MutationResult {
        let id: Identifier = Identifier::create("pd-cds".to_string()).unwrap();
        let cds: Cds = Cds::create(
            sequence,
            id.clone(),
            vec![create(&sequence.reference(), start, start + 3).unwrap()],
            strand,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap();
        let intent: ResidueMutationIntent = ResidueMutationIntent::create(
            ResiduePosition::create(id, 1).unwrap(),
            expected.to_string(),
            replacement.to_string(),
            CodonSelectionPolicy::Supplied(Codon::create(codon.to_string()).unwrap()),
        )
        .unwrap();
        let made: MutationResult = apply_residue(
            sequence,
            cds.clone(),
            Identifier::create("pd-edit".to_string()).unwrap(),
            intent,
        )
        .unwrap();
        verify(sequence, Some(cds), made).unwrap()
    }

    fn spans(design: &PrimerDesign) -> ((u64, u64), (u64, u64)) {
        let pair: PrimerPair = design.pair();
        let forward: NucleotideInterval = pair.clone().forward().span();
        let reverse: NucleotideInterval = pair.reverse().span();
        (
            (forward.start(), forward.end()),
            (reverse.start(), reverse.end()),
        )
    }

    fn texts(design: &PrimerDesign) -> (String, String) {
        let pair: PrimerPair = design.pair();
        (pair.clone().forward().sequence(), pair.reverse().sequence())
    }

    /// Two decimals, as the generating script prints them.
    fn close(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 0.006
    }

    /// The Tm figures of a design: forward, reverse, annealing temperature.
    fn assert_tms(design: &PrimerDesign, forward: f64, reverse: f64, annealing: f64) {
        assert!(
            close(design.forward_tm().celsius(), forward),
            "forward Tm {:.3}, expected {:.2}",
            design.forward_tm().celsius(),
            forward
        );
        assert!(
            close(design.reverse_tm().celsius(), reverse),
            "reverse Tm {:.3}, expected {:.2}",
            design.reverse_tm().celsius(),
            reverse
        );
        assert!(
            close(design.annealing_temperature(), annealing),
            "annealing temperature {:.3}, expected {:.2}",
            design.annealing_temperature(),
            annealing
        );
    }

    fn obstacles_of(error: DomainError) -> Vec<DesignObstacle> {
        match error {
            DomainError::NoPrimerDesignFits(problem) => problem.obstacles,
            other => panic!("expected NoPrimerDesignFits, got {:?}", other),
        }
    }

    fn assert_tm_floor_obstacle(
        error: DomainError,
        five_prime: bool,
        available: u64,
        reached: f64,
    ) {
        assert!(error.category() == ErrorCategory::Unsupported);
        let obstacles: Vec<DesignObstacle> = obstacles_of(error);
        assert!(obstacles.len() == 1, "{:?}", obstacles);
        let (side_ok, got_available, got_reached) = match &obstacles[0] {
            DesignObstacle::FivePrimeTmFloorUnreachable {
                available,
                reached_celsius,
            } => (five_prime, *available, *reached_celsius),
            DesignObstacle::ThreePrimeTmFloorUnreachable {
                available,
                reached_celsius,
            } => (!five_prime, *available, *reached_celsius),
            other => panic!("expected a Tm-floor obstacle, got {:?}", other),
        };
        assert!(side_ok, "wrong side: {:?}", obstacles[0]);
        assert!(got_available == available, "available {}", got_available);
        assert!(close(got_reached, reached), "reached {:.3}", got_reached);
    }

    #[test]
    fn a_single_base_edit_gets_an_abutting_pair_sized_by_tm() {
        let design: PrimerDesign =
            design_primers(&template(), nucleotide(&template(), 30, "A"), request(15)).unwrap();
        assert!(spans(&design) == ((20, 41), (4, 20)));
        assert!(
            texts(&design)
                == (
                    "GTTTAAACCCAGGATCGATCG".to_string(),
                    "TGCTGCTGATTCCAAG".to_string()
                )
        );
        assert!(design.five_prime_flank() == 10);
        assert!(design.three_prime_flank() == 10);
        // The reverse primer met the floor at 15 and grew one base to a G end.
        assert!(design.reverse_length() == 16);
        assert_tms(&design, 56.96, 58.73, 59.96);
        assert!(design.forward_tm().basis() == TmBasis::MismatchedWildTypeSite);
        assert!(design.reverse_tm().basis() == TmBasis::MatchedComplement);
        assert!(design.tm_balance_met());
        assert!(design.pair().abuts());
        assert!(design.pair().reverse().orientation() == PrimerOrientation::Opposite);
    }

    // b = 21: the forward 3′ flank starts at 11 and the reverse at 21.
    #[test]
    fn a_bound_above_twenty_starts_both_primers_longer() {
        let design: PrimerDesign =
            design_primers(&template(), nucleotide(&template(), 35, "A"), request(21)).unwrap();
        assert!(design.five_prime_flank() == 10);
        assert!(design.three_prime_flank() == 11);
        assert!(spans(&design) == ((25, 47), (4, 25)));
        assert!(
            texts(&design)
                == (
                    "AACCCGGGATAGATCGTACGTA".to_string(),
                    "TAAACTGCTGCTGATTCCAAG".to_string()
                )
        );
        assert_tms(&design, 63.96, 62.53, 65.53);
        assert!(design.pair().forward().annealing_overlap() >= 21);
        assert!(design.pair().reverse().annealing_overlap() >= 21);
    }

    // b = 6: the starting lengths are raised to the minimums, then sized by Tm
    // exactly as at b = 15.
    #[test]
    fn a_small_bound_is_raised_to_the_minimums() {
        let small: PrimerDesign =
            design_primers(&template(), nucleotide(&template(), 30, "A"), request(6)).unwrap();
        let usual: PrimerDesign =
            design_primers(&template(), nucleotide(&template(), 30, "A"), request(15)).unwrap();
        assert!(spans(&small) == spans(&usual));
        assert!(small.request().min_annealing_overlap() == 6);
    }

    // GGG (G) -> TGG (W) at [30, 33): one changed base, so the mismatch-aware
    // basis applies. The forward primer grew one base to a C end.
    #[test]
    fn a_codon_edit_is_designed_with_the_codon_inline() {
        let edit: MutationResult = codon_edit(&template(), 30, Strand::Plus, "G", "W", "TGG");
        let design: PrimerDesign = design_primers(&template(), edit, request(15)).unwrap();
        assert!(design.edit_span() == (30, 33));
        assert!(spans(&design) == ((20, 44), (4, 20)));
        assert!(texts(&design).0 == "GTTTAAACCCTGGATCGATCGTAC");
        assert!(design.three_prime_flank() == 11);
        assert!(design.pair().forward().changed_positions() == vec![10, 11, 12]);
        assert!(design.forward_tm().basis() == TmBasis::MismatchedWildTypeSite);
        assert_tms(&design, 60.26, 58.73, 61.73);
    }

    // On the minus strand [30, 33) reads rc(GGG) = CCC (P); TGG (W) is written
    // back as rc(TGG) = CCA. All three bases change, two adjacent mismatches,
    // so the forward Tm falls back to the matched basis and says why.
    #[test]
    fn adjacent_changes_fall_back_to_the_matched_forward_tm() {
        let edit: MutationResult = codon_edit(&template(), 30, Strand::Minus, "P", "W", "TGG");
        let design: PrimerDesign = design_primers(&template(), edit, request(15)).unwrap();
        assert!(spans(&design) == ((20, 43), (3, 20)));
        assert!(
            texts(&design)
                == (
                    "GTTTAAACCCCCAATCGATCGTA".to_string(),
                    "TGCTGCTGATTCCAAGC".to_string()
                )
        );
        assert!(design.forward_tm().basis() == TmBasis::MatchedComplement);
        let reason: String = design.forward_tm().substituted_because().unwrap();
        assert!(reason.contains("tandem mismatch"), "{}", reason);
        assert_tms(&design, 64.51, 62.79, 65.79);
    }

    // At 24 the starting pair does not fit 5′ of the edit; at 25 it fits, but
    // 15 bases upstream cannot make a reverse primer reach 55 °C.
    #[test]
    fn the_five_prime_side_fails_on_availability_then_on_the_tm_floor() {
        let error: DomainError =
            design_primers(&template(), nucleotide(&template(), 24, "G"), request(15)).unwrap_err();
        assert!(error.category() == ErrorCategory::Unsupported);
        assert!(
            obstacles_of(error)
                == vec![DesignObstacle::FivePrimeAvailability {
                    available: 24,
                    needed: 25
                }]
        );

        let error: DomainError =
            design_primers(&template(), nucleotide(&template(), 25, "G"), request(15)).unwrap_err();
        assert_tm_floor_obstacle(error, true, 15, 54.21);
    }

    #[test]
    fn the_three_prime_limit_is_exact() {
        let fits: PrimerDesign =
            design_primers(&template(), nucleotide(&template(), 49, "A"), request(15)).unwrap();
        assert!(spans(&fits) == ((39, 60), (20, 39)));
        assert!(
            texts(&fits)
                == (
                    "CGTACGTACGACATGCATGCA".to_string(),
                    "ATCGATCCCGGGTTTAAAC".to_string()
                )
        );
        assert_tms(&fits, 63.33, 61.85, 64.85);

        let error: DomainError =
            design_primers(&template(), nucleotide(&template(), 50, "A"), request(15)).unwrap_err();
        assert!(
            obstacles_of(error)
                == vec![DesignObstacle::ThreePrimeAvailability {
                    available: 9,
                    needed: 10
                }]
        );
    }

    // 1 + 10 + 10 + 15 = 36 bases is the smallest pair; 35 cannot hold it.
    #[test]
    fn a_template_shorter_than_the_smallest_pair_has_no_design_at_all() {
        let short: Sequence = linear(&TEMPLATE[..35]);
        let error: DomainError =
            design_primers(&short, nucleotide(&short, 25, "G"), request(15)).unwrap_err();
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        assert!(
            error
                == DomainError::NoPrimerDesignPossible(NoPrimerDesignPossibleProblem {
                    template: "pd-a".to_string(),
                    template_length: 35,
                    smallest_pair: 36,
                })
        );
    }

    // B2: a bound so large that the smallest-pair sum overflows `u64` must
    // saturate to `u64::MAX`, which is above every template length, so the
    // failure stays `NoPrimerDesignPossible` (BiologicalValidation) rather
    // than wrapping to a small sum that passes the check and reports
    // `NoPrimerDesignFits` (Unsupported). `balanced()` is 100 bases.
    //
    // With `bound = i64::MAX` the inline sum is `2 * bound + region`, so a
    // 3-base codon substitution overflows (`2 * i64::MAX` is `u64::MAX - 1`)
    // and saturates to `u64::MAX`.
    #[test]
    fn an_overflowing_bound_on_a_substitution_is_no_design_possible() {
        let sequence: Sequence = linear(&balanced());
        let error: DomainError = design_primers(
            &sequence,
            codon_edit(&sequence, 37, Strand::Plus, "R", "K", "AAA"),
            request(i64::MAX),
        )
        .unwrap_err();
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        assert!(
            error
                == DomainError::NoPrimerDesignPossible(NoPrimerDesignPossibleProblem {
                    template: "pd-a".to_string(),
                    template_length: 100,
                    smallest_pair: u64::MAX,
                })
        );
    }

    #[test]
    fn an_overflowing_bound_on_a_deletion_is_no_design_possible() {
        let sequence: Sequence = linear(&balanced());
        let error: DomainError =
            design_primers(&sequence, deletion(&sequence, 50, 53), request(i64::MAX)).unwrap_err();
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        assert!(
            error
                == DomainError::NoPrimerDesignPossible(NoPrimerDesignPossibleProblem {
                    template: "pd-a".to_string(),
                    template_length: 100,
                    smallest_pair: u64::MAX,
                })
        );
    }

    // An insertion writes no template position, so its region is empty and the
    // sum is `2 * bound` exactly: `2 * i64::MAX` is `u64::MAX - 1`, which fits
    // `u64`, so this bound does not overflow the tails sum and the reported
    // `smallest_pair` is `u64::MAX - 1`, not `u64::MAX`. The failure is still
    // `NoPrimerDesignPossible` (BiologicalValidation), as the fix leaves any
    // sum that fits unchanged.
    #[test]
    fn an_overflowing_bound_on_an_insertion_is_no_design_possible() {
        let sequence: Sequence = linear(&balanced());
        let error: DomainError = design_primers(
            &sequence,
            insertion(&sequence, 50, "GAT"),
            request(i64::MAX),
        )
        .unwrap_err();
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        assert!(
            error
                == DomainError::NoPrimerDesignPossible(NoPrimerDesignPossibleProblem {
                    template: "pd-a".to_string(),
                    template_length: 100,
                    smallest_pair: u64::MAX - 1,
                })
        );
    }

    // A replacement of a 7-base region by one base has t = max(7, 1) = 7,
    // above `MAX_INLINE_REPLACEMENT`, so it takes the tails path.
    #[test]
    fn an_overflowing_bound_on_a_tailed_replacement_is_no_design_possible() {
        let sequence: Sequence = linear(&balanced());
        let error: DomainError = design_primers(
            &sequence,
            replacement(&sequence, 50, 57, "T"),
            request(i64::MAX),
        )
        .unwrap_err();
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        assert!(
            error
                == DomainError::NoPrimerDesignPossible(NoPrimerDesignPossibleProblem {
                    template: "pd-a".to_string(),
                    template_length: 100,
                    smallest_pair: u64::MAX,
                })
        );
    }

    #[test]
    fn an_ambiguity_code_under_the_starting_pair_blocks_the_design() {
        let mut text: String = TEMPLATE.to_string();
        text.replace_range(10..11, "N");
        let sequence: Sequence = linear(&text);
        let error: DomainError =
            design_primers(&sequence, nucleotide(&sequence, 30, "A"), request(15)).unwrap_err();
        assert!(error.category() == ErrorCategory::Unsupported);
        assert!(
            obstacles_of(error)
                == vec![DesignObstacle::AmbiguousTemplateBase {
                    position: 10,
                    base: "N".to_string()
                }]
        );
    }

    // ---- D030 branches, on the synthetic templates ------------------------

    // CAG (Q) -> AGA (R) at [55, 58) with 55 AT-rich bases upstream. The
    // reverse primer grows to the floor, its G/C extension fails and reverts,
    // and balancing uses every upstream base without closing the gap.
    #[test]
    fn balance_can_be_left_unmet_when_the_template_runs_out() {
        let sequence: Sequence = linear(&at_upstream());
        let edit: MutationResult = codon_edit(&sequence, 55, Strand::Plus, "Q", "R", "AGA");
        let design: PrimerDesign = design_primers(&sequence, edit, request(15)).unwrap();
        assert!(spans(&design) == ((45, 68), (0, 45)));
        assert!(design.reverse_length() == 45);
        assert!(design.forward_tm().basis() == TmBasis::MatchedComplement);
        assert_tms(&design, 71.55, 55.68, 58.68);
        assert!(!design.tm_balance_met());
    }

    // T -> A at 61, AT-rich downstream. The forward primer grows a long way to
    // reach the floor, its G/C extension reverts, and then the reverse primer
    // is the hotter one, so balancing grows the forward primer.
    #[test]
    fn balance_can_grow_the_forward_primer() {
        let sequence: Sequence = linear(&at_downstream());
        let design: PrimerDesign =
            design_primers(&sequence, nucleotide(&sequence, 61, "A"), request(15)).unwrap();
        assert!(spans(&design) == ((51, 96), (35, 51)));
        assert!(design.three_prime_flank() == 34);
        assert!(design.reverse_length() == 16);
        assert!(design.forward_tm().basis() == TmBasis::MismatchedWildTypeSite);
        assert_tms(&design, 57.91, 62.65, 60.91);
        assert!(design.tm_balance_met());
    }

    // G -> A at 42 on the balanced template: balancing grows the forward
    // primer and then accepts a G/C end that stays below the reverse Tm.
    #[test]
    fn balance_growth_can_finish_on_a_gc_end() {
        let sequence: Sequence = linear(&balanced());
        let design: PrimerDesign =
            design_primers(&sequence, nucleotide(&sequence, 42, "A"), request(15)).unwrap();
        assert!(spans(&design) == ((32, 55), (17, 32)));
        assert!(design.three_prime_flank() == 12);
        assert!(texts(&design).0.ends_with('G'));
        assert_tms(&design, 67.60, 70.90, 70.60);
    }

    // CGC (R) -> AAA (K) at [37, 40): adjacent changes, a hot forward primer,
    // the reverse primer grown by balancing to a G/C end, and an annealing
    // temperature at the 72 °C cap.
    #[test]
    fn the_annealing_temperature_is_capped() {
        let sequence: Sequence = linear(&balanced());
        let edit: MutationResult = codon_edit(&sequence, 37, Strand::Plus, "R", "K", "AAA");
        let design: PrimerDesign = design_primers(&sequence, edit, request(15)).unwrap();
        assert!(spans(&design) == ((27, 50), (8, 27)));
        assert!(design.forward_tm().basis() == TmBasis::MatchedComplement);
        assert_tms(&design, 73.21, 69.59, 72.00);
    }

    // A -> G at 25 with only AT-rich sequence upstream: the reverse primer
    // uses all 15 bases and reaches 30.44 °C.
    #[test]
    fn a_reverse_primer_that_cannot_reach_the_floor_is_a_named_failure() {
        let sequence: Sequence = linear(&at_upstream());
        let error: DomainError =
            design_primers(&sequence, nucleotide(&sequence, 25, "G"), request(15)).unwrap_err();
        assert_tm_floor_obstacle(error, true, 15, 30.44);
    }

    // A -> G at 65 with only AT-rich sequence downstream: the forward primer
    // uses all 34 bases and reaches 54.58 °C.
    #[test]
    fn a_forward_primer_that_cannot_reach_the_floor_is_a_named_failure() {
        let sequence: Sequence = linear(&at_downstream());
        let error: DomainError =
            design_primers(&sequence, nucleotide(&sequence, 65, "G"), request(15)).unwrap_err();
        assert_tm_floor_obstacle(error, false, 34, 54.58);
    }

    #[test]
    fn every_design_meets_the_floor() {
        for (text, position, replacement) in [
            (TEMPLATE.to_string(), 30, "A"),
            (at_downstream(), 61, "A"),
            (balanced(), 42, "A"),
        ] {
            let sequence: Sequence = linear(&text);
            let design: PrimerDesign = design_primers(
                &sequence,
                nucleotide(&sequence, position, replacement),
                request(15),
            )
            .unwrap();
            assert!(design.forward_tm().celsius() >= TM_FLOOR);
            assert!(design.reverse_tm().celsius() >= TM_FLOOR);
        }
    }

    // ---- refusals ----------------------------------------------------------

    #[test]
    fn an_unverified_result_is_refused() {
        let made: MutationResult = calculated_nucleotide(&template(), 30, "A");
        assert!(made.state() == MutationState::Calculated);
        let error: DomainError = design_primers(&template(), made, request(15)).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerDesignRequest(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    #[test]
    fn a_bound_below_one_is_refused() {
        for bound in [0, -1] {
            let error: DomainError = PrimerDesignRequest::create(
                DesignStyle::Q5Polymerase,
                DesignPolicy::LuriaNative,
                bound,
                PrimerPlacement::Inline,
                BufferModel::q5_reaction(),
            )
            .unwrap_err();
            assert!(matches!(error, DomainError::InvalidPrimerDesignRequest(_)));
            assert!(error.category() == ErrorCategory::MalformedInput);
        }
    }

    #[test]
    fn a_circular_template_is_out_of_scope() {
        let circular: Sequence = template_of(TEMPLATE, SequenceTopology::Circular);
        let error: DomainError =
            design_primers(&circular, nucleotide(&circular, 30, "A"), request(15)).unwrap_err();
        assert!(matches!(
            error,
            DomainError::UnsupportedPrimerDesignScope(_)
        ));
        assert!(error.category() == ErrorCategory::Unsupported);
    }

    // A helper for D031: a verified deletion of [start, end), asserting the
    // removed bases read from the template itself.
    fn deletion(sequence: &Sequence, start: i64, end: i64) -> MutationResult {
        let removed: String = sequence.canonical_text()[start as usize..end as usize].to_string();
        let intent: NucleotideDeletionIntent = NucleotideDeletionIntent::create(
            sequence.reference(),
            create(&sequence.reference(), start, end).unwrap(),
            removed,
        )
        .unwrap();
        let made: MutationResult = apply_deletion(
            sequence,
            None,
            Identifier::create("pd-edit".to_string()).unwrap(),
            intent,
        )
        .unwrap();
        verify(sequence, None, made).unwrap()
    }

    // A helper for D032: a verified insertion at `position`, asserting the
    // flanking bases read from the template itself.
    fn insertion(sequence: &Sequence, position: i64, inserted: &str) -> MutationResult {
        let letters: Vec<char> = sequence.letters();
        let intent: NucleotideInsertionIntent = NucleotideInsertionIntent::create(
            sequence.reference(),
            create_point(&sequence.reference(), position).unwrap(),
            letters[position as usize - 1].to_string(),
            letters[position as usize].to_string(),
            inserted.to_string(),
        )
        .unwrap();
        let made: MutationResult = apply_insertion(
            sequence,
            None,
            Identifier::create("pd-edit".to_string()).unwrap(),
            intent,
        )
        .unwrap();
        verify(sequence, None, made).unwrap()
    }

    // A helper for D033: a verified replacement of [start, end) by `text`,
    // asserting the replaced bases read from the template itself.
    fn replacement(sequence: &Sequence, start: i64, end: i64, text: &str) -> MutationResult {
        let replaced: String = sequence.canonical_text()[start as usize..end as usize].to_string();
        let intent: NucleotideReplacementIntent = NucleotideReplacementIntent::create(
            sequence.reference(),
            create(&sequence.reference(), start, end).unwrap(),
            replaced,
            text.to_string(),
        )
        .unwrap();
        let made: MutationResult = apply_replacement(
            sequence,
            None,
            Identifier::create("pd-edit".to_string()).unwrap(),
            intent,
        )
        .unwrap();
        verify(sequence, None, made).unwrap()
    }

    // D031: a deletion of [50, 53) gets a pair whose 5′ ends sit at `e` and
    // `s`, both starting at 15 and sized by Tm. Neither primer carries a
    // mismatch, so both Tms are on the matched basis.
    #[test]
    fn a_deletion_gets_a_pair_that_abuts_across_the_removed_bases() {
        let sequence: Sequence = linear(&balanced());
        let design: PrimerDesign =
            design_primers(&sequence, deletion(&sequence, 50, 53), request(15)).unwrap();
        assert!(design.edit_span() == (50, 53));
        assert!(spans(&design) == ((53, 68), (34, 50)));
        assert!(design.five_prime_flank() == 0);
        assert!(design.forward_annealing_total() == 15);
        assert!(design.deleted_width() == 3);
        assert!(design.pair().abuts());
        assert!(design.forward_tm().basis() == TmBasis::MatchedComplement);
        assert!(design.reverse_tm().basis() == TmBasis::MatchedComplement);
    }

    // D031 is stated on a linear template, so a circular one is refused
    // before any of the deletion geometry is read.
    #[test]
    fn a_deletion_on_a_circular_template_is_out_of_scope() {
        let circular: Sequence = template_of(&balanced(), SequenceTopology::Circular);
        let error: DomainError =
            design_primers(&circular, deletion(&circular, 50, 53), request(15)).unwrap_err();
        assert!(matches!(
            error,
            DomainError::UnsupportedPrimerDesignScope(_)
        ));
        assert!(error.category() == ErrorCategory::Unsupported);
    }

    // D032: an insertion on a circular template is refused before any of the
    // insertion geometry is read, as every other edit is.
    #[test]
    fn an_insertion_on_a_circular_template_is_out_of_scope() {
        let circular: Sequence = template_of(&balanced(), SequenceTopology::Circular);
        let error: DomainError =
            design_primers(&circular, insertion(&circular, 50, "GAT"), request(15)).unwrap_err();
        assert!(matches!(
            error,
            DomainError::UnsupportedPrimerDesignScope(_)
        ));
        assert!(error.category() == ErrorCategory::Unsupported);
    }

    // D033: a 7-base region replaced by a single base has t = max(7, 1) = 7, above
    // `MAX_INLINE_REPLACEMENT`, so it takes the tails path: the forward tail is
    // `text[m/2..]` = the one base and the reverse tail is the reverse complement
    // of `text[..0]`, empty, so the reverse primer is untailed as an insertion's
    // whole-tail case is. The annealing portions are the starting 15 and 17, so
    // the spans are `[57, 72)` and `[33, 50)`.
    //
    // Every figure below is `reference/native-replacement.py`'s `design()` run by
    // the planner on 2026-10-15 (Biopython 1.88), not luria's output: the same
    // geometry as fixture row rep-05, whose forward whole Tm differs only because
    // that row's forward tail is 4 bases rather than this one base.
    #[test]
    fn a_one_base_replacement_on_the_tail_path_has_no_reverse_tail() {
        let sequence: Sequence = linear(&balanced());
        let design: PrimerDesign =
            design_primers(&sequence, replacement(&sequence, 50, 57, "T"), request(15)).unwrap();
        assert!(design.edit_span() == (50, 57));
        assert!(spans(&design) == ((57, 72), (33, 50)));
        assert!(
            texts(&design)
                == (
                    "TGCAGCCGAGCGGCTC".to_string(),
                    "TCCAAGCCATGCGTCCG".to_string()
                )
        );
        assert!(design.pair().forward().tail().unwrap().sequence() == "T");
        assert!(design.pair().reverse().tail().is_none());
        assert_tms(&design, 70.90, 68.76, 71.76);
        assert!(
            close(whole_celsius(design.forward_whole_tm()), 72.45),
            "forward whole Tm {:.3}",
            whole_celsius(design.forward_whole_tm())
        );
        assert!(design.tm_balance_met());
    }

    // D033: an inline replacement of 2 bases by 1 has t = max(2, 1) = 2, so it
    // stays inline, and `m != w` means the forward Tm is sized on the matched
    // basis rather than D030's mismatch-aware one. The written base differs from
    // the two it overwrites, so the template site shifted by `m − w` differs
    // from the primer at index 10 alone: one mismatch, which has published
    // parameters, so a sizing that always asked for the mismatch-aware basis
    // would report 71.71 on that basis rather than the 80.68 below. The basis
    // name and the absence of a `substituted_because` note are asserted
    // alongside the number. Every figure below is
    // `reference/native-replacement.py`'s `design()` run by the planner on
    // 2026-10-15 (Biopython 1.88), not luria's output.
    #[test]
    fn an_unequal_inline_replacement_is_sized_on_the_matched_basis() {
        let sequence: Sequence =
            linear(&[MIX, GC, "ATGGCTTGGA", "GGGGGGGGGGGGGGGGGGGG", GC, MIX].concat());
        let design: PrimerDesign =
            design_primers(&sequence, replacement(&sequence, 50, 52, "C"), request(15)).unwrap();
        assert!(design.edit_span() == (50, 52));
        assert!(spans(&design) == ((40, 62), (23, 40)));
        assert!(
            texts(&design)
                == (
                    "ATGGCTTGGACGGGGGGGGGG".to_string(),
                    "GCGTCCGCGAGCCGCTC".to_string()
                )
        );
        assert_tms(&design, 80.68, 76.28, 72.00);
        assert!(
            design.forward_tm().basis().name() == "matched_complement",
            "forward Tm basis {}",
            design.forward_tm().basis().name()
        );
        assert!(
            design.forward_tm().substituted_because().is_none(),
            "forward Tm substituted_because {:?}",
            design.forward_tm().substituted_because()
        );
    }

    /// A whole-primer Tm in °C; a design that reports none is a failure here.
    fn whole_celsius(temperature: Option<MeltingTemperature>) -> f64 {
        temperature.unwrap().celsius()
    }

    // D032 (ins-01): a 3-base insertion sits wholly on the forward primer's
    // 5′ tail, so the reverse primer is an untailed primer of the insertion.
    // Both annealing portions start at 15 and neither grows, so the spans are
    // the starting ones; the whole-primer Tms are reported but unused.
    #[test]
    fn a_short_insertion_is_one_forward_tail() {
        let sequence: Sequence = linear(&balanced());
        let design: PrimerDesign =
            design_primers(&sequence, insertion(&sequence, 50, "GAT"), request(15)).unwrap();
        assert!(design.edit_span() == (50, 50));
        assert!(spans(&design) == ((50, 65), (35, 50)));
        assert!(design.pair().forward().tail().unwrap().sequence() == "GAT");
        assert!(design.pair().reverse().tail().is_none());
        assert_tms(&design, 63.89, 62.17, 65.17);
        assert!(
            close(whole_celsius(design.forward_whole_tm()), 64.53),
            "forward whole Tm {:.3}",
            whole_celsius(design.forward_whole_tm())
        );
        assert!(
            close(whole_celsius(design.reverse_whole_tm()), 62.17),
            "reverse whole Tm {:.3}",
            whole_celsius(design.reverse_whole_tm())
        );
        assert!(design.forward_tm().basis() == TmBasis::MatchedComplement);
        assert!(design.reverse_tm().basis() == TmBasis::MatchedComplement);
        assert!(design.pair().abuts());
    }

    // D032 (ins-04): a 7-base insertion is above `MAX_WHOLE_TAIL`, so it is
    // split between the two primers: the forward tail takes the last ceil(k/2)
    // bases and the reverse tail the reverse complement of the first floor(k/2).
    #[test]
    fn a_long_insertion_is_split_between_the_primers() {
        let sequence: Sequence = linear(&balanced());
        let design: PrimerDesign =
            design_primers(&sequence, insertion(&sequence, 50, "GATTACA"), request(15)).unwrap();
        assert!(design.edit_span() == (50, 50));
        assert!(spans(&design) == ((50, 65), (35, 50)));
        assert!(design.pair().forward().tail().unwrap().sequence() == "TACA");
        assert!(design.pair().reverse().tail().unwrap().sequence() == "ATC");
        assert_tms(&design, 63.89, 62.17, 65.17);
        assert!(
            close(whole_celsius(design.forward_whole_tm()), 66.66),
            "forward whole Tm {:.3}",
            whole_celsius(design.forward_whole_tm())
        );
        assert!(
            close(whole_celsius(design.reverse_whole_tm()), 65.17),
            "reverse whole Tm {:.3}",
            whole_celsius(design.reverse_whole_tm())
        );
    }

    // D032: only an insertion has a tail, so only an insertion has a separate
    // whole-primer Tm to report.
    #[test]
    fn a_substitution_reports_no_whole_primer_tm() {
        let sequence: Sequence = linear(&balanced());
        let design: PrimerDesign =
            design_primers(&sequence, nucleotide(&sequence, 42, "A"), request(15)).unwrap();
        assert!(design.forward_whole_tm().is_none());
        assert!(design.reverse_whole_tm().is_none());
    }

    #[test]
    fn confine_to_tail_is_refused_before_the_search() {
        let tail: PrimerDesignRequest = PrimerDesignRequest::create(
            DesignStyle::Q5Polymerase,
            DesignPolicy::LuriaNative,
            15,
            PrimerPlacement::ConfineToTail,
            BufferModel::q5_reaction(),
        )
        .unwrap();
        let error: DomainError =
            design_primers(&template(), nucleotide(&template(), 30, "A"), tail).unwrap_err();
        assert!(matches!(error, DomainError::UnsupportedPrimerPlacement(_)));
    }

    #[test]
    fn a_result_from_another_template_is_refused() {
        let other: Sequence = linear(&TEMPLATE.replace("ATGGC", "ATGGG"));
        let error: DomainError =
            design_primers(&template(), nucleotide(&other, 30, "A"), request(15)).unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
    }

    #[test]
    fn the_design_records_its_inputs_and_what_it_left_unresolved() {
        let edit: MutationResult = nucleotide(&template(), 30, "A");
        let design: PrimerDesign = design_primers(&template(), edit.clone(), request(15)).unwrap();
        assert!(design.built_against() == template().reference());
        assert!(design.edited() == edit.reference());
        assert!(design.request() == request(15));
        assert!(design.request().buffer() == BufferModel::q5_reaction());
        assert!(design.forward_tm().buffer() == BufferModel::q5_reaction());
        assert!(design.algorithm_name() == NATIVE_DESIGN_ALGORITHM_NAME);
        assert!(design.algorithm_version() == "0.5.0");
        let inputs: Vec<String> = design.unresolved().into_iter().map(|u| u.input).collect();
        assert!(inputs == vec!["minTm", "maxAnnealingOverlap"]);
    }

    // A warning never stops a design: these are guidelines, not requirements.
    #[test]
    fn guideline_warnings_are_reported_and_do_not_stop_a_design() {
        // The AT-rich downstream case: the forward primer grows to 45 bases to
        // reach the Tm floor, which is well outside the 18-30 nt guideline and
        // far below the 40 % G+C band. The design is still emitted.
        let sequence: Sequence = linear(&at_downstream());
        let design: PrimerDesign =
            design_primers(&sequence, nucleotide(&sequence, 61, "A"), request(15)).unwrap();
        assert!(design.forward_tm().celsius() >= TM_FLOOR);

        let forward: Vec<PrimerWarning> = design.forward_warnings();
        assert!(
            forward.contains(&PrimerWarning::LengthOutsideGuideline { length: 45 }),
            "{:?}",
            forward
        );
        assert!(
            forward
                .iter()
                .any(|w| matches!(w, PrimerWarning::GcContentOutsideBand { gc_percent } if *gc_percent < 40.0)),
            "{:?}",
            forward
        );

        // Every warning names the primer it is about, forward first.
        let all: Vec<(PrimerRole, PrimerWarning)> = design.warnings();
        assert!(all.len() == forward.len() + design.reverse_warnings().len());
        assert!(all
            .iter()
            .take(forward.len())
            .all(|(role, _)| *role == PrimerRole::Forward));
    }

    // The warnings are of the primers this design actually emitted, not of
    // some candidate considered along the way.
    #[test]
    fn the_warnings_are_of_the_emitted_primers() {
        let design: PrimerDesign =
            design_primers(&template(), nucleotide(&template(), 30, "A"), request(15)).unwrap();
        let pair: PrimerPair = design.pair();
        let forward: Vec<char> = pair.clone().forward().sequence().chars().collect();
        let reverse: Vec<char> = pair.reverse().sequence().chars().collect();
        assert!(design.forward_warnings() == crate::primer_quality::inspect(&forward));
        assert!(design.reverse_warnings() == crate::primer_quality::inspect(&reverse));
    }

    #[test]
    fn the_same_request_gives_the_same_design() {
        let first: PrimerDesign =
            design_primers(&template(), nucleotide(&template(), 30, "A"), request(15)).unwrap();
        let second: PrimerDesign =
            design_primers(&template(), nucleotide(&template(), 30, "A"), request(15)).unwrap();
        assert!(first == second);
    }
}
