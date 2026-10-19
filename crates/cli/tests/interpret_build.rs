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
        sha256: template.sequence().content_hash().hex_digest().to_string(),
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
                assert_eq!(
                    parse_request(&echo).ok(),
                    Some(request),
                    "{}: round trip",
                    row[0]
                );
                seen[0] += 1;
            }
            ("needs_clarification", Built::NeedsClarification { questions }) => {
                let mut found: Vec<&str> = questions
                    .iter()
                    .map(|question| question.code.code())
                    .collect();
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
            (state, built) => panic!(
                "{}: the corpus says {}, build gave {:?}",
                row[0], state, built
            ),
        }
    }
    assert_eq!(
        seen,
        [5, 15, 2, 2],
        "proposed, clarification, ungrounded, invalid"
    );
}

#[test]
fn the_ungrounded_corpus_cases_name_the_string() {
    let rows: Vec<Vec<String>> = cases();
    let find = |id: &str| -> Vec<String> {
        rows.iter()
            .find(|row| row[0] == id)
            .expect("the case exists")
            .clone()
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
