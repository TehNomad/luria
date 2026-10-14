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
//! * **No Tm, no `Ta`.** §11 states that the project's Tm function is not yet
//!   defined and that no Tm range is established; §13.3 says reproducing the
//!   reference routine is not evidence about `luria`. Nothing here computes a
//!   thermodynamic value.
//! * **No `confineToTail` placement.** §15 question 8 asks whether `luria`
//!   supports it at all, and §10 would need a style default that no source
//!   establishes. The placement is an explicit argument with no default, and
//!   `ConfineToTail` is refused with `DomainError::UnsupportedPrimerPlacement`
//!   naming the open question.
//! * **No insertion or deletion.** The mutation layer implements substitution
//!   only, so there is no `MutationResult` for those edits to carry, and §5
//!   states them as separate degenerate cases rather than as a general rule.
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
/// the third kind, and appears only under `ConfineToTail`, which this layer
/// refuses.
#[derive(Clone, Debug, PartialEq)]
pub enum PrimerPartRole {
    Context,
    Mutation,
}

impl PrimerPartRole {
    pub fn name(self) -> String {
        match self {
            PrimerPartRole::Context => "context".to_string(),
            PrimerPartRole::Mutation => "mutation".to_string(),
        }
    }

    pub fn carries_mutation(self) -> bool {
        match self {
            PrimerPartRole::Mutation => true,
            PrimerPartRole::Context => false,
        }
    }
}

/// One run of a primer: its text, the template interval it sits at, and the
/// primer index range it occupies. `primer_indices` are half-open and zero
/// based within the primer; `span` is half-open on the template (D004).
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerPart {
    role: PrimerPartRole,
    annealing: bool,
    sequence: String,
    span: NucleotideInterval,
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

    pub fn span(self) -> NucleotideInterval {
        self.span.clone()
    }

    /// Half-open primer index range, zero based within the primer.
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
    /// `L = e - s`; they part only for an insertion or a deletion, which this
    /// layer does not support.
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

    /// The 5′ portion carrying the mutation. Absent for every design this
    /// layer constructs, because `ConfineToTail` is refused; §5 says the
    /// mutated positions sit inside the primer rather than at its 5′ end.
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
/// with the spans it describes.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerPair {
    forward: Primer,
    reverse: Primer,
}

impl PrimerPair {
    pub fn forward(self) -> Primer {
        self.forward.clone()
    }

    pub fn reverse(self) -> Primer {
        self.reverse.clone()
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
    pub fn boundary(self) -> Option<u64> {
        let back: NucleotideInterval = self.reverse.clone().span();
        let front: NucleotideInterval = self.forward.clone().span();
        if back.end() == front.start() {
            return Some(front.start());
        }
        if front.end() == back.start() {
            return Some(back.start());
        }
        None
    }

    pub fn abuts(self) -> bool {
        self.separation() == 0
    }

    /// H5: the pair's relationship to the edit, stated in the output and
    /// derived here rather than carried as a claim (§7 H5, I-8).
    pub fn relationship(self) -> String {
        let back: NucleotideInterval = self.reverse.clone().span();
        let front: NucleotideInterval = self.forward.clone().span();
        let gap: i64 = self.separation();
        let phrase: String = if gap > 0 {
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
            "] and forward span [".to_string(),
            describe(front),
            "]: ".to_string(),
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

    let edited: Sequence = result.edited();
    let before: Vec<char> = template.letters();
    let after: Vec<char> = edited.letters();
    if before.len() != after.len() {
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

    let width: u64 = span.end() - span.start();
    if total as u64 != width {
        let problem = InvalidPrimerCandidateProblem {
            primer: label,
            reason: join(vec![
                "the text is ".to_string(),
                number(total as u64),
                " nucleotides and the span ".to_string(),
                describe(span),
                " selects ".to_string(),
                number(width),
            ]),
        };
        return Err(DomainError::InvalidPrimerCandidate(problem));
    }

    // §4 consequence 2: the primer text comes from the edited artifact, and
    // §6: a reverse primer is stated 5'->3', which is the reverse complement
    // of the region it covers.
    let region: Vec<char> = oriented(
        slice(after.clone(), span.start(), span.end()),
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
    // template position one below its span's end, not its span's start.
    let changed: Vec<u64> =
        changed_within(span.clone(), claimed.clone(), role.clone().is_reverse());

    let reference: SequenceReference = span.reference();
    let parts: Vec<PrimerPart> = runs(
        changed.clone(),
        width,
        span.clone(),
        role.clone().is_reverse(),
        reference,
        letters.clone(),
    )?;
    let annealing_spans: OrderedIntervals = annealing_of(parts.clone());
    let annealing_indices: Vec<u64> = unchanged(changed.clone(), width);
    let annealing_text: String = canonical_text(pick(letters.clone(), annealing_indices.clone()));

    // I-1 / H3 clause 2: the annealing portion is the wild-type template
    // under the intervals it covers, reverse-complemented for a reverse
    // primer. Checked rather than assumed, so a wrong interval list is caught
    // rather than reported.
    let template_region: Vec<char> = oriented(
        slice(before.clone(), span.start(), span.end()),
        role.clone().is_reverse(),
    )?;
    let under: Vec<char> = pick(template_region.clone(), annealing_indices.clone());
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
        edited_span: edited_span_of(role.clone(), span.clone()),
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

/// Join two constructed primers into a pair and check the requirements that
/// belong to a pair rather than to a primer: H2 that the pair covers the
/// changed region of its edit, and H4 that the two primers state opposite
/// orientations. H5 is satisfied by `PrimerPair::relationship`, which derives
/// the statement from the two spans (I-8).
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
    let changed: Vec<u64> = result.clone().target_coordinates();
    let mut uncovered: Vec<u64> = Vec::new();
    for position in changed.clone() {
        if !covers(forward.clone().span(), position) && !covers(reverse.clone().span(), position) {
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

    Ok(PrimerPair {
        forward: forward.clone(),
        reverse: reverse.clone(),
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
            span: piece.clone(),
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
/// per run, so a primer annealing on both sides of the mutation has two.
fn annealing_of(parts: Vec<PrimerPart>) -> OrderedIntervals {
    let mut out: OrderedIntervals = Vec::new();
    for part in parts {
        if part.clone().annealing() {
            out.push(part.clone().span());
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
    use crate::interval::OrderedIntervals;
    use crate::mutation::apply_nucleotide;
    use crate::mutation::apply_residue;
    use crate::mutation::CodonSelectionPolicy;
    use crate::mutation::MutationResult;
    use crate::mutation::NucleotideMutationIntent;
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
                == "reverse span [0, 12] and forward span [12, 30]: abutting"
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
        assert!(pair.clone().relationship() == "reverse span [12, 30] and forward span [0, 20]: overlapping by 8 template positions", "actual=[{}]", pair.clone().relationship());
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
}
