//! The `luria` executable: parse the command line, do the one thing this
//! build can do, and report it with an exit status (plan §3.7).

use std::env;
use std::io::{stderr, stdout, Write};
use std::process::ExitCode;

use luria_cli::args::{parse, Command, UsageError, USAGE};
use luria_cli::exit::ExitStatus;
use luria_cli::version::render_version;

/// Write `text` on stdout; false when the write or the flush failed (plan
/// §3.7: a document that cannot be written is an internal error, exit 1).
fn write_out(text: &str) -> bool {
    let mut stream = stdout().lock();
    if !(stream.write_all(text.as_bytes()).is_ok()) {
        return false;
    }
    stream.flush().is_ok()
}

/// Write `text` on stderr; a failed write changes nothing, because the run
/// still reports the status of its own failure (plan §3.7).
fn write_err(text: &str) -> bool {
    let mut stream = stderr().lock();
    if !(stream.write_all(text.as_bytes()).is_ok()) {
        return false;
    }
    stream.flush().is_ok()
}

/// Print `luria help`'s usage text on stdout (D041: the text is `args`').
fn emit_help() -> ExitStatus {
    if write_out(USAGE) {
        return ExitStatus::Success;
    }
    ExitStatus::Internal
}

/// Print `luria version`'s document on stdout.
fn emit_version() -> ExitStatus {
    if write_out(render_version().as_str()) {
        return ExitStatus::Success;
    }
    ExitStatus::Internal
}

/// `luria q5 design`'s arguments are parsed here, but the command itself is
/// wired in 5g; until then it says so and exits 1.
const NOT_WIRED: &str = "luria: 'q5 design' is not available in this build yet\n";

fn not_wired() -> ExitStatus {
    write_err(NOT_WIRED);
    ExitStatus::Internal
}

/// Report a usage error on stderr, with the usage text repeated (plan §3.7,
/// exit code 2: text on stderr, nothing on stdout).
fn report_usage(problem: UsageError) -> ExitStatus {
    let message: String = format!("luria: {}\n\n{}", problem.message, USAGE);
    write_err(message.as_str());
    ExitStatus::Usage
}

/// Run one `luria` command line.
fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let status: ExitStatus = match parse(&arguments) {
        Ok(Command::Help) => emit_help(),
        Ok(Command::Version) => emit_version(),
        Ok(Command::Q5Design {
            request: _,
            write_edited: _,
        }) => not_wired(),
        Err(problem) => report_usage(problem),
    };

    ExitCode::from(status.code())
}
