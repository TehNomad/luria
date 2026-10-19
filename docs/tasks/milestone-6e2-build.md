# Milestone 6e2: from an extraction to a request or questions

Status: **Waiting on 6d and 6e1, 2026-10-19.** Start only when both are
committed.
Planner: Claude. Parent: `docs/tasks/milestone-6-nl.md` §3, §5.3a, §5.5,
M6-9, M6-10, M6-11, M6-15, M6-16, §6 (6e).

## 1. What and why

This is the trust boundary of plan §3, as one pure function. It takes what
the model said (an `Extraction`, 6a), the user's own text, the record's
CDS catalogue (6c) and the user's design profile (6d), and ends in exactly
one of three ways:

- **ungrounded**: the model reported a string the user never wrote. Luria
  stops (`ungrounded_extraction`, state `failed`).
- **questions**: something the request needs is unstated, unsupported or
  doubtful. Luria asks, with stable codes, all at once.
- **proposed**: a complete `RequestDocument`, built by Rust, with a record
  of which values came from the text and which from the profile.

No model, file, network or clock is touched. The corpus (24 cases) runs
through it here, before any backend exists.

## 2. Requirements this contract applies

- **Plan §3:** the model writes no request field. Every grounded string
  must be in the text (M6-15); the token is checked against its quote
  through 6e1's `evidence`.
- **Plan §5.3a with its tightenings:** the evidence rules are 6e1's, used
  unchanged.
- **Plan §5.5:** template id from the LOCUS name, recorded as an assumption
  (M6-10); `sha256` always filled; the named CDS becomes `scope`; one
  mutation is an `edit`, two or more a `variants` set; numbering from the
  text first, the profile second; `operation: other` or any `unsupported`
  quote is `operation_unsupported`.
- **M6-9:** no named CDS is always a question, whatever the record holds.
- **M6-16:** a combined construct is proposed; core refuses it later.
- **D028:** nothing is defaulted. A value neither the text nor the profile
  states is a question.
- **D003:** no biology here. Whether residue 42 is really alanine, or the
  codon encodes the target, is `luria q5 design`'s to check.
- **D012:** deterministic: the same inputs give the same `Built`.

## 3. Changes

### 3.1 `crates/cli/Cargo.toml`

Add, after `luria-core`:

```toml
luria-nl = { path = "../nl" }
```

This is the dependency plan §5.1 planned (amending D041; recorded as D044
in 6j). Add nothing else. `Cargo.lock` gains the edge.

### 3.2 `crates/cli/src/interpret.rs`

Add `pub mod build;` as the first module line. Change nothing else.

### 3.3 `crates/cli/src/interpret/build.rs` (new)

Module doc: the three outcomes, the order of the steps (§3.4), that
grounding comes before every question, and what is not judged here.

```rust
use crate::interpret::catalogue::CatalogueEntry;
use crate::interpret::catalogue::Usability;
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
    pub fn code(self) -> &'static str;
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

pub fn build(
    text: &str,
    extraction: &Extraction,
    catalogue: &[CatalogueEntry],
    profile: Option<&ProfileDocument>,
    template: &TemplateFacts,
) -> Built;
```

### 3.4 What `build` does, in order

**Step 1, grounding.** Check these strings, in this order, and return
`Built::Ungrounded` for the first that fails:
1. `cds`, if `Some`;
2. for each mutation `i` in order: its `quote`, then its `codon` if `Some`;
3. `numbering`, if `Some`;
4. each `unsupported[j]` in order.

A string fails when `value.trim()` is empty, or when it does not occur in
`text`. Occurrence is an exact substring (`text.contains(value)`), except
for a codon, which is compared ASCII-case-insensitively
(`text.to_ascii_lowercase().contains(&codon.to_ascii_lowercase())`).
`field` is written exactly as in the `Ungrounded` doc comment; `value` is
the string as the model gave it. A mutation's `token` is not grounded here.

**Step 2, the operation.** If `operation` is `Other` or `unsupported` is
not empty, return `NeedsClarification` with exactly one question,
`OperationUnsupported`:
- with an `unsupported` quote: subject the first quote `q`, message
  `luria interprets amino-acid substitutions only, and the request also asks for "q"`;
- without one: subject `operation`, message
  `the request does not ask for amino-acid substitutions, the only operation luria interprets`.

**Step 3, every other question, collected in this order.** Let the
*usable list* be the ids of the catalogue's `Usable` entries in catalogue
order, joined by `, `, or `none` when there are none.

- **a.** No mutations: `MutationUnstated`, subject `mutations`, message
  `the request names no amino-acid substitution`.
- **b.** For each mutation in order, with `t` its token and `q` its quote:
  - `parse_substitution(t)` is `Err(Unsupported { .. })`:
    `MutationNotationUnsupported`, subject `t`, message
    `"t" is not a substitution luria reads: it reads one-letter (A42G) and three-letter (Ala42Gly) residue substitutions only`;
  - `Err(Unstated { parts })`: one question per part, in the parts' order,
    subject `t`, as in the next bullet's three messages;
  - `Ok(s)`: with `e = evidence(q, &s)`, in this order, each subject `t`:
    `!e.number` gives `ResidueNumberUnstated`, message
    `"q" does not state the residue number`; `!e.original` gives
    `OriginalResidueUnstated`, message
    `"q" does not state the residue there now, which luria checks against the CDS rather than assume`;
    `!e.target` gives `TargetResidueUnstated`, message
    `"q" does not state the residue to put there`; and
    `e.original && e.target && !e.in_order` gives
    `SubstitutionDirectionUnclear`, message
    `"q" names both residues, but not the original before the target`.
    Then, if the mutation's `codon` is `None` and the profile states no
    `codon`: `CodonUnstated`, subject `t`, message
    `the text names no codon for C, and the design profile sets no codon policy`,
    with `C` = `s.canonical()`.
- **c.** The CDS. `cds` is `None`: `CdsUnnamed`, subject `cds`, message
  `the text names no CDS; the CDSs luria can use are: L` (`L` the usable
  list). Otherwise, with `n` the name, look for catalogue entries whose
  `id == n` exactly (case-sensitive):
  - none: `CdsNotFound`, subject `n`, message
    `the record has no CDS named "n"; the CDSs luria can use are: L`;
  - some, but none `Usable`: `CdsUnusable`, subject `n`, message
    `luria cannot use the CDS "n": R`, where `R` is the first matching
    entry's `code`, or `code:rule` when it has a rule;
  - any `Usable`: no question.
- **d.** Two or more mutations and `composition` is `None`:
  `CompositionUnstated`, subject `composition`, message
  `the text does not say whether the substitutions are separate variants or one combined construct`.
- **e.** Numbering. `numbering` is `Some(w)` and
  `!numbering_from_cds_start(w)`: `NumberingUnsupported`, subject `w`,
  message `luria numbers residues from the start of the CDS only, and the text says "w"`.
  `numbering` is `None` and the profile states no `numbering`:
  `NumberingUnstated`, subject `numbering`, message
  `the text does not say how residues are numbered, and the design profile does not state it`.
- **f.** No profile, or a profile with no `design`: `DesignUnstated`,
  subject `design`, message
  `no design profile states the primer design, and luria does not default it`.

In the messages, `"q"`, `"t"`, `"n"` and `"w"` mean the value inside
literal double quotes, written as given, with no escaping. If any question
was collected, return `NeedsClarification { questions }` in that order.

**Step 4, the proposal.** Only reached with no question, so every value
below exists. With `n` the CDS name and the parsed substitutions in order:
- each member: `MemberSpec { name: s.canonical(), edit: EditSpec::Residue
  { cds: Some(n), number: s.number, from: s.from.symbol(), to:
  s.to.symbol(), codon } }`, where `codon` is
  `CodonSpec::Supplied(c.to_ascii_uppercase())` for a mutation codon `c`,
  and otherwise the profile's `codon`;
- `operation`: one member is `OperationSpec::Edit(member)`, whatever
  `composition` says; two or more are `OperationSpec::Variants {
  composition, members }`, mapping `SeparateVariants` and
  `CombinedConstruct` to the `CompositionSpec` of the same name;
- `template`: `TemplateSpec::GenBank { id: locus_name, path, sha256:
  Some(sha256), cds: vec![n] }` from `TemplateFacts`;
- `scope`: `Some(n)`; `design`: the profile's `design`;
- `origins`, in this order: `design` from `Profile`; `numbering` from
  `Text` if the extraction states it, else `Profile`; then for each member
  in order, `codon C` (`C` the member name) from `Text` if the mutation
  has a codon, else `Profile`;
- `assumptions`: exactly one, code `template_id_from_locus`, message
  `the template id is the record's LOCUS name, X` (`X` the LOCUS name).

Nothing in step 4 may panic. If a value step 3 guarantees is somehow
missing, do not unwrap: return `NeedsClarification` with the question
that step 3 would have asked for it.

**Not judged here**, by design: whether the residue at the number is the
stated original, whether a codon encodes the target or is a codon at all,
whether two members repeat, and whether the LOCUS name is a valid
identifier. `luria q5 design` checks all of them on the proposal (D003).
The request's round trip through `request_echo` and `parse_request` is
proved by the §4.1 test here, and enforced at run time in 6h, where the
document is rendered.

## 4. Tests (planner-written; add verbatim)

### 4.1 Integration test: `crates/cli/tests/interpret_build.rs`

```rust
//! The natural-language corpus through `build` (contract 6e2): every case
//! whose reply parses ends in the state and codes the corpus records, and
//! each proposal is the hand-written expected request, which survives the
//! echo round trip. See `tests/fixtures/nl/README.md`.

use luria_cli::interpret::build::build;
use luria_cli::interpret::build::Built;
use luria_cli::interpret::build::TemplateFacts;
use luria_cli::interpret::build::Ungrounded;
use luria_cli::interpret::catalogue::catalogue;
use luria_cli::request::parse_profile;
use luria_cli::request::parse_request;
use luria_cli::request::ProfileDocument;
use luria_cli::request::RequestDocument;
use luria_cli::result::request_echo;
use luria_core::genbank::read_genbank;
use luria_core::genbank::GenBankRecord;
use luria_core::identity::Identifier;
use luria_nl::extraction::parse_extraction;
use luria_nl::extraction::Extraction;

const NL: &str = "../../tests/fixtures/nl/";
const GENBANK: &str = "../../tests/fixtures/genbank/";
const CASES_HEADER: &str = "case_id\ttemplate\ttext_file\treply_file\tprofile\tstate\tcodes\trequest_file\tdesign_exit\tnote";

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

fn cases() -> Vec<Vec<String>> {
    let text: String = read(&format!("{}cases.tsv", NL));
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], CASES_HEADER);
    lines[1..]
        .iter()
        .map(|line| line.split('\t').map(|field| field.to_string()).collect())
        .collect()
}

fn record(file: &str) -> GenBankRecord {
    let name: Identifier = Identifier::create("template".to_string()).expect("an identifier");
    read_genbank(name, read(&format!("{}{}", GENBANK, file)))
        .unwrap_or_else(|error| panic!("{}: {}", file, error.message()))
}

/// One corpus case through `build`, exactly as `luria q5 interpret` will
/// assemble its inputs: the text without its file's final newline.
fn run(row: &[String]) -> Built {
    let text_file: String = read(&format!("{}text/{}", NL, row[2]));
    let text: &str = text_file.strip_suffix('\n').unwrap_or(&text_file);
    let reply: Extraction = parse_extraction(&read(&format!("{}reply/{}", NL, row[3])))
        .unwrap_or_else(|error| panic!("{}: {}", row[0], error.message));
    let template: GenBankRecord = record(&row[1]);
    let entries = catalogue(&template).unwrap_or_else(|error| panic!("{}", error.message()));
    let profile: Option<ProfileDocument> = if row[4] == "-" {
        None
    } else {
        Some(
            parse_profile(&read(&format!("{}profile/{}", NL, row[4])))
                .unwrap_or_else(|error| panic!("{}: {:?}", row[4], error)),
        )
    };
    let facts: TemplateFacts = TemplateFacts {
        locus_name: template.locus_name(),
        path: row[1].clone(),
        sha256: template.sequence().content_hash().hex_digest(),
    };
    build(text, &reply, &entries, profile.as_ref(), &facts)
}

#[test]
fn every_corpus_case_ends_where_the_corpus_says() {
    let mut seen: [usize; 4] = [0, 0, 0, 0];
    for row in cases() {
        if row[6] == "model_output_invalid" {
            seen[3] += 1;
            continue;
        }
        match (row[5].as_str(), run(&row)) {
            ("proposed", Built::Proposed { request, .. }) => {
                let expected: RequestDocument =
                    parse_request(&read(&format!("{}request/{}", NL, row[7])))
                        .unwrap_or_else(|error| panic!("{}: {:?}", row[7], error));
                assert_eq!(request, expected, "{}", row[0]);
                let echo: String =
                    serde_json::to_string(&request_echo(&request)).expect("the echo serializes");
                assert_eq!(parse_request(&echo).ok(), Some(request), "{}: round trip", row[0]);
                seen[0] += 1;
            }
            ("needs_clarification", Built::NeedsClarification { questions }) => {
                let mut found: Vec<&str> =
                    questions.iter().map(|question| question.code.code()).collect();
                found.sort();
                found.dedup();
                let mut expected: Vec<&str> = row[6].split(';').collect();
                expected.sort();
                assert_eq!(found, expected, "{}", row[0]);
                seen[1] += 1;
            }
            ("failed", Built::Ungrounded(_)) => {
                assert_eq!(row[6], "ungrounded_extraction", "{}", row[0]);
                seen[2] += 1;
            }
            (state, built) => panic!("{}: the corpus says {}, build gave {:?}", row[0], state, built),
        }
    }
    assert_eq!(seen, [5, 15, 2, 2], "proposed, clarification, ungrounded, invalid");
}

#[test]
fn the_ungrounded_corpus_cases_name_the_string() {
    let rows: Vec<Vec<String>> = cases();
    let find = |id: &str| -> Vec<String> {
        rows.iter().find(|row| row[0] == id).expect("the case exists").clone()
    };
    assert_eq!(
        run(&find("ungrounded-mutation")),
        Built::Ungrounded(Ungrounded {
            field: "mutations[1].quote".to_string(),
            value: "tryptophan 87 to alanine".to_string(),
        })
    );
    assert_eq!(
        run(&find("ungrounded-cds")),
        Built::Ungrounded(Ungrounded {
            field: "cds".to_string(),
            value: "bla".to_string(),
        })
    );
}
```

### 4.2 Unit tests, at the end of `crates/cli/src/interpret/build.rs`

```rust
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
            codon: if codon { Some(CodonSpec::Default) } else { None },
            numbering: if numbering { Some(NumberingSpec::FromCdsStart) } else { None },
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
            Built::NeedsClarification { questions } => {
                questions.iter().map(|question| question.message.clone()).collect()
            }
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
            &reply(r#"{"operation":"q5_residue_substitutions","cds":"synA",
                "mutations":[{"token":"A42G","quote":"glycine 42 to alanine","codon":null}],
                "composition":null,"numbering":"Numbering from the CDS start","unsupported":[]}"#),
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
            vec!["\"glycine 42 to alanine\" names both residues, but not the original before the target".to_string()]
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
        assert_eq!(asked(&built), vec![("residue_number_unstated", "A?G".to_string())]);
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
                        Origin { value: "design".to_string(), source: Source::Profile },
                        Origin { value: "numbering".to_string(), source: Source::Profile },
                        Origin { value: "codon A42G".to_string(), source: Source::Text },
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
                build(text, &reply(case.0), &syna(), Some(&profile(true, true)), &facts()),
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
        assert_eq!(asked(&built), vec![("mutation_unstated", "mutations".to_string())]);
    }

    #[test]
    fn a_cds_name_is_matched_exactly() {
        let built: Built = build(
            "in syna change A42G, numbering from the cds start",
            &reply(r#"{"operation":"q5_residue_substitutions","cds":"syna",
                "mutations":[{"token":"A42G","quote":"A42G","codon":null}],
                "composition":null,"numbering":"numbering from the cds start","unsupported":[]}"#),
            &syna(),
            Some(&profile(true, true)),
            &facts(),
        );
        assert_eq!(asked(&built), vec![("cds_not_found", "syna".to_string())]);
        assert_eq!(
            messages(&built),
            vec!["the record has no CDS named \"syna\"; the CDSs luria can use are: synA".to_string()]
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
            &reply(r#"{"operation":"q5_residue_substitutions","cds":"alt11",
                "mutations":[{"token":"V1A","quote":"valine 1 to alanine","codon":null}],
                "composition":null,"numbering":"numbering from the CDS start","unsupported":[]}"#),
            &entries,
            Some(&profile(true, true)),
            &facts(),
        );
        assert_eq!(
            messages(&built),
            vec!["luria cannot use the CDS \"alt11\": unusable_genbank_cds:alternative_start".to_string()]
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
            &reply(r#"{"operation":"q5_residue_substitutions","cds":"synA",
                "mutations":[{"token":"A42G","quote":"A42G","codon":null}],
                "composition":"separate_variants","numbering":"numbering from the CDS start",
                "unsupported":[]}"#),
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
                        Origin { value: "design".to_string(), source: Source::Profile },
                        Origin { value: "numbering".to_string(), source: Source::Text },
                        Origin { value: "codon A42G".to_string(), source: Source::Profile },
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
        let codes: [(QuestionCode, &str); 15] = [
            (QuestionCode::OperationUnsupported, "operation_unsupported"),
            (QuestionCode::MutationUnstated, "mutation_unstated"),
            (QuestionCode::MutationNotationUnsupported, "mutation_notation_unsupported"),
            (QuestionCode::ResidueNumberUnstated, "residue_number_unstated"),
            (QuestionCode::OriginalResidueUnstated, "original_residue_unstated"),
            (QuestionCode::TargetResidueUnstated, "target_residue_unstated"),
            (QuestionCode::SubstitutionDirectionUnclear, "substitution_direction_unclear"),
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
```

`cargo fmt` may reflow these; do not change content. If a test fails,
stop and report what `build` returned. Do not change a test or a fixture.

### 4.3 Proving the rules are load-bearing (AGENTS.md)

Disable each once, confirm the named test fails, restore, report both:

| Rule, disabled | Test that must fail |
| --- | --- |
| grounding of quotes (skip the substring check) | `the_ungrounded_corpus_cases_name_the_string` |
| the empty-string check | `an_empty_or_absent_string_is_ungrounded_and_first_in_order_wins` |
| the `in_order` question | `a_swapped_token_is_asked_about_not_proposed` |
| the composition question | `every_corpus_case_ends_where_the_corpus_says` |
| one member is an `Edit` (always build `Variants`) | `every_corpus_case_ends_where_the_corpus_says` |
| codon upper-casing | `a_lower_case_codon_is_grounded_and_written_upper_case` |

## 5. Do not

- Read a file, call a model, or touch the network, the environment or the
  clock in `crates/cli/src/interpret/`.
- Re-implement notation (6b), evidence (6e1), catalogue (6c) or profile
  (6d) rules. Call them.
- Check biology: residues against the translation, codons against
  residues. That is `luria q5 design`'s.
- Default any value, or choose a CDS the text does not name (M6-9).
- Change `luria-core`, `luria-nl`, `tests/fixtures/`, `docs/`, or any
  existing file other than `crates/cli/Cargo.toml`, `Cargo.lock` and the
  one line in `interpret.rs`.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the new and changed files, the exact commands and their results,
the §4.3 results, and anything that did not match this contract. Quote
command output or the file for every claim; say plainly what you did not
check.
