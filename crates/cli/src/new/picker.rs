//! The `luria new` file picker's listing (milestone 7m5, contract 7m5 §2.1):
//! one directory's folders and template files, read once with `fs::read_dir`.
//! This is the only file reading the picker does; it writes nothing, and the
//! form, the drawing and the loop see the entries as `list` gives them.
//!
//! # The rules this module applies (contract §2.1)
//!
//! - folders are kept, and files whose extension is one of
//!   `TEMPLATE_EXTENSIONS`, compared case-insensitively;
//! - names starting with `.` are left out, and so are names that are not valid
//!   UTF-8;
//! - folders first, then files, each sorted by name;
//! - a directory that cannot be read gives an empty list;
//! - a symlink counts as what it points at, since `fs::metadata` follows it, and
//!   a broken symlink, whose metadata cannot be read, is left out.

use std::fs;
use std::path::Path;

/// One entry of the listing: the name as the directory holds it, and whether it
/// is a folder. A folder's name carries no `/`; the drawing adds it (contract
/// §2.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub folder: bool,
}

/// The file extensions a template can be read from (M7-50), as they are written
/// in the listing, lower-case.
pub const TEMPLATE_EXTENSIONS: [&str; 6] = ["fa", "fasta", "fna", "gb", "gbk", "genbank"];

/// `dir`'s folders, then its template files, each in name order. A directory
/// that cannot be read lists nothing.
pub fn list(dir: &Path) -> Vec<Entry> {
    let mut folders: Vec<Entry> = Vec::new();
    let mut files: Vec<Entry> = Vec::new();
    let items: std::fs::ReadDir = match fs::read_dir(dir) {
        Ok(found) => found,
        Err(_) => return Vec::new(),
    };
    for item in items {
        let item: fs::DirEntry = match item {
            Ok(found) => found,
            Err(_) => continue,
        };
        // A name that is not valid UTF-8 has no text to show.
        let raw: std::ffi::OsString = item.file_name();
        let name: &str = match raw.to_str() {
            Some(found) => found,
            None => continue,
        };
        if name.starts_with('.') {
            continue;
        }
        // A symlink counts as what it points at, and a broken one is left out.
        let meta: std::fs::Metadata = match fs::metadata(item.path()) {
            Ok(found) => found,
            Err(_) => continue,
        };
        if meta.is_dir() {
            folders.push(Entry {
                name: name.to_string(),
                folder: true,
            });
        } else if is_template(name) {
            files.push(Entry {
                name: name.to_string(),
                folder: false,
            });
        }
    }
    folders.sort_by(|a, b| a.name.cmp(&b.name));
    files.sort_by(|a, b| a.name.cmp(&b.name));
    folders.extend(files);
    folders
}

/// A file whose extension, read after its last `.`, is a template extension,
/// compared case-insensitively. A name with no `.` is not a template file.
fn is_template(name: &str) -> bool {
    match name.rsplit_once('.') {
        Some((_, ext)) => TEMPLATE_EXTENSIONS
            .iter()
            .any(|known| ext.eq_ignore_ascii_case(known)),
        None => false,
    }
}
