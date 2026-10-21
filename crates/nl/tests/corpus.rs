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
    assert!(
        rows.len() == 24,
        "cases.tsv has {} cases, expected 24",
        rows.len()
    );
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
                assert!(
                    !invalid,
                    "{}: parsed, but the corpus calls it invalid",
                    case
                );
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
        refused
            == vec![
                "reply-extra-field".to_string(),
                "reply-wrong-type".to_string()
            ],
        "the refused cases are {:?}",
        refused
    );
    assert!(
        parsed.len() == 22,
        "{} replies parsed, expected 22",
        parsed.len()
    );
}

// A fabricated quote is not this crate's business: it is shape-valid, and
// `luria-cli` refuses it when it grounds the extraction (plan §3, 6e).
#[test]
fn an_ungrounded_reply_is_still_shape_valid_here() {
    for case in ["ungrounded-mutation", "ungrounded-cds"] {
        let path: String = format!("{}reply/{}.json", CORPUS, case);
        let reply: String = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
        parse_extraction(&reply).unwrap_or_else(|error| panic!("{}: {:?}", case, error.message));
    }
}
