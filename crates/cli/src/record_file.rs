//! One sequence file to the record the view and `luria find` read it as
//! (contract 8i2 §4.3; D048's M8-14, D049, D056). This is the reader 8h's view
//! will share: the byte ceiling, the text, the record name the file gives, and
//! `luria_core::view_record::open_record` — nothing else.
//!
//! The ceiling is the cli's, not core's (D048): `open_record` takes text and
//! never a file, so the only place a file's length can be checked is here, and
//! the check comes before the bytes are read. `MAX_RECORD_BYTES` is 64 MiB
//! (D056): a fully annotated 10 Mb GenBank record is 25-30 MB and a 10 Mb FASTA
//! about 10 MB, so a record inside core's base ceiling is never refused by
//! bytes, while a gigabase FASTA never reaches a parser. Core's own ceiling,
//! `MAX_RECORD_BASES`, counts bases and stays core's (D048, M8-14); neither has
//! a flag or an environment variable (plan §8).
//!
//! What the file *is* is decided by `open_record` and by nothing here: the
//! format comes from the content, the topology from the record, and a refusal
//! comes back as the `DomainError` core gave, code, message and category
//! unchanged (D049, D053). The record's name is the file's name, sanitised to
//! the single printable ASCII token `Identifier` is (D007), because a file name
//! is free text and `open_record`'s messages quote the name they name.

use std::fs;
use std::path::Path;

use luria_core::error::DomainError;
use luria_core::identity::Identifier;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;

use crate::exit::ExitStatus;

/// The ceiling on the bytes of a file luria reads: exactly 64 MiB, checked on
/// the file's own metadata before its bytes are read (D056, D048 M8-14). The
/// boundary is tested at this number and at one byte past it (8i2 test 11).
pub const MAX_RECORD_BYTES: u64 = 67_108_864;

/// Why a path did not become a record: the file itself, or `open_record`'s own
/// refusal returned exactly as core gave it.
#[derive(Clone, Debug, PartialEq)]
pub enum RecordFileProblem {
    /// Bigger than `MAX_RECORD_BYTES`; `bytes` is the length the metadata gave.
    TooLarge { bytes: u64 },
    /// The file could not be read at all; `cause` is what the OS said.
    Unreadable { cause: String },
    /// The bytes are not UTF-8 text.
    NotUtf8,
    /// `open_record` refused the text: the format, the reader's own rule, or
    /// the base ceiling (§4.3 hands these back unchanged).
    Record(DomainError),
}

impl RecordFileProblem {
    /// The one line the run prints for this problem — no `luria: ` prefix, no
    /// trailing newline — and the status it exits with (§4.3): the byte ceiling
    /// is `Unsupported`, a file that cannot be read or is not text is
    /// `MalformedInput`, and a record refusal carries its own category, exactly
    /// as `ExitStatus::for_failure` reads it (D056, docs/cli.md §5-§6).
    pub fn report(&self, file: &Path) -> (String, ExitStatus) {
        match self {
            RecordFileProblem::TooLarge { bytes } => (
                format!(
                    "{} is {} bytes; luria reads files of at most {} bytes (64 MiB)",
                    file.display(),
                    bytes,
                    MAX_RECORD_BYTES
                ),
                ExitStatus::Unsupported,
            ),
            RecordFileProblem::Unreadable { cause } => (
                format!("cannot read {}: {}", file.display(), cause),
                ExitStatus::MalformedInput,
            ),
            RecordFileProblem::NotUtf8 => (
                format!("{} is not UTF-8 text", file.display()),
                ExitStatus::MalformedInput,
            ),
            RecordFileProblem::Record(error) => {
                (error.message(), ExitStatus::for_failure(error.category()))
            }
        }
    }
}

/// The record name the file gives it (§4.3): the file's name with every
/// character outside printable ASCII — which includes every space, as does
/// nothing at or below a space and anything above `~` — replaced by `_`, and
/// `record` when that leaves nothing. `Identifier` is one printable ASCII token
/// with no whitespace (D007), and a file name is free text — a path with a
/// space, a name with a newline in it, a non-ASCII name — so the characters that
/// cannot be in an identifier are replaced rather than dropped, which keeps the
/// name the length of the file name it came from. `open_record` is then named by
/// that value in every message it writes (D007).
fn record_name(path: &Path) -> Identifier {
    let written: String = match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        // `..`, `/`, a trailing separator: no last component to name the record
        // by, which is the same empty name the rule below answers `record` for.
        None => String::new(),
    };

    let clean: String = written
        .chars()
        .map(|ch| {
            // 0x21..=0x7e is printable ASCII; a space, a control character and a
            // non-ASCII character all fall outside it and become `_`.
            if ch <= ' ' || ch > '~' {
                '_'
            } else {
                ch
            }
        })
        .collect();

    let clean: String = if clean.is_empty() {
        "record".to_string()
    } else {
        clean
    };

    match Identifier::create(clean) {
        Ok(name) => name,
        // Unreachable: every character left is in the range `Identifier::create`
        // accepts (0x21..=0x7e, `_` included), and an empty name became `record`.
        Err(_) => unreachable!("a sanitised file name is always a valid identifier"),
    }
}

/// Read one file into the record `open_record` builds from it: the byte ceiling
/// first, on the length the metadata gives, before any byte is read (D048); then
/// the whole text, refused when it is not UTF-8; then `open_record`, with the
/// name the file gives it. The order is §4.3's, and the first failure is the one
/// reported. Nothing here looks at the content: the format, the topology, the
/// base ceiling and every record rule are `open_record`'s, and its refusal is
/// returned as it came (D049).
pub fn read_record_file(path: &Path) -> Result<ViewRecord, RecordFileProblem> {
    let bytes: u64 = match fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(cause) => {
            return Err(RecordFileProblem::Unreadable {
                cause: cause.to_string(),
            })
        }
    };
    if bytes > MAX_RECORD_BYTES {
        return Err(RecordFileProblem::TooLarge { bytes });
    }

    let raw: Vec<u8> = match fs::read(path) {
        Ok(raw) => raw,
        Err(cause) => {
            return Err(RecordFileProblem::Unreadable {
                cause: cause.to_string(),
            })
        }
    };
    let text: String = match String::from_utf8(raw) {
        Ok(text) => text,
        Err(_) => return Err(RecordFileProblem::NotUtf8),
    };

    match open_record(record_name(path), text) {
        Ok(record) => Ok(record),
        Err(error) => Err(RecordFileProblem::Record(error)),
    }
}
