//! Primer representation, decomposition, and validation.
//!
//! This is the part of `docs/q5-primer-design.md` that the contract has
//! already settled, and nothing beyond it. The contract fixes what a primer
//! *is* and what must hold of one before it fixes how one is found: §4 names
//! the reference sequence a primer is built against, §5 defines the
//! decomposition and the overlap count, §6 fixes 5′→3′ representation and
//! orientation, and §7 states the requirements that are validity rather than
//! preference. What it leaves open is where a primer comes from — §15
//! questions 1, 3, 4, 5, 6 and 8 — so this module takes a proposed primer and
//! decides whether it is one. It never proposes a span, a length, or a
//! candidate.
//!
//! What the module therefore does:
//!
//! * `construct_primer` turns a proposed primer text plus a template span into
//!   a `Primer` record, deriving the parts, the annealing portion, the changed
//!   positions, the contract's annealing overlap, and the whole-primer match
//!   count. It re-checks the one structural invariant of §2.1 — that a
//!   `MutationResult`'s edited artifact differs from its source at positions
//!   inside the coordinates it claims — and reports a mismatch as a defect in the
//!   mutation layer (I-12), not as something repaired here.
//! * `assemble_pair` joins two constructed primers and checks the pair-level
//!   requirements: H2 (the pair covers the changed region) and H4 (opposite
//!   orientations). H5 is satisfied by `PrimerPair::relationship`, which
//!   derives the statement from the two spans rather than storing it, which is
//!   what I-8 requires.
//! * `require_design_constraints` applies H1, the only requirement that needs
//!   a caller's number: every primer's annealing overlap at or above
//!   `minAnnealingOverlap`.
//!
//! What it deliberately does not do, and why:
//!
//! * **No candidate search, no ranking, no preference.** P1–P4 (§8) need a
//!   search to be meaningful, and §15 question 1 leaves the primer-length
//!   question open. A layer that picked a span would be choosing a design rule
//!   the contract has not written.
//! * **No Tm, no `Ta`.** Nothing here computes a thermodynamic value: `tm.rs`
//!   defines the Tm and its bases, and `primer_design.rs` sizes primers by it
//!   and reports the annealing temperature (D030).
//! * **No `ConfineToTail` as a caller's choice.** `construct_primer` takes the
//!   placement as an argument and refuses `ConfineToTail` with
//!   `DomainError::UnsupportedPrimerPlacement`: §15 question 8 asks whether
//!   `luria` supports the placement at all, and §10 would need a style default
//!   that no source establishes. `construct_tailed_primer` fills the placement
//!   itself, deriving `ConfineToTail` from the shape of the primer it built.
//! * **Insertion and deletion design lives in `primer_design.rs`, not here.**
//!   Both are represented here — a deletion pair by D031's invariant, and an
//!   insertion pair by D032's — as is a tail-path replacement pair by D033's,
//!   and those two are the pairs whose primers fill the `tail` field. Nothing
//!   here proposes a span, a length or a candidate for any of them;
//!   `primer_design.rs` does. §5 still states the insertion and the deletion
//!   as separate degenerate cases rather than as one general rule.
//! * **No combined-variant design.** §15 questions 5 and 6 leave the
//!   representation of a composed request and the rule for overlapping spans
//!   open. One pair per edit is all this module claims.
//!
//! The decomposition is stated over a **set** of changed template positions,
//! not over one contiguous interval. §5 describes a single substitution at
//! `[s, e)`, which is contiguous, but a residue edit carries `OrderedIntervals`
//! and a CDS that wraps the origin of a circular reference gives two of them.
//! Maximal runs of changed and unchanged positions are what the parts and the
//! annealing spans are built from, so a non-contiguous changed set needs no
//! special case and no assumption that §5 documented one.

use crate::error::AnnealingOverlapBelowBoundProblem;
use crate::error::DomainError;
use crate::error::EmptySequenceProblem;
use crate::error::InvalidPrimerCandidateProblem;
use crate::error::InvalidSequenceCharactersProblem;
use crate::error::MutationVerificationFailureProblem;
use crate::error::PrimerAnnealingNotTemplateProblem;
use crate::error::PrimerOrientationConflictProblem;
use crate::error::PrimerPairCoverageProblem;
use crate::error::PrimerPartitionProblem;
use crate::error::PrimerTextAtSpanMismatchProblem;
use crate::error::UnsupportedPrimerPlacementProblem;
use crate::identity::SequenceReference;
use crate::interval::create;
use crate::interval::require_sequence;
use crate::interval::NucleotideInterval;
use crate::interval::OrderedIntervals;
use crate::mutation::MutationResult;
use crate::sequence::canonical_text;
use crate::sequence::is_dna_letter;
use crate::sequence::reverse_complement;
use crate::sequence::Sequence;

/// Identity of this layer's code path (D021).
pub const PRIMER_DESIGN_NAME: &str = "luria-core-primer";

/// Version of this layer's code path (D021).
pub const PRIMER_DESIGN_VERSION: &str = "0.1.0";

/// Which of the two primers of a Q5 pair this is. §6 defines the roles by the
/// construct's own orientation, not by a sequence operation: the forward
/// primer reads on the construct's strand and the reverse primer reads on the
/// opposite one.
#[derive(Clone, Debug, PartialEq)]
pub enum PrimerRole {
    Forward,
    Reverse,
}

impl PrimerRole {
    pub fn name(self) -> String {
        match self {
            PrimerRole::Forward => "forward".to_string(),
            PrimerRole::Reverse => "reverse".to_string(),
        }
    }

    pub fn is_forward(self) -> bool {
        match self {
            PrimerRole::Forward => true,
            PrimerRole::Reverse => false,
        }
    }

    pub fn is_reverse(self) -> bool {
        match self {
            PrimerRole::Reverse => true,
            PrimerRole::Forward => false,
        }
    }
}

/// The strand a primer reads on, relative to the template it was built
/// against (§3, §6). A forward primer runs in the direction of increasing
/// template coordinates; a reverse primer runs against it, and its text is the
/// reverse complement of the region it covers.
///
/// The value is relative rather than absolute because the layer is handed a
/// template, which carries no strand of its own. `same` and `opposite` are the
/// two values the reference records in `designs.json` use, so a reference
/// record and a `Primer` state orientation in one vocabulary.
#[derive(Clone, Debug, PartialEq)]
pub enum PrimerOrientation {
    Same,
    Opposite,
}

impl PrimerOrientation {
    pub fn name(self) -> String {
        match self {
            PrimerOrientation::Same => "same".to_string(),
            PrimerOrientation::Opposite => "opposite".to_string(),
        }
    }

    /// H4: the two primers of a pair must state opposite orientations.
    pub fn opposite_of(self, other: PrimerOrientation) -> bool {
        match self {
            PrimerOrientation::Same => other == PrimerOrientation::Opposite,
            PrimerOrientation::Opposite => other == PrimerOrientation::Same,
        }
    }
}

/// Where the mutation sits in a primer (§2.2, §5). A caller's choice, not a
/// biological rule — which is why it is an argument with no default rather
/// than a field the layer fills in. §10 would have a style default to name
/// here, and §15 question 8 records that no source establishes one.
#[derive(Clone, Debug, PartialEq)]
pub enum PrimerPlacement {
    Inline,
    ConfineToTail,
}

impl PrimerPlacement {
    pub fn name(self) -> String {
        match self {
            PrimerPlacement::Inline => "inline".to_string(),
            PrimerPlacement::ConfineToTail => "confine_to_tail".to_string(),
        }
    }

    /// Only `Inline` is supported today; see §15 question 8.
    pub fn is_supported(self) -> bool {
        match self {
            PrimerPlacement::Inline => true,
            PrimerPlacement::ConfineToTail => false,
        }
    }
}

/// A run of primer positions that does one thing: wild-type context that
/// anneals, or positions carrying the mutation that do not (§5). A 5′ tail is
/// the third kind, and `construct_tailed_primer` builds one for an insertion
/// (D032) and for a replacement too wide to sit inline (D033): its bases are
/// written by the edit rather than read from the template, so it covers no
/// template coordinate and does not anneal. What this layer still refuses is the
/// caller's `ConfineToTail` **placement**, which is a design rule (§15
/// question 8), not the part itself.
#[derive(Clone, Debug, PartialEq)]
pub enum PrimerPartRole {
    Context,
    Mutation,
    Tail,
}

impl PrimerPartRole {
    pub fn name(self) -> String {
        match self {
            PrimerPartRole::Context => "context".to_string(),
            PrimerPartRole::Mutation => "mutation".to_string(),
            PrimerPartRole::Tail => "tail".to_string(),
        }
    }

    pub fn carries_mutation(self) -> bool {
        match self {
            PrimerPartRole::Mutation => true,
            PrimerPartRole::Context => false,
            PrimerPartRole::Tail => false,
        }
    }
}

/// One run of a primer: its text, the template interval it sits at, and the
/// primer index range it occupies. `primer_indices` are half-open and zero
/// based within the primer; `span` is half-open on the template (D004), and
/// absent only for a `Tail` part, whose bases sit in a gap rather than at
/// template coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerPart {
    role: PrimerPartRole,
    annealing: bool,
    sequence: String,
    span: Option<NucleotideInterval>,
    primer_start: u64,
    primer_end: u64,
}

impl PrimerPart {
    pub fn role(self) -> PrimerPartRole {
        self.role.clone()
    }

    /// Whether this part counts toward the annealing overlap. §5: context
    /// anneals, the mutation and the 5′ tail do not.
    pub fn annealing(self) -> bool {
        self.annealing
    }

    pub fn sequence(self) -> String {
        self.sequence.clone()
    }

    /// The template interval this part sits at. `None` only for a `Tail` part:
    /// an inserted run covers no template coordinate at all.
    pub fn span(self) -> Option<NucleotideInterval> {
        self.span.clone()
    }
    pub fn primer_indices(self) -> (u64, u64) {
        (self.primer_start, self.primer_end)
    }

    pub fn length(self) -> u64 {
        self.primer_end - self.primer_start
    }
}

/// A primer: its text as it is to be typed into the kit, the template interval
/// it covers, and every quantity §3 and §5 require of it.
///
/// The text and the span name two different sequences (§4, consequence 2): the
/// text comes from the edited artifact and the span is an interval of the
/// pristine template. Both are stored, and `construct_primer` checks them
/// against each other, which is the invariant that catches an off-by-one
/// (H3, I-2).
#[derive(Clone, Debug, PartialEq)]
pub struct Primer {
    role: PrimerRole,
    orientation: PrimerOrientation,
    placement: PrimerPlacement,
    built_against: SequenceReference,
    sequence: String,
    length: u64,
    span: NucleotideInterval,
    edited_span: Option<NucleotideInterval>,
    parts: Vec<PrimerPart>,
    tail: Option<PrimerPart>,
    tail_reason: String,
    changed_positions: Vec<u64>,
    annealing_sequence: String,
    annealing_spans: OrderedIntervals,
    annealing_length: u64,
    annealing_overlap: u64,
    overlap_positions: Vec<u64>,
    whole_primer_match_count: u64,
}

impl Primer {
    pub fn role(self) -> PrimerRole {
        self.role.clone()
    }

    pub fn orientation(self) -> PrimerOrientation {
        self.orientation.clone()
    }

    pub fn placement(self) -> PrimerPlacement {
        self.placement.clone()
    }

    /// The template this primer was built against, by digest (H7, I-6).
    pub fn built_against(self) -> SequenceReference {
        self.built_against.clone()
    }

    pub fn sequence(self) -> String {
        self.sequence.clone()
    }

    pub fn length(self) -> u64 {
        self.length
    }

    /// The interval of the pristine template this primer covers (D004).
    pub fn span(self) -> NucleotideInterval {
        self.span.clone()
    }

    /// The interval of the **edited** artifact the primer text was taken from.
    ///
    /// H3 states this clause for the forward primer and states the reverse
    /// primer against the template instead, so the value is present exactly
    /// where the contract states it. For a substitution the two intervals
    /// coincide, which §5 records as the edited region `[s, s + L)` with
    /// `L = e - s`. For a deletion of `[s, e)` the forward primer's edited
    /// interval is its template span shifted left by the deleted width and
    /// named against the edited artifact, which is D031.
    pub fn edited_span(self) -> Option<NucleotideInterval> {
        self.edited_span.clone()
    }

    /// 1-based inclusive form of `span` for any header (D015).
    pub fn one_based_span(self) -> (u64, u64) {
        (self.span.start() + 1, self.span.end())
    }

    pub fn parts(self) -> Vec<PrimerPart> {
        self.parts.clone()
    }

    /// The 5′ portion carrying the mutation. Present on the primers built by
    /// `construct_tailed_primer`: an insertion primer, whose tail carries
    /// inserted bases, and a tail-path replacement primer, whose tail carries
    /// part of a replacement too wide to sit inline. Every other primer this
    /// layer constructs has none, because §5 places mutated positions inside
    /// the primer rather than at its 5′ end.
    pub fn tail(self) -> Option<PrimerPart> {
        self.tail.clone()
    }

    /// Why `tail` is absent, stated rather than left to the reader (§10).
    pub fn tail_reason(self) -> String {
        self.tail_reason.clone()
    }

    /// Primer indices whose text carries the mutation (§6: which positions
    /// changed is stated here, never by case).
    pub fn changed_positions(self) -> Vec<u64> {
        self.changed_positions.clone()
    }

    pub fn annealing_sequence(self) -> String {
        self.annealing_sequence.clone()
    }

    /// The template intervals the annealing portion covers (§3, I-1).
    pub fn annealing_spans(self) -> OrderedIntervals {
        self.annealing_spans.clone()
    }

    pub fn annealing_length(self) -> u64 {
        self.annealing_length
    }

    /// The contract's overlap: annealing positions whose text equals the
    /// template underneath them, counted position by position (§5, I-5).
    pub fn annealing_overlap(self) -> u64 {
        self.annealing_overlap
    }

    /// Which primer positions those are, so the count is checkable and not
    /// merely reported (§3).
    pub fn overlap_positions(self) -> Vec<u64> {
        self.overlap_positions.clone()
    }

    /// The quantity §5 keeps apart from `annealing_overlap`: the same count
    /// taken over the whole primer, mutated positions included. For the SID-1
    /// forward primer these are 20 and 21, and the difference is the base the
    /// replacement shares with the original, which is a mutated position and
    /// therefore not annealing (H8, I-13).
    pub fn whole_primer_match_count(self) -> u64 {
        self.whole_primer_match_count
    }
}

/// A Q5 pair: the two primers, and nothing else.
///
/// The relationship between them is **derived from the two spans** rather than
/// stored, which is what I-8 requires: there is no field that can disagree
/// with the spans it describes. The one stored width is that of the region the
/// pair straddles: the deleted region for a deletion, the replaced region for a
/// tail-path replacement, and 0 otherwise. D031 states the deletion case as the
/// pair invariant `reverse end + w = forward start`; the spans alone cannot say
/// how many template positions the edit sets aside between the two 5′ ends.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerPair {
    forward: Primer,
    reverse: Primer,
    deleted_width: u64,
    replaced: bool,
}

impl PrimerPair {
    pub fn forward(self) -> Primer {
        self.forward.clone()
    }

    pub fn reverse(self) -> Primer {
        self.reverse.clone()
    }

    /// The number of template positions between the pair's two annealing
    /// portions: the deleted region for a deletion, the replaced region for a
    /// tail-path replacement, and 0 otherwise. D031 states the deletion case as
    /// the pair invariant `reverse end + w = forward start`.
    pub fn deleted_width(self) -> u64 {
        self.deleted_width
    }

    /// Signed separation of the two spans along the template: positive when
    /// they are apart, negative when they overlap, zero when they abut.
    pub fn separation(self) -> i64 {
        let back: NucleotideInterval = self.reverse.clone().span();
        let front: NucleotideInterval = self.forward.clone().span();
        let one: i64 = signed(back.start()) - signed(front.end());
        let two: i64 = signed(front.start()) - signed(back.end());
        if one > two {
            return one;
        }
        two
    }

    /// Where the two 5′ ends meet, when they do. §15 question 3 asks whether a
    /// Q5 pair has to abut at all, so abutment is reported, never required.
    ///
    /// When `deleted_width` is greater than 0 — the deleted region of a
    /// deletion, or the replaced region of a tail-path replacement — the two
    /// spans are apart on the template, and the coordinate reported is the
    /// reverse primer's end: the start of that region, where D031's invariant,
    /// generalised by D033, puts it. For a deletion the two ends then meet in
    /// the **product**; for a tail-path replacement the replacement text sits
    /// between them, carried by the two tails.
    pub fn boundary(self) -> Option<u64> {
        let back: NucleotideInterval = self.reverse.clone().span();
        let front: NucleotideInterval = self.forward.clone().span();
        if self.deleted_width > 0 && back.clone().end() + self.deleted_width == front.start() {
            return Some(back.end());
        }
        if back.end() == front.start() {
            return Some(front.start());
        }
        if front.end() == back.start() {
            return Some(back.start());
        }
        None
    }

    /// D031's generalisation of abutment: the pair abuts when the gap between
    /// the two spans equals `deleted_width`, the number of template positions
    /// between the two annealing portions — the deleted region for a deletion,
    /// the replaced region for a tail-path replacement, and 0 otherwise — so
    /// that `reverse end + w = forward start` holds. The substitution case is
    /// `w = 0`, template abutment as §15 question 3 states it.
    pub fn abuts(self) -> bool {
        let width: u64 = self.deleted_width;
        self.separation() == signed(width)
    }

    /// H5: the pair's relationship to the edit, stated in the output and
    /// derived here rather than carried as a claim (§7 H5, I-8).
    pub fn relationship(self) -> String {
        let back: NucleotideInterval = self.reverse.clone().span();
        let front: NucleotideInterval = self.forward.clone().span();
        let width: u64 = self.deleted_width;
        let replaced: bool = self.replaced;
        let gap: i64 = self.separation();
        let phrase: String = if width > 0 && gap == signed(width) {
            if replaced {
                join(vec![
                    "abutting across a replaced region of ".to_string(),
                    number(width),
                    " template positions".to_string(),
                ])
            } else {
                join(vec![
                    "abutting across a deletion of ".to_string(),
                    number(width),
                    " template positions".to_string(),
                ])
            }
        } else if gap > 0 {
            join(vec![
                "apart by ".to_string(),
                number(negate(gap)),
                " template positions".to_string(),
            ])
        } else if gap == 0 {
            "abutting".to_string()
        } else {
            join(vec![
                "overlapping by ".to_string(),
                number(negate(gap)),
                " template positions".to_string(),
            ])
        };
        join(vec![
            "reverse span [".to_string(),
            describe(back),
            ") and forward span [".to_string(),
            describe(front),
            "): ".to_string(),
            phrase.clone(),
        ])
    }
}

/// Decide whether a proposed primer text at a proposed template span is a
/// primer under the contract, and report every quantity §3 and §5 define.
///
/// The inputs are the ones §2.1 requires: a validated `MutationResult`, not a
/// bare sequence and a coordinate. A design whose input were `(sequence,
/// position)` would have lost the provenance that makes it reproducible, and
/// this signature is where that rule is enforced.
///
/// The changed positions come from the result, and the result is re-checked
/// against the template on one point only — that its edited artifact differs
/// from the template at positions inside the coordinates it claims. That is the
/// structural invariant §2.1 names, and a mismatch is reported as a defect in
/// the mutation layer (I-12), not repaired here.
pub fn construct_primer(
    template: &Sequence,
    result: MutationResult,
    role: PrimerRole,
    placement: PrimerPlacement,
    text: String,
    span: NucleotideInterval,
) -> Result<Primer, DomainError> {
    let label: String = join(vec![
        role.clone().name(),
        " primer on ".to_string(),
        span.reference().describe(),
    ]);

    require_sequence(span.reference(), template)?;
    span.within(template)?;
    require_sequence(result.source().clone(), template)?;

    if !placement.clone().is_supported() {
        let problem = UnsupportedPrimerPlacementProblem {
            placement: placement.clone().name(),
            reason: join(vec![
                "§15 question 8 asks whether luria supports this placement at all, and §10 "
                    .to_string(),
                "would need a style default that no source establishes; the placement is "
                    .to_string(),
                "therefore refused rather than defaulted".to_string(),
            ]),
        };
        return Err(DomainError::UnsupportedPrimerPlacement(problem));
    }

    // D031: a deletion is a length change, and the mutation layer verified it
    // independently (`mutation::verify`). The two checks below compare the source
    // and the edited artifact position by position, which is meaningless across
    // a length change, so both are skipped for a deletion, for an insertion and
    // for a replacement (D033), which rewrites a region at any width.
    let deletion: bool = result.clone().is_deletion();
    let insertion: Option<(u64, String)> = insertion_claim(result.clone());
    let replacement: Option<(u64, u64, String)> = replacement_claim(result.clone());
    let edited: Sequence = result.edited();
    let before: Vec<char> = template.letters();
    let after: Vec<char> = edited.letters();
    if !deletion && insertion.is_none() && replacement.is_none() && before.len() != after.len() {
        let problem = MutationVerificationFailureProblem {
            result: result.describe(),
            reason: join(vec![
                "the source and the edited artifact differ in length, so the result is not a "
                    .to_string(),
                "substitution against this template".to_string(),
            ]),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    let claimed: Vec<u64> = result.target_coordinates();
    if !deletion && insertion.is_none() && replacement.is_none() {
        let actual: Vec<u64> = differing(before.clone(), after.clone());
        // §2.1's structural invariant, stated as containment: the edit must have
        // written inside the coordinates the result claims. Equality would reject
        // the contract's own worked case: §5's SID-1 pair replaces GAT with AAC and
        // the middle base is identical in both, so a residue edit claims the whole
        // codon and differs at only part of it. A claimed position whose letter did
        // not change is still a changed position, and §5 counts it as one.
        let stray: Vec<u64> = outside(actual.clone(), claimed.clone());
        if !stray.is_empty() {
            let problem = MutationVerificationFailureProblem {
                result: result.describe(),
                reason: join(vec![
                    "the edited artifact differs from the template at ".to_string(),
                    position_text(actual),
                    "; the result claims ".to_string(),
                    position_text(claimed),
                    ", and the difference at ".to_string(),
                    position_text(stray),
                    " lies outside that claim".to_string(),
                ]),
            };
            return Err(DomainError::MutationVerificationFailure(problem));
        }
    }

    // D031: neither primer of a deletion carries a non-template base, so a primer
    // may not reach into the removed region `[s, e)`. The pair invariant fixes
    // where the two primers of a pair sit; this refuses one primer on its own.
    let removed: (u64, u64) = deleted_region(claimed.clone());
    if deletion && span.clone().start() < removed.1 && span.clone().end() > removed.0 {
        let problem = InvalidPrimerCandidateProblem {
            primer: label,
            reason: join(vec![
                "the span ".to_string(),
                describe(span.clone()),
                " overlaps the deleted region [".to_string(),
                number(removed.0),
                ", ".to_string(),
                number(removed.1),
                "): a primer of a deletion covers template only, never a deleted base".to_string(),
            ]),
        };
        return Err(DomainError::InvalidPrimerCandidate(problem));
    }

    // D032: an untailed primer of an insertion carries template only, so it may
    // not read straight across the gap: the inserted bases belong to a 5′ tail,
    // which `construct_tailed_primer` builds. A span that ends at the point or
    // starts at it is fine — it abuts the gap without crossing it.
    if insertion.is_some() {
        let point: u64 = insertion.clone().unwrap().0;
        if span.clone().start() < point && point < span.clone().end() {
            let problem = InvalidPrimerCandidateProblem {
                primer: label,
                reason: join(vec![
                    "the span ".to_string(),
                    describe(span.clone()),
                    " straddles the insertion point at ".to_string(),
                    number(point),
                    ": an untailed primer covers template only, so it may not read ".to_string(),
                    "across the insertion point; the inserted bases belong to a 5' tail"
                        .to_string(),
                ]),
            };
            return Err(DomainError::InvalidPrimerCandidate(problem));
        }
    }

    // D033: a replacement's reverse primer states the template beside the
    // region, so it may not cover a replaced base. Its partner on the inline
    // path is the forward primer, and covering the region is that primer's job,
    // so the rule is stated for the reverse role only.
    if replacement.is_some() && role.clone().is_reverse() {
        let region: (u64, u64, String) = replacement.clone().unwrap();
        if span.clone().start() < region.1 && span.clone().end() > region.0 {
            let problem = InvalidPrimerCandidateProblem {
                primer: label,
                reason: join(vec![
                    "the span ".to_string(),
                    describe(span.clone()),
                    " overlaps the replaced region [".to_string(),
                    number(region.0),
                    ", ".to_string(),
                    number(region.1),
                    "): a reverse primer of a replacement states the template, so it may "
                        .to_string(),
                    "not cover a replaced base; the forward primer carries the replacement"
                        .to_string(),
                ]),
            };
            return Err(DomainError::InvalidPrimerCandidate(problem));
        }
    }

    if text.is_empty() {
        let problem = EmptySequenceProblem { sequence: label };
        return Err(DomainError::EmptySequenceContent(problem));
    }

    let letters: Vec<char> = text.chars().collect();
    let total: usize = letters.len();
    let mut index: usize = 0;
    while index < total {
        if !is_dna_letter(letters[index]) {
            let problem = InvalidSequenceCharactersProblem {
                sequence: label,
                position: index as u64,
                characters: letters[index].to_string(),
            };
            return Err(DomainError::InvalidSequenceCharacters(problem));
        }
        index += 1;
    }

    let span_width: u64 = span.end() - span.start();
    // D033: a replacement's forward primer names a template span but reads the
    // edited artifact, whose coordinates at and past the region's end have moved
    // by m − w, so its text is m − w longer or shorter than the span it names.
    // Every other primer's text is the width of its span.
    let shift: i64 = replacement_shift(role.clone(), replacement.clone());
    let text_width: i64 = signed(span_width) + shift;
    if text_width < 0 || total as u64 != text_width as u64 {
        let problem = InvalidPrimerCandidateProblem {
            primer: label,
            reason: join(vec![
                "the text is ".to_string(),
                number(total as u64),
                " nucleotides and the span ".to_string(),
                describe(span),
                " selects ".to_string(),
                number(span_width),
                shift_reason(shift),
            ]),
        };
        return Err(DomainError::InvalidPrimerCandidate(problem));
    }
    let width: u64 = total as u64;

    // §4 consequence 2: the primer text comes from the edited artifact, and
    // §6: a reverse primer is stated 5'->3', which is the reverse complement
    // of the region it covers. For a deletion, and for an insertion, the text
    // comes from the **template** instead: neither carries a non-template base,
    // so each primer reads wild-type context on whichever side of the edit it
    // sits (D031, D032).
    let letters_of_span: Vec<char> = if deletion || insertion.is_some() {
        before.clone()
    } else if replacement.is_some() && role.clone().is_reverse() {
        // D033: only a replacement's forward primer reads the edited artifact;
        // its reverse partner states the template.
        before.clone()
    } else {
        after.clone()
    };
    // D033: the forward primer of a replacement names a template interval but
    // reads the edited artifact, whose coordinates past the region's end have
    // moved by m − w; the text sits at [span.start, span.end + m − w), which the
    // width check above already bounded inside the edited artifact.
    let text_end: u64 = if replacement.is_some() && role.clone().is_forward() {
        (signed(span.end()) + shift) as u64
    } else {
        span.end()
    };
    let region: Vec<char> = oriented(
        slice(letters_of_span.clone(), span.start(), text_end),
        role.clone().is_reverse(),
    )?;
    if canonical_text(region.clone()) != canonical_text(letters.clone()) {
        let problem = PrimerTextAtSpanMismatchProblem {
            primer: label,
            span: describe(span),
            expected: canonical_text(region.clone()),
            actual: canonical_text(letters.clone()),
        };
        return Err(DomainError::PrimerTextAtSpanMismatch(problem));
    }

    // The edit's changed template coordinates, kept only where the primer
    // covers them, and mapped into the primer's own index space. §6: a reverse
    // primer reads against increasing coordinates, so its primer index 0 is the
    // template position one below its span's end, not its span's start. A
    // deletion writes nothing inside either primer, so it contributes no changed
    // position and every part of the primer is context.
    let claimed_within: Vec<u64> = if deletion {
        Vec::new()
    } else {
        claimed.clone()
    };
    let changed: Vec<u64> = if replacement.is_some() && role.clone().is_forward() {
        // D033: the replacement's bases occupy the edited interval [s, s + m), so
        // the changed primer indices are that interval inside the primer's own
        // edited range. With m = w this is `changed_within` unchanged, which is
        // why a substitution needs no special case here.
        replacement_changed(span.clone(), replacement.clone().unwrap())
    } else if replacement.is_some() {
        // D033: a replacement's reverse primer carries no changed base on either
        // path: it states the template beside the region.
        Vec::new()
    } else {
        changed_within(span.clone(), claimed_within, role.clone().is_reverse())
    };

    let reference: SequenceReference = span.reference();
    let edited_span: Option<NucleotideInterval> = if deletion {
        deletion_edited_span(
            role.clone(),
            span.clone(),
            removed.0,
            removed.1,
            edited.clone().reference(),
        )?
    } else if insertion.is_some() {
        // D032: the edited artifact shifts its coordinates by the inserted width
        // at the insertion point, so neither primer's text is one interval of it.
        None
    } else if replacement.is_some() {
        // D033: the primer reads the edited artifact over its whole span, so the
        // formula `span.end + m − w` only holds when the span covers the region:
        // a span that starts or ends inside the region would carry only part of
        // the replacement, and its length would not follow from the region's
        // width. Refuse it rather than state a span that does not name the text.
        if role.clone().is_forward()
            && (span.clone().start() > replacement.clone().unwrap().0
                || span.clone().end() < replacement.clone().unwrap().1)
        {
            let problem = InvalidPrimerCandidateProblem {
                primer: label,
                reason: join(vec![
                    "a forward primer of a replacement reads the edited artifact over ".to_string(),
                    describe(span.clone()),
                    ", whose length follows from the region only when the span covers the replaced region [".to_string(),
                    number(replacement.clone().unwrap().0),
                    ", ".to_string(),
                    number(replacement.clone().unwrap().1),
                    ")".to_string(),
                ]),
            };
            return Err(DomainError::InvalidPrimerCandidate(problem));
        }
        // D033: the forward primer's text is one interval of the edited artifact,
        // shifted by m − w past the region's end; the reverse primer states the
        // template and has none.
        replacement_edited_span(
            role.clone(),
            span.clone(),
            replacement.clone().unwrap(),
            edited.clone().reference(),
        )?
    } else {
        edited_span_of(role.clone(), span.clone())
    };
    let parts: Vec<PrimerPart> = if replacement.is_some() && role.clone().is_forward() {
        // D033: the primer's text and its span differ in width, so each run sits
        // at its own template interval rather than at one shifted image of the
        // span; `runs` assumes one shift for the whole primer.
        replacement_runs(
            changed.clone(),
            width,
            span.clone(),
            replacement.clone().unwrap(),
            reference,
            letters.clone(),
        )?
    } else {
        runs(
            changed.clone(),
            width,
            span.clone(),
            role.clone().is_reverse(),
            reference,
            letters.clone(),
        )?
    };
    let annealing_spans: OrderedIntervals = annealing_of(parts.clone());
    let annealing_indices: Vec<u64> = unchanged(changed.clone(), width);
    let annealing_text: String = canonical_text(pick(letters.clone(), annealing_indices.clone()));

    // I-1 / H3 clause 2: the annealing portion is the wild-type template
    // under the intervals it covers, reverse-complemented for a reverse
    // primer. Checked rather than assumed, so a wrong interval list is caught
    // rather than reported.
    let template_region: Vec<char> = if replacement.is_some() && role.clone().is_forward() {
        // D033: the text is longer or shorter than the span, so the template
        // under it is aligned to the primer's indices: the region's own bases
        // stand where the replacement wrote over them, and the template past the
        // region resumes at `e`.
        replacement_template(
            before.clone(),
            span.clone(),
            replacement.clone().unwrap(),
            letters.clone(),
        )
    } else {
        oriented(
            slice(before.clone(), span.start(), span.end()),
            role.clone().is_reverse(),
        )?
    };
    let under: Vec<char> = pick(template_region.clone(), annealing_indices.clone());
    // Defensive check: the text-at-span check above already compared this
    // primer's text with the slice it reads, and `changed` is exactly the
    // result's claimed coordinates inside that slice, so every annealing
    // index reads a coordinate the two artifacts agree on. For a deletion,
    // an insertion and a replacement's reverse primer, `letters_of_span`
    // is `before` itself, so `region` and `template_region` are the same
    // slice and the two texts are equal by construction; for a
    // substitution, the §2.1 containment check (`stray`, reported as
    // MutationVerificationFailure) is what makes the edited artifact equal
    // the template outside the claimed coordinates; and for a
    // replacement's forward primer, `replacement_template` aligns the
    // template to the splice `mutation::apply_replacement` built. A
    // mismatch here would mean one of those checks was bypassed, so it is
    // reported rather than assumed.
    if canonical_text(under.clone()) != annealing_text.clone() {
        let problem = PrimerAnnealingNotTemplateProblem {
            primer: label,
            position: span.start(),
            expected: canonical_text(under.clone()),
            actual: annealing_text,
        };
        return Err(DomainError::PrimerAnnealingNotTemplate(problem));
    }

    // H8 / I-13: the parts partition the primer text, and the mutation parts
    // are exactly the changed positions.
    let fault: String = partition_fault(parts.clone(), changed.clone(), width);
    // Defensive check: `partition_fault` re-reads the parts `runs` and
    // `replacement_runs` just built from the same `changed` and `width`, so
    // contiguity, non-empty parts and full coverage hold by construction,
    // and the mutation runs are the maximal runs of that same list. The
    // last clause is the only one that could fail, and only for a `changed`
    // that is not ascending and duplicate-free inside `[0, width)`; both
    // producers keep it there. `changed_within` filters the result's
    // `target_coordinates` — ascending and duplicate-free, since they come
    // from `OrderedIntervals` — and `reversed` puts a reverse primer's
    // inverted mapping back into ascending order, while `replacement_changed`
    // returns one ascending range within the primer's own width. The fault
    // is reported rather than assumed, so a change to either run builder is
    // caught here.
    if !fault.is_empty() {
        let problem = PrimerPartitionProblem {
            primer: label,
            reason: fault,
        };
        return Err(DomainError::PrimerPartition(problem));
    }

    // §5: overlap is counted position by position, over annealing positions
    // only. The whole-primer count is kept apart under its own name.
    let mut overlap_positions: Vec<u64> = Vec::new();
    let mut whole_matches: u64 = 0;
    let mut position: u64 = 0;
    while position < width {
        if template_region[position as usize] == letters[position as usize] {
            whole_matches += 1;
            if !is_changed(changed.clone(), position) {
                overlap_positions.push(position);
            }
        }
        position += 1;
    }

    let reason: String = if changed.is_empty() {
        "carries no mutated base: the whole primer is wild-type context".to_string()
    } else {
        join(vec![
            "no 5' tail: the mutation is placed inline at primer indices ".to_string(),
            position_text(changed.clone()),
            ", not at the primer's 5' end".to_string(),
        ])
    };

    Ok(Primer {
        role: role.clone(),
        orientation: oriented_of(role.clone()),
        placement: placement.clone(),
        built_against: template.reference(),
        sequence: canonical_text(letters.clone()),
        length: width,
        span: span.clone(),
        edited_span: edited_span.clone(),
        parts: parts.clone(),
        tail: None,
        tail_reason: reason,
        changed_positions: changed.clone(),
        annealing_sequence: annealing_text.clone(),
        annealing_spans: annealing_spans.clone(),
        annealing_length: annealing_indices.len() as u64,
        annealing_overlap: overlap_positions.len() as u64,
        overlap_positions: overlap_positions.clone(),
        whole_primer_match_count: whole_matches,
    })
}

/// Build a primer whose 5′ end is a non-annealing tail: the text is `tail`
/// followed by `annealing`, and `span` covers the **annealing portion only**,
/// because the tail's bases are written by the edit rather than read from the
/// template (D032, D033). This is the only shape in which the `tail` field is
/// filled, and both the edits that use it fill it: an insertion writes no
/// template position, so the bases it adds have to sit somewhere, and a
/// replacement too wide to sit inline carries its bases in the two tails
/// instead of over the region (D033).
///
/// `construct_primer` stays as it is for the other edits and for an untailed
/// primer of an insertion or of a replacement; a caller that wants no tail
/// uses it, and this constructor refuses an empty tail rather than
/// defaulting one.
///
/// The requirements are the ones §2 states for an insertion pair and D033 for
/// a tail-path replacement, applied to one primer: the annealing portion is
/// wild-type template at the span, the tail carries the bases the edit writes
/// — inserted bases, or part of a replacement — and anneals
/// nowhere, and nothing under this primer changed. Which bases the tail must
/// carry is a **pair** rule — the two tails together rebuild the insertion or
/// the replacement — so it is checked in `assemble_pair`, not here.
pub fn construct_tailed_primer(
    template: &Sequence,
    result: MutationResult,
    role: PrimerRole,
    tail: String,
    annealing: String,
    span: NucleotideInterval,
) -> Result<Primer, DomainError> {
    let label: String = join(vec![
        role.clone().name(),
        " primer on ".to_string(),
        span.reference().describe(),
    ]);

    require_sequence(span.reference(), template)?;
    span.within(template)?;
    require_sequence(result.source().clone(), template)?;

    // D032 and D033: a tail carries what the primer's own span cannot — the
    // bases an insertion adds, or the half of a replacement too wide to sit
    // inline. A substitution rewrites template positions inside the primer, and
    // a deletion removes positions that no primer may carry, so neither has a
    // tail to fill.
    if !result.clone().is_insertion() && !result.clone().is_replacement() {
        let problem = InvalidPrimerCandidateProblem {
            primer: label,
            reason: join(vec![
                "a 5′ tail carries inserted bases into a gap, and the edit ".to_string(),
                result.describe(),
                " inserts nothing: it writes or removes template positions, which ".to_string(),
                "sit inside the annealing portion rather than at the primer's 5' end".to_string(),
            ]),
        };
        return Err(DomainError::InvalidPrimerCandidate(problem));
    }

    // Review B1 (H2, H3, H8): a tailed primer's annealing portion sits beside
    // the edit, in a gap of the template, and its tail carries the bases the
    // edit writes there. A span laid over the edit anneals over bases that are
    // no longer the template's, and its tail then rebuilds nothing the pair
    // needs, so the pair would not contain the edit. The span must therefore
    // lie wholly on one side of the edit: outside the replaced region [s, e),
    // and not straddling the insertion point p.
    let replacement: Option<(u64, u64, String)> = replacement_claim(result.clone());
    if replacement.is_some() {
        let region: (u64, u64, String) = replacement.clone().unwrap();
        if span.start() < region.1 && span.end() > region.0 {
            let problem = InvalidPrimerCandidateProblem {
                primer: label,
                reason: join(vec![
                    "the replacement is [".to_string(),
                    number(region.0),
                    ", ".to_string(),
                    number(region.1),
                    "), so a tailed primer's annealing portion lies outside the ".to_string(),
                    "replaced region: the span ".to_string(),
                    describe(span),
                    " overlaps it and would anneal over bases the replacement overwrites"
                        .to_string(),
                ]),
            };
            return Err(DomainError::InvalidPrimerCandidate(problem));
        }
    }
    let insertion: Option<(u64, String)> = insertion_claim(result.clone());
    if insertion.is_some() {
        let point: u64 = insertion.clone().unwrap().0;
        if span.start() < point && span.end() > point {
            let problem = InvalidPrimerCandidateProblem {
                primer: label,
                reason: join(vec![
                    "the insertion point is at ".to_string(),
                    number(point),
                    ", so a tailed primer's annealing portion may not straddle it: the "
                        .to_string(),
                    "span ".to_string(),
                    describe(span),
                    " starts before ".to_string(),
                    number(point),
                    " and ends after it".to_string(),
                ]),
            };
            return Err(DomainError::InvalidPrimerCandidate(problem));
        }
    }

    if tail.is_empty() {
        let problem = InvalidPrimerCandidateProblem {
            primer: label,
            reason: join(vec![
                "there is no tail to place: a primer without a 5′ tail is built by ".to_string(),
                "construct_primer, which is not defaulted here".to_string(),
            ]),
        };
        return Err(DomainError::InvalidPrimerCandidate(problem));
    }
    if annealing.is_empty() {
        let problem = EmptySequenceProblem { sequence: label };
        return Err(DomainError::EmptySequenceContent(problem));
    }

    let tail_letters: Vec<char> = tail.chars().collect();
    let annealing_letters: Vec<char> = annealing.chars().collect();

    // Every letter of both halves is checked, at its index into the primer's
    // own text: the tail occupies the primer's first indices and the annealing
    // portion follows it.
    let mut letters: Vec<char> = tail_letters.clone();
    letters.extend(annealing_letters.clone());
    let total: usize = letters.len();
    let mut index: usize = 0;
    while index < total {
        if !is_dna_letter(letters[index]) {
            let problem = InvalidSequenceCharactersProblem {
                sequence: label,
                position: index as u64,
                characters: letters[index].to_string(),
            };
            return Err(DomainError::InvalidSequenceCharacters(problem));
        }
        index += 1;
    }

    let width: u64 = span.end() - span.start();
    if annealing_letters.len() as u64 != width {
        let problem = InvalidPrimerCandidateProblem {
            primer: label,
            reason: join(vec![
                "the text is ".to_string(),
                number(annealing_letters.len() as u64),
                " nucleotides and the span ".to_string(),
                describe(span),
                " selects ".to_string(),
                number(width),
            ]),
        };
        return Err(DomainError::InvalidPrimerCandidate(problem));
    }

    // The annealing portion is wild-type template: an insertion changes no
    // template position, so both primers of its pair read the source, and a
    // reverse primer states its region 5'->3' against the increasing
    // coordinates (§6).
    let before: Vec<char> = template.letters();
    let region: Vec<char> = oriented(
        slice(before.clone(), span.start(), span.end()),
        role.clone().is_reverse(),
    )?;
    if canonical_text(region.clone()) != canonical_text(annealing_letters.clone()) {
        let problem = PrimerTextAtSpanMismatchProblem {
            primer: label,
            span: describe(span),
            expected: canonical_text(region.clone()),
            actual: canonical_text(annealing_letters.clone()),
        };
        return Err(DomainError::PrimerTextAtSpanMismatch(problem));
    }

    let tail_length: u64 = tail_letters.len() as u64;
    let tail_part: PrimerPart = PrimerPart {
        role: PrimerPartRole::Tail,
        annealing: false,
        sequence: canonical_text(tail_letters.clone()),
        span: None,
        primer_start: 0,
        primer_end: tail_length,
    };
    let context_part: PrimerPart = PrimerPart {
        role: PrimerPartRole::Context,
        annealing: true,
        sequence: canonical_text(annealing_letters.clone()),
        span: Some(span.clone()),
        primer_start: tail_length,
        primer_end: tail_length + width,
    };
    let parts: Vec<PrimerPart> = vec![tail_part.clone(), context_part];

    // The annealing portion is one whole run, so its indices are every primer
    // index past the tail, and each of them matches the template underneath it
    // by the check above.
    let annealing_indices: Vec<u64> = range_of(tail_length, tail_length + width);
    let annealing_text: String = canonical_text(annealing_letters.clone());

    // D032 and D033: the tail carries what the primer's own span cannot, and
    // which of the two it carries follows from the edit.
    let tail_kind: String = if result.clone().is_replacement() {
        "part of the replacement".to_string()
    } else {
        "the insertion".to_string()
    };

    Ok(Primer {
        role: role.clone(),
        orientation: oriented_of(role.clone()),
        // The mutation sits wholly in the tail, which is what `ConfineToTail`
        // names. That placement is refused in `construct_primer` only as a
        // caller's choice (§15 question 8); here it is derived from the shape
        // of the primer this constructor built.
        placement: PrimerPlacement::ConfineToTail,
        built_against: template.reference(),
        sequence: join(vec![
            canonical_text(tail_letters.clone()),
            canonical_text(annealing_letters.clone()),
        ]),
        length: tail_length + width,
        span: span.clone(),
        // Neither role has an edited interval: this constructor states the
        // primer against the template span its annealing portion covers, and
        // the tail's bases are named by the tail part itself, not by an
        // interval of either artifact.
        edited_span: None,
        parts: parts.clone(),
        tail: Some(tail_part.clone()),
        tail_reason: join(vec![
            "a 5′ tail of ".to_string(),
            number(tail_length),
            " bases carries ".to_string(),
            tail_kind,
            " and does not anneal".to_string(),
        ]),
        // No template position under this primer changed: an insertion writes
        // into a gap, and a tail-path replacement's bases sit in the tails,
        // with the checks here keeping this primer's span clear of the
        // replaced region.
        changed_positions: Vec::new(),
        annealing_sequence: annealing_text.clone(),
        annealing_spans: vec![span.clone()],
        annealing_length: width,
        annealing_overlap: width,
        overlap_positions: annealing_indices.clone(),
        whole_primer_match_count: width,
    })
}

/// Join two constructed primers into a pair and check the requirements that
/// belong to a pair rather than to a primer: H2 that the pair covers the
/// changed region of its edit, and H4 that the two primers state opposite
/// orientations. Across a deletion H2 does not apply and D031's pair invariant
/// replaces it; across an insertion H2 does not apply either, and D032's rule
/// does: both annealing portions meet at the insertion point, and the two tails
/// rebuild the inserted text. H5 is satisfied by `PrimerPair::relationship`,
/// which derives the statement from the two spans (I-8).
///
/// Abutment is **not** required here. §15 question 3 asks whether a Q5 pair
/// has to abut or only cover the edit, and no fixture establishes a rule for
/// `luria`, so the pair reports its separation and leaves the decision open.
pub fn assemble_pair(
    forward: Primer,
    reverse: Primer,
    result: MutationResult,
) -> Result<PrimerPair, DomainError> {
    if !forward.clone().role().is_forward() {
        let problem = InvalidPrimerCandidateProblem {
            primer: forward.clone().sequence(),
            reason: "the forward slot needs a primer whose role is forward".to_string(),
        };
        return Err(DomainError::InvalidPrimerCandidate(problem));
    }
    if !reverse.clone().role().is_reverse() {
        let problem = InvalidPrimerCandidateProblem {
            primer: reverse.clone().sequence(),
            reason: "the reverse slot needs a primer whose role is reverse".to_string(),
        };
        return Err(DomainError::InvalidPrimerCandidate(problem));
    }

    if forward.clone().built_against() != reverse.clone().built_against() {
        let problem = InvalidPrimerCandidateProblem {
            primer: "pair".to_string(),
            reason: join(vec![
                "the two primers were built against different templates: ".to_string(),
                forward.built_against().describe(),
                " and ".to_string(),
                reverse.built_against().describe(),
            ]),
        };
        return Err(DomainError::InvalidPrimerCandidate(problem));
    }
    if forward.clone().built_against() != result.clone().source() {
        let problem = InvalidPrimerCandidateProblem {
            primer: "pair".to_string(),
            reason: join(vec![
                "the pair was built against ".to_string(),
                forward.built_against().describe(),
                " and the edit against ".to_string(),
                result.source().describe(),
            ]),
        };
        return Err(DomainError::InvalidPrimerCandidate(problem));
    }

    // Defensive check: H4 cannot fail for a pair this module built. Both
    // orientations come from `oriented_of(role)`, and the two role checks
    // above refuse any primer whose role is not the slot it stands in, so
    // the comparison always sees `Same` against `Opposite` — the one pair
    // `opposite_of` accepts. A `Primer` whose role and orientation disagree
    // is not an input this API accepts: the struct's fields are private,
    // `construct_primer` and `construct_tailed_primer` are the only
    // constructors, and neither offers an orientation argument. The check
    // stays so H4 remains stated rather than assumed.
    if !forward
        .clone()
        .orientation()
        .opposite_of(reverse.clone().orientation())
    {
        let problem = PrimerOrientationConflictProblem {
            forward: forward.clone().orientation().name(),
            reverse: reverse.clone().orientation().name(),
        };
        return Err(DomainError::PrimerOrientationConflict(problem));
    }

    // H2: every changed template position must sit inside one of the two
    // primers' spans. Which positions went uncovered is reported, because
    // "the pair does not cover the edit" alone is not actionable (§10).
    let claimed: Vec<u64> = result.clone().target_coordinates();
    let deletion: bool = result.clone().is_deletion();
    let insertion: Option<(u64, String)> = insertion_claim(result.clone());
    let replacement: Option<(u64, u64, String)> = replacement_claim(result.clone());
    let mut deleted_width: u64 = 0;
    let mut replaced: bool = false;
    if deletion {
        // D031's pair invariant: reverse end + w = forward start. A deletion's
        // removed bases lie between the primers by design, so H2's coverage
        // check does not apply to it; the invariant is what pins the pair to the
        // edges of the deletion instead.
        let removed: (u64, u64) = deleted_region(claimed.clone());
        let reverse_end: u64 = reverse.clone().span().end();
        let forward_start: u64 = forward.clone().span().start();
        if reverse_end != removed.0 || forward_start != removed.1 {
            let problem = InvalidPrimerCandidateProblem {
                primer: "pair".to_string(),
                reason: join(vec![
                    "the deletion is [".to_string(),
                    number(removed.0),
                    ", ".to_string(),
                    number(removed.1),
                    "), so D031 requires the reverse primer to end at ".to_string(),
                    number(removed.0),
                    " and the forward primer to start at ".to_string(),
                    number(removed.1),
                    "; the reverse primer ends at ".to_string(),
                    number(reverse_end),
                    " and the forward primer starts at ".to_string(),
                    number(forward_start),
                ]),
            };
            return Err(DomainError::InvalidPrimerCandidate(problem));
        }
        deleted_width = removed.1 - removed.0;
    } else if insertion.is_some() {
        // D032: H2's coverage check does not apply to an insertion. It changes
        // no template position, so `claimed` is empty and the loop below would
        // be vacuous — the skip is stated here rather than left to that.
        // D032's own rule pins the pair instead: both annealing portions abut
        // at the insertion point, and the two tails together rebuild the
        // inserted text. `deleted_width` stays 0, so `abuts()` is the template
        // abutment of §15 question 3, which here means the two portions touch
        // at the point.
        let point: u64 = insertion.clone().unwrap().0;
        let inserted: String = insertion.clone().unwrap().1;
        let reverse_end: u64 = reverse.clone().span().end();
        let forward_start: u64 = forward.clone().span().start();
        if forward_start != point || reverse_end != point {
            let problem = InvalidPrimerCandidateProblem {
                primer: "pair".to_string(),
                reason: join(vec![
                    "the insertion is at point ".to_string(),
                    number(point),
                    ", so D032 requires the forward primer's annealing portion to start at "
                        .to_string(),
                    number(point),
                    " and the reverse primer's to end at ".to_string(),
                    number(point),
                    "; the forward primer starts at ".to_string(),
                    number(forward_start),
                    " and the reverse primer ends at ".to_string(),
                    number(reverse_end),
                ]),
            };
            return Err(DomainError::InvalidPrimerCandidate(problem));
        }

        // D032: the two tails are the whole insertion, split between the primers:
        // `require_tails_rebuild` states which half comes from which primer and
        // what a mismatch reports. A primer with no tail contributes an empty
        // string, which is what a `k <= 6` insertion gives the reverse primer.
        require_tails_rebuild(&forward, &reverse, &inserted, "insertion")?;
    } else if replacement.is_some()
        && (forward.clone().tail().is_some() || reverse.clone().tail().is_some())
    {
        // D033: the tail path. A replacement too wide to sit inline puts its
        // bases in the two tails, and each primer's span then lies outside the
        // region. What decides this branch is that one of the primers carries a
        // tail — the forward one always does, ceil(m / 2) >= 1, and the reverse
        // one may not, floor(m / 2) = 0 when m = 1 — and not where the forward
        // span happens to start: a forward primer annealed straight over the
        // region would pass H2's coverage check below while its partner's tail
        // rebuilds nothing, so the pair is refused at the edge check instead.
        // An inline pair has no tail on either primer, so it falls through to
        // the substitution branch below, where H2's coverage check applies and
        // `deleted_width` stays 0.
        let region: (u64, u64, String) = replacement.clone().unwrap();
        let reverse_end: u64 = reverse.clone().span().end();
        let forward_start: u64 = forward.clone().span().start();
        if reverse_end != region.0 || forward_start != region.1 {
            let problem = InvalidPrimerCandidateProblem {
                primer: "pair".to_string(),
                reason: join(vec![
                    "the replacement is [".to_string(),
                    number(region.0),
                    ", ".to_string(),
                    number(region.1),
                    "), so the reverse primer's annealing portion must end at ".to_string(),
                    number(region.0),
                    " and the forward primer's must start at ".to_string(),
                    number(region.1),
                    "; the reverse primer ends at ".to_string(),
                    number(reverse_end),
                    " and the forward primer starts at ".to_string(),
                    number(forward_start),
                ]),
            };
            return Err(DomainError::InvalidPrimerCandidate(problem));
        }

        // D033: the two tails are the whole replacement, split between the primers;
        // `require_tails_rebuild` states which half comes from which primer and
        // what a mismatch reports, in the same shape D032's insertion check uses.
        require_tails_rebuild(&forward, &reverse, &region.2, "replacement")?;
        // D033: the annealing portions sit at the two edges of the replaced
        // region, so `abuts()` means what it means across a deletion: the gap
        // between the spans is the width of the region the edit overwrote.
        deleted_width = region.1 - region.0;
        replaced = true;
    } else {
        let changed: Vec<u64> = claimed.clone();
        let mut uncovered: Vec<u64> = Vec::new();
        for position in changed.clone() {
            if !covers(forward.clone().span(), position)
                && !covers(reverse.clone().span(), position)
            {
                uncovered.push(position);
            }
        }
        if !uncovered.is_empty() {
            let problem = PrimerPairCoverageProblem {
                pair: join(vec![
                    "forward ".to_string(),
                    describe(forward.clone().span()),
                    " reverse ".to_string(),
                    describe(reverse.clone().span()),
                ]),
                changed: position_text(changed.clone()),
                uncovered: position_text(uncovered),
            };
            return Err(DomainError::PrimerPairCoverage(problem));
        }
    }

    Ok(PrimerPair {
        forward: forward.clone(),
        reverse: reverse.clone(),
        deleted_width,
        replaced,
    })
}

/// Apply H1, the only hard requirement that needs a caller's number: every
/// primer's annealing overlap at or above `minAnnealingOverlap` (§2.2, §9).
///
/// The bound is a bound, not a target, and it is a bound on the **annealing**
/// overlap. `whole_primer_match_count` is a different quantity and is never
/// consulted here — that is the whole point of §5 and of H8.
pub fn require_design_constraints(
    pair: PrimerPair,
    min_annealing_overlap: u64,
) -> Result<(), DomainError> {
    if min_annealing_overlap < 1 {
        let problem = InvalidPrimerCandidateProblem {
            primer: "minAnnealingOverlap".to_string(),
            reason: "§2.2 types it as an integer of at least 1; 0 is not a bound".to_string(),
        };
        return Err(DomainError::InvalidPrimerCandidate(problem));
    }

    let first: Primer = pair.clone().forward();
    let second: Primer = pair.clone().reverse();
    if first.clone().annealing_overlap() < min_annealing_overlap {
        let problem = AnnealingOverlapBelowBoundProblem {
            primer: join(vec![
                first.clone().role().name(),
                " primer ".to_string(),
                first.clone().sequence(),
            ]),
            overlap: first.clone().annealing_overlap(),
            bound: min_annealing_overlap,
        };
        return Err(DomainError::AnnealingOverlapBelowBound(problem));
    }
    if second.clone().annealing_overlap() < min_annealing_overlap {
        let problem = AnnealingOverlapBelowBoundProblem {
            primer: join(vec![
                second.clone().role().name(),
                " primer ".to_string(),
                second.clone().sequence(),
            ]),
            overlap: second.clone().annealing_overlap(),
            bound: min_annealing_overlap,
        };
        return Err(DomainError::AnnealingOverlapBelowBound(problem));
    }
    Ok(())
}

/// The orientation a role implies (§6). Derived rather than supplied, so H4
/// cannot be argued into a different answer; §7 still checks it on the output.
fn oriented_of(role: PrimerRole) -> PrimerOrientation {
    if role.is_forward() {
        return PrimerOrientation::Same;
    }
    PrimerOrientation::Opposite
}

/// H3 states the edited-sequence clause for the forward primer and states the
/// reverse primer against the template instead, so the edited interval is
/// present exactly where the contract states it.
fn edited_span_of(role: PrimerRole, span: NucleotideInterval) -> Option<NucleotideInterval> {
    if role.is_forward() {
        return Some(span.clone());
    }
    None
}

/// The removed region `[s, e)` of a deletion result, from the coordinates it
/// claims: they are ascending and contiguous, so `s` is the first and `e` is
/// one past the last. A deletion always removes at least one base, since
/// `interval::create` rejects an empty region; the guard keeps the helper total
/// rather than panicking on a claim that cannot come from `mutation.rs`.
pub(crate) fn deleted_region(claimed: Vec<u64>) -> (u64, u64) {
    if claimed.is_empty() {
        return (0, 0);
    }
    (claimed[0], claimed[claimed.len() - 1] + 1)
}

/// The insertion point `p` and the inserted text of an insertion result, or
/// `None` for every other edit. An insertion contributes no target coordinate
/// — it writes into a gap — so this is how the primer layer reaches the two
/// quantities its rules need. `MutationResult` gains no public accessor for
/// the pair: the match stays inside this module. `primer_design.rs` reads the
/// same pair through this reader, so one match states it for both layers.
pub(crate) fn insertion_claim(result: MutationResult) -> Option<(u64, String)> {
    match result {
        MutationResult::Insertion(record) => Some((record.point().position(), record.inserted())),
        MutationResult::Nucleotide(_) => None,
        MutationResult::Residue(_) => None,
        MutationResult::Deletion(_) => None,
        MutationResult::Replacement(_) => None,
    }
}

/// The replaced region `[s, e)` and the text written in its place, or `None` for
/// every other edit. D033: the primer layer needs the region's width and the
/// replacement's width to know how far the edited artifact's coordinates have
/// moved past the region's end. `MutationResult` gains no public accessor for
/// the triple: the match stays inside this module. `primer_design.rs` reads the
/// same triple through this reader, so one match states it for both layers.
pub(crate) fn replacement_claim(result: MutationResult) -> Option<(u64, u64, String)> {
    match result {
        MutationResult::Replacement(record) => {
            let region: NucleotideInterval = record.region();
            Some((region.start(), region.end(), record.replacement()))
        }
        MutationResult::Nucleotide(_) => None,
        MutationResult::Residue(_) => None,
        MutationResult::Deletion(_) => None,
        MutationResult::Insertion(_) => None,
    }
}

/// The width of a replacement's text, in nucleotides.
fn written_width(claim: (u64, u64, String)) -> u64 {
    claim.2.clone().chars().count() as u64
}

/// How far the edited artifact's coordinates have moved, as seen from a primer:
/// `m − w` for a replacement's forward primer, which reads the edited artifact,
/// and `0` for every other primer, which reads the template or reads the edited
/// artifact on a side where nothing moved.
fn replacement_shift(role: PrimerRole, claim: Option<(u64, u64, String)>) -> i64 {
    if claim.is_none() || role.is_reverse() {
        return 0;
    }
    let region: (u64, u64, String) = claim.clone().unwrap();
    signed(written_width(region.clone())) - signed(region.1 - region.0)
}

/// The reason a primer's text may differ in length from the span it names: a
/// replacement's forward primer reads the edited artifact, which is `m − w`
/// nucleotides longer or shorter than the template. `""` when nothing moved,
/// so every other edit keeps its message unchanged.
fn shift_reason(shift: i64) -> String {
    if shift == 0 {
        return "".to_string();
    }
    if shift < 0 {
        return join(vec![
            "; a replacement's forward primer reads the edited artifact, which is ".to_string(),
            number(negate(shift)),
            " nucleotides shorter than the template over this span".to_string(),
        ]);
    }
    join(vec![
        "; a replacement's forward primer reads the edited artifact, which is ".to_string(),
        number(shift as u64),
        " nucleotides longer than the template over this span".to_string(),
    ])
}

/// The primer indices a forward primer of a replacement carries the replacement
/// at, as a half-open range: the replacement occupies the edited interval
/// `[s, s + m)` and the primer reads the edited artifact over
/// `[span.start, span.end + m − w)`, so the changed indices are where the two
/// overlap. An empty range means the primer does not reach the region. With
/// `m = w` this is the substitution's changed run, which is why `changed_within`
/// needs no replacement case of its own.
fn replacement_junction(span: NucleotideInterval, claim: (u64, u64, String)) -> (i64, i64) {
    let region_start: i64 = signed(claim.0);
    let region_end: i64 = signed(claim.1);
    let written: i64 = signed(written_width(claim));
    let edited_start: i64 = signed(span.clone().start());
    let edited_end: i64 = signed(span.clone().end()) + written - (region_end - region_start);
    let lo: i64 = if edited_start > region_start {
        edited_start
    } else {
        region_start
    };
    let hi: i64 = if edited_end < region_start + written {
        edited_end
    } else {
        region_start + written
    };
    if hi <= lo {
        return (0, 0);
    }
    (lo - edited_start, hi - edited_start)
}

/// The changed primer indices of a forward primer of a replacement.
fn replacement_changed(span: NucleotideInterval, claim: (u64, u64, String)) -> Vec<u64> {
    let junction: (i64, i64) = replacement_junction(span.clone(), claim.clone());
    if junction.1 <= junction.0 {
        return Vec::new();
    }
    range_of(junction.0 as u64, junction.1 as u64)
}

/// D033: the forward primer of a replacement reads one interval of the **edited**
/// artifact: it starts where its template span starts, since everything before
/// the region's start is unchanged, and ends at `span.end + m − w`, since the
/// replacement moved the template by that much at the region's end. A reverse
/// primer states the template and has no edited interval (H3).
fn replacement_edited_span(
    role: PrimerRole,
    span: NucleotideInterval,
    claim: (u64, u64, String),
    edited_reference: SequenceReference,
) -> Result<Option<NucleotideInterval>, DomainError> {
    if role.clone().is_reverse() {
        return Ok(None);
    }
    let shift: i64 = replacement_shift(role.clone(), Some(claim.clone()));
    let interval: NucleotideInterval = create(
        &edited_reference,
        signed(span.clone().start()),
        signed(span.clone().end()) + shift,
    )?;
    Ok(Some(interval))
}

/// The template letters under a forward primer of a replacement, one per primer
/// index: primer index `i` reads the edited coordinate `span.start + i`, and that
/// coordinate came from the template at itself before the region, from inside the
/// region, or from `m − w` further along past it. A base the replacement added
/// has no template base underneath it, so it cannot match: the aligned template
/// carries a letter other than it there, and such an index is a changed one, so
/// it never reaches the annealing text.
fn replacement_template(
    before: Vec<char>,
    span: NucleotideInterval,
    claim: (u64, u64, String),
    letters: Vec<char>,
) -> Vec<char> {
    let region_start: i64 = signed(claim.0);
    let region_end: i64 = signed(claim.1);
    let written: i64 = signed(written_width(claim));
    let width: u64 = letters.len() as u64;
    let mut out: Vec<char> = Vec::new();
    let mut index: u64 = 0;
    while index < width {
        let at: i64 = signed(span.clone().start()) + signed(index);
        if at < region_start {
            out.push(before[(signed(span.clone().start()) + signed(index)) as usize]);
        } else if at < region_start + written {
            let offset: i64 = at - region_start;
            if offset < region_end - region_start {
                out.push(before[(region_start + offset) as usize]);
            } else {
                out.push(other_than(letters[index as usize]));
            }
        } else {
            out.push(before[(at - (written - (region_end - region_start))) as usize]);
        }
        index += 1;
    }
    out
}

/// A letter that is not `letter`: there is no template base under a base the
/// replacement added, so it cannot match the primer text there.
fn other_than(letter: char) -> char {
    if letter == 'A' {
        return 'C';
    }
    'A'
}

/// The runs of a forward primer of a replacement, each at the template interval
/// it really sits at: the context left of the region sits where it does in the
/// template, the mutation part covers the region's coordinates `[s, e)` — the
/// bases the replacement overwrote — and the context right of the region sits at
/// `e` onward, since the replacement moved the template by `m − w` there. `runs`
/// assumes one shift for the whole primer, which is what a deletion has and a
/// replacement does not.
fn replacement_runs(
    changed: Vec<u64>,
    width: u64,
    span: NucleotideInterval,
    claim: (u64, u64, String),
    reference: SequenceReference,
    letters: Vec<char>,
) -> Result<Vec<PrimerPart>, DomainError> {
    let region_start: i64 = signed(claim.0);
    let region_end: i64 = signed(claim.1);
    let written: i64 = signed(written_width(claim));
    let mut out: Vec<PrimerPart> = Vec::new();
    let mut start: u64 = 0;
    while start < width {
        let mutation: bool = is_changed(changed.clone(), start);
        let stop: u64 = run_end(changed.clone(), mutation, start, width);
        let bounds: (i64, i64) = if mutation {
            let clipped_start: i64 = if region_start > signed(span.clone().start()) {
                region_start
            } else {
                signed(span.clone().start())
            };
            let clipped_end: i64 = if region_end < signed(span.clone().end()) {
                region_end
            } else {
                signed(span.clone().end())
            };
            (clipped_start, clipped_end)
        } else if signed(span.clone().start()) + signed(start) < region_start {
            (
                signed(span.clone().start()) + signed(start),
                signed(span.clone().start()) + signed(stop),
            )
        } else {
            let back: i64 = (region_end - region_start) - written;
            (
                signed(span.clone().start()) + signed(start) + back,
                signed(span.clone().start()) + signed(stop) + back,
            )
        };
        let piece: NucleotideInterval = create(&reference, bounds.0, bounds.1)?;
        let indices: Vec<u64> = range_of(start, stop);
        let part = PrimerPart {
            role: if mutation {
                PrimerPartRole::Mutation
            } else {
                PrimerPartRole::Context
            },
            annealing: !mutation,
            sequence: canonical_text(pick(letters.clone(), indices.clone())),
            span: Some(piece.clone()),
            primer_start: start,
            primer_end: stop,
        };
        out.push(part);
        start = stop;
    }
    Ok(out)
}

/// The text a primer's 5′ tail carries, or `""` for a primer with no tail: a
/// short insertion gives one primer no tail at all, and it contributes nothing
/// to rebuilding the insertion.
fn tail_text_of(primer: Primer) -> String {
    let part: Option<PrimerPart> = primer.clone().tail();
    if part.is_none() {
        return "".to_string();
    }
    part.clone().unwrap().sequence()
}

/// The tail's text as it would read in the template's own direction. A reverse
/// primer states its tail 5′→3′ against the increasing coordinates, so the
/// bases it carries are the reverse complement of the run of inserted text
/// they stand for (§6).
fn tail_in_template_direction(primer: Primer) -> Result<String, DomainError> {
    let text: String = tail_text_of(primer.clone());
    let letters: Vec<char> = text.chars().collect();
    let back: Vec<char> = reverse_complement(letters.clone())?;
    Ok(canonical_text(back.clone()))
}

/// D032 and D033, one check: the two 5′ tails together are the whole written
/// run — the inserted text or the replacement — split between the primers. The
/// reverse primer states its tail 5'->3' on the opposite strand, so its
/// reverse complement is the first part of the run in the template's own
/// direction and the forward tail follows it; a primer with no tail
/// contributes an empty string. `kind` names the edit, `insertion` or
/// `replacement`, and is the only difference between the two pair checks this
/// helper serves: it appears twice in the reason, once naming the written run
/// and once naming what the tails must rebuild, so the reason strings stay as
/// each branch states them.
fn require_tails_rebuild(
    forward: &Primer,
    reverse: &Primer,
    written: &str,
    kind: &str,
) -> Result<(), DomainError> {
    let forward_tail: String = tail_text_of(forward.clone());
    let reverse_tail: String = tail_text_of(reverse.clone());
    let reverse_head: String = tail_in_template_direction(reverse.clone())?;
    let rebuilt: String = join(vec![reverse_head.clone(), forward_tail.clone()]);
    if rebuilt == written {
        return Ok(());
    }
    let problem = InvalidPrimerCandidateProblem {
        primer: "pair".to_string(),
        reason: join(vec![
            "the reverse primer's tail reads ".to_string(),
            reverse_tail,
            ", which stands for ".to_string(),
            reverse_head,
            " in the template's direction, and the forward primer's tail reads ".to_string(),
            forward_tail,
            ": together they give ".to_string(),
            rebuilt,
            " and the ".to_string(),
            kind.to_string(),
            " is ".to_string(),
            written.to_string(),
            ", so the two tails do not rebuild the ".to_string(),
            kind.to_string(),
        ]),
    };
    Err(DomainError::InvalidPrimerCandidate(problem))
}

/// D031: the forward primer of a deletion reads the **edited** artifact, whose
/// coordinates run `w` to the left of the template's past the deletion, so its
/// edited interval is the template span shifted left by the deleted width. A
/// primer wholly left of the deletion needs no shift. `construct_primer` has
/// already refused a span that overlaps the deleted region, so one of the two
/// holds; a reverse primer states its region against the template and has no
/// edited interval (H3).
fn deletion_edited_span(
    role: PrimerRole,
    span: NucleotideInterval,
    region_start: u64,
    region_end: u64,
    edited_reference: SequenceReference,
) -> Result<Option<NucleotideInterval>, DomainError> {
    if role.is_reverse() {
        return Ok(None);
    }
    let width: u64 = region_end - region_start;
    if span.clone().start() >= region_end {
        let interval: NucleotideInterval = create(
            &edited_reference,
            signed(span.clone().start() - width),
            signed(span.clone().end() - width),
        )?;
        return Ok(Some(interval));
    }
    if span.clone().end() <= region_start {
        let interval: NucleotideInterval = create(
            &edited_reference,
            signed(span.clone().start()),
            signed(span.clone().end()),
        )?;
        return Ok(Some(interval));
    }
    Ok(None)
}

fn signed(value: u64) -> i64 {
    value as i64
}

/// Absolute value of a signed separation, as an unsigned count.
fn negate(value: i64) -> u64 {
    if value < 0 {
        return (0 - value) as u64;
    }
    value as u64
}

fn number(value: u64) -> String {
    value.to_string()
}

fn describe(span: NucleotideInterval) -> String {
    join(vec![
        number(span.start()),
        ", ".to_string(),
        number(span.end()),
    ])
}

/// Concatenate text pieces in order. Rust has no `&str + String` chain, so
/// every multi-part message in this module is assembled here.
fn join(pieces: Vec<String>) -> String {
    let mut out: String = String::new();
    for piece in pieces {
        out += &piece;
    }
    out
}

fn position_text(positions: Vec<u64>) -> String {
    if positions.is_empty() {
        return "none".to_string();
    }
    let mut text: String = String::new();
    let total: usize = positions.len();
    let mut index: usize = 0;
    while index < total {
        if index > 0 {
            text += ", ";
        }
        text += &positions[index].to_string();
        index += 1;
    }
    text
}

fn slice(letters: Vec<char>, start: u64, end: u64) -> Vec<char> {
    let mut out: Vec<char> = Vec::new();
    let mut position: u64 = start;
    while position < end {
        out.push(letters[position as usize]);
        position += 1;
    }
    out
}

/// A reverse primer reads 5′→3′ against the template, so the region it covers
/// is seen as its reverse complement (§6, §5's reverse-primer rule).
fn oriented(letters: Vec<char>, reverse: bool) -> Result<Vec<char>, DomainError> {
    if reverse {
        return reverse_complement(letters);
    }
    Ok(letters)
}

fn pick(letters: Vec<char>, indices: Vec<u64>) -> Vec<char> {
    let mut out: Vec<char> = Vec::new();
    for index in indices {
        out.push(letters[index as usize]);
    }
    out
}

/// Template positions at which the two artifacts differ, ascending.
fn differing(before: Vec<char>, after: Vec<char>) -> Vec<u64> {
    let mut out: Vec<u64> = Vec::new();
    let total: usize = before.len();
    let mut index: usize = 0;
    while index < total {
        if before[index] != after[index] {
            out.push(index as u64);
        }
        index += 1;
    }
    out
}

/// The edit's changed template coordinates that this primer covers, as
/// indices into the primer's own text. For a reverse primer the mapping is
/// inverted (§6), and the result is returned in increasing primer-index order
/// either way, so the decomposition has one shape.
fn changed_within(span: NucleotideInterval, claimed: Vec<u64>, reverse: bool) -> Vec<u64> {
    let mut out: Vec<u64> = Vec::new();
    for position in claimed.clone() {
        if position >= span.start() && position < span.end() {
            if reverse {
                out.push(span.clone().end() - 1 - position);
            } else {
                out.push(position - span.clone().start());
            }
        }
    }
    if reverse {
        return reversed(out.clone());
    }
    out
}

/// Reverse a list. Used to put a reverse primer's changed indices, which come
/// out of the inverted mapping in decreasing order, back into increasing order.
fn reversed(values: Vec<u64>) -> Vec<u64> {
    let mut out: Vec<u64> = Vec::new();
    let total: usize = values.len();
    let mut index: usize = total;
    while index > 0 {
        index -= 1;
        out.push(values[index]);
    }
    out
}

/// The template interval a primer index range covers, given the primer's
/// orientation (§6). A forward primer's range [a, b) sits at span.start()+a;
/// a reverse primer's sits at span.end()-b, because its primer index 0 is the
/// template position one below its span's end.
fn template_range(span: NucleotideInterval, reverse: bool, start: u64, stop: u64) -> (u64, u64) {
    if reverse {
        return (span.clone().end() - stop, span.clone().end() - start);
    }
    (span.clone().start() + start, span.clone().start() + stop)
}

fn is_changed(changed: Vec<u64>, position: u64) -> bool {
    for candidate in changed {
        if candidate == position {
            return true;
        }
    }
    false
}

fn unchanged(changed: Vec<u64>, width: u64) -> Vec<u64> {
    let mut out: Vec<u64> = Vec::new();
    let mut position: u64 = 0;
    while position < width {
        if !is_changed(changed.clone(), position) {
            out.push(position);
        }
        position += 1;
    }
    out
}

/// Maximal runs of the primer, split where the mutation sits. Changed
/// positions need not be contiguous, so the run count is not capped at one
/// mutation part.
fn runs(
    changed: Vec<u64>,
    width: u64,
    span: NucleotideInterval,
    reverse: bool,
    reference: SequenceReference,
    letters: Vec<char>,
) -> Result<Vec<PrimerPart>, DomainError> {
    let mut out: Vec<PrimerPart> = Vec::new();
    let mut start: u64 = 0;
    while start < width {
        let mutation: bool = is_changed(changed.clone(), start);
        let stop: u64 = run_end(changed.clone(), mutation, start, width);
        let bounds: (u64, u64) = template_range(span.clone(), reverse, start, stop);
        let piece: NucleotideInterval = create(&reference, signed(bounds.0), signed(bounds.1))?;
        let indices: Vec<u64> = range_of(start, stop);
        let part = PrimerPart {
            role: if mutation {
                PrimerPartRole::Mutation
            } else {
                PrimerPartRole::Context
            },
            annealing: !mutation,
            sequence: canonical_text(pick(letters.clone(), indices.clone())),
            span: Some(piece.clone()),
            primer_start: start,
            primer_end: stop,
        };
        out.push(part);
        start = stop;
    }
    Ok(out)
}

fn run_end(changed: Vec<u64>, mutation: bool, start: u64, width: u64) -> u64 {
    let mut stop: u64 = start;
    while stop < width && is_changed(changed.clone(), stop) == mutation {
        stop += 1;
    }
    stop
}

fn range_of(start: u64, stop: u64) -> Vec<u64> {
    let mut out: Vec<u64> = Vec::new();
    let mut position: u64 = start;
    while position < stop {
        out.push(position);
        position += 1;
    }
    out
}

/// The template intervals the annealing parts cover (§3, I-1). One interval
/// per run, so a primer annealing on both sides of the mutation has two. A
/// part with no span is a 5′ tail: it covers no template coordinate, so it
/// contributes no interval here even before `annealing` excludes it.
fn annealing_of(parts: Vec<PrimerPart>) -> OrderedIntervals {
    let mut out: OrderedIntervals = Vec::new();
    for part in parts {
        let span: Option<NucleotideInterval> = part.clone().span();
        if part.clone().annealing() && span.is_some() {
            out.push(span.clone().unwrap());
        }
    }
    out
}

/// H8 / I-13: the parts must partition the primer text, and the mutation parts
/// must be exactly the changed positions. Returns `""` when they do, and the
/// reason when they do not.
fn partition_fault(parts: Vec<PrimerPart>, changed: Vec<u64>, width: u64) -> String {
    let mut covered: u64 = 0;
    for part in parts.clone() {
        let indices: (u64, u64) = part.primer_indices();
        if indices.0 != covered {
            return join(vec![
                "the parts are not contiguous at primer index ".to_string(),
                number(covered),
            ]);
        }
        if indices.1 <= indices.0 {
            return "a part is empty".to_string();
        }
        covered = indices.1;
    }
    if covered != width {
        return join(vec![
            "the parts stop at primer index ".to_string(),
            number(covered),
            " and the primer is ".to_string(),
            number(width),
        ]);
    }
    let mut mutation_indices: Vec<u64> = Vec::new();
    for part in parts {
        if part.clone().role().carries_mutation() {
            let indices: (u64, u64) = part.clone().primer_indices();
            mutation_indices.extend(range_of(indices.0, indices.1));
        }
    }
    if mutation_indices != changed {
        return join(vec![
            "the mutation parts cover ".to_string(),
            position_text(mutation_indices),
            " and the changed positions are ".to_string(),
            position_text(changed),
        ]);
    }
    String::new()
}

/// Positions of `inner` that lie outside `outer`. §2.1's containment check.
fn outside(inner: Vec<u64>, outer: Vec<u64>) -> Vec<u64> {
    let mut out: Vec<u64> = Vec::new();
    for value in inner.clone() {
        if !is_changed(outer.clone(), value) {
            out.push(value);
        }
    }
    out
}

fn covers(span: NucleotideInterval, position: u64) -> bool {
    position >= span.start() && position < span.end()
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
    use crate::identity::Identifier;
    use crate::interval::create;
    use crate::interval::create_point;
    use crate::interval::OrderedIntervals;
    use crate::mutation::apply_deletion;
    use crate::mutation::apply_insertion;
    use crate::mutation::apply_nucleotide;
    use crate::mutation::apply_replacement;
    use crate::mutation::apply_residue;
    use crate::mutation::CodonSelectionPolicy;
    use crate::mutation::MutationResult;
    use crate::mutation::NucleotideDeletionIntent;
    use crate::mutation::NucleotideInsertionIntent;
    use crate::mutation::NucleotideMutationIntent;
    use crate::mutation::NucleotideReplacementIntent;
    use crate::mutation::ResidueMutationIntent;
    use crate::sequence::normalize;
    use crate::sequence::Sequence;
    use crate::sequence::SequenceTopology;
    use crate::strand::Strand;

    // "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG" is ten whole codons over a 30-nucleotide
    // CDS: ATG GCT TGG AAT CAG CAG CAG CAG CAG CAG. Residue 4 is AAT, which the
    // standard genetic code reads as N, and replacing it with M writes ATG. The
    // codon's first DNA position is unchanged and its last two are not, so the
    // edit claims coordinates 9, 10 and 11 while differing only at 10 and 11.
    // That shape is what separates the contract's annealing overlap from the
    // whole-primer match count, and §13.1's SID-1 pair has the same shape.
    fn seq_a() -> Sequence {
        normalize(
            Identifier::create("seq-a".to_string()).unwrap(),
            "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    // Same content, different name: the identity check must catch a primer built
    // against a differently named artifact (H7, I-6).
    fn seq_b() -> Sequence {
        normalize(
            Identifier::create("seq-b".to_string()).unwrap(),
            "ATGGCTTGGAATCAGCAGCAGCAGCAGCAGCAG".to_string(),
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

    // The one edit every test shares: residue 4, N -> M, DNA AAT -> ATG at
    // reference coordinates 9, 10, 11. The edited artifact is
    // ATGGCTTGGATGCAGCAGCAGCAGCAGCAG.
    fn edit() -> MutationResult {
        let intent: ResidueMutationIntent = ResidueMutationIntent::create(
            residue(4),
            "N".to_string(),
            "M".to_string(),
            CodonSelectionPolicy::Default,
        )
        .unwrap();
        apply_residue(&seq_a().clone(), cds_of(30), result_name(), intent).unwrap()
    }

    fn forward_primer(start: i64, end: i64, text: String) -> Primer {
        construct_primer(
            &seq_a().clone(),
            edit().clone(),
            PrimerRole::Forward,
            PrimerPlacement::Inline,
            text.clone(),
            at(start, end),
        )
        .unwrap()
    }

    fn reverse_primer(start: i64, end: i64, text: String) -> Primer {
        construct_primer(
            &seq_a().clone(),
            edit().clone(),
            PrimerRole::Reverse,
            PrimerPlacement::Inline,
            text.clone(),
            at(start, end),
        )
        .unwrap()
    }

    fn pair_of(forward: Primer, reverse: Primer) -> PrimerPair {
        assemble_pair(forward.clone(), reverse.clone(), edit().clone()).unwrap()
    }

    #[test]
    fn an_inline_forward_primer_reports_every_derived_quantity() {
        let primer: Primer = forward_primer(0, 20, "ATGGCTTGGATGCAGCAGCA".to_string());
        assert!(primer.clone().length() == 20);
        assert!(primer.clone().sequence() == "ATGGCTTGGATGCAGCAGCA");
        assert!(primer.clone().changed_positions() == vec![9, 10, 11]);
        assert!(primer.clone().parts().len() == 3);
        assert!(primer.clone().annealing_spans().len() == 2);
        assert!(primer.clone().annealing_sequence() == "ATGGCTTGGCAGCAGCA");
        assert!(primer.clone().annealing_length() == 17);
        assert!(primer.clone().orientation() == PrimerOrientation::Same);
        assert!(primer.clone().edited_span().is_some());
        assert!(primer.clone().tail().is_none());
        assert!(primer.clone().one_based_span() == (1, 20));
    }

    #[test]
    fn a_changed_position_whose_letter_did_not_change_is_not_annealing() {
        let primer: Primer = forward_primer(0, 20, "ATGGCTTGGATGCAGCAGCA".to_string());
        assert!(primer.clone().annealing_overlap() == 17);
        assert!(primer.clone().whole_primer_match_count() == 18);
        assert!(primer.clone().annealing_overlap() < primer.clone().whole_primer_match_count());
    }

    #[test]
    fn the_overlap_positions_are_reported_alongside_the_count() {
        let primer: Primer = forward_primer(0, 20, "ATGGCTTGGATGCAGCAGCA".to_string());
        let positions: Vec<u64> = primer.clone().overlap_positions();
        assert!(positions.len() as u64 == primer.clone().annealing_overlap());
        assert!(positions == vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 13, 14, 15, 16, 17, 18, 19]);
    }

    #[test]
    fn the_parts_of_a_constructed_primer_partition_it() {
        let primer: Primer = forward_primer(0, 20, "ATGGCTTGGATGCAGCAGCA".to_string());
        assert!(partition_fault(
            primer.clone().parts(),
            primer.clone().changed_positions(),
            primer.clone().length()
        )
        .is_empty());
    }

    #[test]
    fn a_reverse_primer_states_the_reverse_complement_of_the_region_it_covers() {
        let primer: Primer = reverse_primer(12, 30, "CTGCTGCTGCTGCTGCTG".to_string());
        assert!(primer.clone().orientation() == PrimerOrientation::Opposite);
        assert!(primer.clone().edited_span().is_none());
        assert!(primer.clone().one_based_span() == (13, 30));
        assert!(primer.clone().changed_positions().is_empty());
        assert!(primer.clone().annealing_overlap() == 18);
        assert!(primer.clone().whole_primer_match_count() == 18);
        assert!(!primer.clone().tail_reason().is_empty());
    }

    #[test]
    fn an_abutting_pair_reports_where_its_five_prime_ends_meet() {
        let pair: PrimerPair = pair_of(
            forward_primer(12, 30, "CAGCAGCAGCAGCAGCAG".to_string()),
            reverse_primer(0, 12, "CATCCAAGCCAT".to_string()),
        );
        assert!(pair.clone().abuts());
        assert!(pair.clone().separation() == 0);
        assert!(pair.clone().boundary() == Some(12));
        assert!(
            pair.clone().relationship()
                == "reverse span [0, 12) and forward span [12, 30): abutting"
        );
        assert!(pair
            .clone()
            .forward()
            .orientation()
            .opposite_of(pair.clone().reverse().orientation()));
    }

    #[test]
    fn an_overlapping_pair_is_reported_rather_than_repaired() {
        let pair: PrimerPair = pair_of(
            forward_primer(0, 20, "ATGGCTTGGATGCAGCAGCA".to_string()),
            reverse_primer(12, 30, "CTGCTGCTGCTGCTGCTG".to_string()),
        );
        assert!(pair.clone().separation() == -8);
        assert!(!pair.clone().abuts());
        assert!(pair.clone().boundary().is_none());
        assert!(pair.clone().relationship() == "reverse span [12, 30) and forward span [0, 20): overlapping by 8 template positions", "actual=[{}]", pair.clone().relationship());
    }

    #[test]
    fn a_pair_that_leaves_a_changed_position_uncovered_is_rejected() {
        let forward: Primer = forward_primer(20, 30, "GCAGCAGCAG".to_string());
        let reverse: Primer = reverse_primer(0, 9, "CCAAGCCAT".to_string());
        let error: DomainError =
            assemble_pair(forward.clone(), reverse.clone(), edit().clone()).unwrap_err();
        assert!(matches!(error, DomainError::PrimerPairCoverage(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn a_slot_holding_the_wrong_role_is_rejected() {
        let first: Primer = forward_primer(0, 20, "ATGGCTTGGATGCAGCAGCA".to_string());
        let second: Primer = forward_primer(12, 30, "CAGCAGCAGCAGCAGCAG".to_string());
        let error: DomainError =
            assemble_pair(first.clone(), second.clone(), edit().clone()).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    #[test]
    fn h1_accepts_a_primer_exactly_at_the_bound() {
        let pair: PrimerPair = pair_of(
            forward_primer(0, 20, "ATGGCTTGGATGCAGCAGCA".to_string()),
            reverse_primer(12, 30, "CTGCTGCTGCTGCTGCTG".to_string()),
        );
        assert!(require_design_constraints(pair.clone(), 17).is_ok());
    }

    #[test]
    fn h1_rejects_a_primer_below_the_bound() {
        let pair: PrimerPair = pair_of(
            forward_primer(0, 20, "ATGGCTTGGATGCAGCAGCA".to_string()),
            reverse_primer(12, 30, "CTGCTGCTGCTGCTGCTG".to_string()),
        );
        let error: DomainError = require_design_constraints(pair.clone(), 18).unwrap_err();
        assert!(matches!(error, DomainError::AnnealingOverlapBelowBound(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn h1_rejects_a_bound_of_zero() {
        let pair: PrimerPair = pair_of(
            forward_primer(12, 30, "CAGCAGCAGCAGCAGCAG".to_string()),
            reverse_primer(0, 12, "CATCCAAGCCAT".to_string()),
        );
        let error: DomainError = require_design_constraints(pair.clone(), 0).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    #[test]
    fn confine_to_tail_is_refused_rather_than_defaulted() {
        let error: DomainError = construct_primer(
            &seq_a().clone(),
            edit().clone(),
            PrimerRole::Forward,
            PrimerPlacement::ConfineToTail,
            "ATGGCTTGGATGCAGCAGCA".to_string(),
            at(0, 20),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::UnsupportedPrimerPlacement(_)));
        assert!(error.category() == ErrorCategory::Unsupported);
    }

    #[test]
    fn primer_text_taken_from_the_template_instead_of_the_edit_is_rejected() {
        let error: DomainError = construct_primer(
            &seq_a().clone(),
            edit().clone(),
            PrimerRole::Forward,
            PrimerPlacement::Inline,
            "ATGGCTTGGAATCAGCAGCA".to_string(),
            at(0, 20),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::PrimerTextAtSpanMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn a_primer_built_against_another_artifact_is_rejected() {
        let span: NucleotideInterval = create(&seq_b().reference(), 0, 20).unwrap();
        let error: DomainError = construct_primer(
            &seq_a().clone(),
            edit().clone(),
            PrimerRole::Forward,
            PrimerPlacement::Inline,
            "ATGGCTTGGATGCAGCAGCA".to_string(),
            span.clone(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    // §2.1, containment rather than equality: a residue edit claims its whole
    // codon, and the SID-1 shape writes a letter that the original already had.
    // A tampered result is out of reach here, since the mutation layer's record
    // fields are private, and mutation.rs's own tests cover that path.
    #[test]
    fn a_claim_wider_than_the_difference_it_wrote_is_accepted_and_a_stray_is_not() {
        assert!(outside(vec![10, 11], vec![9, 10, 11]).is_empty());
        assert!(outside(vec![12], vec![9, 10, 11]) == vec![12]);
        assert!(outside(vec![], vec![9, 10, 11]).is_empty());
    }

    #[test]
    fn the_decomposition_takes_a_set_of_changed_positions() {
        let changed: Vec<u64> = vec![2, 3, 7, 8];
        let parts: Vec<PrimerPart> = runs(
            changed.clone(),
            12,
            at(0, 12),
            false,
            seq_a().reference(),
            seq_a().letters(),
        )
        .unwrap();
        assert!(parts.len() == 5);
        assert!(annealing_of(parts.clone()).len() == 3);
        assert!(unchanged(changed.clone(), 12).len() == 8);
        assert!(partition_fault(parts.clone(), changed.clone(), 12).is_empty());
    }

    // The initial native scope of §18 is one inline nucleotide substitution
    // (L = 1). The tests above all use a three-base residue edit, so these pin
    // the same layer on a single changed position. The edit is coordinate 15
    // of seq-a, C -> T: ATGGCTTGGAATCAG[C]AGCAGCAGCAGCAG becomes
    // ATGGCTTGGAATCAGTAGCAGCAGCAGCAG. Expected texts were written out by hand
    // from that string: a forward primer reads the edited text, a reverse
    // primer the reverse complement of the region it covers.
    fn nucleotide_edit() -> MutationResult {
        let intent: NucleotideMutationIntent = NucleotideMutationIntent::create(
            seq_a().reference(),
            at(15, 16),
            "C".to_string(),
            "T".to_string(),
        )
        .unwrap();
        apply_nucleotide(&seq_a().clone(), None, result_name(), intent).unwrap()
    }

    fn nucleotide_primer(role: PrimerRole, start: i64, end: i64, text: &str) -> Primer {
        construct_primer(
            &seq_a().clone(),
            nucleotide_edit(),
            role,
            PrimerPlacement::Inline,
            text.to_string(),
            at(start, end),
        )
        .unwrap()
    }

    fn spans_of(spans: OrderedIntervals) -> Vec<(u64, u64)> {
        spans
            .iter()
            .map(|span| (span.start(), span.end()))
            .collect()
    }

    // [12,19) of the edited text is CAG T AGC: three unchanged bases, the
    // substituted base, three unchanged bases.
    #[test]
    fn a_single_nucleotide_substitution_is_one_mutation_part_between_two_flanks() {
        let primer: Primer = nucleotide_primer(PrimerRole::Forward, 12, 19, "CAGTAGC");
        assert!(primer.clone().length() == 7);
        assert!(primer.clone().one_based_span() == (13, 19));
        assert!(primer.clone().changed_positions() == vec![3]);
        assert!(primer.clone().parts().len() == 3);
        assert!(spans_of(primer.clone().annealing_spans()) == vec![(12, 15), (16, 19)]);
        assert!(primer.clone().annealing_sequence() == "CAGAGC");
        assert!(primer.clone().annealing_overlap() == 6);
    }

    // The residue edit above claims a position whose letter did not change, so
    // its annealing overlap is below its whole-primer match count. A single
    // substitution changes the letter it claims, so the two counts agree.
    #[test]
    fn a_single_substituted_letter_makes_annealing_and_whole_primer_counts_agree() {
        let primer: Primer = nucleotide_primer(PrimerRole::Forward, 12, 19, "CAGTAGC");
        assert!(primer.clone().whole_primer_match_count() == 6);
        assert!(primer.clone().annealing_overlap() == primer.clone().whole_primer_match_count());
    }

    // [9,16) ends on the substituted base: one flank, then the mutation part.
    #[test]
    fn a_substitution_at_the_three_prime_end_leaves_one_annealing_flank() {
        let primer: Primer = nucleotide_primer(PrimerRole::Forward, 9, 16, "AATCAGT");
        assert!(primer.clone().changed_positions() == vec![6]);
        assert!(primer.clone().parts().len() == 2);
        assert!(spans_of(primer.clone().annealing_spans()) == vec![(9, 15)]);
        assert!(primer.clone().annealing_sequence() == "AATCAG");
        assert!(primer.clone().annealing_overlap() == 6);
    }

    // A reverse primer over [12,19) states rc(CAGTAGC) = GCTACTG. Its index 0
    // is template position 18, so the substituted position 15 is index 3.
    #[test]
    fn a_reverse_primer_over_a_single_substitution_maps_it_from_the_three_prime_end() {
        let primer: Primer = nucleotide_primer(PrimerRole::Reverse, 12, 19, "GCTACTG");
        assert!(primer.clone().orientation() == PrimerOrientation::Opposite);
        assert!(primer.clone().changed_positions() == vec![3]);
        assert!(primer.clone().parts().len() == 3);
        assert!(primer.clone().annealing_sequence() == "GCTCTG");
        assert!(primer.clone().annealing_overlap() == 6);
    }

    // Forward [12,19) with six annealing bases, and a reverse primer of six
    // bases over [6,12), rc(TGGAAT) = ATTCCA, meeting it at 12.
    #[test]
    fn a_single_nucleotide_pair_abuts_and_meets_h1_exactly_at_the_bound() {
        let forward: Primer = nucleotide_primer(PrimerRole::Forward, 12, 19, "CAGTAGC");
        let reverse: Primer = nucleotide_primer(PrimerRole::Reverse, 6, 12, "ATTCCA");
        let pair: PrimerPair = assemble_pair(forward, reverse, nucleotide_edit()).unwrap();
        assert!(pair.clone().abuts());
        assert!(pair.clone().boundary() == Some(12));
        assert!(pair.clone().separation() == 0);
        assert!(require_design_constraints(pair.clone(), 6).is_ok());
        let error: DomainError = require_design_constraints(pair.clone(), 7).unwrap_err();
        assert!(matches!(error, DomainError::AnnealingOverlapBelowBound(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    // The wild-type text at [12,19) is CAGCAGC: a primer that keeps the
    // original base does not carry the edit.
    #[test]
    fn a_single_nucleotide_primer_that_keeps_the_original_base_is_rejected() {
        let built = construct_primer(
            &seq_a().clone(),
            nucleotide_edit(),
            PrimerRole::Forward,
            PrimerPlacement::Inline,
            "CAGCAGC".to_string(),
            at(12, 19),
        );
        assert!(matches!(
            built.unwrap_err(),
            DomainError::PrimerTextAtSpanMismatch(_)
        ));
    }

    // Milestone 4b-1: the deletion layer. The one deletion every test below
    // shares removes [9, 12) of seq-a, which holds AAT; the edited artifact is
    // ATGGCTTGG + CAGCAGCAGCAGCAGCAG, 27 nucleotides, and the deleted width is
    // 3. Every expected text below was written out by hand from those two
    // strings, not read back from the code.
    fn deletion() -> MutationResult {
        let intent: NucleotideDeletionIntent =
            NucleotideDeletionIntent::create(seq_a().reference(), at(9, 12), "AAT".to_string())
                .unwrap();
        apply_deletion(&seq_a().clone(), None, result_name(), intent).unwrap()
    }

    fn deletion_forward_primer(start: i64, end: i64, text: String) -> Primer {
        construct_primer(
            &seq_a().clone(),
            deletion().clone(),
            PrimerRole::Forward,
            PrimerPlacement::Inline,
            text.clone(),
            at(start, end),
        )
        .unwrap()
    }

    fn deletion_reverse_primer(start: i64, end: i64, text: String) -> Primer {
        construct_primer(
            &seq_a().clone(),
            deletion().clone(),
            PrimerRole::Reverse,
            PrimerPlacement::Inline,
            text.clone(),
            at(start, end),
        )
        .unwrap()
    }

    fn deletion_pair_of(forward: Primer, reverse: Primer) -> PrimerPair {
        assemble_pair(forward.clone(), reverse.clone(), deletion().clone()).unwrap()
    }

    // D031: the forward primer reads the template at [12, 22), CAGCAGCAGC, and
    // carries no changed position, so all ten positions anneal. Its text is the
    // edited artifact's [9, 19): the template span shifted left by w = 3.
    #[test]
    fn a_forward_primer_of_a_deletion_is_all_template() {
        let primer: Primer = deletion_forward_primer(12, 22, "CAGCAGCAGC".to_string());
        assert!(primer.clone().length() == 10);
        assert!(primer.clone().sequence() == "CAGCAGCAGC");
        assert!(primer.clone().changed_positions().is_empty());
        assert!(primer.clone().annealing_overlap() == 10);
        assert!(primer.clone().tail().is_none());
        let edited: NucleotideInterval = primer.clone().edited_span().unwrap();
        assert!(edited.start() == 9);
        assert!(edited.end() == 19);
        assert!(edited.reference() == deletion().reference());
    }

    // rc(ATGGCTTGG) = CCAAGCCAT: the reverse primer states the template region
    // [0, 9) against the template, so it has no edited interval.
    #[test]
    fn a_reverse_primer_of_a_deletion_is_stated_against_the_template() {
        let primer: Primer = deletion_reverse_primer(0, 9, "CCAAGCCAT".to_string());
        assert!(primer.clone().sequence() == "CCAAGCCAT");
        assert!(primer.clone().orientation() == PrimerOrientation::Opposite);
        assert!(primer.clone().edited_span().is_none());
        assert!(primer.clone().annealing_overlap() == 9);
    }

    // [11, 21) reaches the deleted base at 11, so the primer would carry a
    // removed letter. The text TCAGCAGCAG is the template there, so only the
    // deletion rule refuses it.
    #[test]
    fn a_primer_may_not_cover_a_deleted_base() {
        let error: DomainError = construct_primer(
            &seq_a().clone(),
            deletion().clone(),
            PrimerRole::Forward,
            PrimerPlacement::Inline,
            "TCAGCAGCAG".to_string(),
            at(11, 21),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        let reason: String = match error {
            DomainError::InvalidPrimerCandidate(problem) => problem.reason.clone(),
            _ => "".to_string(),
        };
        assert!(reason.contains("deleted"));
        assert!(reason.contains("the deleted region [9, 12)"));
    }

    // The last base of the template at [12, 22) is C, not A: a primer text that
    // is not the template at its span is refused, whatever the edit.
    #[test]
    fn the_text_of_a_deletion_primer_must_be_the_template() {
        let error: DomainError = construct_primer(
            &seq_a().clone(),
            deletion().clone(),
            PrimerRole::Forward,
            PrimerPlacement::Inline,
            "CAGCAGCAGA".to_string(),
            at(12, 22),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::PrimerTextAtSpanMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    // D031's pair invariant: reverse end 9 + w 3 = forward start 12, so the
    // pair abuts across the deletion and its 5' ends meet at 9 in the product.
    #[test]
    fn a_deletion_pair_abuts_across_the_deletion() {
        let pair: PrimerPair = deletion_pair_of(
            deletion_forward_primer(12, 22, "CAGCAGCAGC".to_string()),
            deletion_reverse_primer(0, 9, "CCAAGCCAT".to_string()),
        );
        assert!(pair.clone().deleted_width() == 3);
        assert!(pair.clone().separation() == 3);
        assert!(pair.clone().abuts());
        assert!(pair.clone().boundary() == Some(9));
        assert!(pair.clone().relationship().contains("deletion of 3"));
    }

    // AGCAGCAGCA is the template at [13, 23), so the primer is well formed; it
    // is refused because it starts one position right of the deletion's edge.
    #[test]
    fn a_forward_primer_away_from_the_deletion_edge_is_refused() {
        let forward: Primer = construct_primer(
            &seq_a().clone(),
            deletion().clone(),
            PrimerRole::Forward,
            PrimerPlacement::Inline,
            "AGCAGCAGCA".to_string(),
            at(13, 23),
        )
        .unwrap();
        let reverse: Primer = deletion_reverse_primer(0, 9, "CCAAGCCAT".to_string());
        let error: DomainError =
            assemble_pair(forward.clone(), reverse.clone(), deletion().clone()).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        let reason: String = match error {
            DomainError::InvalidPrimerCandidate(problem) => problem.reason.clone(),
            _ => "".to_string(),
        };
        assert!(reason.contains("the deletion is [9, 12)"));
    }

    // rc(ATGGCTTG) = CAAGCCAT: a well-formed reverse primer that stops one
    // position before the deletion's edge, so the invariant fails.
    #[test]
    fn a_reverse_primer_away_from_the_deletion_edge_is_refused() {
        let forward: Primer = deletion_forward_primer(12, 22, "CAGCAGCAGC".to_string());
        let reverse: Primer = deletion_reverse_primer(0, 8, "CAAGCCAT".to_string());
        let error: DomainError =
            assemble_pair(forward.clone(), reverse.clone(), deletion().clone()).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        let reason: String = match error {
            DomainError::InvalidPrimerCandidate(problem) => problem.reason.clone(),
            _ => "".to_string(),
        };
        assert!(reason.contains("the deletion is [9, 12)"));
    }

    // A substitution writes rather than removes, so its pair's width is 0 and
    // D031's invariant is the template abutment of §15 question 3.
    #[test]
    fn a_substitution_pair_has_no_deleted_width() {
        let pair: PrimerPair = pair_of(
            forward_primer(12, 30, "CAGCAGCAGCAGCAGCAG".to_string()),
            reverse_primer(0, 12, "CATCCAAGCCAT".to_string()),
        );
        assert!(pair.clone().deleted_width() == 0);
        assert!(pair.clone().abuts());
    }

    // D032: the insertion layer. The one insertion every test below shares puts
    // bases into the gap at point 12 of seq-a, which the flanking bases T (index
    // 11) and C (index 12) assert. It changes no template coordinate, so
    // `target_coordinates()` is empty and both primers read wild-type template:
    // the forward primer's annealing portion is template [12, 22) = CAGCAGCAGC,
    // and the reverse primer's is rc(template [2, 12)) = rc(GGCTTGGAAT) =
    // ATTCCAAGCC. Every expected text below was written out by hand from those
    // two slices, not read back from the code.
    fn insertion(position: i64, before: &str, after: &str, inserted: &str) -> MutationResult {
        let intent: NucleotideInsertionIntent = NucleotideInsertionIntent::create(
            seq_a().reference(),
            create_point(&seq_a().reference(), position).unwrap(),
            before.to_string(),
            after.to_string(),
            inserted.to_string(),
        )
        .unwrap();
        apply_insertion(&seq_a().clone(), None, result_name(), intent).unwrap()
    }

    fn tailed_forward(
        result: MutationResult,
        tail: &str,
        annealing: &str,
        start: i64,
        end: i64,
    ) -> Result<Primer, DomainError> {
        construct_tailed_primer(
            &seq_a().clone(),
            result.clone(),
            PrimerRole::Forward,
            tail.to_string(),
            annealing.to_string(),
            at(start, end),
        )
    }

    fn tailed_reverse(
        result: MutationResult,
        tail: &str,
        annealing: &str,
        start: i64,
        end: i64,
    ) -> Result<Primer, DomainError> {
        construct_tailed_primer(
            &seq_a().clone(),
            result.clone(),
            PrimerRole::Reverse,
            tail.to_string(),
            annealing.to_string(),
            at(start, end),
        )
    }

    // A `k <= 6` insertion gives the reverse primer no tail, so it is built by
    // `construct_primer` as an untailed primer of the insertion.
    fn untailed_reverse(
        result: MutationResult,
        text: &str,
        start: i64,
        end: i64,
    ) -> Result<Primer, DomainError> {
        construct_primer(
            &seq_a().clone(),
            result.clone(),
            PrimerRole::Reverse,
            PrimerPlacement::Inline,
            text.to_string(),
            at(start, end),
        )
    }

    // D032: an untailed primer covers template only, so it may not read straight
    // across the gap. [8, 18) holds the insertion point at 12, and GAATCAGCAG is
    // the template there, so only the straddle rule refuses it.
    #[test]
    fn an_untailed_primer_may_not_read_across_the_insertion_point() {
        let result: MutationResult = insertion(12, "T", "C", "GGG");
        let built: Result<Primer, DomainError> = construct_primer(
            &seq_a().clone(),
            result.clone(),
            PrimerRole::Forward,
            PrimerPlacement::Inline,
            "GAATCAGCAG".to_string(),
            at(8, 18),
        );
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        let reason: String = match error {
            DomainError::InvalidPrimerCandidate(problem) => problem.reason.clone(),
            _ => "".to_string(),
        };
        assert!(reason.contains("insertion point"));
    }

    #[test]
    fn a_tailed_forward_primer_reports_its_tail() {
        let result: MutationResult = insertion(12, "T", "C", "GGG");
        let primer: Primer = tailed_forward(result.clone(), "GGG", "CAGCAGCAGC", 12, 22).unwrap();
        assert!(primer.clone().sequence() == "GGGCAGCAGCAGC");
        assert!(primer.clone().length() == 13);
        assert!(primer.clone().annealing_overlap() == 10);
        assert!(primer.clone().annealing_length() == 10);
        assert!(primer.clone().changed_positions().is_empty());
        assert!(primer.clone().edited_span().is_none());
        let tail: PrimerPart = primer.clone().tail().unwrap();
        assert!(tail.clone().role() == PrimerPartRole::Tail);
        assert!(tail.clone().span().is_none());
        assert!(tail.clone().primer_indices() == (0, 3));
        assert!(primer.clone().parts().len() == 2);
        assert!(primer.clone().tail_reason().contains("does not anneal"));
    }

    #[test]
    fn the_annealing_portion_of_a_tailed_primer_must_be_the_template() {
        let result: MutationResult = insertion(12, "T", "C", "GGG");
        let error: DomainError =
            tailed_forward(result.clone(), "GGG", "CAGCAGCAGA", 12, 22).unwrap_err();
        assert!(matches!(error, DomainError::PrimerTextAtSpanMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn the_annealing_length_of_a_tailed_primer_must_match_its_span() {
        let result: MutationResult = insertion(12, "T", "C", "GGG");
        let error: DomainError =
            tailed_forward(result.clone(), "GGG", "CAGCAGCAG", 12, 22).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    #[test]
    fn a_tail_is_refused_for_a_substitution() {
        let error: DomainError = tailed_forward(edit(), "GGG", "CAGCAGCAGC", 12, 22).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        let reason: String = match error {
            DomainError::InvalidPrimerCandidate(problem) => problem.reason.clone(),
            _ => "".to_string(),
        };
        assert!(reason.contains("5′ tail"));
    }

    #[test]
    fn the_parts_of_a_tailed_primer_partition_it() {
        let result: MutationResult = insertion(12, "T", "C", "GGG");
        let primer: Primer = tailed_forward(result.clone(), "GGG", "CAGCAGCAGC", 12, 22).unwrap();
        assert!(partition_fault(
            primer.clone().parts(),
            primer.clone().changed_positions(),
            primer.clone().length()
        )
        .is_empty());
    }

    #[test]
    fn a_short_insertion_pairs_with_an_untailed_reverse_primer() {
        let result: MutationResult = insertion(12, "T", "C", "GGG");
        let forward: Primer = tailed_forward(result.clone(), "GGG", "CAGCAGCAGC", 12, 22).unwrap();
        let reverse: Primer = untailed_reverse(result.clone(), "ATTCCAAGCC", 2, 12).unwrap();
        let pair: PrimerPair =
            assemble_pair(forward.clone(), reverse.clone(), result.clone()).unwrap();
        assert!(pair.clone().abuts());
        assert!(pair.clone().boundary() == Some(12));
        assert!(pair.clone().deleted_width() == 0);
    }

    // Seven bases at point 12, so the forward tail carries the last ceil(7 / 2)
    // = 4 (TACA) and the reverse tail the reverse complement of the first floor
    // (7 / 2) = 3: rc(GAT) = ATC.
    #[test]
    fn a_split_insertion_needs_both_tails_to_rebuild_it() {
        let result: MutationResult = insertion(12, "T", "C", "GATTACA");
        let forward: Primer = tailed_forward(result.clone(), "TACA", "CAGCAGCAGC", 12, 22).unwrap();
        let reverse: Primer = tailed_reverse(result.clone(), "ATC", "ATTCCAAGCC", 2, 12).unwrap();
        let pair: PrimerPair =
            assemble_pair(forward.clone(), reverse.clone(), result.clone()).unwrap();
        assert!(pair.clone().abuts());
    }

    #[test]
    fn tails_that_do_not_rebuild_the_insertion_are_refused() {
        let result: MutationResult = insertion(12, "T", "C", "GATTACA");
        let forward: Primer = tailed_forward(result.clone(), "TACG", "CAGCAGCAGC", 12, 22).unwrap();
        let reverse: Primer = tailed_reverse(result.clone(), "ATC", "ATTCCAAGCC", 2, 12).unwrap();
        let error: DomainError =
            assemble_pair(forward.clone(), reverse.clone(), result.clone()).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        let reason: String = match error {
            DomainError::InvalidPrimerCandidate(problem) => problem.reason.clone(),
            _ => "".to_string(),
        };
        assert!(reason.contains("rebuild the insertion"));
    }

    // rc(template [1, 11)) = rc(TGGCTTGGAA) = TTCCAAGCCA: a well-formed reverse
    // primer that stops one position before the insertion point, so the pair is
    // refused even though each primer on its own is fine.
    #[test]
    fn an_annealing_portion_away_from_the_insertion_point_is_refused() {
        let result: MutationResult = insertion(12, "T", "C", "GGG");
        let forward: Primer = tailed_forward(result.clone(), "GGG", "CAGCAGCAGC", 12, 22).unwrap();
        let reverse: Primer = untailed_reverse(result.clone(), "TTCCAAGCCA", 1, 11).unwrap();
        let error: DomainError =
            assemble_pair(forward.clone(), reverse.clone(), result.clone()).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    // The forward primer is 13 nucleotides but anneals only 10, so a bound of
    // 11 is refused: the tail carries the insertion and anneals nowhere.
    #[test]
    fn a_tail_does_not_count_toward_the_annealing_bound() {
        let result: MutationResult = insertion(12, "T", "C", "GGG");
        let forward: Primer = tailed_forward(result.clone(), "GGG", "CAGCAGCAGC", 12, 22).unwrap();
        let reverse: Primer = untailed_reverse(result.clone(), "ATTCCAAGCC", 2, 12).unwrap();
        let pair: PrimerPair =
            assemble_pair(forward.clone(), reverse.clone(), result.clone()).unwrap();
        assert!(require_design_constraints(pair.clone(), 10).is_ok());
        let error: DomainError = require_design_constraints(pair.clone(), 11).unwrap_err();
        assert!(matches!(error, DomainError::AnnealingOverlapBelowBound(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    // D033: a replacement over [9, 12) of seq-a, which holds AAT there. `AAT`
    // -> `GG` gives the edited artifact ATGGCTTGG + GG + CAGCAGCAGCAGCAGCAG,
    // 29 nucleotides, so m - w = -1: a forward primer's text is one nucleotide
    // shorter than the template span it names, and its edited coordinates are
    // [span.start, span.end - 1). `AAT` -> `GATTACAG` gives an 8-nucleotide
    // artifact, so m - w = 5 and the junction is too wide to sit inline: the
    // forward tail is the last 4 replacement bases ACAG and the reverse tail is
    // the reverse complement of the first 4, rc(GATT) = AATC. Every expected
    // text below was written out by hand from those two strings, not read back
    // from the code.
    fn replacement(start: i64, end: i64, expected: &str, text: &str) -> MutationResult {
        let intent: NucleotideReplacementIntent = NucleotideReplacementIntent::create(
            seq_a().reference(),
            at(start, end),
            expected.to_string(),
            text.to_string(),
        )
        .unwrap();
        apply_replacement(&seq_a().clone(), None, result_name(), intent).unwrap()
    }

    fn replacement_forward_primer(
        start: i64,
        end: i64,
        text: String,
    ) -> Result<Primer, DomainError> {
        construct_primer(
            &seq_a().clone(),
            replacement(9, 12, "AAT", "GG"),
            PrimerRole::Forward,
            PrimerPlacement::Inline,
            text.clone(),
            at(start, end),
        )
    }

    fn replacement_reverse_primer(
        start: i64,
        end: i64,
        text: String,
    ) -> Result<Primer, DomainError> {
        construct_primer(
            &seq_a().clone(),
            replacement(9, 12, "AAT", "GG"),
            PrimerRole::Reverse,
            PrimerPlacement::Inline,
            text.clone(),
            at(start, end),
        )
    }

    // The forward primer names the template span [0, 20) but reads the edited
    // artifact over [0, 19): ATGGCTTGG, then the two bases the replacement wrote,
    // then the template from 12 on. The replacement wrote 2 bases, so primer
    // indices 9 and 10 carry it and its edited interval is [0, 19).
    #[test]
    fn a_replacement_forward_primer_takes_its_text_from_the_edited_artifact() {
        let primer: Primer =
            replacement_forward_primer(0, 20, "ATGGCTTGGGGCAGCAGCA".to_string()).unwrap();
        assert!(primer.clone().length() == 19);
        assert!(primer.clone().changed_positions() == vec![9, 10]);
        let edited: NucleotideInterval = primer.clone().edited_span().unwrap();
        assert!(edited.start() == 0);
        assert!(edited.end() == 19);
    }

    // The 19th nucleotide of the edited artifact at [0, 19) is A, not T: a
    // forward primer of a replacement is checked against the edited artifact,
    // so a text that is one base off there is refused.
    #[test]
    fn the_text_of_a_replacement_primer_must_match_the_edited_artifact() {
        let error: DomainError =
            replacement_forward_primer(0, 20, "ATGGCTTGGGGCAGCAGCT".to_string()).unwrap_err();
        assert!(matches!(error, DomainError::PrimerTextAtSpanMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    // The reverse primer states the template: rc(template [0, 9)) = rc(ATGGCTTGG)
    // = CCAAGCCAT, it carries no changed base, and it has no edited interval.
    #[test]
    fn a_replacement_reverse_primer_is_pure_template() {
        let primer: Primer = replacement_reverse_primer(0, 9, "CCAAGCCAT".to_string()).unwrap();
        assert!(primer.clone().changed_positions().is_empty());
        assert!(primer.clone().edited_span().is_none());
    }

    // [0, 10) reaches the replaced coordinate 9, so the reverse primer would
    // state a base the replacement overwrote as if it were still template.
    #[test]
    fn a_replacement_reverse_primer_may_not_cover_a_replaced_base() {
        let error: DomainError =
            replacement_reverse_primer(0, 10, "TCCAAGCCAT".to_string()).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        let reason: String = match error {
            DomainError::InvalidPrimerCandidate(problem) => problem.reason.clone(),
            _ => "".to_string(),
        };
        assert!(reason.contains("replaced"));
        assert!(reason.contains("the replaced region [9, 12)"));
    }

    // D033's tail path: the replacement is 8 nucleotides, so the junction is
    // wider than 6 and the bases go into the two tails. The forward primer
    // anneals to template [12, 22) = CAGCAGCAGC and its 5' tail carries the last
    // 4 replacement bases, ACAG.
    #[test]
    fn a_tailed_forward_primer_of_a_replacement_carries_its_half() {
        let result: MutationResult = replacement(9, 12, "AAT", "GATTACAG");
        let forward: Primer = tailed_forward(result.clone(), "ACAG", "CAGCAGCAGC", 12, 22).unwrap();
        assert!(forward.clone().sequence() == "ACAGCAGCAGCAGC");
        assert!(forward.clone().length() == 14);
        let tail: PrimerPart = forward.clone().tail().unwrap();
        assert!(tail.primer_indices() == (0, 4));
    }

    // rc(AATC) + ACAG = GATT + ACAG = GATTACAG, so the two tails are the whole
    // replacement, and the annealing portions sit at the two edges of the
    // replaced region: reverse end 9 + w 3 = forward start 12.
    #[test]
    fn a_replacement_pair_rebuilds_the_replacement_from_its_tails() {
        let result: MutationResult = replacement(9, 12, "AAT", "GATTACAG");
        let forward: Primer = tailed_forward(result.clone(), "ACAG", "CAGCAGCAGC", 12, 22).unwrap();
        let reverse: Primer = tailed_reverse(result.clone(), "AATC", "CCAAGCCAT", 0, 9).unwrap();
        let pair: PrimerPair =
            assemble_pair(forward.clone(), reverse.clone(), result.clone()).unwrap();
        assert!(pair.clone().deleted_width() == 3);
        assert!(pair.clone().abuts());
    }

    // The forward tail ACAC stands for a replacement of GATTACAC, not GATTACAG:
    // the two tails must rebuild the replacement the result wrote.
    #[test]
    fn tails_that_do_not_rebuild_the_replacement_are_refused() {
        let result: MutationResult = replacement(9, 12, "AAT", "GATTACAG");
        let forward: Primer = tailed_forward(result.clone(), "ACAC", "CAGCAGCAGC", 12, 22).unwrap();
        let reverse: Primer = tailed_reverse(result.clone(), "AATC", "CCAAGCCAT", 0, 9).unwrap();
        let error: DomainError =
            assemble_pair(forward.clone(), reverse.clone(), result.clone()).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        let reason: String = match error {
            DomainError::InvalidPrimerCandidate(problem) => problem.reason.clone(),
            _ => "".to_string(),
        };
        assert!(reason.contains("rebuild the replacement"));
    }

    // The reverse primer's annealing portion ends at 8, not at the region's
    // start 9, so a nucleotide of the replaced region belongs to neither primer.
    #[test]
    fn a_replacement_pair_must_sit_at_the_regions_edges() {
        let result: MutationResult = replacement(9, 12, "AAT", "GATTACAG");
        let forward: Primer = tailed_forward(result.clone(), "ACAG", "CAGCAGCAGC", 12, 22).unwrap();
        let reverse: Primer = tailed_reverse(result.clone(), "AATC", "CAAGCCAT", 0, 8).unwrap();
        let error: DomainError =
            assemble_pair(forward.clone(), reverse.clone(), result.clone()).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        let reason: String = match error {
            DomainError::InvalidPrimerCandidate(problem) => problem.reason.clone(),
            _ => "".to_string(),
        };
        assert!(reason.contains("the replacement is [9, 12)"));
    }

    // D033: a forward primer of a replacement reads the edited artifact over its
    // whole span, so its text length only follows from the region's width when the
    // span covers the region. [10, 20) starts inside [9, 12), so it would name only
    // the two replacement bases that lie at 10 and 11 and none of the replaced base
    // at 9: its 9-nucleotide text GCAGCAGCA, the edited [10, 19), is refused rather
    // than stated as a span that does not name it.
    #[test]
    fn a_replacement_forward_primer_must_cover_the_whole_region() {
        let error: DomainError =
            replacement_forward_primer(10, 20, "GCAGCAGCA".to_string()).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        let reason: String = match error {
            DomainError::InvalidPrimerCandidate(problem) => problem.reason.clone(),
            _ => "".to_string(),
        };
        assert!(reason.contains("covers the replaced region"));
    }

    // Review B1: a tailed primer's annealing portion sits beside the edit, in a
    // gap of the template, so its span may not be laid over the edit and carry a
    // 5' tail that rebuilds nothing. The §1 reproduction scaled to seq-a: the
    // region is [9, 12) and the forward primer anneals over [8, 18), which holds
    // it, with a tail of GGG that stands for none of the replacement's bases.
    #[test]
    fn a_tailed_replacement_primer_may_not_anneal_over_the_region() {
        let result: MutationResult = replacement(9, 12, "AAT", "AAAA");
        let error: DomainError =
            tailed_forward(result.clone(), "GGG", "GAATCAGCAG", 8, 18).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        let reason: String = match error {
            DomainError::InvalidPrimerCandidate(problem) => problem.reason.clone(),
            _ => "".to_string(),
        };
        assert!(reason.contains("replaced region"));
        assert!(reason.contains("the replacement is [9, 12)"));
    }

    // The same rule on the insertion side: a tailed primer's annealing portion
    // lies wholly on one side of the point, so a span that straddles it is
    // refused even though its text is wild-type template throughout.
    #[test]
    fn a_tailed_insertion_primer_may_not_straddle_the_point() {
        let result: MutationResult = insertion(12, "T", "C", "GGG");
        let error: DomainError =
            tailed_forward(result.clone(), "GGG", "GAATCAGCAG", 8, 18).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        let reason: String = match error {
            DomainError::InvalidPrimerCandidate(problem) => problem.reason.clone(),
            _ => "".to_string(),
        };
        assert!(reason.contains("straddle"));
    }

    // D033's tail branch is what decides such a pair, not where the forward span
    // happens to start: here the forward primer is inline and covers the region,
    // so H2's coverage check would pass, and only the reverse primer carries a
    // tail. Its span ends exactly at the region's start, which 3.1 allows, so the
    // pair is refused at the edge check: the forward primer does not start at 12.
    #[test]
    fn an_inline_replacement_pair_with_a_tailed_reverse_primer_is_refused() {
        let result: MutationResult = replacement(9, 12, "AAT", "GG");
        let forward: Primer =
            replacement_forward_primer(0, 20, "ATGGCTTGGGGCAGCAGCA".to_string()).unwrap();
        let reverse: Primer = tailed_reverse(result.clone(), "TGC", "CCAAGCCAT", 0, 9).unwrap();
        let error: DomainError =
            assemble_pair(forward.clone(), reverse.clone(), result.clone()).unwrap_err();
        assert!(matches!(error, DomainError::InvalidPrimerCandidate(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    // On the tail path the reverse primer may have no tail at all: a
    // replacement of one base gives floor(1 / 2) = 0 bases to the reverse tail
    // and ceil(1 / 2) = 1 to the forward one. The pair is still a tail pair, so
    // it is the primer's own tail that decides the branch, and it is accepted.
    // The region is [2, 12), ten template bases replaced by G, so the annealing
    // portions are template [12, 22) = CAGCAGCAGC and rc(template [0, 2)): the
    // template there is AT, whose reverse complement is AT itself, written out
    // here rather than read back from the code.
    #[test]
    fn a_tail_path_replacement_with_an_untailed_reverse_primer_is_accepted() {
        let result: MutationResult = replacement(2, 12, "GGCTTGGAAT", "G");
        let forward: Primer = tailed_forward(result.clone(), "G", "CAGCAGCAGC", 12, 22).unwrap();
        let reverse: Primer = untailed_reverse(result.clone(), "AT", 0, 2).unwrap();
        let pair: PrimerPair =
            assemble_pair(forward.clone(), reverse.clone(), result.clone()).unwrap();
        assert!(pair.clone().deleted_width() == 10);
        assert!(pair.clone().abuts());
        assert!(pair.clone().boundary() == Some(2));
    }

    // Review B5: the same pair, described in words. The region is [2, 12), so
    // the two annealing portions are 10 template positions apart and the pair
    // straddles a *replaced* region, not a deletion: nothing was removed, the
    // ten bases were rewritten as G. The sentence is therefore the replaced
    // region one, and it must not fall back to the deletion wording.
    #[test]
    fn a_tail_path_replacement_pair_states_a_replaced_region() {
        let result: MutationResult = replacement(2, 12, "GGCTTGGAAT", "G");
        let forward: Primer = tailed_forward(result.clone(), "G", "CAGCAGCAGC", 12, 22).unwrap();
        let reverse: Primer = untailed_reverse(result.clone(), "AT", 0, 2).unwrap();
        let pair: PrimerPair =
            assemble_pair(forward.clone(), reverse.clone(), result.clone()).unwrap();
        let statement: String = pair.clone().relationship();
        assert!(statement.contains("abutting across a replaced region of 10 template positions"));
        assert!(!statement.contains("deletion"));
    }

    // The tail of that forward primer carries one of the ten replacement bases,
    // not an insertion: the insertion wording is wrong for it.
    #[test]
    fn a_tailed_replacement_primer_says_its_tail_carries_the_replacement() {
        let result: MutationResult = replacement(2, 12, "GGCTTGGAAT", "G");
        let primer: Primer = tailed_forward(result.clone(), "G", "CAGCAGCAGC", 12, 22).unwrap();
        let reason: String = primer.clone().tail_reason();
        assert!(reason.contains("part of the replacement"));
        assert!(!reason.contains("insertion"));
    }

    // Regression on the insertion side: a tailed insertion primer keeps the
    // wording it has always had, and no existing test asserted this phrase.
    #[test]
    fn a_tailed_insertion_primer_says_its_tail_carries_the_insertion() {
        let result: MutationResult = insertion(12, "T", "C", "GGG");
        let primer: Primer = tailed_forward(result.clone(), "GGG", "CAGCAGCAGC", 12, 22).unwrap();
        let reason: String = primer.clone().tail_reason();
        assert!(reason.contains("carries the insertion"));
    }
}
