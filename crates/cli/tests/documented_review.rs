//! `docs/cli.md` §11 against the code (contract 7h): the command lines the page
//! shows are the ones `luria help` prints, and its order table's scale rule is
//! `luria_cli::review::order::scale`'s, the one the export writes rows with.

use luria_cli::args::USAGE;
use luria_cli::review::order::scale;
use std::fs;

const PAGE: &str = "../../docs/cli.md";

/// Everything after §11's heading, so a marker below is found in §11 alone.
fn section_11() -> String {
    let page: String = fs::read_to_string(PAGE).expect("docs/cli.md is readable");
    page.split("## 11. Reviewing in the terminal: `luria review`")
        .nth(1)
        .expect("docs/cli.md has §11")
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

/// The scale table after `<!-- luria:order-scales -->`, one row as its two
/// length ends and the code the page gives for that range.
fn documented_scales() -> Vec<(u64, u64, String)> {
    section_11()
        .split("<!-- luria:order-scales -->")
        .nth(1)
        .expect("the order-scales marker")
        .lines()
        .skip_while(|line| !line.starts_with('|'))
        .take_while(|line| line.starts_with('|'))
        .skip(2)
        .map(|line| {
            let cells: Vec<&str> = line.split('|').map(|cell| cell.trim()).collect();
            let ends: Vec<&str> = cells[1].split('-').collect();
            (
                ends[0].parse().expect("a row's first length"),
                ends[1]
                    .trim_end_matches(" nt")
                    .parse()
                    .expect("a row's last length"),
                cells[2].trim_matches('`').to_string(),
            )
        })
        .collect()
}

#[test]
fn the_documented_command_lines_are_the_usage_text() {
    let section: String = section_11();
    let shown: String = fenced_after(&section, "docs/tasks/milestone-7-tui.md`.");
    let lines: Vec<&str> = shown.lines().map(|line| line.trim()).collect();
    assert_eq!(lines.len(), 4, "§11 shows four command lines");
    for line in lines {
        assert!(USAGE.contains(line), "luria help does not show {:?}", line);
    }
}

#[test]
fn the_documented_scales_are_the_rule() {
    let rows: Vec<(u64, u64, String)> = documented_scales();
    assert_eq!(rows.len(), 4, "the page's scale table gives four rows");
    for (first, last, code) in &rows {
        assert_eq!(
            scale(*first),
            Some(code.as_str()),
            "the page gives {} for {}-{} nt, so scale({}) gives it",
            code,
            first,
            last,
            first
        );
        assert_eq!(
            scale(*last),
            Some(code.as_str()),
            "the page gives {} for {}-{} nt, so scale({}) gives it",
            code,
            first,
            last,
            last
        );
    }
    assert_eq!(
        scale(14),
        None,
        "the page's table starts at 15 nt, so 14 is outside it"
    );
    assert_eq!(
        scale(201),
        None,
        "the page's table ends at 200 nt, so 201 is outside it"
    );
}
