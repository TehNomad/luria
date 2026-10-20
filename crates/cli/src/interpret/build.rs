//! From what a model says the user asked for to a request, questions, or a
//! string the user never wrote (contract 6e2, plan §3's trust boundary): one
//! pure function, `build`, over the user's own text, the model's `Extraction`
//! (6a), the record's CDS catalogue (6c) and the user's design profile (6d).
//!
//! It ends in exactly one of three outcomes (plan §1):
//!
//! - `Built::Ungrounded`: the model reported a string the user never wrote.
//!   Grounding is step 1, before every question, so a fabricated string never
//!   earns a question about something else: nothing the model said is trusted.
//! - `Built::NeedsClarification`: what a request would need that neither the
//!   text nor the profile states, supports or leaves doubtful. Every one is
//!   collected in the order of §3.4 step 3 and returned all at once, each
//!   with the stable code of `QuestionCode`.
//! - `Built::Proposed`: a complete `RequestDocument` built here, with the
//!   origin of every value that came from the text or the profile (`Origin`)
//!   and the assumptions the user should check (`Assumption`).
//!
//! The steps, in order (§3.4): grounding, then the operation question, then
//! every other question, then the proposal. Nothing is defaulted (D028): a
//! value neither the text nor the profile states is a question, and a CDS the
//! text does not name is never chosen (M6-9). The evidence rules are 6e1's
//! and the notation rules are 6b's, called and never re-implemented; the
//! catalogue's usability is core's, and a refusal is reported as its code.
//!
//! What is not judged here, by design (D003): whether the residue at the
//! number really is the stated original, whether a codon encodes the target
//! or is a codon at all, whether two members repeat, and whether the LOCUS
//! name is a valid identifier. `luria q5 design` checks all of them on the
//! proposal. Nothing here touches a model, a file, the network, the
//! environment or the clock, and the same inputs give the same `Built`
//! (D012).

use crate::interpret::catalogue::CatalogueEntry;
use crate::interpret::catalogue::Usability;
use crate::interpret::evidence::digit_runs;
use crate::interpret::evidence::evidence;
use crate::interpret::evidence::numbering_from_cds_start;
use crate::interpret::notation::parse_substitution;
use crate::interpret::notation::NotationError;
use crate::interpret::notation::Part;
use crate::interpret::notation::Substitution;
use crate::request::CodonSpec;
use crate::request::CompositionSpec;
use crate::request::EditSpec;
use crate::request::MemberSpec;
use crate::request::OperationSpec;
use crate::request::ProfileDocument;
use crate::request::RequestDocument;
use crate::request::TemplateSpec;
use luria_nl::extraction::Composition;
use luria_nl::extraction::Extraction;
use luria_nl::extraction::Operation;

/// What the request will say about its template (plan §5.5).
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateFacts {
    /// The record's LOCUS name: the template's id (M6-10).
    pub locus_name: String,
    /// The template path as the request will write it.
    pub path: String,
    /// The SHA-256 of the record's normalized sequence.
    pub sha256: String,
}

/// Why luria is asking (plan §5.5, §5.3a). Each has a stable code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestionCode {
    OperationUnsupported,
    MutationUnstated,
    MutationNotationUnsupported,
    ResidueNumberUnstated,
    OriginalResidueUnstated,
    TargetResidueUnstated,
    SubstitutionDirectionUnclear,
    MutationQuoteAmbiguous,
    CodonUnstated,
    CdsUnnamed,
    CdsNotFound,
    CdsUnusable,
    CompositionUnstated,
    NumberingUnstated,
    NumberingUnsupported,
    DesignUnstated,
}

impl QuestionCode {
    /// The snake_case code: `operation_unsupported`, `mutation_unstated`,
    /// and so on, one per variant, the variant name in snake case.
    pub fn code(self) -> &'static str {
        match self {
            QuestionCode::OperationUnsupported => "operation_unsupported",
            QuestionCode::MutationUnstated => "mutation_unstated",
            QuestionCode::MutationNotationUnsupported => "mutation_notation_unsupported",
            QuestionCode::ResidueNumberUnstated => "residue_number_unstated",
            QuestionCode::OriginalResidueUnstated => "original_residue_unstated",
            QuestionCode::TargetResidueUnstated => "target_residue_unstated",
            QuestionCode::SubstitutionDirectionUnclear => "substitution_direction_unclear",
            QuestionCode::MutationQuoteAmbiguous => "mutation_quote_ambiguous",
            QuestionCode::CodonUnstated => "codon_unstated",
            QuestionCode::CdsUnnamed => "cds_unnamed",
            QuestionCode::CdsNotFound => "cds_not_found",
            QuestionCode::CdsUnusable => "cds_unusable",
            QuestionCode::CompositionUnstated => "composition_unstated",
            QuestionCode::NumberingUnstated => "numbering_unstated",
            QuestionCode::NumberingUnsupported => "numbering_unsupported",
            QuestionCode::DesignUnstated => "design_unstated",
        }
    }
}

/// One question: its code, what it is about, and one line for a person.
#[derive(Clone, Debug, PartialEq)]
pub struct Question {
    pub code: QuestionCode,
    pub subject: String,
    pub message: String,
}

/// Where a value of a proposed request came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Text,
    Profile,
}

/// One value of a proposed request and where it came from (plan §5.4).
#[derive(Clone, Debug, PartialEq)]
pub struct Origin {
    pub value: String,
    pub source: Source,
}

/// Something luria took as given in a proposal, for the user to check.
#[derive(Clone, Debug, PartialEq)]
pub struct Assumption {
    pub code: String,
    pub message: String,
}

/// A string the model reported that the text does not contain. `field` is
/// its place in the extraction: `cds`, `mutations[1].quote`,
/// `mutations[0].codon`, `numbering` or `unsupported[0]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Ungrounded {
    pub field: String,
    pub value: String,
}

/// How an interpretation ends (plan §1).
#[derive(Clone, Debug, PartialEq)]
// The request is not boxed, unlike `prepare.rs`: a caller of this adapter
// matches on `Built::Proposed` and compares the request it binds with a
// `RequestDocument` parsed from a fixture (plan §4).
#[allow(clippy::large_enum_variant)]
pub enum Built {
    Proposed {
        request: RequestDocument,
        origins: Vec<Origin>,
        assumptions: Vec<Assumption>,
    },
    NeedsClarification {
        questions: Vec<Question>,
    },
    Ungrounded(Ungrounded),
}

/// Whether luria can use this CDS, as the catalogue says (6c).
fn is_usable(usability: &Usability) -> bool {
    match usability {
        Usability::Usable { .. } => true,
        Usability::Refused { .. } => false,
    }
}

/// Why the catalogue cannot use this CDS, as §3.4 writes it: the refusal's
/// `code`, or `code:rule` when it has a rule. `None` when it can.
fn refusal_of(entry: &CatalogueEntry) -> Option<String> {
    let usability: Usability = entry.usability.clone();
    match usability {
        Usability::Usable { .. } => None,
        Usability::Refused {
            code,
            rule: Some(rule),
        } => Some(format!("{}:{}", code.clone(), rule.clone())),
        Usability::Refused { code, rule: None } => Some(code.clone()),
    }
}

/// Whether the text states `value` (§3.4 step 1): not blank, and an exact
/// substring of the user's own text.
fn stated_in(text: &str, value: &str) -> bool {
    !value.trim().is_empty() && text.contains(value)
}

/// Whether the text states a codon: like `stated_in`, and the one string
/// step 1 compares ASCII-case-insensitively, since a user writes a codon
/// either way (§3.4 step 1).
fn stated_codon_in(text: &str, codon: &str) -> bool {
    !codon.trim().is_empty()
        && text
            .to_ascii_lowercase()
            .contains(&codon.to_ascii_lowercase())
}

/// Step 1: the first string the model reported that the text does not hold,
/// in the order of §3.4 step 1, named by its place in the extraction and
/// given as the model gave it. A mutation's `token` is not grounded here:
/// it is checked against its quote through 6e1's `evidence` instead.
fn first_ungrounded(text: &str, extraction: &Extraction) -> Option<Ungrounded> {
    let named: Option<String> = extraction.cds.clone();
    if let Some(cds) = named {
        if !stated_in(text, cds.as_str()) {
            return Some(Ungrounded {
                field: "cds".to_string(),
                value: cds.clone(),
            });
        }
    }
    for pair in extraction.mutations.iter().enumerate() {
        let index: usize = pair.0;
        let mutation = pair.1.clone();
        if !stated_in(text, mutation.quote.as_str()) {
            return Some(Ungrounded {
                field: format!("mutations[{}].quote", index),
                value: mutation.quote.clone(),
            });
        }
        if let Some(codon) = mutation.codon {
            if !stated_codon_in(text, codon.as_str()) {
                return Some(Ungrounded {
                    field: format!("mutations[{}].codon", index),
                    value: codon.clone(),
                });
            }
        }
    }
    let numbering: Option<String> = extraction.numbering.clone();
    if let Some(words) = numbering {
        if !stated_in(text, words.as_str()) {
            return Some(Ungrounded {
                field: "numbering".to_string(),
                value: words.clone(),
            });
        }
    }
    for pair in extraction.unsupported.iter().enumerate() {
        let quote: String = pair.1.clone();
        if !stated_in(text, quote.as_str()) {
            return Some(Ungrounded {
                field: format!("unsupported[{}]", pair.0),
                value: quote.clone(),
            });
        }
    }
    None
}

/// Step 3f's question, and step 4's guard for it: no profile, or one that
/// states no design (§3.4 step 3f).
fn design_unstated() -> Question {
    Question {
        code: QuestionCode::DesignUnstated,
        subject: "design".to_string(),
        message: "no design profile states the primer design, and luria does not default it"
            .to_string(),
    }
}

/// Whether the profile states a design, `false` with no profile.
fn stated_design(profile: Option<&ProfileDocument>) -> bool {
    match profile {
        None => false,
        Some(document) => document.design.is_some(),
    }
}

/// The profile's codon policy, `None` with no profile or none stated.
fn profile_codon(profile: Option<&ProfileDocument>) -> Option<CodonSpec> {
    match profile {
        None => None,
        Some(document) => document.codon.clone(),
    }
}

/// Whether the profile states a numbering, `false` with no profile.
fn stated_numbering(profile: Option<&ProfileDocument>) -> bool {
    match profile {
        None => false,
        Some(document) => document.numbering.is_some(),
    }
}

/// Step 3a: the request names no substitution at all (§3.4 step 3a).
fn mutation_unstated() -> Question {
    Question {
        code: QuestionCode::MutationUnstated,
        subject: "mutations".to_string(),
        message: "the request names no amino-acid substitution".to_string(),
    }
}

/// Step 3d: two or more substitutions, and the text never said whether they
/// are separate variants or one construct (§3.4 step 3d).
fn composition_unstated() -> Question {
    Question {
        code: QuestionCode::CompositionUnstated,
        subject: "composition".to_string(),
        message: "the text does not say whether the substitutions are separate variants or one combined construct".to_string(),
    }
}

/// Step 3e: the text does not say how residues are numbered and the profile
/// does not state it (§3.4 step 3e).
fn numbering_unstated() -> Question {
    Question {
        code: QuestionCode::NumberingUnstated,
        subject: "numbering".to_string(),
        message: "the text does not say how residues are numbered, and the design profile does not state it".to_string(),
    }
}

/// Step 3c: the text names no CDS, and `listed` names the CDSs luria can
/// use (§3.4 step 3c).
fn cds_unnamed(listed: String) -> Question {
    Question {
        code: QuestionCode::CdsUnnamed,
        subject: "cds".to_string(),
        message: format!(
            "the text names no CDS; the CDSs luria can use are: {}",
            listed.clone()
        ),
    }
}

/// Step 3b's codon question: `token` as the model gave it, about the
/// substitution `name` in its canonical form (§3.4 step 3b).
fn codon_unstated(token: String, name: String) -> Question {
    Question {
        code: QuestionCode::CodonUnstated,
        subject: token.clone(),
        message: format!(
            "the text names no codon for {}, and the design profile sets no codon policy",
            name.clone()
        ),
    }
}

/// One question, alone: step 4's guard asks the question step 3 would have
/// asked for a value that is somehow missing (§3.4 step 4).
fn clarify(question: Question) -> Built {
    Built::NeedsClarification {
        questions: vec![question],
    }
}

/// Step 3b's question for one part of the substitution the quote does not
/// show, or that a `?` stood for: the same three messages of §3.4 either way.
fn part_unstated(token: String, quote: String, part: Part) -> Question {
    match part {
        Part::Original => Question {
            code: QuestionCode::OriginalResidueUnstated,
            subject: token.clone(),
            message: format!(
                "\"{}\" does not state the residue there now, which luria checks against the CDS rather than assume",
                quote.clone(),
            ),
        },
        Part::Number => Question {
            code: QuestionCode::ResidueNumberUnstated,
            subject: token.clone(),
            message: format!("\"{}\" does not state the residue number", quote.clone()),
        },
        Part::Target => Question {
            code: QuestionCode::TargetResidueUnstated,
            subject: token.clone(),
            message: format!("\"{}\" does not state the residue to put there", quote.clone()),
        },
    }
}

/// The composition a variant set states, `None` when the text said nothing.
fn composition_of(stated: Option<Composition>) -> Option<CompositionSpec> {
    stated.map(|each| match each {
        Composition::SeparateVariants => CompositionSpec::SeparateVariants,
        Composition::CombinedConstruct => CompositionSpec::CombinedConstruct,
    })
}

#[derive(Clone, Debug, PartialEq)]
/// Step 3's reading of one mutation, in extraction order: the questions it
/// earned, and, when its token is a substitution luria reads, that
/// substitution and the codon the mutation states. Step 4 answers with the
/// questions of any mutation it cannot read, so nothing in it unwraps.
struct MutationRead {
    questions: Vec<Question>,
    token: String,
    substitution: Option<Substitution>,
    codon: Option<String>,
}

/// Step 3b: the questions one mutation earns, in the order of §3.4, and its
/// substitution when its token is one luria reads (`None` when the questions
/// say why it is not, which is what step 4's guard answers with). A quote
/// with more than one number is asked about as `mutation_quote_ambiguous`
/// (D045), and its number, original, target and direction checks are skipped.
/// The codon check runs in both cases.
fn read_mutation(
    token: String,
    quote: String,
    codon: Option<String>,
    profile: Option<&ProfileDocument>,
) -> MutationRead {
    let mut asked: Vec<Question> = Vec::new();
    let read: Option<Substitution> = match parse_substitution(&token) {
        Ok(substitution) => {
            // D045 (B-1): a quote holding more than one digit run is asked
            // about as one question, and its number, original, target and
            // direction checks are skipped.
            if digit_runs(&quote).len() >= 2 {
                asked.push(Question {
                    code: QuestionCode::MutationQuoteAmbiguous,
                    subject: token.clone(),
                    message: format!(
                        "\"{}\" holds more than one number, so it does not show which is the residue number",
                        quote.clone(),
                    ),
                });
            } else {
                // `shown` is `crate::interpret::evidence::Evidence`: what the
                // quote shows of this substitution, under 6e1's rules.
                let shown = evidence(&quote, &substitution);
                if !shown.number {
                    asked.push(part_unstated(token.clone(), quote.clone(), Part::Number));
                }
                if !shown.original {
                    asked.push(part_unstated(token.clone(), quote.clone(), Part::Original));
                }
                if !shown.target {
                    asked.push(part_unstated(token.clone(), quote.clone(), Part::Target));
                }
                if shown.original && shown.target && !shown.in_order {
                    asked.push(Question {
                        code: QuestionCode::SubstitutionDirectionUnclear,
                        subject: token.clone(),
                        message: format!(
                            "\"{}\" does not name the original residue, then the number, then the target",
                            quote.clone(),
                        ),
                    });
                }
            }
            if codon.is_none() && profile_codon(profile).is_none() {
                asked.push(codon_unstated(token.clone(), substitution.canonical()));
            }
            Some(substitution)
        }
        Err(NotationError::Unsupported { .. }) => {
            asked.push(Question {
                code: QuestionCode::MutationNotationUnsupported,
                subject: token.clone(),
                message: format!(
                    "\"{}\" is not a substitution luria reads: it reads one-letter (A42G) and three-letter (Ala42Gly) residue substitutions only",
                    token.clone(),
                ),
            });
            None
        }
        Err(NotationError::Unstated { parts }) => {
            for part in parts.iter() {
                asked.push(part_unstated(token.clone(), quote.clone(), *part));
            }
            None
        }
    };
    MutationRead {
        questions: asked.clone(),
        token: token.clone(),
        substitution: read.clone(),
        codon: codon.clone(),
    }
}

pub fn build(
    text: &str,
    extraction: &Extraction,
    catalogue: &[CatalogueEntry],
    profile: Option<&ProfileDocument>,
    template: &TemplateFacts,
) -> Built {
    // Step 1: grounding, before every question (§3.4 step 1).
    if let Some(problem) = first_ungrounded(text, extraction) {
        return Built::Ungrounded(problem.clone());
    }

    // Step 2: the operation. Anything outside amino-acid substitutions, or
    // any quote the fields above cannot hold, is refused as a whole (§3.4
    // step 2).
    if extraction.operation == Operation::Other || !extraction.unsupported.is_empty() {
        let questions: Vec<Question> = match extraction.unsupported.first() {
            Some(quote) => {
                vec![
                    Question {
                        code: QuestionCode::OperationUnsupported,
                        subject: quote.clone(),
                        message: format!(
                            "luria interprets amino-acid substitutions only, and the request also asks for \"{}\"",
                            quote.clone(),
                        ),
                    }
                ]
            }
            None => {
                vec![
                    Question {
                        code: QuestionCode::OperationUnsupported,
                        subject: "operation".to_string(),
                        message: "the request does not ask for amino-acid substitutions, the only operation luria interprets".to_string(),
                    }
                ]
            }
        };
        return Built::NeedsClarification {
            questions: questions.clone(),
        };
    }

    // Step 3: every other question, collected in the order of §3.4, and what
    // step 4 needs of each mutation.
    let mut asked: Vec<Question> = Vec::new();
    let mut reads: Vec<MutationRead> = Vec::new();

    // The usable list: the ids of the catalogue's `Usable` entries, in
    // catalogue order, joined by `, `, or `none` when there are none.
    let mut offered: Vec<String> = Vec::new();
    for entry in catalogue.iter() {
        if is_usable(&entry.usability) {
            offered.push(entry.id.clone());
        }
    }
    let sep: String = ", ".to_string();
    let listed: String = if offered.is_empty() {
        "none".to_string()
    } else {
        offered.join(&sep)
    };

    // 3a: the request names no substitution at all.
    if extraction.mutations.is_empty() {
        asked.push(mutation_unstated());
    }

    // 3b: each mutation in order.
    for mutation in extraction.mutations.iter() {
        let reading: MutationRead = read_mutation(
            mutation.token.clone(),
            mutation.quote.clone(),
            mutation.codon.clone(),
            profile,
        );
        asked.extend(reading.questions.clone());
        reads.push(reading.clone());
    }

    // 3c: the CDS, matched exactly against the catalogue: never one the
    // text does not name, and never one core cannot use (M6-9, D042).
    let named: Option<String> = extraction.cds.clone();
    match named.clone() {
        None => {
            asked.push(cds_unnamed(listed.clone()));
        }
        Some(name) => {
            let mut usable_here: bool = false;
            let mut found_here: bool = false;
            let mut reason: Option<String> = None;
            for entry in catalogue.iter() {
                if entry.id == name.clone() {
                    found_here = true;
                    if is_usable(&entry.usability) {
                        usable_here = true;
                    } else if reason.is_none() {
                        reason = refusal_of(entry);
                    }
                }
            }
            if usable_here {
                // The record has this CDS and luria can use it: no question.
            } else if !found_here {
                asked.push(Question {
                    code: QuestionCode::CdsNotFound,
                    subject: name.clone(),
                    message: format!(
                        "the record has no CDS named \"{}\"; the CDSs luria can use are: {}",
                        name.clone(),
                        listed.clone(),
                    ),
                });
            } else {
                if let Some(text) = reason {
                    asked.push(Question {
                        code: QuestionCode::CdsUnusable,
                        subject: name.clone(),
                        message: format!(
                            "luria cannot use the CDS \"{}\": {}",
                            name.clone(),
                            text.clone(),
                        ),
                    });
                }
            }
        }
    }

    // 3d: two or more substitutions and no statement of the composition.
    if extraction.mutations.len() >= 2 && extraction.composition.is_none() {
        asked.push(composition_unstated());
    }

    // 3e: the numbering - the text first, the profile second (§5.5).
    let numbering_words: Option<String> = extraction.numbering.clone();
    match numbering_words {
        Some(words) => {
            if !numbering_from_cds_start(words.as_str()) {
                asked.push(Question {
                    code: QuestionCode::NumberingUnsupported,
                    subject: words.clone(),
                    message: format!(
                        "luria numbers residues from the start of the CDS only, and the text says \"{}\"",
                        words.clone(),
                    ),
                });
            }
        }
        None => {
            if !stated_numbering(profile) {
                asked.push(numbering_unstated());
            }
        }
    }

    // 3f: the design, which nothing here defaults (D028).
    if !stated_design(profile) {
        asked.push(design_unstated());
    }

    // Any question at all: ask all of them at once, in the order collected.
    if !asked.is_empty() {
        return Built::NeedsClarification {
            questions: asked.clone(),
        };
    }

    // Step 4: the proposal. Step 3 guarantees every value below; each guard
    // keeps this total by answering with the question step 3 would have
    // asked, so nothing here unwraps (§3.4 step 4).
    let stated_design = match profile {
        None => {
            return clarify(design_unstated());
        }
        Some(document) => document.design.clone(),
    };
    let design = match stated_design {
        None => {
            return clarify(design_unstated());
        }
        Some(each) => each.clone(),
    };

    // The origins, in the order of §3.4 step 4: the design, the numbering,
    // then one codon per member.
    let mut origins: Vec<Origin> = Vec::new();
    origins.push(Origin {
        value: "design".to_string(),
        source: Source::Profile,
    });
    let numbering_source: Source = match extraction.numbering {
        Some(_) => Source::Text,
        None => match stated_numbering(profile) {
            false => {
                return clarify(numbering_unstated());
            }
            true => Source::Profile,
        },
    };
    origins.push(Origin {
        value: "numbering".to_string(),
        source: numbering_source,
    });

    // The CDS the text named, which step 3c guarantees.
    let scope: String = match named.clone() {
        None => {
            return clarify(cds_unnamed(listed.clone()));
        }
        Some(name) => name.clone(),
    };

    let mut members: Vec<MemberSpec> = Vec::new();
    for read in reads.iter() {
        match read.substitution.clone() {
            None => {
                // Step 3b asked about this token; the same questions answer
                // here (§3.4 step 4).
                return Built::NeedsClarification {
                    questions: read.questions.clone(),
                };
            }
            Some(substitution) => {
                let name: String = substitution.canonical();
                let codon = match read.codon.clone() {
                    Some(codon) => {
                        origins.push(Origin {
                            value: format!("codon {}", name.clone()),
                            source: Source::Text,
                        });
                        CodonSpec::Supplied(codon.to_ascii_uppercase())
                    }
                    None => match profile_codon(profile) {
                        None => {
                            return clarify(codon_unstated(read.token.clone(), name.clone()));
                        }
                        Some(policy) => {
                            origins.push(Origin {
                                value: format!("codon {}", name.clone()),
                                source: Source::Profile,
                            });
                            policy.clone()
                        }
                    },
                };
                members.push(MemberSpec {
                    name: name.clone(),
                    edit: EditSpec::Residue {
                        cds: Some(scope.clone()),
                        number: substitution.number,
                        from: substitution.from.symbol(),
                        to: substitution.to.symbol(),
                        codon: codon.clone(),
                    },
                });
            }
        }
    }

    // One mutation is an `edit` whatever the composition says, two or more a
    // `variants` set; a combined construct is proposed faithfully, so that
    // core, not this adapter, refuses it (M6-16).
    let operation: OperationSpec = match members.len() {
        0 => {
            return clarify(mutation_unstated());
        }
        1 => OperationSpec::Edit(members[0].clone()),
        _ => match composition_of(extraction.composition) {
            Some(spec) => OperationSpec::Variants {
                composition: spec,
                members: members.clone(),
            },
            None => {
                return clarify(composition_unstated());
            }
        },
    };

    Built::Proposed {
        request: RequestDocument {
            // The template id is the record's LOCUS name, recorded as an
            // assumption the user should check (M6-10); `sha256` is always
            // filled (§5.5).
            template: TemplateSpec::GenBank {
                id: template.locus_name.clone(),
                path: template.path.clone(),
                sha256: Some(template.sha256.clone()),
                cds: vec![scope.clone()],
            },
            scope: Some(scope.clone()),
            operation: operation.clone(),
            design: design.clone(),
        },
        origins: origins.clone(),
        assumptions: vec![Assumption {
            code: "template_id_from_locus".to_string(),
            message: format!(
                "the template id is the record's LOCUS name, {}",
                template.locus_name.clone()
            ),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpret::catalogue::Usability;
    use crate::request::BufferSpec;
    use crate::request::DesignSpec;
    use crate::request::NumberingSpec;
    use crate::request::PlacementSpec;
    use crate::request::PolicySpec;
    use crate::request::StyleSpec;
    use luria_core::strand::Strand;
    use luria_nl::extraction::parse_extraction;

    fn design() -> DesignSpec {
        DesignSpec {
            policy: PolicySpec::LuriaNative,
            style: StyleSpec::Q5Polymerase,
            min_annealing_overlap: 15,
            placement: PlacementSpec::Inline,
            buffer: BufferSpec::Q5Reaction,
        }
    }

    fn profile(codon: bool, numbering: bool) -> ProfileDocument {
        ProfileDocument {
            design: Some(design()),
            codon: if codon {
                Some(CodonSpec::Default)
            } else {
                None
            },
            numbering: if numbering {
                Some(NumberingSpec::FromCdsStart)
            } else {
                None
            },
        }
    }

    fn syna() -> Vec<CatalogueEntry> {
        vec![CatalogueEntry {
            id: "synA".to_string(),
            gene: Some("synA".to_string()),
            product: None,
            usability: Usability::Usable {
                residues: 100,
                strand: Strand::Plus,
            },
        }]
    }

    fn facts() -> TemplateFacts {
        TemplateFacts {
            locus_name: "synlin".to_string(),
            path: "synthetic-linear.gb".to_string(),
            sha256: "a8b4ddcfa617179143d428dc103aabb7b13cf27f7b323407d11db8a01105bbd7".to_string(),
        }
    }

    fn reply(json: &str) -> Extraction {
        parse_extraction(json).unwrap_or_else(|error| panic!("{}: {}", json, error.message))
    }

    fn asked(built: &Built) -> Vec<(&'static str, String)> {
        match built {
            Built::NeedsClarification { questions } => questions
                .iter()
                .map(|question| (question.code.code(), question.subject.clone()))
                .collect(),
            other => panic!("expected questions, got {:?}", other),
        }
    }

    fn messages(built: &Built) -> Vec<String> {
        match built {
            Built::NeedsClarification { questions } => questions
                .iter()
                .map(|question| question.message.clone())
                .collect(),
            other => panic!("expected questions, got {:?}", other),
        }
    }

    const NONE: &str = r#""composition":null,"numbering":null,"unsupported":[]"#;

    // The swap of plan §5.3a: synA's residue 42 really is alanine, so only
    // this check stands between "glycine 42 to alanine" and an A42G edit.
    #[test]
    fn a_swapped_token_is_asked_about_not_proposed() {
        let text: &str = "In synA, change glycine 42 to alanine. Numbering from the CDS start.";
        let built: Built = build(
            text,
            &reply(
                r#"{"operation":"q5_residue_substitutions","cds":"synA",
                "mutations":[{"token":"A42G","quote":"glycine 42 to alanine","codon":null}],
                "composition":null,"numbering":"Numbering from the CDS start","unsupported":[]}"#,
            ),
            &syna(),
            Some(&profile(true, true)),
            &facts(),
        );
        assert_eq!(
            asked(&built),
            vec![("substitution_direction_unclear", "A42G".to_string())]
        );
        assert_eq!(
            messages(&built),
            vec![
                "\"glycine 42 to alanine\" does not name the original residue, then the number, then the target"
                    .to_string()
            ]
        );
    }

    // B-1 (D045): a quote holding two numbers shows no one of them as the
    // residue number, so it is asked about and never proposed.
    #[test]
    fn a_quote_holding_two_substitutions_is_asked_about() {
        let text: &str =
            "In synA, make A37G and A42S as separate variants. Number residues from the beginning of the annotated CDS.";
        let built: Built = build(
            text,
            &reply(
                r#"{"operation":"q5_residue_substitutions","cds":"synA",
                "mutations":[{"token":"A37S","quote":"A37G and A42S","codon":null}],
                "composition":null,"numbering":"from the beginning of the annotated CDS","unsupported":[]}"#,
            ),
            &syna(),
            Some(&profile(true, true)),
            &facts(),
        );
        assert_eq!(
            asked(&built),
            vec![("mutation_quote_ambiguous", "A37S".to_string())]
        );
        assert_eq!(
            messages(&built),
            vec![
                "\"A37G and A42S\" holds more than one number, so it does not show which is the residue number"
                    .to_string()
            ]
        );
    }

    // B-2 (D045): the 42 in the CDS name ydj42 is part of the name, not the
    // residue number, and the user stated no number.
    #[test]
    fn a_number_inside_a_cds_name_is_not_the_residue_number() {
        let entries: Vec<CatalogueEntry> = vec![CatalogueEntry {
            id: "ydj42".to_string(),
            gene: Some("ydj42".to_string()),
            product: None,
            usability: Usability::Usable {
                residues: 200,
                strand: Strand::Plus,
            },
        }];
        let text: &str =
            "In ydj42, change the alanine to a glycine. Number residues from the CDS start.";
        let built: Built = build(
            text,
            &reply(
                r#"{"operation":"q5_residue_substitutions","cds":"ydj42",
                "mutations":[{"token":"A42G","quote":"In ydj42, change the alanine to a glycine","codon":null}],
                "composition":null,"numbering":"from the CDS start","unsupported":[]}"#,
            ),
            &entries,
            Some(&profile(true, true)),
            &facts(),
        );
        assert_eq!(
            asked(&built),
            vec![("residue_number_unstated", "A42G".to_string())]
        );
    }

    // D045: the number must sit between the original and the target, so a
    // quote whose number stands after both earns the direction question.
    #[test]
    fn a_number_after_both_residues_is_asked_about() {
        let text: &str =
            "In synA, change the alanine to a glycine; see figure 42. Numbering from the CDS start.";
        let built: Built = build(
            text,
            &reply(
                r#"{"operation":"q5_residue_substitutions","cds":"synA",
                "mutations":[{"token":"A42G","quote":"change the alanine to a glycine; see figure 42","codon":null}],
                "composition":null,"numbering":"Numbering from the CDS start","unsupported":[]}"#,
            ),
            &syna(),
            Some(&profile(true, true)),
            &facts(),
        );
        assert_eq!(
            asked(&built),
            vec![("substitution_direction_unclear", "A42G".to_string())]
        );
    }

    #[test]
    fn a_question_mark_names_the_unstated_part() {
        let text: &str = "In synA, change alanine to glycine.";
        let built: Built = build(
            text,
            &reply(&format!(
                r#"{{"operation":"q5_residue_substitutions","cds":"synA",
                "mutations":[{{"token":"A?G","quote":"alanine to glycine","codon":null}}],{}}}"#,
                NONE
            )),
            &syna(),
            Some(&profile(true, true)),
            &facts(),
        );
        assert_eq!(
            asked(&built),
            vec![("residue_number_unstated", "A?G".to_string())]
        );
    }

    #[test]
    fn a_lower_case_codon_is_grounded_and_written_upper_case() {
        let text: &str = "In synA, change A42 to glycine using codon ggc.";
        let built: Built = build(
            text,
            &reply(&format!(
                r#"{{"operation":"q5_residue_substitutions","cds":"synA",
                "mutations":[{{"token":"A42G","quote":"A42 to glycine","codon":"ggc"}}],{}}}"#,
                NONE
            )),
            &syna(),
            Some(&profile(false, true)),
            &facts(),
        );
        match built {
            Built::Proposed {
                request, origins, ..
            } => {
                assert_eq!(
                    request.operation,
                    OperationSpec::Edit(MemberSpec {
                        name: "A42G".to_string(),
                        edit: EditSpec::Residue {
                            cds: Some("synA".to_string()),
                            number: 42,
                            from: "A".to_string(),
                            to: "G".to_string(),
                            codon: CodonSpec::Supplied("GGC".to_string()),
                        },
                    })
                );
                assert_eq!(
                    origins,
                    vec![
                        Origin {
                            value: "design".to_string(),
                            source: Source::Profile
                        },
                        Origin {
                            value: "numbering".to_string(),
                            source: Source::Profile
                        },
                        Origin {
                            value: "codon A42G".to_string(),
                            source: Source::Text
                        },
                    ]
                );
            }
            other => panic!("expected a proposal, got {:?}", other),
        }
    }

    #[test]
    fn an_empty_or_absent_string_is_ungrounded_and_first_in_order_wins() {
        let text: &str = "In synA, change alanine 42 to glycine.";
        let cases: [(&str, &str, &str); 4] = [
            (
                r#"{"operation":"q5_residue_substitutions","cds":"",
                "mutations":[{"token":"A42G","quote":"","codon":null}],
                "composition":null,"numbering":null,"unsupported":[]}"#,
                "cds",
                "",
            ),
            (
                r#"{"operation":"q5_residue_substitutions","cds":"synA",
                "mutations":[{"token":"A42G","quote":"","codon":null}],
                "composition":null,"numbering":"  ","unsupported":[]}"#,
                "mutations[0].quote",
                "",
            ),
            (
                r#"{"operation":"q5_residue_substitutions","cds":"synA",
                "mutations":[{"token":"A42G","quote":"alanine 42 to glycine","codon":"GCG"}],
                "composition":null,"numbering":"  ","unsupported":[]}"#,
                "mutations[0].codon",
                "GCG",
            ),
            (
                r#"{"operation":"other","cds":"synA","mutations":[],
                "composition":null,"numbering":null,"unsupported":["walk the dog"]}"#,
                "unsupported[0]",
                "walk the dog",
            ),
        ];
        for case in cases {
            assert_eq!(
                build(
                    text,
                    &reply(case.0),
                    &syna(),
                    Some(&profile(true, true)),
                    &facts()
                ),
                Built::Ungrounded(Ungrounded {
                    field: case.1.to_string(),
                    value: case.2.to_string(),
                }),
                "{}",
                case.1
            );
        }
    }

    #[test]
    fn no_mutation_is_a_question() {
        let built: Built = build(
            "In synA, numbering from the CDS start.",
            &reply(&format!(
                r#"{{"operation":"q5_residue_substitutions","cds":"synA","mutations":[],{}}}"#,
                NONE
            )),
            &syna(),
            Some(&profile(true, true)),
            &facts(),
        );
        assert_eq!(
            asked(&built),
            vec![("mutation_unstated", "mutations".to_string())]
        );
    }

    #[test]
    fn a_cds_name_is_matched_exactly() {
        let built: Built = build(
            "in syna change A42G, numbering from the cds start",
            &reply(
                r#"{"operation":"q5_residue_substitutions","cds":"syna",
                "mutations":[{"token":"A42G","quote":"A42G","codon":null}],
                "composition":null,"numbering":"numbering from the cds start","unsupported":[]}"#,
            ),
            &syna(),
            Some(&profile(true, true)),
            &facts(),
        );
        assert_eq!(asked(&built), vec![("cds_not_found", "syna".to_string())]);
        assert_eq!(
            messages(&built),
            vec![
                "the record has no CDS named \"syna\"; the CDSs luria can use are: synA"
                    .to_string()
            ]
        );
    }

    #[test]
    fn an_unusable_cds_reports_core_s_refusal() {
        let entries: Vec<CatalogueEntry> = vec![
            CatalogueEntry {
                id: "alt11".to_string(),
                gene: Some("alt11".to_string()),
                product: None,
                usability: Usability::Refused {
                    code: "unusable_genbank_cds".to_string(),
                    rule: Some("alternative_start".to_string()),
                },
            },
            CatalogueEntry {
                id: "p1".to_string(),
                gene: Some("p1".to_string()),
                product: None,
                usability: Usability::Usable {
                    residues: 3,
                    strand: Strand::Plus,
                },
            },
        ];
        let built: Built = build(
            "In alt11, change valine 1 to alanine, numbering from the CDS start.",
            &reply(
                r#"{"operation":"q5_residue_substitutions","cds":"alt11",
                "mutations":[{"token":"V1A","quote":"valine 1 to alanine","codon":null}],
                "composition":null,"numbering":"numbering from the CDS start","unsupported":[]}"#,
            ),
            &entries,
            Some(&profile(true, true)),
            &facts(),
        );
        assert_eq!(
            messages(&built),
            vec![
                "luria cannot use the CDS \"alt11\": unusable_genbank_cds:alternative_start"
                    .to_string()
            ]
        );
    }

    // D028: with no profile, every value the request needs and the text
    // does not state is asked about, all at once, in the order of §3.4.
    #[test]
    fn every_unstated_value_is_asked_about_at_once_in_order() {
        let built: Built = build(
            "Make A42G and W87A.",
            &reply(&format!(
                r#"{{"operation":"q5_residue_substitutions","cds":null,
                "mutations":[{{"token":"A42G","quote":"A42G","codon":null}},
                             {{"token":"W87A","quote":"W87A","codon":null}}],{}}}"#,
                NONE
            )),
            &syna(),
            None,
            &facts(),
        );
        assert_eq!(
            asked(&built),
            vec![
                ("codon_unstated", "A42G".to_string()),
                ("codon_unstated", "W87A".to_string()),
                ("cds_unnamed", "cds".to_string()),
                ("composition_unstated", "composition".to_string()),
                ("numbering_unstated", "numbering".to_string()),
                ("design_unstated", "design".to_string()),
            ]
        );
        assert_eq!(
            messages(&built)[2],
            "the text names no CDS; the CDSs luria can use are: synA"
        );
    }

    #[test]
    fn one_mutation_is_an_edit_whatever_the_composition_says() {
        let built: Built = build(
            "In synA, make A42G as separate variants, numbering from the CDS start.",
            &reply(
                r#"{"operation":"q5_residue_substitutions","cds":"synA",
                "mutations":[{"token":"A42G","quote":"A42G","codon":null}],
                "composition":"separate_variants","numbering":"numbering from the CDS start",
                "unsupported":[]}"#,
            ),
            &syna(),
            Some(&profile(true, false)),
            &facts(),
        );
        match built {
            Built::Proposed {
                request,
                origins,
                assumptions,
            } => {
                assert!(matches!(request.operation, OperationSpec::Edit(_)));
                assert_eq!(request.scope, Some("synA".to_string()));
                assert_eq!(
                    request.template,
                    TemplateSpec::GenBank {
                        id: "synlin".to_string(),
                        path: "synthetic-linear.gb".to_string(),
                        sha256: Some(facts().sha256),
                        cds: vec!["synA".to_string()],
                    }
                );
                assert_eq!(request.design, design());
                assert_eq!(
                    origins,
                    vec![
                        Origin {
                            value: "design".to_string(),
                            source: Source::Profile
                        },
                        Origin {
                            value: "numbering".to_string(),
                            source: Source::Text
                        },
                        Origin {
                            value: "codon A42G".to_string(),
                            source: Source::Profile
                        },
                    ]
                );
                assert_eq!(
                    assumptions,
                    vec![Assumption {
                        code: "template_id_from_locus".to_string(),
                        message: "the template id is the record's LOCUS name, synlin".to_string(),
                    }]
                );
            }
            other => panic!("expected a proposal, got {:?}", other),
        }
    }

    #[test]
    fn every_question_code_is_pinned() {
        let codes: [(QuestionCode, &str); 16] = [
            (QuestionCode::OperationUnsupported, "operation_unsupported"),
            (QuestionCode::MutationUnstated, "mutation_unstated"),
            (
                QuestionCode::MutationNotationUnsupported,
                "mutation_notation_unsupported",
            ),
            (
                QuestionCode::ResidueNumberUnstated,
                "residue_number_unstated",
            ),
            (
                QuestionCode::OriginalResidueUnstated,
                "original_residue_unstated",
            ),
            (
                QuestionCode::TargetResidueUnstated,
                "target_residue_unstated",
            ),
            (
                QuestionCode::SubstitutionDirectionUnclear,
                "substitution_direction_unclear",
            ),
            (
                QuestionCode::MutationQuoteAmbiguous,
                "mutation_quote_ambiguous",
            ),
            (QuestionCode::CodonUnstated, "codon_unstated"),
            (QuestionCode::CdsUnnamed, "cds_unnamed"),
            (QuestionCode::CdsNotFound, "cds_not_found"),
            (QuestionCode::CdsUnusable, "cds_unusable"),
            (QuestionCode::CompositionUnstated, "composition_unstated"),
            (QuestionCode::NumberingUnstated, "numbering_unstated"),
            (QuestionCode::NumberingUnsupported, "numbering_unsupported"),
            (QuestionCode::DesignUnstated, "design_unstated"),
        ];
        for pair in codes {
            assert_eq!(pair.0.code(), pair.1);
        }
    }
}
