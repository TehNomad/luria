//! `docs/cli.md` §13 against the code (contract 8n): the command lines the
//! page shows are the ones `luria help` prints, and the two `luria find`
//! examples print what the page says they print.

use luria_cli::args::USAGE;
use std::fs;
use std::process::Command;

const PAGE: &str = "../../docs/cli.md";

/// Everything after §13's heading, so a marker below is found in §13 alone.
fn section_13() -> String {
    let page: String = fs::read_to_string(PAGE).expect("docs/cli.md is readable");
    page.split("## 13. Looking at a sequence: `luria FILE` and `luria find`")
        .nth(1)
        .expect("docs/cli.md has §13")
        .to_string()
}

/// Every fenced block of `text`, in order, without its fences.
fn fenced_blocks(text: &str) -> Vec<String> {
    text.split("```")
        .skip(1)
        .step_by(2)
        .map(|block| block.split_once('\n').expect("a fence line").1.to_string())
        .collect()
}

/// The words of `line`, as the page and the program align them differently.
fn words(line: &str) -> Vec<&str> {
    line.split_whitespace().collect()
}

#[test]
fn the_documented_command_lines_are_the_usage_text() {
    let blocks: Vec<String> = fenced_blocks(&section_13());
    let shown: &String = blocks.first().expect("§13 has a command block");
    let lines: Vec<&str> = shown.lines().map(|line| line.trim()).collect();
    assert_eq!(lines.len(), 2, "§13 shows two command lines");
    let usage: Vec<&str> = USAGE.lines().map(|line| line.trim()).collect();
    for line in lines {
        assert!(usage.contains(&line), "luria help does not show {:?}", line);
    }
}

#[test]
fn the_documented_find_examples_are_what_find_prints() {
    let examples: Vec<String> = fenced_blocks(&section_13())
        .into_iter()
        .filter(|block: &String| {
            block
                .lines()
                .next()
                .is_some_and(|line: &str| line.starts_with("$ luria find "))
        })
        .collect();
    assert_eq!(examples.len(), 2, "§13 shows two find examples");
    for example in &examples {
        let mut lines: std::str::Lines<'_> = example.lines();
        let first: &str = lines.next().expect("the example's $ command line");
        let arguments: Vec<&str> = first.trim_start_matches("$ ").split(' ').skip(1).collect();
        let output = Command::new(env!("CARGO_BIN_EXE_luria"))
            .args(&arguments)
            .current_dir("../..")
            .output()
            .expect("the luria executable runs");
        assert_eq!(
            output.status.code(),
            Some(0),
            "luria {:?} exits 0",
            arguments
        );
        let text: String = String::from_utf8(output.stdout).expect("stdout is text");
        let printed: Vec<&str> = text.lines().collect();
        assert_eq!(
            printed.len() + 1,
            example.lines().count(),
            "one example and luria find's output have a different number of lines"
        );
        let want: Vec<Vec<&str>> = lines.map(|line: &str| words(line)).collect();
        let have: Vec<Vec<&str>> = printed.iter().map(|line: &&str| words(line)).collect();
        assert_eq!(want, have, "one example line differs from what find prints");
    }
}
