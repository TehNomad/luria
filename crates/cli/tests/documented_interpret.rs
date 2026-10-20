//! `docs/cli.md` §10 against the code (contract 6j): the command line it
//! shows is the one `luria help` prints, its question table lists exactly
//! the codes `build` can ask, and its profile example is a profile luria
//! reads, the corpus's own.

use luria_cli::args::USAGE;
use luria_cli::interpret::build::QuestionCode;
use luria_cli::request::parse_profile;
use std::fs;

const PAGE: &str = "../../docs/cli.md";

fn section_10() -> String {
    let page: String = fs::read_to_string(PAGE).expect("docs/cli.md is readable");
    page.split("## 10. Natural language: `luria q5 interpret`")
        .nth(1)
        .expect("docs/cli.md has §10")
        .to_string()
}

/// The first fenced block after `marker` in `text`, without its fences.
fn fenced_after(text: &str, marker: &str) -> String {
    let after: &str = text
        .split(marker)
        .nth(1)
        .unwrap_or_else(|| panic!("no {:?}", marker));
    let block: &str = after.split("```").nth(1).expect("a fenced block");
    block.split_once('\n').expect("a fence line").1.to_string()
}

#[test]
fn the_documented_command_line_is_the_usage_text() {
    let section: String = section_10();
    let shown: String = fenced_after(&section, "D044 and `docs/tasks/milestone-6-nl.md`.");
    let lines: Vec<&str> = shown.lines().map(|line| line.trim()).collect();
    assert_eq!(lines.len(), 3);
    for line in lines {
        assert!(USAGE.contains(line), "luria help does not show {:?}", line);
    }
}

#[test]
fn the_question_table_lists_every_code_build_can_ask() {
    let section: String = section_10();
    let table: &str = section
        .split("<!-- luria:question-codes -->")
        .nth(1)
        .expect("the question-codes marker");
    let mut documented: Vec<String> = table
        .lines()
        .skip_while(|line| line.trim().is_empty())
        .take_while(|line| line.starts_with('|'))
        .skip(2)
        .map(|line| {
            line.split('`')
                .nth(1)
                .expect("a code in backticks")
                .to_string()
        })
        .collect();
    documented.sort();
    let mut codes: Vec<String> = [
        QuestionCode::OperationUnsupported,
        QuestionCode::MutationUnstated,
        QuestionCode::MutationNotationUnsupported,
        QuestionCode::ResidueNumberUnstated,
        QuestionCode::OriginalResidueUnstated,
        QuestionCode::TargetResidueUnstated,
        QuestionCode::SubstitutionDirectionUnclear,
        QuestionCode::CodonUnstated,
        QuestionCode::CdsUnnamed,
        QuestionCode::CdsNotFound,
        QuestionCode::CdsUnusable,
        QuestionCode::CompositionUnstated,
        QuestionCode::NumberingUnstated,
        QuestionCode::NumberingUnsupported,
        QuestionCode::DesignUnstated,
    ]
    .iter()
    .map(|code| code.code().to_string())
    .collect();
    codes.sort();
    assert_eq!(documented, codes);
}

#[test]
fn the_documented_profile_is_the_corpus_profile() {
    let section: String = section_10();
    let shown: String = fenced_after(&section, "and pass it with `--design`:");
    let corpus: String = fs::read_to_string("../../tests/fixtures/nl/profile/full.json")
        .expect("the corpus profile");
    assert_eq!(parse_profile(&shown).ok(), parse_profile(&corpus).ok());
    assert!(parse_profile(&shown).is_ok());
}

#[test]
fn d044_is_recorded() {
    let decisions: String = fs::read_to_string("../../docs/decisions.md").expect("decisions");
    assert_eq!(
        decisions
            .matches("## D044: Natural-language interpretation (Milestone 6)")
            .count(),
        1
    );
}
