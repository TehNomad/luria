# Milestone 6a: the `luria-nl` crate and the extraction parser

Status: **Ready for the worker, 2026-10-19.**
Planner: Claude. Parent: `docs/tasks/contracts/milestone-6-nl.md` §5.1, §5.2, §6 (6a).

## 1. What and why

Milestone 6 turns a user's prose into a typed request. This contract adds
the crate that talks to a model, and the one thing it must do before any
network code exists: **read the model's reply strictly**.

The reply is a small JSON document, the *extraction* (plan §5.2): what the
model says the user asked for, in the user's own words. This crate parses
it and stops there. It attaches no biological meaning, checks nothing
against a sequence, and builds no request: `luria-cli` grounds the
extraction against the user's text and builds the request (plan §3,
contract 6e). Keeping the two apart is what lets the trust boundary be
tested.

No HTTP in this contract. The backend trait, the prompt and `ureq` are 6f
and 6g.

## 2. Requirements this contract applies

- **Plan §5.1:** `luria-nl` holds model I/O only. It may depend on `serde`
  and `serde_json` (and later `ureq`). It does **not** depend on
  `luria-core` or `luria-cli`, and holds no scientific logic (D003).
- **Plan §5.2:** the extraction's six members, their types and their
  nullability, exactly as §4.2 restates them.
- **Plan §3, step 1:** the parser is strict. An unknown field, a missing
  field, a wrong type, an unknown enum value, a duplicate key or trailing
  text is a refusal, never a repair. The CLI reports every refusal as
  `model_output_invalid`.
- **Plan §5.7:** the reply is capped in size (review item B-8).
- **D012:** parsing is deterministic.

### 2.1 Facts established by the planner, 2026-10-19

Verified in a throwaway crate against the versions in `Cargo.lock`, so §4.2
can rely on them. Do not re-litigate these; the §4.3 tests prove them
again:

1. `#[serde(deny_unknown_fields)]` refuses an unknown field:
   `` unknown field `z`, expected `cds` or `plain` at line 1 column 28 ``.
2. A plain `Option<T>` field **is allowed to be missing**, and becomes
   `None`. That is why §4.2 puts `#[serde(deserialize_with = "present")]`
   on every nullable member: with `deserialize_with`, a missing field is
   `` missing field `cds` at line 1 column 14 ``.
3. A duplicate key is `` duplicate field `cds` at line 1 column 16 ``.
4. Trailing text is `trailing characters at line 1 column 27`.

## 3. Files

### 3.1 `Cargo.toml` (root)

Append `"crates/nl"` to `members`, after `"crates/cli"`. Change nothing
else.

### 3.2 `crates/nl/Cargo.toml`

```toml
[package]
name = "luria-nl"
description = "luria's model adapter: what a language model returns, read strictly. No scientific logic."
version = "0.1.0"
edition = "2021"

[lib]
name = "luria_nl"
path = "src/lib.rs"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

No `ureq`, and no `luria-core`. `Cargo.lock` changes as cargo resolves
these; both crates are already in the lock file, so nothing new should be
fetched. If the network is unavailable, retry with `--offline`.

### 3.3 `crates/nl/src/lib.rs`

```rust
//! luria's model adapter: the request sent to a language model and the
//! reply read back from it. This crate is the only place a model is
//! spoken to, and it holds no scientific logic and no biology (D003):
//! it stops at what the model said. `luria-cli` grounds that against the
//! user's own text and builds the typed request (milestone 6 plan §3).

pub mod extraction;
```

### 3.4 `crates/nl/src/extraction.rs`

Module doc: what an extraction is, that it is shape-only, that every
refusal becomes `model_output_invalid` in the CLI, and the list of what is
refused (§2's third bullet).

```rust
use serde::de::Deserializer;
use serde::Deserialize;
use serde::Serialize;

/// The most bytes a model reply may hold (plan §5.7, review item B-8).
/// 6h confirms the number against the other input caps.
pub const MAX_REPLY_BYTES: usize = 64 * 1024;

/// A reply luria will not read. `message` is one line, no trailing
/// newline. The CLI reports every one of these as `model_output_invalid`;
/// this type carries no code of its own.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtractionError {
    pub message: String,
}

/// What the model was asked for: the one operation milestone 6 supports,
/// or `other` for anything else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Operation {
    #[serde(rename = "q5_residue_substitutions")]
    Q5ResidueSubstitutions,
    #[serde(rename = "other")]
    Other,
}

/// Whether the user asked for separate variants or one combined
/// construct. Null when they did not say; the CLI asks (plan §5.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Composition {
    #[serde(rename = "separate_variants")]
    SeparateVariants,
    #[serde(rename = "combined_construct")]
    CombinedConstruct,
}

/// One mutation the model reports, with the user's words that say so.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Mutation {
    /// The mutation in canonical one-letter form, such as `A42G`. The
    /// model may write this even when the user did not, so it is the one
    /// string that is not grounded in the text directly: it is checked
    /// against `quote` instead (plan §5.3a).
    pub token: String,
    /// The span of the user's text that supports `token`, copied
    /// verbatim.
    pub quote: String,
    /// The codon the user named, copied verbatim, or null.
    #[serde(deserialize_with = "present")]
    pub codon: Option<String>,
}

/// What the model made of the user's request (plan §5.2). Shape only: no
/// value here has been grounded, parsed as notation, or checked against a
/// sequence.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Extraction {
    pub operation: Operation,
    /// The CDS name as the user wrote it, or null.
    #[serde(deserialize_with = "present")]
    pub cds: Option<String>,
    pub mutations: Vec<Mutation>,
    #[serde(deserialize_with = "present")]
    pub composition: Option<Composition>,
    /// The user's own words about numbering, or null.
    #[serde(deserialize_with = "present")]
    pub numbering: Option<String>,
    /// Verbatim quotes of anything the fields above cannot hold.
    pub unsupported: Vec<String>,
}

/// Read an `Option<T>` that must be present, as `null` or as a value.
/// A plain `Option<T>` field may be left out of the document altogether
/// (§2.1); every nullable member of an extraction has to be there.
fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// Parse one model reply. Nothing is repaired: see the module doc for
/// everything this refuses.
pub fn parse_extraction(text: &str) -> Result<Extraction, ExtractionError>;
```

`parse_extraction`:
1. if `text.len() > MAX_REPLY_BYTES`, refuse with the exact message
   `format!("the reply is {} bytes, above the {} the schema allows", text.len(), MAX_REPLY_BYTES)`;
2. otherwise `serde_json::from_str::<Extraction>(text)`, and on failure
   refuse with serde's own message, `error.to_string()`, unchanged — it
   already ends in ` at line L column C`, as `luria-cli`'s request parser
   also reports (5e1).

No other behaviour. In particular `parse_extraction` does **not** judge
whether `mutations` is empty, whether a token parses as notation, or
whether a quote occurs in any text: an empty `mutations` array parses, and
grounding is 6e's.

## 4. Tests (planner-written; add verbatim)

### 4.1 Unit tests in `crates/nl/src/extraction.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// The plan's own §5.2 example.
    const EXAMPLE: &str = r#"{
  "operation": "q5_residue_substitutions",
  "cds": "synA",
  "mutations": [
    { "token": "A42G", "quote": "alanine 42 to glycine", "codon": null },
    { "token": "W87A", "quote": "W87A", "codon": "GCG" }
  ],
  "composition": "separate_variants",
  "numbering": null,
  "unsupported": []
}"#;

    /// The six members of a complete reply, each as its own JSON text.
    const MEMBERS: [(&str, &str); 6] = [
        ("operation", "\"q5_residue_substitutions\""),
        ("cds", "\"synA\""),
        (
            "mutations",
            "[{ \"token\": \"A42G\", \"quote\": \"alanine 42 to glycine\", \"codon\": null }]",
        ),
        ("composition", "null"),
        ("numbering", "null"),
        ("unsupported", "[]"),
    ];

    /// A complete reply, less the members named.
    fn document(without: &[&str]) -> String {
        let kept: Vec<String> = MEMBERS
            .iter()
            .filter(|member| !without.contains(&member.0))
            .map(|member| format!("\"{}\": {}", member.0, member.1))
            .collect();
        format!("{{{}}}", kept.join(","))
    }

    /// The message a reply is refused with, or a panic if it parsed.
    fn refusal(text: &str) -> String {
        match parse_extraction(text) {
            Err(error) => error.message,
            Ok(extraction) => panic!("{:?} parsed as {:?}", text, extraction),
        }
    }

    #[test]
    fn the_plan_example_parses_member_by_member() {
        let extraction: Extraction =
            parse_extraction(EXAMPLE).expect("the plan's §5.2 example parses");
        assert_eq!(extraction.operation, Operation::Q5ResidueSubstitutions);
        assert_eq!(extraction.cds, Some("synA".to_string()));
        assert_eq!(extraction.mutations.len(), 2);
        assert_eq!(extraction.mutations[0].token, "A42G");
        assert_eq!(extraction.mutations[0].quote, "alanine 42 to glycine");
        assert_eq!(extraction.mutations[0].codon, None);
        assert_eq!(extraction.mutations[1].token, "W87A");
        assert_eq!(extraction.mutations[1].quote, "W87A");
        assert_eq!(extraction.mutations[1].codon, Some("GCG".to_string()));
        assert_eq!(extraction.composition, Some(Composition::SeparateVariants));
        assert_eq!(extraction.numbering, None);
        assert!(extraction.unsupported.is_empty());
    }

    #[test]
    fn every_nullable_member_takes_null_and_a_value() {
        let nulls: &str = r#"{"operation":"other","cds":null,"mutations":[],
            "composition":null,"numbering":null,"unsupported":[]}"#;
        let empty: Extraction = parse_extraction(nulls).expect("every null parses");
        assert_eq!(empty.operation, Operation::Other);
        assert_eq!(empty.cds, None);
        assert_eq!(empty.composition, None);
        assert_eq!(empty.numbering, None);
        assert!(empty.mutations.is_empty() && empty.unsupported.is_empty());

        let values: &str = r#"{"operation":"q5_residue_substitutions","cds":"synA",
            "mutations":[{"token":"A42G","quote":"A42G","codon":"GGT"}],
            "composition":"combined_construct","numbering":"from the CDS start",
            "unsupported":["add a His tag"]}"#;
        let full: Extraction = parse_extraction(values).expect("every value parses");
        assert_eq!(full.cds, Some("synA".to_string()));
        assert_eq!(full.composition, Some(Composition::CombinedConstruct));
        assert_eq!(full.numbering, Some("from the CDS start".to_string()));
        assert_eq!(full.mutations[0].codon, Some("GGT".to_string()));
        assert_eq!(full.unsupported, vec!["add a His tag".to_string()]);
    }

    // Every member is required, the nullable ones included: a model that
    // leaves one out has not answered, and null is not the same as absent.
    #[test]
    fn a_complete_document_parses_and_each_missing_member_is_refused() {
        parse_extraction(&document(&[])).expect("the complete document parses");
        for member in MEMBERS {
            let message: String = refusal(&document(&[member.0]));
            assert!(
                message.contains(&format!("missing field `{}`", member.0)),
                "without {}: {:?}",
                member.0,
                message
            );
        }
    }

    #[test]
    fn each_missing_mutation_member_is_refused() {
        let cases: [(&str, &str); 3] = [
            ("token", "{\"quote\":\"A42G\",\"codon\":null}"),
            ("quote", "{\"token\":\"A42G\",\"codon\":null}"),
            ("codon", "{\"token\":\"A42G\",\"quote\":\"A42G\"}"),
        ];
        for case in cases {
            let text: String = format!(
                "{{\"operation\":\"other\",\"cds\":null,\"mutations\":[{}],\"composition\":null,\"numbering\":null,\"unsupported\":[]}}",
                case.1
            );
            let message: String = refusal(&text);
            assert!(
                message.contains(&format!("missing field `{}`", case.0)),
                "a mutation without {}: {:?}",
                case.0,
                message
            );
        }
    }

    #[test]
    fn an_unknown_field_is_refused_at_either_level() {
        let outer: &str = r#"{"operation":"other","cds":null,"mutations":[],
            "composition":null,"numbering":null,"unsupported":[],"confidence":0.92}"#;
        assert!(
            refusal(outer).contains("unknown field `confidence`"),
            "{:?}",
            refusal(outer)
        );
        let inner: &str = r#"{"operation":"other","cds":null,
            "mutations":[{"token":"A42G","quote":"A42G","codon":null,"score":1}],
            "composition":null,"numbering":null,"unsupported":[]}"#;
        assert!(
            refusal(inner).contains("unknown field `score`"),
            "{:?}",
            refusal(inner)
        );
    }

    #[test]
    fn a_wrong_type_is_refused() {
        let cases: [&str; 4] = [
            // mutations as a bare string
            r#"{"operation":"other","cds":null,"mutations":"A42G","composition":null,
                "numbering":null,"unsupported":[]}"#,
            // a token as a number
            r#"{"operation":"other","cds":null,"mutations":[{"token":42,"quote":"A42G",
                "codon":null}],"composition":null,"numbering":null,"unsupported":[]}"#,
            // unsupported as an object
            r#"{"operation":"other","cds":null,"mutations":[],"composition":null,
                "numbering":null,"unsupported":{}}"#,
            // operation as null: it is not nullable
            r#"{"operation":null,"cds":null,"mutations":[],"composition":null,
                "numbering":null,"unsupported":[]}"#,
        ];
        for text in cases {
            let message: String = refusal(text);
            assert!(
                message.contains("line") && message.contains("column"),
                "{:?} was refused with {:?}, which names no place",
                text,
                message
            );
        }
    }

    #[test]
    fn an_unknown_enum_value_is_refused() {
        let operation: &str = r#"{"operation":"q5_deletions","cds":null,"mutations":[],
            "composition":null,"numbering":null,"unsupported":[]}"#;
        assert!(
            refusal(operation).contains("unknown variant `q5_deletions`"),
            "{:?}",
            refusal(operation)
        );
        let composition: &str = r#"{"operation":"other","cds":null,"mutations":[],
            "composition":"both","numbering":null,"unsupported":[]}"#;
        assert!(
            refusal(composition).contains("unknown variant `both`"),
            "{:?}",
            refusal(composition)
        );
    }

    #[test]
    fn a_duplicate_key_and_trailing_text_are_refused() {
        let duplicate: &str = r#"{"operation":"other","cds":"a","cds":"b","mutations":[],
            "composition":null,"numbering":null,"unsupported":[]}"#;
        assert!(
            refusal(duplicate).contains("duplicate field `cds`"),
            "{:?}",
            refusal(duplicate)
        );
        let trailing: String = format!("{} {{}}", document(&[]));
        assert!(
            refusal(&trailing).contains("trailing characters"),
            "{:?}",
            refusal(&trailing)
        );
    }

    #[test]
    fn a_reply_above_the_cap_is_refused_before_it_is_parsed() {
        // A valid document, padded past the cap inside its own quote, so
        // that only the cap can refuse it.
        let padding: String = "x".repeat(MAX_REPLY_BYTES);
        let text: String = format!(
            "{{\"operation\":\"other\",\"cds\":null,\"mutations\":[],\"composition\":null,\"numbering\":\"{}\",\"unsupported\":[]}}",
            padding
        );
        assert!(text.len() > MAX_REPLY_BYTES);
        let message: String = refusal(&text);
        assert!(
            message == format!(
                "the reply is {} bytes, above the {} the schema allows",
                text.len(),
                MAX_REPLY_BYTES
            ),
            "{:?}",
            message
        );
        // One byte under the cap, the same shape parses.
        let under: String = format!(
            "{{\"operation\":\"other\",\"cds\":null,\"mutations\":[],\"composition\":null,\"numbering\":\"{}\",\"unsupported\":[]}}",
            "x".repeat(MAX_REPLY_BYTES - 120)
        );
        assert!(under.len() <= MAX_REPLY_BYTES);
        parse_extraction(&under).expect("a reply inside the cap parses");
    }

    // The CLI records the parsed extraction in its interpretation document
    // (plan §5.7), so what this crate writes it must read back.
    #[test]
    fn an_extraction_round_trips_through_its_own_json() {
        let first: Extraction = parse_extraction(EXAMPLE).expect("the example parses");
        let text: String = serde_json::to_string(&first).expect("an extraction serializes");
        let again: Extraction =
            parse_extraction(&text).expect("the serialized form parses back");
        assert_eq!(first, again);
    }
}
```

### 4.2 Integration test: `crates/nl/tests/corpus.rs`

```rust
//! Every scripted reply in the natural-language corpus, read by the
//! parser: the two cases the corpus calls `model_output_invalid` must be
//! refused, and every other reply must parse. The corpus is the fixture
//! of contracts 6e and 6i; `tests/fixtures/nl/README.md` records how each
//! case was derived.

use luria_nl::extraction::parse_extraction;

const CORPUS: &str = "../../tests/fixtures/nl/";
const CASES_HEADER: &str = "case_id\ttemplate\ttext_file\treply_file\tprofile\tstate\tcodes\trequest_file\tdesign_exit\tnote";

/// The rows of `cases.tsv` past its header, refusing a header it does not
/// have.
fn cases() -> Vec<Vec<String>> {
    let path: String = format!("{}cases.tsv", CORPUS);
    let text: String = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
    let lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    assert!(
        !lines.is_empty() && lines[0] == CASES_HEADER,
        "{}: first row {:?}, expected {:?}",
        path,
        lines.first(),
        CASES_HEADER
    );
    lines[1..]
        .iter()
        .map(|line| line.split('\t').map(|field| field.to_string()).collect())
        .collect()
}

#[test]
fn every_corpus_reply_parses_unless_the_corpus_calls_it_invalid() {
    let rows: Vec<Vec<String>> = cases();
    assert!(rows.len() == 24, "cases.tsv has {} cases, expected 24", rows.len());
    let mut refused: Vec<String> = Vec::new();
    let mut parsed: Vec<String> = Vec::new();
    for row in rows.iter() {
        let case: String = row[0].clone();
        let path: String = format!("{}reply/{}", CORPUS, row[3]);
        let reply: String = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
        let invalid: bool = row[6] == "model_output_invalid";
        match parse_extraction(&reply) {
            Ok(_) => {
                assert!(!invalid, "{}: parsed, but the corpus calls it invalid", case);
                parsed.push(case);
            }
            Err(error) => {
                assert!(
                    invalid,
                    "{}: refused with {:?}, but the corpus expects {:?}",
                    case, error.message, row[6]
                );
                refused.push(case);
            }
        }
    }
    assert!(
        refused == vec!["reply-extra-field".to_string(), "reply-wrong-type".to_string()],
        "the refused cases are {:?}",
        refused
    );
    assert!(parsed.len() == 22, "{} replies parsed, expected 22", parsed.len());
}

// A fabricated quote is not this crate's business: it is shape-valid, and
// `luria-cli` refuses it when it grounds the extraction (plan §3, 6e).
#[test]
fn an_ungrounded_reply_is_still_shape_valid_here() {
    for case in ["ungrounded-mutation", "ungrounded-cds"] {
        let path: String = format!("{}reply/{}.json", CORPUS, case);
        let reply: String = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
        parse_extraction(&reply)
            .unwrap_or_else(|error| panic!("{}: {:?}", case, error.message));
    }
}
```

`cargo fmt` may reflow these; do not change their content. If one fails,
stop and report what the parser returned. Do not change the test or the
fixture.

### 4.3 Proving the rules are load-bearing (AGENTS.md)

For each, disable the rule once, confirm the named test fails, restore it,
and report both outcomes:

| Rule | Test that must fail |
| --- | --- |
| `deny_unknown_fields` on `Extraction` | `an_unknown_field_is_refused_at_either_level` |
| `deny_unknown_fields` on `Mutation` | `an_unknown_field_is_refused_at_either_level` |
| `deserialize_with = "present"` on `composition` | `a_complete_document_parses_and_each_missing_member_is_refused` |
| the `MAX_REPLY_BYTES` check in `parse_extraction` | `a_reply_above_the_cap_is_refused_before_it_is_parsed` |

## 5. Do not

- Add any dependency beyond the two in §3.2. No `ureq`, no HTTP, no
  `luria-core`, no `luria-cli`.
- Write any network, process, file or environment access in `crates/nl/src`.
  `parse_extraction` takes text and returns a value.
- Add the backend trait, the prompt, `PROMPT_VERSION` or a scripted
  backend: those are 6f.
- Ground anything, parse a mutation token, read a template, or build a
  request: those are 6b, 6c and 6e.
- Change anything under `crates/cli/`, `crates/core/`, `tests/fixtures/` or
  `docs/`.
- Use `serde_json::Value` or `json!` to build or inspect the extraction.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp` (the sandbox's
`/tmp` cannot be read back; see AGENTS.md):
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo test -p luria-nl` (the new crate on its own)
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the new and changed files, the exact commands and their results, the
§4.3 results, what `Cargo.lock` changed, and anything that did not match
this contract.
