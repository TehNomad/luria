//! Milestone 7k2's proof (contract 7k2 §3): the forms of
//! `tests/fixtures/new/reference/requests.py`, written out in Rust as the
//! inputs they are, build the request documents `request-<case>.json` holds.
//! Every template is read from `tests/fixtures/` at test time, as the form of
//! 7k3 carries it, so the form's text is the file's text, not a copy. The
//! request is rendered with `result::request_echo` and compared as a
//! `serde_json::Value` with the fixture, and `request::parse_request` of that
//! rendering must give the same document back. A refusal is the code and the
//! line of `refusals.tsv`.

use luria_cli::new::build;
use luria_cli::new::Built;
use luria_cli::new::CdsChoice;
use luria_cli::new::CodonChoice;
use luria_cli::new::Form;
use luria_cli::new::TemplateInput;
use luria_cli::request::parse_request;
use luria_cli::request::RequestDocument;
use luria_cli::request::StrandSpec;
use luria_cli::request::TopologySpec;
use luria_cli::result::request_echo;
use luria_cli::result::RequestEcho;
use serde_json::Value;
use std::fs;

const FIXTURES: &str = "../../tests/fixtures/";

/// `requests.py`'s `SYN`: the 100 bases of the pasted sequence.
const SYN: &str = "ATGGCTTGGAATCAGCAGCAGCCGAGCGGCTCGCGGACGCATGGCTTGGAATCAGCAGCA\
                   GCCGAGCGGCTCGCGGACGCATGGCTTGGAATCAGCAGCA";

/// The `BUILT` cases, in `requests.py`'s order.
const BUILT: &[&str] = &[
    "sid1-codon",
    "sid1-set",
    "synlin-genbank",
    "syn-paste",
    "syn-minus",
];

/// One fixture template, read at test time (contract §3).
fn read_file(relative: &str) -> String {
    let path: String = format!("{}{}", FIXTURES, relative);
    fs::read_to_string(&path).unwrap_or_else(|problem| panic!("cannot read {}: {}", path, problem))
}

/// The `sid-1` FASTA form, as `requests.py`'s `sid1` writes it: the file's
/// name and text, the whole sequence as the CDS on the plus strand, and the
/// lines, codon choice and overlap of the case.
fn sid1(lines: &[&str], codon: CodonChoice, overlap: Option<u64>) -> Form {
    Form {
        template: TemplateInput::Fasta {
            file_name: "sid-1_cDNA.fa".to_string(),
            text: read_file("example-construct/input/sid-1_cDNA.fa"),
            name: "sid-1".to_string(),
            topology: TopologySpec::Linear,
            cds: CdsChoice::Whole(StrandSpec::Plus),
        },
        lines: lines.iter().map(|line| line.to_string()).collect(),
        codon,
        overlap,
    }
}

/// The pasted `syn` form, as `requests.py`'s `syn_paste` writes it: the paste
/// text as typed, the name it is given, the file name it is saved as, the CDS
/// the form states, and the lines, codon choice and overlap of the case.
fn paste(
    lines: &[&str],
    typed: &str,
    save_as: &str,
    cds: CdsChoice,
    codon: CodonChoice,
    overlap: Option<u64>,
) -> Form {
    Form {
        template: TemplateInput::Paste {
            text: typed.to_string(),
            name: "syn".to_string(),
            save_as: save_as.to_string(),
            topology: TopologySpec::Linear,
            cds,
        },
        lines: lines.iter().map(|line| line.to_string()).collect(),
        codon,
        overlap,
    }
}

/// The pasted text of `requests.py`'s `syn_paste`: `SYN[:50]`, a newline and
/// two spaces, then `SYN[50:]` lower case.
fn typed_paste() -> String {
    format!("{}\n  {}", &SYN[..50], SYN[50..].to_lowercase())
}

/// One `BUILT` form: the 1-based inclusive bases `requests.py` states for the
/// `syn-paste` CDS.
fn syn_bases() -> CdsChoice {
    CdsChoice::Bases {
        first: 1,
        last: 99,
        strand: StrandSpec::Plus,
    }
}

/// The form of one case, `BUILT` or `REFUSED`, with the values of
/// `requests.py`'s `BUILT` and `REFUSED` lists.
fn form_of(case: &str) -> Form {
    match case {
        "sid1-codon" => sid1(&["D404N:AAC"], CodonChoice::TypedPerMutation, Some(15)),
        "sid1-set" => sid1(
            &[
                "D404N",
                "del 1210-1212",
                "",
                "ins GAT after 1209",
                "1210 G>A",
                "1210-1212 GAT>AAC",
            ],
            CodonChoice::DefaultPolicy,
            Some(15),
        ),
        "synlin-genbank" => Form {
            template: TemplateInput::GenBank {
                file_name: "synthetic-linear.gb".to_string(),
                text: read_file("genbank/synthetic-linear.gb"),
                cds: "synA".to_string(),
            },
            lines: ["A42G", "W87A"]
                .iter()
                .map(|line| line.to_string())
                .collect(),
            codon: CodonChoice::DefaultPolicy,
            overlap: Some(20),
        },
        "syn-paste" => paste(
            &["del 46", "51-53 ATC>TT"],
            &typed_paste(),
            "syn.fa",
            syn_bases(),
            CodonChoice::TypedPerMutation,
            Some(15),
        ),
        "syn-minus" => paste(
            &["ins GATTACAGATTA after 50"],
            SYN,
            "syn-minus.fa",
            CdsChoice::Whole(StrandSpec::Minus),
            CodonChoice::TypedPerMutation,
            Some(15),
        ),

        // The `REFUSED` forms, in `requests.py`'s order.
        "no-lines" => sid1(&["", "  "], CodonChoice::DefaultPolicy, Some(15)),
        "bad-line" => sid1(&["D404N", "D404*"], CodonChoice::DefaultPolicy, Some(15)),
        "wrong-from" => sid1(&["1210 A>G"], CodonChoice::DefaultPolicy, Some(15)),
        "past-end" => sid1(&["del 3150-3154"], CodonChoice::DefaultPolicy, Some(15)),
        "insert-at-end" => sid1(
            &["ins GAT after 3153"],
            CodonChoice::DefaultPolicy,
            Some(15),
        ),
        "insert-at-start" => sid1(&["ins GAT after 0"], CodonChoice::DefaultPolicy, Some(15)),
        "no-codon" => sid1(&["D404N"], CodonChoice::TypedPerMutation, Some(15)),
        "no-overlap" => sid1(&["D404N"], CodonChoice::DefaultPolicy, None),
        "same-twice" => sid1(&["D404N", "d404n"], CodonChoice::DefaultPolicy, Some(15)),
        "paste-not-dna" => paste(
            &["del 46", "51-53 ATC>TT"],
            "ATGGCT1GGAATC",
            "syn.fa",
            syn_bases(),
            CodonChoice::TypedPerMutation,
            Some(15),
        ),
        "cds-outside" => paste(
            &["del 46", "51-53 ATC>TT"],
            &typed_paste(),
            "syn.fa",
            CdsChoice::Bases {
                first: 1,
                last: 101,
                strand: StrandSpec::Plus,
            },
            CodonChoice::TypedPerMutation,
            Some(15),
        ),
        _ => panic!("{} is not a case of requests.py", case),
    }
}

/// A paste case's file name, `None` when the template came from a file.
fn paste_name(case: &str) -> Option<String> {
    match case {
        "syn-paste" => Some("syn.fa".to_string()),
        "syn-minus" => Some("syn-minus.fa".to_string()),
        _ => None,
    }
}

/// The request as a request file's text: `result::request_echo`'s rendering,
/// the JSON `request::parse_request` reads.
fn rendered(request: &RequestDocument) -> String {
    let echo: RequestEcho = request_echo(request);
    serde_json::to_string_pretty(&echo).expect("the echo is JSON")
}

/// One fixture as the JSON value it holds (key order free, contract §3).
fn fixture(relative: &str) -> Value {
    serde_json::from_str(&read_file(relative)).expect("the fixture is JSON")
}

/// One `BUILT` case: the rendered request is `request-<case>.json`, and a
/// paste's `Built.paste` is its file name and the text of `paste-<case>.fa`.
fn check_built(case: &str) {
    let built: Built = build(&form_of(case)).unwrap_or_else(|(refusal, line)| {
        panic!(
            "{}: refused as {} at line {:?}: {}",
            case,
            refusal.code(),
            line,
            refusal.message()
        )
    });
    assert_eq!(
        serde_json::from_str::<Value>(&rendered(&built.request)).expect("the rendering is JSON"),
        fixture(&format!("new/request-{}.json", case)),
        "the request of {}",
        case
    );
    let wanted: Option<(String, String)> =
        paste_name(case).map(|name| (name, read_file(&format!("new/paste-{}.fa", case))));
    assert_eq!(built.paste, wanted, "the paste of {}", case);
}

#[test]
fn builds_sid1_codon() {
    check_built("sid1-codon");
}

#[test]
fn builds_sid1_set() {
    check_built("sid1-set");
}

#[test]
fn builds_synlin_genbank() {
    check_built("synlin-genbank");
}

#[test]
fn builds_syn_paste() {
    check_built("syn-paste");
}

#[test]
fn builds_syn_minus() {
    check_built("syn-minus");
}

#[test]
fn every_refusal_matches() {
    let text: String = read_file("new/refusals.tsv");
    let rows: Vec<&str> = text.lines().collect();
    assert_eq!(rows[0], "case_id\tcode\tline");
    for row in rows[1..].iter() {
        let columns: Vec<&str> = row.split('\t').collect();
        assert_eq!(
            columns.len(),
            3,
            "the row {:?} is not three columns",
            columns
        );
        let case: &str = columns[0];
        match build(&form_of(case)) {
            Ok(got) => panic!("{} was not refused: {:?}", case, got),
            Err((refusal, line)) => {
                assert_eq!(refusal.code(), columns[1], "the code of {}", case);
                let wanted: Option<usize> = if columns[2] == "-" {
                    None
                } else {
                    Some(columns[2].parse::<usize>().expect("a line number"))
                };
                assert_eq!(line, wanted, "the line of {}", case);
            }
        }
    }
}

#[test]
fn a_built_request_round_trips() {
    for case in BUILT.iter() {
        let built: Built = build(&form_of(case)).expect("the form of the case builds");
        let request: RequestDocument =
            parse_request(&rendered(&built.request)).expect("the rendering is a request");
        assert_eq!(request, built.request, "the round trip of {}", case);
    }
}
