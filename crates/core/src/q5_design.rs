//! Where the Q5 workflow meets the primer-design layer.
//!
//! `q5.rs` resolves a target annotation, applies an edit and verifies it;
//! `primer_design.rs` turns a verified `MutationResult` into a primer pair
//! under `luriaNative` (D029–D033). Neither knows the other, and this module
//! composes them: it adds no biological rule of its own and re-derives nothing.
//!
//! What that fixes here:
//!
//! * the template handed to the design layer is the one the run's own request
//!   names, so a design is built against the artifact the edit was applied to
//!   (I-6, D007); no template comes from the caller;
//! * the `MutationResult` is passed through as the run produced it. This layer
//!   does not re-verify: `design_primers` refuses a result that is not `Verified`,
//!   and that refusal is the guard (D013);
//! * a run that needs clarification designs nothing, and that is an `Ok` outcome
//!   rather than a failure: the clarification is the answer, carried through
//!   intact (D005, "No sequence calculation performed"), and no field of the
//!   design request is read on that path;
//! * every `DomainError` the design layer raises is returned as it stands, with
//!   its own variant and category. This module adds no variant, no message and
//!   no category.
//!
//! A variant set is designed under one request, so every member shares the
//! bound, style, policy, placement and buffer, and each member reports its own
//! outcome: a `PrimerDesign`, or the `DomainError` that stopped it. One member's
//! design failure does not discard the others' designs (owner, 2026-10-14,
//! `docs/q5-sdm.md` §12; D030's 55 °C floor makes such a failure ordinary on an
//! AT-rich site). A failure of the *edits* still fails the whole set, which is
//! `q5::execute_set`'s rule: it runs every member before any design happens.
//!
//! These records are public structs with private fields and no public
//! constructor, so nothing outside this module can build one, valid or invalid,
//! and `design_run` and `design_set` are the only producers. The types do not
//! make an invalid pairing unrepresentable; that is stated rather than assumed.
//! `designs()` holds one entry per member because `q5::execute_set` produces
//! one result per member in request order, an invariant of that module this one
//! relies on and a test pins.

use crate::error::DomainError;
use crate::identity::Identifier;
use crate::identity::SequenceReference;
use crate::mutation::MutationResult;
use crate::primer_design::design_primers;
use crate::primer_design::PrimerDesign;
use crate::primer_design::PrimerDesignRequest;
use crate::q5::Q5Variant;
use crate::q5::Q5VariantResult;
use crate::q5::Q5VariantSetExecution;
use crate::q5::Q5WorkflowExecution;
use crate::sequence::Sequence;

/// A Q5 run carried whole, with the primer design for its edit.
///
/// The run is not flattened: what the workflow understood and decided stays
/// inspectable beside the design it led to (D004, D016).
#[derive(Clone, Debug, PartialEq)]
pub struct Q5DesignedRun {
    run: Q5WorkflowExecution,
    design: Option<PrimerDesign>,
}

impl Q5DesignedRun {
    /// The run, exactly as `q5::execute` produced it.
    pub fn run(&self) -> Q5WorkflowExecution {
        self.run.clone()
    }

    /// The design, present exactly when the run carried a verified mutation.
    pub fn design(&self) -> Option<PrimerDesign> {
        self.design.clone()
    }

    pub fn needs_clarification(&self) -> bool {
        self.run.needs_clarification()
    }

    /// The artifact the run was about, by identity, in every state (D007).
    pub fn template_reference(&self) -> SequenceReference {
        self.run.template_reference()
    }

    /// Short label for reporting. Never used for classification.
    pub fn describe(&self) -> String {
        match self.design() {
            None => self.run.describe(),
            Some(design) => {
                let (start, end) = design.edit_span();
                format!("{} designed [{}, {})", self.run.describe(), start, end)
            }
        }
    }
}

/// What became of one member: a pair, or the reason there is none.
#[derive(Clone, Debug, PartialEq)]
pub enum MemberDesignOutcome {
    /// Boxed because a `PrimerDesign` is much larger than a `DomainError`, and
    /// an unboxed pair would make every outcome the size of the larger one.
    Designed(Box<PrimerDesign>),
    /// The design-layer error as it stands, with its own variant and category.
    NotDesigned(DomainError),
}

/// One member's outcome, paired with the name of the artifact it was designed
/// for, so a design in a set of several stays traceable to its variant.
#[derive(Clone, Debug, PartialEq)]
pub struct Q5MemberDesign {
    result_name: Identifier,
    outcome: MemberDesignOutcome,
}

impl Q5MemberDesign {
    pub fn result_name(&self) -> Identifier {
        self.result_name.clone()
    }

    pub fn outcome(&self) -> MemberDesignOutcome {
        self.outcome.clone()
    }

    /// The design, when this member got one.
    pub fn design(&self) -> Option<PrimerDesign> {
        match &self.outcome {
            MemberDesignOutcome::Designed(design) => Some(design.as_ref().clone()),
            MemberDesignOutcome::NotDesigned(_error) => None,
        }
    }

    /// Why this member got none, when it got none.
    pub fn failure(&self) -> Option<DomainError> {
        match &self.outcome {
            MemberDesignOutcome::Designed(_design) => None,
            MemberDesignOutcome::NotDesigned(error) => Some(error.clone()),
        }
    }

    pub fn is_designed(&self) -> bool {
        self.design().is_some()
    }
}

/// A variant set carried whole, with one design per member.
#[derive(Clone, Debug, PartialEq)]
pub struct Q5DesignedSet {
    run: Q5VariantSetExecution,
    members: Vec<Q5MemberDesign>,
}

impl Q5DesignedSet {
    /// The set run, exactly as `q5::execute_set` produced it.
    pub fn run(&self) -> Q5VariantSetExecution {
        self.run.clone()
    }

    /// One outcome per member, in request order. Empty for a clarification.
    pub fn members(&self) -> Vec<Q5MemberDesign> {
        self.members.clone()
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// The members that got a pair, in request order.
    pub fn designs(&self) -> Vec<PrimerDesign> {
        self.members
            .iter()
            .filter_map(Q5MemberDesign::design)
            .collect()
    }

    pub fn designed_count(&self) -> usize {
        self.designs().len()
    }

    /// The members that did not, in request order.
    pub fn failures(&self) -> Vec<Q5MemberDesign> {
        self.members
            .iter()
            .filter(|member| !member.is_designed())
            .cloned()
            .collect()
    }

    /// True when every member got a pair. False for a clarification, which has
    /// no members at all.
    pub fn all_designed(&self) -> bool {
        !self.members.is_empty() && self.failures().is_empty()
    }

    pub fn needs_clarification(&self) -> bool {
        self.run.needs_clarification()
    }

    pub fn template_reference(&self) -> SequenceReference {
        self.run.template_reference()
    }

    /// Short label for reporting. Never used for classification.
    pub fn describe(&self) -> String {
        format!(
            "{} designed {} of {}",
            self.run.describe(),
            self.designed_count(),
            self.member_count()
        )
    }
}

/// Design the primer pair for a Q5 run.
///
/// A run that needs clarification performs no edit, so it carries no mutation
/// and gets no design, and that is an `Ok` outcome: the clarification is what
/// the caller asked for and it survives unchanged.
pub fn design_run(
    run: Q5WorkflowExecution,
    request: PrimerDesignRequest,
) -> Result<Q5DesignedRun, DomainError> {
    let mutation: Option<MutationResult> = run.mutation();
    if mutation.is_none() {
        return Ok(Q5DesignedRun { run, design: None });
    }

    // The run's own template, never a caller's: a design is built against the
    // artifact its edit was applied to.
    let template: Sequence = run.scope().request().template();
    let design: PrimerDesign = design_primers(&template, mutation.unwrap(), request)?;

    Ok(Q5DesignedRun {
        run,
        design: Some(design),
    })
}

/// Design one primer pair per member of a variant set, under one request.
///
/// Members are designed in request order, and each carries its own outcome: a
/// member that cannot be designed records its error and the rest are designed
/// anyway (owner, 2026-10-14, `docs/q5-sdm.md` §12). The result is always
/// `Ok`: reading the set's members and results cannot fail, and a member's
/// design failure is carried per member rather than returned as the set's.
/// A failure of the *edits* is not this function's to report: it fails the
/// whole set in `q5::execute_set` before any design is attempted.
pub fn design_set(
    run: Q5VariantSetExecution,
    request: PrimerDesignRequest,
) -> Result<Q5DesignedSet, DomainError> {
    if run.needs_clarification() {
        // D005: nothing was chosen, so nothing was edited, so there is
        // nothing to design for any member.
        return Ok(Q5DesignedSet {
            run,
            members: Vec::new(),
        });
    }

    let members: Vec<Q5Variant> = run.request().variants();
    let results: Vec<Q5VariantResult> = run.results();
    let mut outcomes: Vec<Q5MemberDesign> = Vec::new();
    for (member, result) in members.iter().zip(results.iter()) {
        // Each member's own template, which `Q5VariantSetRequest::create`
        // has already required to be one artifact across the set.
        let template: Sequence = member.request().template();
        let outcome: MemberDesignOutcome =
            match design_primers(&template, result.mutation(), request.clone()) {
                Ok(design) => MemberDesignOutcome::Designed(Box::new(design)),
                Err(error) => MemberDesignOutcome::NotDesigned(error),
            };
        outcomes.push(Q5MemberDesign {
            result_name: result.identifier(),
            outcome,
        });
    }

    Ok(Q5DesignedSet {
        run,
        members: outcomes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cds::Cds;
    use crate::cds::Completeness;
    use crate::cds::Numbering;
    use crate::cds::ResiduePosition;
    use crate::error::DesignObstacle;
    use crate::error::ErrorCategory;
    use crate::interval::create;
    use crate::interval::create_point;
    use crate::mutation::Codon;
    use crate::mutation::CodonSelectionPolicy;
    use crate::mutation::NucleotideDeletionIntent;
    use crate::mutation::NucleotideInsertionIntent;
    use crate::mutation::NucleotideMutationIntent;
    use crate::mutation::NucleotideReplacementIntent;
    use crate::mutation::ResidueMutationIntent;
    use crate::primer::PrimerPlacement;
    use crate::primer_design::DesignPolicy;
    use crate::primer_design::DesignStyle;
    use crate::q5::execute;
    use crate::q5::execute_set;
    use crate::q5::Q5Composition;
    use crate::q5::Q5EditWarning;
    use crate::q5::Q5MutationIntent;
    use crate::q5::Q5SdmRequest;
    use crate::q5::Q5VariantSetRequest;
    use crate::q5::ResidueIntentWithoutCds;
    use crate::sequence::normalize;
    use crate::sequence::SequenceTopology;
    use crate::strand::Strand;
    use crate::tm::BufferModel;

    // A 90-nucleotide plus-strand template whose whole length is one CDS of 30
    // codons: `ATG` 14 times, then `GCT`, then `ATG` 15 times. So residue 15 is
    // A at coordinates 42, 43, 44, residue 20 is M at 57, 58, 59, and residue
    // 30 is M at 87, 88, 89. No stop codon and no ambiguous codon.
    //
    // Every expected primer below was produced by
    // `tests/fixtures/q5-primer/reference/native-design.py`, which restates
    // D029 and D030 in plain Python and computes Tm with Biopython; it shares
    // no code with this crate. The junction is 10 bases 5′ of the edit and
    // both primers grow at their 3′ ends until their Tms reach 55 °C, prefer a
    // G/C last base, and are balanced to within 5 °C.
    fn template_text() -> String {
        let mut text: String = String::new();
        let mut index: usize = 0;
        while index < 14 {
            text += "ATG";
            index += 1;
        }
        text += "GCT";
        index = 0;
        while index < 15 {
            text += "ATG";
            index += 1;
        }
        text
    }

    fn template() -> Sequence {
        normalize(
            Identifier::create("q5d-a".to_string()).unwrap(),
            template_text(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    // A 30-nucleotide artifact: too short for any pair, since the smallest is
    // L + max(b, 20) + max(b, 15) = 3 + 20 + 15 = 38 bases.
    fn short_template() -> Sequence {
        normalize(
            Identifier::create("q5d-short".to_string()).unwrap(),
            "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    fn name(value: &str) -> Identifier {
        Identifier::create(value.to_string()).unwrap()
    }

    /// The whole artifact as one complete plus-strand CDS.
    fn cds_of(sequence: &Sequence, id: &str) -> Cds {
        Cds::create(
            sequence,
            name(id),
            vec![create(&sequence.reference(), 0, sequence.length() as i64).unwrap()],
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap()
    }

    /// A residue intent naming its CDS, with the replacement codon supplied so
    /// the expectations do not depend on the default codon table.
    fn residue_intent(
        cds: &str,
        number: i64,
        expected: &str,
        replacement: &str,
        codon: &str,
    ) -> Q5MutationIntent {
        Q5MutationIntent::Residue(
            ResidueMutationIntent::create(
                ResiduePosition::create(name(cds), number).unwrap(),
                expected.to_string(),
                replacement.to_string(),
                CodonSelectionPolicy::Supplied(Codon::create(codon.to_string()).unwrap()),
            )
            .unwrap(),
        )
    }

    fn request_of(
        sequence: Sequence,
        scope: Option<&str>,
        intent: Q5MutationIntent,
    ) -> Q5SdmRequest {
        Q5SdmRequest::create(sequence, scope.map(name), intent).unwrap()
    }

    fn design_request(bound: i64) -> PrimerDesignRequest {
        PrimerDesignRequest::create(
            DesignStyle::Q5Polymerase,
            DesignPolicy::LuriaNative,
            bound,
            PrimerPlacement::Inline,
            BufferModel::q5_reaction(),
        )
        .unwrap()
    }

    /// Run a one-request workflow over the 90-base template's single CDS, then
    /// design it.
    fn designed(intent: Q5MutationIntent, scope: Option<&str>) -> Q5DesignedRun {
        let run: Q5WorkflowExecution = execute(
            request_of(template(), scope, intent),
            vec![cds_of(&template(), "cds-1")],
            name("q5d-edited"),
        )
        .unwrap();
        design_run(run, design_request(15)).unwrap()
    }

    fn primer_text(design: &PrimerDesign) -> (String, String) {
        (
            design.pair().forward().sequence(),
            design.pair().reverse().sequence(),
        )
    }

    fn primer_spans(design: &PrimerDesign) -> ((u64, u64), (u64, u64)) {
        let pair = design.pair();
        let (forward, reverse) = (pair.clone().forward().span(), pair.reverse().span());
        (
            (forward.start(), forward.end()),
            (reverse.start(), reverse.end()),
        )
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

    // Residue 15, A -> W, codon GCT -> TGG at [42, 45). Two adjacent bases
    // change, so the forward Tm is on the matched basis. The forward primer
    // grows two bases to a G end; the reverse grows to the floor and then to
    // within 5 °C of the forward: forward [32, 57), reverse [8, 32).
    #[test]
    fn a_residue_run_reaches_a_primer_pair() {
        let designed: Q5DesignedRun = designed(residue_intent("cds-1", 15, "A", "W", "TGG"), None);
        assert!(!designed.needs_clarification());
        assert!(designed.run().is_verified());

        let design: PrimerDesign = designed.design().unwrap();
        assert!(design.edit_span() == (42, 45));
        assert!(primer_spans(&design) == ((32, 57), (8, 32)));
        assert!(
            primer_text(&design)
                == (
                    "GATGATGATGTGGATGATGATGATG".to_string(),
                    "ATCATCATCATCATCATCATCATC".to_string()
                )
        );
        assert!(design.pair().abuts());
        assert!(design.tm_balance_met());
        assert!(design.forward_tm().celsius() >= 55.0 && design.reverse_tm().celsius() >= 55.0);
    }

    // Coordinate 45, A -> C. Both primers grow to reach the floor: forward
    // [35, 57), reverse [14, 35).
    #[test]
    fn a_nucleotide_run_reaches_a_primer_pair() {
        let intent: Q5MutationIntent = Q5MutationIntent::Nucleotide(
            NucleotideMutationIntent::create(
                template().reference(),
                create(&template().reference(), 45, 46).unwrap(),
                "A".to_string(),
                "C".to_string(),
            )
            .unwrap(),
        );
        let design: PrimerDesign = designed(intent, None).design().unwrap();
        assert!(design.edit_span() == (45, 46));
        assert!(primer_spans(&design) == ((35, 57), (14, 35)));
        assert!(
            primer_text(&design)
                == (
                    "GATGATGGCTCTGATGATGATG".to_string(),
                    "ATCATCATCATCATCATCATC".to_string()
                )
        );
    }

    // The same edit as the named-residue case, reached through the intent form
    // that names no CDS: the gate resolves the only annotation, so the design
    // must be identical.
    #[test]
    fn an_unbound_residue_run_reaches_the_same_pair_as_the_named_form() {
        let unbound: Q5MutationIntent = Q5MutationIntent::ResidueWithoutCds(
            ResidueIntentWithoutCds::create(
                15,
                "A".to_string(),
                "W".to_string(),
                CodonSelectionPolicy::Supplied(Codon::create("TGG".to_string()).unwrap()),
            )
            .unwrap(),
        );
        let bound: PrimerDesign = designed(residue_intent("cds-1", 15, "A", "W", "TGG"), None)
            .design()
            .unwrap();
        let resolved: PrimerDesign = designed(unbound, None).design().unwrap();
        assert!(primer_spans(&resolved) == primer_spans(&bound));
        assert!(primer_text(&resolved) == primer_text(&bound));
    }

    // Two annotations and a request that names neither: the gate cannot choose,
    // so there is no edit to design for.
    #[test]
    fn a_clarification_run_designs_nothing_and_keeps_its_clarification() {
        let unbound: Q5MutationIntent = Q5MutationIntent::ResidueWithoutCds(
            ResidueIntentWithoutCds::create(
                15,
                "A".to_string(),
                "W".to_string(),
                CodonSelectionPolicy::Supplied(Codon::create("TGG".to_string()).unwrap()),
            )
            .unwrap(),
        );
        let run: Q5WorkflowExecution = execute(
            request_of(template(), None, unbound),
            vec![cds_of(&template(), "cds-1"), cds_of(&template(), "cds-2")],
            name("q5d-edited"),
        )
        .unwrap();
        assert!(run.needs_clarification());

        let designed: Q5DesignedRun = design_run(run, design_request(15)).unwrap();
        assert!(designed.needs_clarification());
        assert!(designed.design().is_none());
        assert!(!designed.run().scope().clarifications().is_empty());
        assert!(designed.template_reference() == template().reference());
    }

    #[test]
    fn a_design_is_built_against_the_run_that_produced_it() {
        let designed: Q5DesignedRun = designed(residue_intent("cds-1", 15, "A", "W", "TGG"), None);
        let design: PrimerDesign = designed.design().unwrap();
        assert!(design.built_against() == designed.template_reference());
        assert!(design.built_against() == template().reference());
        assert!(design.edited() == designed.run().mutation().unwrap().reference());
    }

    #[test]
    fn the_same_run_and_request_give_the_same_design() {
        let first: Q5DesignedRun = designed(residue_intent("cds-1", 15, "A", "W", "TGG"), None);
        let second: Q5DesignedRun = designed(residue_intent("cds-1", 15, "A", "W", "TGG"), None);
        assert!(first == second);
    }

    /// A set of the named-residue edits given, each with its own result name.
    fn variant_set(members: Vec<(Q5MutationIntent, &str)>) -> Q5VariantSetRequest {
        let variants: Vec<Q5Variant> = members
            .into_iter()
            .map(|(intent, result)| {
                Q5Variant::create(request_of(template(), None, intent), name(result))
            })
            .collect();
        Q5VariantSetRequest::create(Q5Composition::SeparateVariants, variants).unwrap()
    }

    // Residue 15 at [42, 45) and residue 20 at [57, 60): the second pair sits
    // 15 bases further along, and its reverse primer spans the unedited GCT,
    // so the two designs differ in both span and text.
    #[test]
    fn a_two_member_set_gets_one_design_per_member_in_request_order() {
        let set: Q5VariantSetRequest = variant_set(vec![
            (residue_intent("cds-1", 15, "A", "W", "TGG"), "variant-1"),
            (residue_intent("cds-1", 20, "M", "W", "TGG"), "variant-2"),
        ]);
        let run: Q5VariantSetExecution =
            execute_set(set, vec![cds_of(&template(), "cds-1")]).unwrap();
        let designed: Q5DesignedSet = design_set(run, design_request(15)).unwrap();

        assert!(designed.member_count() == 2);
        assert!(designed.designed_count() == 2);
        assert!(designed.all_designed());
        assert!(designed.member_count() == designed.run().result_count());
        let members: Vec<Q5MemberDesign> = designed.members();
        assert!(members[0].result_name() == name("variant-1"));
        assert!(members[1].result_name() == name("variant-2"));

        let (first, second) = (members[0].design().unwrap(), members[1].design().unwrap());
        assert!(primer_spans(&first) == ((32, 57), (8, 32)));
        assert!(primer_spans(&second) == ((47, 72), (26, 47)));
        assert!(
            primer_text(&second)
                == (
                    "GATGATGATGTGGATGATGATGATG".to_string(),
                    "ATAGCCATCATCATCATCATC".to_string()
                )
        );
        assert!(first != second);
    }

    // Residue 30 is the last codon, at [87, 90), so no template base is left
    // for the forward primer's 3′ flank. The member's edit succeeds and only
    // its design fails, which the set records against that member alone.
    #[test]
    fn a_member_that_cannot_be_designed_does_not_discard_the_others() {
        let set: Q5VariantSetRequest = variant_set(vec![
            (residue_intent("cds-1", 15, "A", "W", "TGG"), "variant-1"),
            (residue_intent("cds-1", 30, "M", "W", "TGG"), "variant-2"),
            (residue_intent("cds-1", 20, "M", "W", "TGG"), "variant-3"),
        ]);
        let run: Q5VariantSetExecution =
            execute_set(set, vec![cds_of(&template(), "cds-1")]).unwrap();
        assert!(run.result_count() == 3);
        assert!(run.is_verified());

        let designed: Q5DesignedSet = design_set(run, design_request(15)).unwrap();
        assert!(designed.member_count() == 3);
        assert!(designed.designed_count() == 2);
        assert!(!designed.all_designed());

        // The members that could be designed are, and in request order.
        let members: Vec<Q5MemberDesign> = designed.members();
        assert!(members[0].is_designed() && members[2].is_designed());
        assert!(primer_spans(&members[0].design().unwrap()) == ((32, 57), (8, 32)));
        assert!(primer_spans(&members[2].design().unwrap()) == ((47, 72), (26, 47)));

        // The one that could not carries its own error, unchanged.
        let failures: Vec<Q5MemberDesign> = designed.failures();
        assert!(failures.len() == 1);
        assert!(failures[0].result_name() == name("variant-2"));
        assert!(!members[1].is_designed() && members[1].design().is_none());
        let error: DomainError = members[1].failure().unwrap();
        assert!(error.category() == ErrorCategory::Unsupported);
        match error {
            DomainError::NoPrimerDesignFits(problem) => {
                assert!(problem.edit_start == 87 && problem.edit_end == 90);
                assert!(
                    problem.obstacles
                        == vec![DesignObstacle::ThreePrimeAvailability {
                            available: 0,
                            needed: 10
                        }]
                );
            }
            other => panic!("expected NoPrimerDesignFits, got {:?}", other),
        }
    }

    // A set every member of which fails is still `Ok`: the per-member reasons
    // are the answer, and the caller is not left guessing which member failed.
    #[test]
    fn a_set_no_member_of_which_can_be_designed_reports_every_reason() {
        let set: Q5VariantSetRequest = variant_set(vec![
            (residue_intent("cds-1", 30, "M", "W", "TGG"), "variant-1"),
            (residue_intent("cds-1", 29, "M", "W", "TGG"), "variant-2"),
        ]);
        let run: Q5VariantSetExecution =
            execute_set(set, vec![cds_of(&template(), "cds-1")]).unwrap();
        let designed: Q5DesignedSet = design_set(run, design_request(15)).unwrap();
        assert!(designed.member_count() == 2);
        assert!(designed.designed_count() == 0);
        assert!(!designed.all_designed());
        assert!(designed.designs().is_empty());
        for member in designed.members() {
            assert!(matches!(
                member.failure().unwrap(),
                DomainError::NoPrimerDesignFits(_)
            ));
        }
    }

    // An edit that fails still fails the whole set, which is `execute_set`'s
    // rule and is not changed by per-member design reporting.
    #[test]
    fn a_failing_edit_still_fails_the_whole_set_before_any_design() {
        let set: Q5VariantSetRequest = variant_set(vec![
            (residue_intent("cds-1", 15, "A", "W", "TGG"), "variant-1"),
            (residue_intent("cds-1", 20, "A", "W", "TGG"), "variant-2"),
        ]);
        let error: DomainError = execute_set(set, vec![cds_of(&template(), "cds-1")]).unwrap_err();
        assert!(matches!(error, DomainError::ExpectedResidueMismatch(_)));
    }

    #[test]
    fn a_set_that_needs_clarification_designs_nothing() {
        let unbound: Q5MutationIntent = Q5MutationIntent::ResidueWithoutCds(
            ResidueIntentWithoutCds::create(
                15,
                "A".to_string(),
                "W".to_string(),
                CodonSelectionPolicy::Supplied(Codon::create("TGG".to_string()).unwrap()),
            )
            .unwrap(),
        );
        let set: Q5VariantSetRequest = variant_set(vec![(unbound, "variant-1")]);
        let run: Q5VariantSetExecution = execute_set(
            set,
            vec![cds_of(&template(), "cds-1"), cds_of(&template(), "cds-2")],
        )
        .unwrap();
        assert!(run.needs_clarification());

        let designed: Q5DesignedSet = design_set(run, design_request(15)).unwrap();
        assert!(designed.needs_clarification());
        assert!(designed.member_count() == 0);
        assert!(designed.designed_count() == 0);
        // A clarification has no members, so "every member was designed" is
        // false rather than vacuously true.
        assert!(!designed.all_designed());
    }

    // The design layer's failures reach the caller unchanged: 30 bases cannot
    // hold the smallest pair, 3 + 20 + 15 = 38.
    #[test]
    fn a_template_too_short_for_any_pair_fails_with_the_design_layers_own_error() {
        let run: Q5WorkflowExecution = execute(
            request_of(
                short_template(),
                None,
                residue_intent("cds-s", 2, "A", "W", "TGG"),
            ),
            vec![cds_of(&short_template(), "cds-s")],
            name("q5d-edited"),
        )
        .unwrap();
        assert!(run.is_verified());

        let error: DomainError = design_run(run, design_request(15)).unwrap_err();
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        match error {
            DomainError::NoPrimerDesignPossible(problem) => {
                assert!(problem.template_length == 30);
                assert!(problem.smallest_pair == 38);
            }
            other => panic!("expected NoPrimerDesignPossible, got {:?}", other),
        }
    }

    // A DNA-level deletion on the 90-base template: the region and the exact
    // text removed, as Milestone 4a's intent asserts it. `[45, 48)` is the `ATG`
    // that follows the GCT, wholly inside the one whole-length CDS.
    fn deletion_intent(start: i64, end: i64, bases: String) -> Q5MutationIntent {
        Q5MutationIntent::Deletion(
            NucleotideDeletionIntent::create(
                template().reference(),
                create(&template().reference(), start, end).unwrap(),
                bases,
            )
            .unwrap(),
        )
    }

    // D031, row `del` of `tests/fixtures/q5-primer/reference/native-deletion.py`'s
    // `design()` over this template with bound 15: three coding bases removed is
    // a multiple of 3, so the reading frame is kept and the run warns nothing.
    // The forward primer's 5' end sits at 48, the far edge of the removed region,
    // and the reverse's at 45, so the 3-base gap between the two spans is the
    // deletion itself.
    #[test]
    fn an_in_frame_deletion_run_reaches_a_d031_pair() {
        let designed: Q5DesignedRun = designed(deletion_intent(45, 48, "ATG".to_string()), None);
        assert!(!designed.needs_clarification());

        let run: Q5WorkflowExecution = designed.run();
        assert!(run.is_verified());
        assert!(run.mutation().unwrap().is_deletion());
        assert!(run.warnings().is_empty());

        let design: PrimerDesign = designed.design().unwrap();
        assert!(design.edit_span() == (45, 48));
        assert!(primer_spans(&design) == ((48, 69), (26, 45)));
        assert!(
            primer_text(&design)
                == (
                    "ATGATGATGATGATGATGATG".to_string(),
                    "AGCCATCATCATCATCATC".to_string()
                )
        );
        assert!(design.deleted_width() == 3);
        assert!(design.pair().abuts());
        assert_tms(&design, 56.52, 58.66, 59.52);
    }

    // One coding base removed from the same CDS: 1 is not a multiple of 3, so
    // D031's owner decision keeps the edit, warns, and the design still happens
    // under the same request. The pair straddles a 1-base gap at [46, 47).
    #[test]
    fn a_frameshift_deletion_is_still_designed_with_its_warning() {
        let designed: Q5DesignedRun = designed(deletion_intent(46, 47, "T".to_string()), None);
        assert!(!designed.needs_clarification());

        let run: Q5WorkflowExecution = designed.run();
        assert!(run.is_verified());
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(warnings.len() == 1);
        assert!(
            warnings[0]
                == Q5EditWarning::Frameshift {
                    cds: name("cds-1"),
                    coding_bases_removed: 1,
                    wholly_inside: true,
                }
        );

        let design: PrimerDesign = designed.design().unwrap();
        assert!(design.edit_span() == (46, 47));
        assert!(primer_spans(&design) == ((47, 69), (26, 46)));
        assert!(
            primer_text(&design)
                == (
                    "GATGATGATGATGATGATGATG".to_string(),
                    "TAGCCATCATCATCATCATC".to_string()
                )
        );
        assert!(design.deleted_width() == 1);
        assert!(design.pair().abuts());
        assert_tms(&design, 57.83, 58.37, 60.83);
    }

    // Both deletions as members of one set under one request: each gets its own
    // pair, in request order, and the forward spans 48:69 then 47:69 say which
    // design belongs to which member.
    #[test]
    fn a_set_of_deletions_designs_every_member() {
        let set: Q5VariantSetRequest = variant_set(vec![
            (deletion_intent(45, 48, "ATG".to_string()), "d1"),
            (deletion_intent(46, 47, "T".to_string()), "d2"),
        ]);
        let run: Q5VariantSetExecution =
            execute_set(set, vec![cds_of(&template(), "cds-1")]).unwrap();
        let designed: Q5DesignedSet = design_set(run, design_request(15)).unwrap();

        assert!(designed.member_count() == 2);
        assert!(designed.designed_count() == 2);
        assert!(designed.all_designed());
        assert!(designed.member_count() == designed.run().result_count());

        let members: Vec<Q5MemberDesign> = designed.members();
        assert!(members[0].result_name() == name("d1"));
        assert!(members[1].result_name() == name("d2"));
        assert!(primer_spans(&members[0].design().unwrap()) == ((48, 69), (26, 45)));
        assert!(primer_spans(&members[1].design().unwrap()) == ((47, 69), (26, 46)));

        let (first, second) = (members[0].design().unwrap(), members[1].design().unwrap());
        assert!(first.deleted_width() == 3 && second.deleted_width() == 1);
        assert!(first.pair().abuts() && second.pair().abuts());
        assert!(first != second);
    }

    // A DNA-level insertion on the 90-base template: the gap at `position`,
    // with the two flanking bases read from the template itself at indices
    // `position - 1` and `position` — the pattern `deletion_intent` sets up,
    // with the assertion of the original state moved off the test.
    fn insertion_intent(position: i64, inserted: &str) -> Q5MutationIntent {
        let before: String = template()
            .letter_at((position - 1) as u64)
            .unwrap()
            .to_string();
        let after: String = template().letter_at(position as u64).unwrap().to_string();
        Q5MutationIntent::Insertion(
            NucleotideInsertionIntent::create(
                template().reference(),
                create_point(&template().reference(), position).unwrap(),
                before.clone(),
                after.clone(),
                inserted.to_string(),
            )
            .unwrap(),
        )
    }

    // D032, row `ins` of `tests/fixtures/q5-primer/reference/native-insertion.py`'s
    // `design()` over this template with bound 15: three coding bases inserted at
    // the gap at 45 is a multiple of 3, so the reading frame is kept and the run
    // warns nothing. The forward primer's 5' end sits at 45, the near edge of the
    // gap, and the reverse's at 45 too, so the two spans meet at the insertion
    // point and the inserted `GCT` is the forward primer's tail.
    #[test]
    fn an_in_frame_insertion_run_reaches_a_d032_pair() {
        let designed: Q5DesignedRun = designed(insertion_intent(45, "GCT"), None);
        assert!(!designed.needs_clarification());

        let run: Q5WorkflowExecution = designed.run();
        assert!(run.is_verified());
        assert!(run.mutation().unwrap().is_insertion());
        assert!(run.warnings().is_empty());

        let design: PrimerDesign = designed.design().unwrap();
        assert!(design.edit_span() == (45, 45));
        assert!(primer_spans(&design) == ((45, 66), (26, 45)));
        assert!(
            primer_text(&design)
                == (
                    "GCTATGATGATGATGATGATGATG".to_string(),
                    "AGCCATCATCATCATCATC".to_string()
                )
        );
        assert!(design.pair().forward().tail().unwrap().sequence() == "GCT");
        assert!(design.pair().reverse().tail().is_none());
        assert_tms(&design, 56.52, 58.66, 59.52);
        assert!(
            close(design.forward_whole_tm().unwrap().celsius(), 60.54),
            "forward whole Tm {:.3}, expected {:.2}",
            design.forward_whole_tm().unwrap().celsius(),
            60.54,
        );
        assert!(
            close(design.reverse_whole_tm().unwrap().celsius(), 58.66),
            "reverse whole Tm {:.3}, expected {:.2}",
            design.reverse_whole_tm().unwrap().celsius(),
            58.66,
        );
    }

    // One base inserted into the same gap of the same CDS: 1 is not a multiple of
    // 3, so D032's owner decision keeps the edit, warns, and the design still
    // happens under the same request. The spans are unchanged; only the forward
    // primer's tail is the single inserted base.
    #[test]
    fn a_frameshift_insertion_is_still_designed_with_its_warning() {
        let designed: Q5DesignedRun = designed(insertion_intent(45, "G"), None);
        assert!(!designed.needs_clarification());

        let run: Q5WorkflowExecution = designed.run();
        assert!(run.is_verified());
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(
            warnings
                == vec![Q5EditWarning::InsertionFrameshift {
                    cds: name("cds-1"),
                    bases_inserted: 1,
                },]
        );

        let design: PrimerDesign = designed.design().unwrap();
        assert!(design.edit_span() == (45, 45));
        assert!(primer_spans(&design) == ((45, 66), (26, 45)));
        assert!(design.pair().forward().sequence() == "GATGATGATGATGATGATGATG");
        assert!(
            close(design.forward_whole_tm().unwrap().celsius(), 57.83),
            "forward whole Tm {:.3}, expected {:.2}",
            design.forward_whole_tm().unwrap().celsius(),
            57.83,
        );
    }

    // Both insertions as members of one set under one request: each gets its own
    // pair, in request order, and the frameshift warning of one member does not
    // stop the other.
    #[test]
    fn a_set_of_insertions_designs_every_member() {
        let set: Q5VariantSetRequest = variant_set(vec![
            (insertion_intent(45, "GCT"), "i1"),
            (insertion_intent(45, "G"), "i2"),
        ]);
        let run: Q5VariantSetExecution =
            execute_set(set, vec![cds_of(&template(), "cds-1")]).unwrap();
        let designed: Q5DesignedSet = design_set(run, design_request(15)).unwrap();

        assert!(designed.designed_count() == 2);
        assert!(designed.all_designed());
    }

    // A DNA-level replacement on the 90-base template: the region it overwrites
    // and the exact text there, as Milestone 4e-2's intent asserts it — the
    // pattern `deletion_intent` sets up. `[45, 48)` is the `ATG` that follows the
    // GCT, wholly inside the one whole-length CDS.
    fn replacement_intent(
        start: i64,
        end: i64,
        expected: String,
        replacement: String,
    ) -> Q5MutationIntent {
        Q5MutationIntent::Replacement(
            NucleotideReplacementIntent::create(
                template().reference(),
                create(&template().reference(), start, end).unwrap(),
                expected,
                replacement,
            )
            .unwrap(),
        )
    }

    // D033, row `rep` of `tests/fixtures/q5-primer/reference/native-replacement.py`'s
    // `design()` over this template with bound 15: three bases overwritten by six
    // leave `m − w` at 3, a multiple of 3, so the reading frame is kept and the run
    // warns nothing. `t = max(w, m)` is 6, at `MAX_INLINE_REPLACEMENT`, so the pair
    // is inline: the forward span is 3 template bases longer than the 23 it names
    // and the reverse is exactly the 30 it names.
    #[test]
    fn an_inline_replacement_run_reaches_a_d033_pair() {
        let designed: Q5DesignedRun = designed(
            replacement_intent(45, 48, "ATG".to_string(), "TGGTGG".to_string()),
            None,
        );
        assert!(!designed.needs_clarification());

        let run: Q5WorkflowExecution = designed.run();
        assert!(run.is_verified());
        assert!(run.mutation().unwrap().is_replacement());
        assert!(run.warnings().is_empty());

        let design: PrimerDesign = designed.design().unwrap();
        assert!(primer_spans(&design) == ((35, 58), (5, 35)));
        assert!(
            primer_text(&design)
                == (
                    "GATGATGGCTTGGTGGATGATGATGA".to_string(),
                    "ATCATCATCATCATCATCATCATCATCATC".to_string(),
                )
        );
        assert!(design.pair().forward().tail().is_none());
        assert!(design.pair().reverse().tail().is_none());
        assert_tms(&design, 68.70, 64.00, 67.00);
    }

    // Three bases overwritten by two: `m − w` is −1, not a multiple of 3, so D033's
    // owner decision keeps the edit, warns, and the design still happens under the
    // same request. The forward primer is one base shorter than the 25 its span
    // names.
    #[test]
    fn a_frameshift_replacement_is_still_designed_with_its_warning() {
        let designed: Q5DesignedRun = designed(
            replacement_intent(45, 48, "ATG".to_string(), "TG".to_string()),
            None,
        );
        assert!(!designed.needs_clarification());

        let run: Q5WorkflowExecution = designed.run();
        assert!(run.is_verified());
        let warnings: Vec<Q5EditWarning> = run.warnings();
        assert!(
            warnings
                == vec![Q5EditWarning::ReplacementFrameshift {
                    cds: name("cds-1"),
                    bases_replaced: 3,
                    bases_written: 2,
                    wholly_inside: true,
                },]
        );

        let design: PrimerDesign = designed.design().unwrap();
        assert!(primer_spans(&design) == ((35, 60), (11, 35)));
        assert!(
            primer_text(&design)
                == (
                    "GATGATGGCTTGATGATGATGATG".to_string(),
                    "ATCATCATCATCATCATCATCATC".to_string(),
                )
        );
        assert_tms(&design, 63.36, 59.48, 62.48);
    }

    // Three bases overwritten by nine: `t = max(w, m)` is 9, past
    // `MAX_INLINE_REPLACEMENT`, so the pair takes tails. The forward tail is the
    // first five bases written and the reverse's the reverse complement of the last
    // four, and the annealing spans are the template's 21 and 19 bases.
    #[test]
    fn a_long_replacement_run_reaches_a_tail_pair() {
        let designed: Q5DesignedRun = designed(
            replacement_intent(45, 48, "ATG".to_string(), "TGGTGGTGG".to_string()),
            None,
        );
        assert!(!designed.needs_clarification());

        let run: Q5WorkflowExecution = designed.run();
        assert!(run.is_verified());
        assert!(run.warnings().is_empty());

        let design: PrimerDesign = designed.design().unwrap();
        assert!(primer_spans(&design) == ((48, 69), (26, 45)));
        assert!(
            primer_text(&design)
                == (
                    "GGTGGATGATGATGATGATGATGATG".to_string(),
                    "ACCAAGCCATCATCATCATCATC".to_string(),
                )
        );
        assert!(design.pair().forward().tail().unwrap().sequence() == "GGTGG");
        assert!(design.pair().reverse().tail().unwrap().sequence() == "ACCA");
        assert_tms(&design, 56.52, 58.66, 59.52);
        assert!(
            close(design.forward_whole_tm().unwrap().celsius(), 65.03),
            "forward whole Tm {:.3}, expected {:.2}",
            design.forward_whole_tm().unwrap().celsius(),
            65.03,
        );
        assert!(
            close(design.reverse_whole_tm().unwrap().celsius(), 64.69),
            "reverse whole Tm {:.3}, expected {:.2}",
            design.reverse_whole_tm().unwrap().celsius(),
            64.69,
        );
    }

    // Both replacements as members of one set under one request: each gets its own
    // pair, in request order, and the inline and tailed geometries sit side by side
    // in the same set.
    #[test]
    fn a_set_of_replacements_designs_every_member() {
        let set: Q5VariantSetRequest = variant_set(vec![
            (
                replacement_intent(45, 48, "ATG".to_string(), "TGGTGG".to_string()),
                "r1",
            ),
            (
                replacement_intent(45, 48, "ATG".to_string(), "TGGTGGTGG".to_string()),
                "r2",
            ),
        ]);
        let run: Q5VariantSetExecution =
            execute_set(set, vec![cds_of(&template(), "cds-1")]).unwrap();
        let designed: Q5DesignedSet = design_set(run, design_request(15)).unwrap();

        assert!(designed.designed_count() == 2);
        assert!(designed.all_designed());
    }
}
