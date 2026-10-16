//! D033's degenerate expansion, at the request layer: one request, one member
//! per concrete sequence.
//!
//! A replacement that carries an IUPAC ambiguity code cannot reach `mutation.rs`
//! as it stands: an ambiguity code is not a base that can be asserted or
//! written, and `NucleotideReplacementIntent::create` refuses one in either
//! text. D033's addendum therefore puts the expansion at the **request** layer,
//! before any edit — each concrete sequence becomes its own ordinary,
//! non-degenerate replacement, its own member of a `SeparateVariants` set, and
//! the set's existing per-member reporting gives each its own outcome. This
//! module is that expansion and nothing else: it calls
//! `sequence::expand_degenerate` as it stands, builds each member through
//! `mutation.rs`'s `NucleotideReplacementIntent::create` and `q5.rs`'s
//! `Q5SdmRequest::create`, and hands the set to `q5::execute_set` and
//! `q5_design::design_set` unchanged. It adds no mutation kind, no error
//! variant, no biological rule of its own, and it never runs or designs
//! anything: `mutation.rs` still sees only `A`, `C`, `G`, `T`.
//!
//! What the expansion decides, in the order the steps below carry out:
//!
//! * the member order is `expand_degenerate`'s — each position's bases in
//!   `A`, `C`, `G`, `T` order, the leftmost position varying fastest, so `NS`
//!   gives `AC, CC, GC, TC, AG, CG, GG, TG`. Nothing here re-derives that
//!   order or sorts it;
//! * a concrete sequence equal to the bases the region already holds would be
//!   a no-op edit, and is skipped with a warning naming it (D033's wild-type
//!   addendum). An expansion with nothing left after skipping is refused as
//!   the no-op it is, rather than as an empty set, which is a set-level rule
//!   of `q5.rs` this module does not restate;
//! * past `EXPANSION_WARNING_THRESHOLD` emitted members the expansion carries
//!   a `ManyDesigns` warning and every member is still emitted — there is no
//!   cap (D033's expansion count, warned rather than refused the way D032's
//!   100-nt limit is treated);
//! * the count that warns is the number emitted *after* skipping, and a
//!   `ManyDesigns` warning follows any `WildTypeSkipped` ones.
//!
//! Naming is D033's: the caller's base name, a hyphen, and the concrete
//! sequence, so `mut-1-AC`. Each name is validated by `Identifier::create`,
//! and the sequences are distinct by construction, which is what the variant
//! set's existing duplicate-name rule needs — no rule is restated here.
//!
//! Sets do not nest: a degenerate replacement inside an existing variant set
//! is refused by `mutation.rs` before this module could see it, so there is
//! nothing here to add and a test pins that it holds.
//!
//! As in `q5.rs` and `q5_design.rs`, these records are public structs with
//! private fields and no public constructor, so nothing outside this module
//! can build one, valid or invalid, and `expand_replacement` is the only
//! producer. The types do not make an invalid pairing unrepresentable; that
//! is stated rather than assumed.

use crate::error::DomainError;
use crate::error::InvalidMutationRequestProblem;
use crate::identity::Identifier;
use crate::interval::NucleotideInterval;
use crate::mutation::NucleotideReplacementIntent;
use crate::q5::Q5Composition;
use crate::q5::Q5MutationIntent;
use crate::q5::Q5SdmRequest;
use crate::q5::Q5Variant;
use crate::q5::Q5VariantSetRequest;
use crate::sequence::expand_degenerate;
use crate::sequence::Sequence;

/// D033: past this many emitted members an expansion carries a warning.
pub const EXPANSION_WARNING_THRESHOLD: usize = 64;

/// Something the expansion wants to say about itself, and nothing else.
///
/// Neither is a failure: a wild-type member is a no-op edit that is left out,
/// and a large expansion is a request the caller may not have counted. Every
/// member that can be emitted is emitted in both cases, and the warning is
/// carried beside the set rather than folded into it.
#[derive(Clone, Debug, PartialEq)]
pub enum Q5ExpansionWarning {
    /// The expansion could write the bases the region already holds, so that
    /// member edits nothing and is left out.
    WildTypeSkipped { sequence: String },
    /// More members were emitted than `EXPANSION_WARNING_THRESHOLD`, so the
    /// caller is being asked to read a long set.
    ManyDesigns { count: usize, threshold: usize },
}

impl Q5ExpansionWarning {
    /// Reporting text: what was skipped, or how many designs there are.
    pub fn describe(&self) -> String {
        match self {
            Q5ExpansionWarning::WildTypeSkipped { sequence } => {
                format!(
                    "{} equals the bases already in the region and was skipped",
                    sequence,
                )
            }
            Q5ExpansionWarning::ManyDesigns { count, threshold } => {
                format!(
                    "{} designs, more than {}; every one is still emitted",
                    count, threshold,
                )
            }
        }
    }
}

/// A degenerate replacement as the variant-set layer reads it: the members,
/// and what the expansion wants to say about producing them.
#[derive(Clone, Debug, PartialEq)]
pub struct Q5Expansion {
    set: Q5VariantSetRequest,
    warnings: Vec<Q5ExpansionWarning>,
}

impl Q5Expansion {
    /// The set, ready for `q5::execute_set` and `q5_design::design_set`.
    pub fn set(&self) -> Q5VariantSetRequest {
        self.set.clone()
    }

    /// The warnings, in the order the expansion raised them: any skipped
    /// member first, then the count.
    pub fn warnings(&self) -> Vec<Q5ExpansionWarning> {
        self.warnings.clone()
    }
}

/// Expand a degenerate replacement into one member per concrete sequence.
///
/// The steps are the contract's, in this order, and each one is reached
/// through the module that owns the rule rather than re-derived here:
///
/// 1. an empty replacement text is refused before anything is expanded, as
///    `InvalidMutationRequest` (`MalformedInput`);
/// 2. `sequence::expand_degenerate` gives the concrete sequences, in its own
///    order, and an invalid letter is its own `InvalidSequenceCharacters`,
///    propagated unchanged;
/// 3. each sequence equal to `expected_original` is skipped with a
///    `WildTypeSkipped` warning naming it; every other one becomes an
///    ordinary `NucleotideReplacementIntent` over `region`, a
///    `Q5SdmRequest` carrying the caller's scope, and a `Q5Variant` named
///    `base_name-concrete`;
/// 4. no member left is refused as the no-op it is, with the same
///    `InvalidMutationRequest`;
/// 5. more members than `EXPANSION_WARNING_THRESHOLD` add a `ManyDesigns`
///    warning after any `WildTypeSkipped` ones, and every member is kept;
/// 6. the members are composed as `SeparateVariants`, which is what
///    `Q5VariantSetRequest::create` then checks as it stands: one template
///    artifact across the set, agreeing CDS claims, distinct result names.
///
/// Nothing here runs an edit or designs a pair: the set is returned as a
/// request, and `mutation.rs` never sees an ambiguity code.
pub fn expand_replacement(
    template: Sequence,
    scope: Option<Identifier>,
    region: NucleotideInterval,
    expected_original: String,
    degenerate: String,
    base_name: Identifier,
) -> Result<Q5Expansion, DomainError> {
    if degenerate.is_empty() {
        let problem = InvalidMutationRequestProblem {
            request: "q5-degenerate-replacement".to_string(),
            reason: "a degenerate replacement must write at least one base".to_string(),
        };
        return Err(DomainError::InvalidMutationRequest(problem));
    }

    // The concrete sequences, in `expand_degenerate`'s order: each position's
    // bases in `A`, `C`, `G`, `T` order, leftmost position varying fastest. An
    // invalid letter is its `InvalidSequenceCharacters`, returned as it stands.
    let concrete: Vec<String> = expand_degenerate(&degenerate)?;

    let mut variants: Vec<Q5Variant> = Vec::new();
    let mut warnings: Vec<Q5ExpansionWarning> = Vec::new();
    for sequence in concrete.iter() {
        if *sequence == expected_original {
            // D033's wild-type member: writing back what is already there
            // edits nothing, so it is left out and named.
            warnings.push(Q5ExpansionWarning::WildTypeSkipped {
                sequence: sequence.clone(),
            });
            continue;
        }

        // An ordinary replacement, with the region's own bases asserted:
        // `mutation.rs` sees no ambiguity code anywhere.
        let intent: NucleotideReplacementIntent = NucleotideReplacementIntent::create(
            template.reference(),
            region.clone(),
            expected_original.clone(),
            sequence.clone(),
        )?;
        let request: Q5SdmRequest = Q5SdmRequest::create(
            template.clone(),
            scope.clone(),
            Q5MutationIntent::Replacement(intent),
        )?;
        let result: Identifier = Identifier::create(format!("{}-{}", base_name.value(), sequence))?;
        variants.push(Q5Variant::create(request, result));
    }

    if variants.is_empty() {
        let problem = InvalidMutationRequestProblem {
            request: "q5-degenerate-replacement".to_string(),
            reason: format!(
                "every sequence the expansion could write is the text that is already there; the request would edit nothing: {}",
                expected_original,
            ),
        };
        return Err(DomainError::InvalidMutationRequest(problem));
    }

    // D033's expansion count: the threshold counts the members actually
    // emitted, after skipping, and a warning rather than a refusal.
    if variants.len() > EXPANSION_WARNING_THRESHOLD {
        warnings.push(Q5ExpansionWarning::ManyDesigns {
            count: variants.len(),
            threshold: EXPANSION_WARNING_THRESHOLD,
        });
    }

    let set: Q5VariantSetRequest =
        Q5VariantSetRequest::create(Q5Composition::SeparateVariants, variants.clone())?;
    Ok(Q5Expansion { set, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cds::Cds;
    use crate::cds::Completeness;
    use crate::cds::Numbering;
    use crate::error::DomainError;
    use crate::error::ErrorCategory;
    use crate::identity::Identifier;
    use crate::interval::create;
    use crate::interval::NucleotideInterval;
    use crate::mutation::NucleotideReplacementIntent;
    use crate::primer::PrimerPlacement;
    use crate::primer_design::DesignPolicy;
    use crate::primer_design::DesignStyle;
    use crate::primer_design::PrimerDesignRequest;
    use crate::q5::execute_set;
    use crate::q5::Q5Variant;
    use crate::q5::Q5VariantSetExecution;
    use crate::q5_design::design_set;
    use crate::q5_design::Q5DesignedSet;
    use crate::sequence::normalize;
    use crate::sequence::SequenceTopology;
    use crate::strand::Strand;
    use crate::tm::BufferModel;

    // The 30-nucleotide artifact `q5.rs`'s tests use, so the region texts
    // below are plain string slices of it (planner, 2026-10-15): `[3, 5)` is
    // `GC`, `[9, 11)` is `AA`, `[9, 12)` is `AAT`, `[9, 13)` is `AATC`.
    fn template() -> Sequence {
        normalize(
            Identifier::create("q5-a".to_string()).unwrap(),
            "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    fn name(value: &str) -> Identifier {
        Identifier::create(value.to_string()).unwrap()
    }

    fn region(start: i64, end: i64) -> NucleotideInterval {
        create(&template().reference(), start, end).unwrap()
    }

    fn base_name() -> Identifier {
        name("mut-1")
    }

    fn member_names(set: Q5VariantSetRequest) -> Vec<String> {
        set.variants()
            .iter()
            .map(|variant: &Q5Variant| variant.result_name().value().to_string())
            .collect()
    }

    // X1: `NS` over the region at `[9, 11)`, whose bases are `AA`. The eight
    // concrete sequences are `expand_degenerate`'s own order, which its tests
    // pin: `AC, CC, GC, TC, AG, CG, GG, TG`.
    #[test]
    fn ns_expands_to_eight_named_members_in_order() {
        let expansion: Q5Expansion = expand_replacement(
            template(),
            None,
            region(9, 11),
            "AA".to_string(),
            "NS".to_string(),
            base_name(),
        )
        .unwrap();
        let set: Q5VariantSetRequest = expansion.set();
        assert!(set.composition().is_separate_variants());
        assert!(
            member_names(set)
                == vec![
                    "mut-1-AC".to_string(),
                    "mut-1-CC".to_string(),
                    "mut-1-GC".to_string(),
                    "mut-1-TC".to_string(),
                    "mut-1-AG".to_string(),
                    "mut-1-CG".to_string(),
                    "mut-1-GG".to_string(),
                    "mut-1-TG".to_string(),
                ]
        );
        assert!(expansion.warnings().is_empty());
    }

    // X2: the region at `[3, 5)` already holds `GC`, which `NS` can write, so
    // that member is a no-op edit and is left out. The other seven keep their
    // relative order.
    #[test]
    fn a_wild_type_member_is_skipped_with_a_warning() {
        let expansion: Q5Expansion = expand_replacement(
            template(),
            None,
            region(3, 5),
            "GC".to_string(),
            "NS".to_string(),
            base_name(),
        )
        .unwrap();
        assert!(
            member_names(expansion.set())
                == vec![
                    "mut-1-AC".to_string(),
                    "mut-1-CC".to_string(),
                    "mut-1-TC".to_string(),
                    "mut-1-AG".to_string(),
                    "mut-1-CG".to_string(),
                    "mut-1-GG".to_string(),
                    "mut-1-TG".to_string(),
                ]
        );
        assert!(
            expansion.warnings()
                == vec![Q5ExpansionWarning::WildTypeSkipped {
                    sequence: "GC".to_string(),
                },]
        );
    }

    // X3: three fully ambiguous positions give 4 × 4 × 4 = 64 members, and
    // 64 is not past the threshold. None of them is `AA`: every concrete
    // sequence here is three bases long, the region two.
    #[test]
    fn sixty_four_members_do_not_warn() {
        let expansion: Q5Expansion = expand_replacement(
            template(),
            None,
            region(9, 11),
            "AA".to_string(),
            "NNN".to_string(),
            base_name(),
        )
        .unwrap();
        assert!(expansion.set().variant_count() == 64);
        assert!(expansion.warnings().is_empty());
    }

    // X4: the count is taken after skipping. `[9, 12)` holds `AAT`, which
    // `NNN` can write, so 64 expansions leave 63 members — at the threshold,
    // not past it — and the only warning names the skipped sequence.
    #[test]
    fn the_count_is_taken_after_skipping() {
        let expansion: Q5Expansion = expand_replacement(
            template(),
            None,
            region(9, 12),
            "AAT".to_string(),
            "NNN".to_string(),
            base_name(),
        )
        .unwrap();
        assert!(expansion.set().variant_count() == 63);
        assert!(
            expansion.warnings()
                == vec![Q5ExpansionWarning::WildTypeSkipped {
                    sequence: "AAT".to_string(),
                },]
        );
    }

    // X5: `NNNS` is 4 × 4 × 4 × 2 = 128 members, all of them four bases long,
    // so none equals the two bases already at `[9, 11)`. Past the threshold
    // the expansion warns, and every member is still emitted.
    #[test]
    fn more_than_sixty_four_warns_and_still_emits_every_member() {
        let expansion: Q5Expansion = expand_replacement(
            template(),
            None,
            region(9, 11),
            "AA".to_string(),
            "NNNS".to_string(),
            base_name(),
        )
        .unwrap();
        assert!(expansion.set().variant_count() == 128);
        assert!(
            expansion.warnings()
                == vec![Q5ExpansionWarning::ManyDesigns {
                    count: 128,
                    threshold: 64,
                },]
        );
    }

    // X6: `[9, 13)` holds `AATC`, which `NNNN` can write, so 256 expansions
    // leave 255 members. Both warnings are raised, and the count follows the
    // skipped member rather than coming first.
    #[test]
    fn warnings_are_ordered() {
        let expansion: Q5Expansion = expand_replacement(
            template(),
            None,
            region(9, 13),
            "AATC".to_string(),
            "NNNN".to_string(),
            base_name(),
        )
        .unwrap();
        assert!(expansion.set().variant_count() == 255);
        assert!(
            expansion.warnings()
                == vec![
                    Q5ExpansionWarning::WildTypeSkipped {
                        sequence: "AATC".to_string(),
                    },
                    Q5ExpansionWarning::ManyDesigns {
                        count: 255,
                        threshold: 64,
                    },
                ]
        );
    }

    // X7: a text with no ambiguity code expands to itself, so the expansion is
    // a one-member set — the same shape a caller would have built by hand.
    #[test]
    fn a_concrete_replacement_is_a_one_member_set() {
        let expansion: Q5Expansion = expand_replacement(
            template(),
            None,
            region(9, 11),
            "AA".to_string(),
            "GG".to_string(),
            base_name(),
        )
        .unwrap();
        assert!(member_names(expansion.set()) == vec!["mut-1-GG".to_string()]);
        assert!(expansion.warnings().is_empty());
    }

    // X8: the only sequence the expansion could write is the one already in
    // the region, so the request would edit nothing. It is refused as the no-op
    // it is, not as the empty set `q5.rs` refuses.
    #[test]
    fn an_expansion_with_nothing_left_is_refused() {
        let attempt: Result<Q5Expansion, DomainError> = expand_replacement(
            template(),
            None,
            region(9, 12),
            "AAT".to_string(),
            "AAT".to_string(),
            base_name(),
        );
        assert!(attempt.is_err());
        let error: DomainError = attempt.unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        if let DomainError::InvalidMutationRequest(problem) = error {
            assert!(problem.request == "q5-degenerate-replacement");
            assert!(problem.reason.contains("already there"));
        }
    }

    // X9: sets do not nest. A degenerate replacement inside an existing
    // variant set is refused where the intent is built, before this module
    // could expand anything — `mutation.rs` does not loosen.
    #[test]
    fn a_degenerate_member_cannot_be_built_directly() {
        let attempt: Result<NucleotideReplacementIntent, DomainError> =
            NucleotideReplacementIntent::create(
                template().reference(),
                region(9, 11),
                "AA".to_string(),
                "NS".to_string(),
            );
        assert!(attempt.is_err());
        let error: DomainError = attempt.unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    // X10: an empty text has nothing to expand, and `X` is outside the
    // alphabet `sequence.rs` knows. Each refusal is the one its own owner
    // raises, with its own variant and category.
    #[test]
    fn an_empty_or_invalid_replacement_text_is_refused() {
        let empty: Result<Q5Expansion, DomainError> = expand_replacement(
            template(),
            None,
            region(9, 11),
            "AA".to_string(),
            "".to_string(),
            base_name(),
        );
        assert!(empty.is_err());
        let error: DomainError = empty.unwrap_err();
        assert!(matches!(error, DomainError::InvalidMutationRequest(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        if let DomainError::InvalidMutationRequest(problem) = error {
            assert!(problem.request == "q5-degenerate-replacement");
            assert!(problem.reason.contains("at least one"));
        }

        let invalid: Result<Q5Expansion, DomainError> = expand_replacement(
            template(),
            None,
            region(9, 11),
            "AA".to_string(),
            "NX".to_string(),
            base_name(),
        );
        assert!(invalid.is_err());
        let error: DomainError = invalid.unwrap_err();
        assert!(matches!(error, DomainError::InvalidSequenceCharacters(_)));
        assert!(error.category() == ErrorCategory::MalformedInput);
        if let DomainError::InvalidSequenceCharacters(problem) = error {
            assert!(problem.sequence == "replacement");
            assert!(problem.position == 1);
            assert!(problem.characters == "X");
        }
    }

    // X11: the caller's scope reaches every member, so the set resolves one
    // CDS rather than asking each member on its own (D016).
    #[test]
    fn the_scope_reaches_every_member() {
        let expansion: Q5Expansion = expand_replacement(
            template(),
            Some(name("cds-1")),
            region(9, 11),
            "AA".to_string(),
            "NS".to_string(),
            base_name(),
        )
        .unwrap();
        let wanted: Option<Identifier> = Some(name("cds-1"));
        let members: Vec<Q5Variant> = expansion.set().variants();
        assert!(members.len() == 8);
        for variant in members.iter() {
            assert!(variant.request().scope() == wanted);
        }
        assert!(expansion.set().target_cds_name() == wanted);
    }

    // X12: an expansion runs and designs end to end. The 90-base template and
    // its whole-length CDS are `q5_design.rs`'s, so the pair each member
    // reaches is one those fixtures already pin the shape of; `TGS` expands to
    // `TGC` and `TGG`, the last position varying because the two before it are
    // fixed. Both replacements are three bases over a three-base region, so
    // neither changes the reading frame and both are designed.
    fn long_text() -> String {
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

    fn long_template() -> Sequence {
        normalize(
            Identifier::create("q5d-a".to_string()).unwrap(),
            long_text(),
            SequenceTopology::Linear,
        )
        .unwrap()
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

    #[test]
    fn an_expansion_designs_end_to_end() {
        let expansion: Q5Expansion = expand_replacement(
            long_template(),
            None,
            create(&long_template().reference(), 45, 48).unwrap(),
            "ATG".to_string(),
            "TGS".to_string(),
            name("r"),
        )
        .unwrap();
        assert!(expansion.warnings().is_empty());
        assert!(member_names(expansion.set()) == vec!["r-TGC".to_string(), "r-TGG".to_string(),]);

        let run: Q5VariantSetExecution =
            execute_set(expansion.set(), vec![cds_of(&long_template(), "cds-1")]).unwrap();
        assert!(!run.needs_clarification());
        assert!(run.result_count() == 2);

        let designed: Q5DesignedSet = design_set(run, design_request(15)).unwrap();
        assert!(designed.designed_count() == 2);
        assert!(designed.all_designed());
    }
}
