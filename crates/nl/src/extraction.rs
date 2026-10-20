//! What a model says the user asked for, read strictly (plan §5.2).
//!
//! An `Extraction` is the model's reply: its six members, with the user's
//! own words kept in `quote` and `numbering`. It is shape only: no value
//! here has been grounded against the user's text, parsed as mutation
//! notation, or checked against a sequence. `luria-cli` grounds it and
//! builds the typed request (milestone 6 plan §3, step 1).
//!
//! Every reply this refuses becomes `model_output_invalid` in the CLI.
//! `parse_extraction` refuses, and never repairs:
//! - a reply above `MAX_REPLY_BYTES` (plan §5.7, review item B-8);
//! - an unknown field, at the extraction or the mutation level;
//! - a missing field, the nullable members included: `null` is present, an
//!   absent member is not;
//! - a wrong type;
//! - an array where an object is required, at the reply level or as a
//!   `mutations` element: serde's derived deserializer reads a sequence
//!   into a struct by position, which skips the field names and their
//!   `deny_unknown_fields` along with them;
//! - an unknown enum value;
//! - a duplicate key;
//! - trailing text after the document.
//!
//! In particular it does not judge whether `mutations` is empty, whether a
//! token parses as notation, or whether a quote occurs in any text: that is
//! grounding, and it belongs to `luria-cli`.

use serde::de::Deserializer;
use serde::Deserialize;
use serde::Serialize;

/// The most bytes a model reply may hold (plan §5.7, review item B-8).
/// 6h confirms the number against the other input caps.
pub const MAX_REPLY_BYTES: usize = 64 * 1024;

/// The most characters a refusal message may hold: serde's own text quotes
/// the model's strings, which are as long and as broken up as the model
/// likes.
const MAX_MESSAGE_CHARS: usize = 300;

/// A reply luria will not read. `message` is one line of at most
/// `MAX_MESSAGE_CHARS` (300) chars, no trailing newline: each control char
/// is a space, and a longer message is cut at 297 chars to end with `...`.
/// The CLI reports every one of these as `model_output_invalid`; this type
/// carries no code of its own.
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

/// One line of at most `MAX_MESSAGE_CHARS` chars: each control char is a
/// space, and a longer message is cut and ends with `...`.
fn one_line(message: String) -> String {
    let line: String = message
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    if line.chars().count() <= MAX_MESSAGE_CHARS {
        return line;
    }
    // The cut keeps the first 297 chars, so with `...` the line is 300.
    let head: String = line.chars().take(MAX_MESSAGE_CHARS - "...".len()).collect();
    head + "..."
}

/// The reply's shape, read as plain JSON after the typed parse has
/// accepted its values. A sequence where an object is required parses
/// either way through the derived deserializer, which takes the values by
/// position and skips the field names and their `deny_unknown_fields`
/// along with them; the reply itself is an object, and so is each
/// `mutations` element (plan §3 step 1: a wrong type is a failure, not a
/// repair). Failing to read `text` as a value after the typed parse
/// succeeded cannot happen; if it does, its own text is the message.
fn check_shape(text: &str) -> Result<(), String> {
    let value: serde_json::Value = match serde_json::from_str::<serde_json::Value>(text) {
        Err(error) => return Err(error.to_string()),
        Ok(value) => value,
    };
    if !value.is_object() {
        return Err("the reply is not a JSON object".to_string());
    }
    // The typed parse has read `mutations` as a list, so an element that
    // is not an object is the only shape left to refuse.
    if let Some(list) = value.get("mutations").and_then(|member| member.as_array()) {
        for (index, element) in list.iter().enumerate() {
            if !element.is_object() {
                return Err(format!("mutations[{}] is not a JSON object", index));
            }
        }
    }
    Ok(())
}

/// Parse one model reply. Nothing is repaired: see the module doc for
/// everything this refuses.
pub fn parse_extraction(text: &str) -> Result<Extraction, ExtractionError> {
    if text.len() > MAX_REPLY_BYTES {
        return Err(ExtractionError {
            message: one_line(format!(
                "the reply is {} bytes, above the {} the schema allows",
                text.len(),
                MAX_REPLY_BYTES,
            )),
        });
    }
    match serde_json::from_str::<Extraction>(text) {
        Err(error) => Err(ExtractionError {
            message: one_line(error.to_string()),
        }),
        Ok(extraction) => match check_shape(text) {
            Err(message) => Err(ExtractionError {
                message: one_line(message),
            }),
            Ok(()) => Ok(extraction),
        },
    }
}

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

    // serde's derived deserializer reads a JSON sequence into a struct,
    // skipping the field names and `deny_unknown_fields` along with them,
    // so an array is not the reply this type documents (plan §3 step 1).
    #[test]
    fn an_array_is_refused_where_an_object_is_required() {
        let array: &str =
            r#"["q5_residue_substitutions","synA",[["A42G","A42G",null]],null,null,[]]"#;
        assert_eq!(refusal(array), "the reply is not a JSON object");
        let mutation: &str = r#"{"operation":"q5_residue_substitutions","cds":"synA","mutations":[["A42G","A42G","GGT"]],"composition":null,"numbering":null,"unsupported":[]}"#;
        assert_eq!(refusal(mutation), "mutations[0] is not a JSON object");
        let later: &str = r#"{"operation":"q5_residue_substitutions","cds":"synA","mutations":[{"token":"A42G","quote":"A42G","codon":null},["W87A","W87A",null]],"composition":null,"numbering":null,"unsupported":[]}"#;
        assert_eq!(refusal(later), "mutations[1] is not a JSON object");
    }

    #[test]
    fn an_error_message_is_one_line() {
        // The newlines are JSON escapes, so the document itself is valid
        // JSON; only the model's own text carries the control chars.
        let reply: &str = r#"{"operation":"a\nb\nINJECTED","cds":null,"mutations":[],
            "composition":null,"numbering":null,"unsupported":[]}"#;
        let message: String = refusal(reply);
        assert!(
            message.chars().all(|c| !c.is_control()),
            "the reply {:?} was refused with a message that holds a control char: {:?}",
            reply,
            message
        );
    }

    #[test]
    fn a_long_error_message_is_cut() {
        let reply: String = format!(
            "{{\"operation\":\"{}\",\"cds\":null,\"mutations\":[],\"composition\":null,\"numbering\":null,\"unsupported\":[]}}",
            "x".repeat(1000)
        );
        let message: String = refusal(&reply);
        assert_eq!(
            message.chars().count(),
            300,
            "the reply {:?} was refused with {:?} chars, not 300",
            reply,
            message.chars().count()
        );
        assert!(message.ends_with("..."), "{:?}", message);
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
            message
                == format!(
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
        let again: Extraction = parse_extraction(&text).expect("the serialized form parses back");
        assert_eq!(first, again);
    }
}
