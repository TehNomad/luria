//! The `luria` executable, run as a process: exit codes, which stream each
//! document goes to, and the exact text of `luria version`.

use luria_core::mutation::{MUTATION_ALGORITHM_NAME, MUTATION_ALGORITHM_VERSION};
use luria_core::primer::{PRIMER_DESIGN_NAME, PRIMER_DESIGN_VERSION};
use luria_core::primer_design::{NATIVE_DESIGN_ALGORITHM_NAME, NATIVE_DESIGN_ALGORITHM_VERSION};
use luria_core::q5::{Q5_WORKFLOW_NAME, Q5_WORKFLOW_VERSION};
use luria_core::tm::{TM_FUNCTION_NAME, TM_FUNCTION_VERSION};
use std::process::{Command, Output};

fn luria(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(arguments)
        .output()
        .expect("the luria executable runs")
}

fn algorithm_block(role: &str, name: &str, version: &str, last: bool) -> String {
    format!(
        "    {{\n      \"role\": \"{}\",\n      \"name\": \"{}\",\n      \"version\": \"{}\"\n    }}{}\n",
        role,
        name,
        version,
        if last { "" } else { "," }
    )
}

// The whole document is pinned, so the key order and the layout are too.
#[test]
fn version_prints_the_pinned_document() {
    let output = luria(&["version"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let blocks: String = [
        algorithm_block(
            "mutation",
            MUTATION_ALGORITHM_NAME,
            MUTATION_ALGORITHM_VERSION,
            false,
        ),
        algorithm_block("primer", PRIMER_DESIGN_NAME, PRIMER_DESIGN_VERSION, false),
        algorithm_block("tm", TM_FUNCTION_NAME, TM_FUNCTION_VERSION, false),
        algorithm_block(
            "native_design",
            NATIVE_DESIGN_ALGORITHM_NAME,
            NATIVE_DESIGN_ALGORITHM_VERSION,
            false,
        ),
        algorithm_block("q5_workflow", Q5_WORKFLOW_NAME, Q5_WORKFLOW_VERSION, true),
    ]
    .concat();
    let expected: String = format!(
        "{{\n  \"schema\": \"luria.version/1\",\n  \"package\": \"luria-cli\",\n  \"version\": \"{}\",\n  \"algorithms\": [\n{}  ],\n  \"schemas\": {{\n    \"request\": \"luria.q5-request/1\",\n    \"result\": \"luria.q5-result/1\"\n  }}\n}}\n",
        env!("CARGO_PKG_VERSION"),
        blocks
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
}

#[test]
fn version_is_byte_identical_run_to_run() {
    assert_eq!(luria(&["version"]).stdout, luria(&["--version"]).stdout);
}

#[test]
fn help_prints_the_usage_to_stdout() {
    for spelling in ["help", "--help", "-h"] {
        let output = luria(&[spelling]);
        assert_eq!(output.status.code(), Some(0), "{}", spelling);
        assert!(output.stderr.is_empty(), "{}", spelling);
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            luria_cli::args::USAGE
        );
    }
}

#[test]
fn a_usage_error_exits_2_with_text_on_stderr_and_nothing_on_stdout() {
    let cases: [(&[&str], &str); 4] = [
        (&[], "no command given"),
        (&["frobnicate"], "unknown command 'frobnicate'"),
        (&["q5", "design"], "'q5 design' needs a request file"),
        (&["q5", "design", "a.json", "-v"], "unknown option '-v'"),
    ];
    for (arguments, message) in cases {
        let output = luria(arguments);
        assert_eq!(output.status.code(), Some(2), "{:?}", arguments);
        assert!(output.stdout.is_empty(), "{:?}", arguments);
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("luria: {}\n\n{}", message, luria_cli::args::USAGE),
            "{:?}",
            arguments
        );
    }
}

// Added in review: a document that cannot be written is an I/O failure,
// exit 1 (plan §3.7), not a panic (101). /dev/full refuses every write.
#[cfg(target_os = "linux")]
#[test]
fn a_failed_write_to_stdout_exits_1() {
    for arguments in [&["version"][..], &["help"][..]] {
        let full = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .expect("/dev/full opens");
        let output = Command::new(env!("CARGO_BIN_EXE_luria"))
            .args(arguments)
            .stdout(full)
            .output()
            .expect("the luria executable runs");
        assert_eq!(output.status.code(), Some(1), "{:?}", arguments);
    }
}
