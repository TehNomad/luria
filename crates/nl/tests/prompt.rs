//! The shipped prompt is the measured one: `SYSTEM_PROMPT`,
//! `EXTRACTION_SCHEMA` and the user message's layout against prompt v3 of
//! `docs/research/m6-local-model-extraction-2026-10-19.md`, and the schema
//! against the extraction parser.

use luria_nl::extraction::parse_extraction;
use luria_nl::prompt::prompt;
use luria_nl::prompt::CdsLine;
use luria_nl::prompt::CdsState;
use luria_nl::prompt::EXTRACTION_SCHEMA;
use luria_nl::prompt::SYSTEM_PROMPT;
use serde_json::Value;

const NOTE: &str = "../../docs/research/m6-local-model-extraction-2026-10-19.md";
const NL: &str = "../../tests/fixtures/nl/";

/// The fenced block that follows `label` in the note's "Prompt v3" section,
/// without its fences.
fn block(label: &str) -> String {
    let note: String = std::fs::read_to_string(NOTE).expect("the research note is readable");
    let section: &str = note
        .split("## Prompt v3")
        .nth(1)
        .expect("the note has a Prompt v3 section");
    let after: &str = section
        .split(label)
        .nth(1)
        .unwrap_or_else(|| panic!("no {:?} in Prompt v3", label));
    let fenced: &str = after.split("```").nth(1).expect("a fenced block follows");
    let body: &str = fenced.split_once('\n').expect("the fence line ends").1;
    body.strip_suffix('\n').unwrap_or(body).to_string()
}

#[test]
fn the_system_prompt_is_the_measured_one() {
    assert_eq!(SYSTEM_PROMPT, block("System message:"));
}

#[test]
fn the_schema_is_the_measured_one() {
    assert_eq!(
        EXTRACTION_SCHEMA,
        block("`response_format.json_schema.schema`:")
    );
}

#[test]
fn the_user_message_has_the_measured_layout() {
    let syna: CdsLine = CdsLine {
        id: "synA".to_string(),
        gene: Some("synA".to_string()),
        product: Some("synthetic protein A".to_string()),
        state: CdsState::Usable {
            residues: 100,
            plus_strand: true,
        },
    };
    assert_eq!(
        prompt("<the user text>", &[syna]).user,
        block("User message (catalogue, then the text):")
    );
}

// The schema asks for what the parser accepts: the six members, each
// required, and the three of each mutation, with nothing else allowed.
#[test]
fn the_schema_describes_what_the_parser_reads() {
    let schema: Value = serde_json::from_str(EXTRACTION_SCHEMA).expect("the schema is JSON");
    let names = |value: &Value| -> Vec<String> {
        let mut found: Vec<String> = value
            .as_array()
            .expect("an array of names")
            .iter()
            .map(|name| name.as_str().expect("a name").to_string())
            .collect();
        found.sort();
        found
    };
    assert_eq!(
        names(&schema["required"]),
        vec![
            "cds",
            "composition",
            "mutations",
            "numbering",
            "operation",
            "unsupported"
        ]
    );
    assert_eq!(schema["additionalProperties"], Value::Bool(false));
    let item: &Value = &schema["properties"]["mutations"]["items"];
    assert_eq!(names(&item["required"]), vec!["codon", "quote", "token"]);
    assert_eq!(item["additionalProperties"], Value::Bool(false));
    let mut properties: Vec<String> = schema["properties"]
        .as_object()
        .expect("properties")
        .keys()
        .cloned()
        .collect();
    properties.sort();
    assert_eq!(properties, names(&schema["required"]));
}

// The research note's finding 4: the model copied a number from a prompt
// example. No corpus text may contain an example token, or a corpus case
// could pass by copying.
#[test]
fn no_corpus_text_contains_a_prompt_example() {
    for example in ["\"S12T\"", "\"S?T\""] {
        assert!(
            SYSTEM_PROMPT.contains(example),
            "the prompt no longer has {}",
            example
        );
    }
    let mut checked: usize = 0;
    for entry in std::fs::read_dir(format!("{}text", NL)).expect("the corpus texts") {
        let path = entry.expect("a directory entry").path();
        let text: String = std::fs::read_to_string(&path).expect("a readable text");
        for example in ["S12T", "S?T"] {
            assert!(!text.contains(example), "{:?} contains {}", path, example);
        }
        checked += 1;
    }
    assert_eq!(checked, 24);
}

// Every corpus reply that parses is what this schema describes: the model's
// constrained output and the parser agree.
#[test]
fn the_corpus_replies_the_parser_reads_fit_the_schema() {
    let schema: Value = serde_json::from_str(EXTRACTION_SCHEMA).expect("the schema is JSON");
    let allowed: Vec<String> = schema["properties"]
        .as_object()
        .expect("properties")
        .keys()
        .cloned()
        .collect();
    for entry in std::fs::read_dir(format!("{}reply", NL)).expect("the corpus replies") {
        let path = entry.expect("a directory entry").path();
        let text: String = std::fs::read_to_string(&path).expect("a readable reply");
        if parse_extraction(&text).is_ok() {
            let reply: Value = serde_json::from_str(&text).expect("a parsed reply is JSON");
            for key in reply.as_object().expect("an object").keys() {
                assert!(
                    allowed.contains(key),
                    "{:?}: {} is not in the schema",
                    path,
                    key
                );
            }
        }
    }
}
