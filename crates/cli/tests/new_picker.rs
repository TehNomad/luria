//! Milestone 7m5's proof (contract 7m5 §3, tests 1 and 2): `new::picker::list`
//! keeps the folders and the template files of one directory, and nothing else,
//! in the order the picker draws them. The directories are built under
//! `target/tmp`, created fresh in each test, so the listing is of the files the
//! test wrote and not of what a previous run left.

use luria_cli::new::picker::list;
use luria_cli::new::picker::Entry;
use std::fs;
use std::path::{Path, PathBuf};

/// A fresh directory under `target/tmp`: the old one with this name is removed,
/// so the listing holds only what the test creates.
fn fresh(name: &str) -> PathBuf {
    let dir: PathBuf = PathBuf::from("target/tmp").join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)
        .unwrap_or_else(|problem| panic!("cannot create {}: {}", dir.display(), problem));
    dir
}

/// Writes one file in `dir`, naming the failure.
fn write(dir: &Path, name: &str, text: &str) {
    let path: PathBuf = dir.join(name);
    fs::write(&path, text)
        .unwrap_or_else(|problem| panic!("cannot write {}: {}", path.display(), problem));
}

#[test]
fn list_keeps_templates_and_folders() {
    let dir: PathBuf = fresh("7m5-templates");
    write(&dir, "a.fa", ">a\nATG\n");
    write(&dir, "B.GB", "LOCUS       B                   3 bp\n");
    write(&dir, "c.txt", "not a template\n");
    write(&dir, ".hidden.fa", ">hidden\nATG\n");
    write(&dir, "d.fasta.bak", "not a template\n");
    fs::create_dir(dir.join("sub")).expect("the folder is created");

    let expected: Vec<Entry> = vec![
        Entry {
            name: "sub".to_string(),
            folder: true,
        },
        Entry {
            name: "B.GB".to_string(),
            folder: false,
        },
        Entry {
            name: "a.fa".to_string(),
            folder: false,
        },
    ];
    assert_eq!(
        list(&dir),
        expected,
        "the folder comes first, then the template files in name order, with no \
         hidden, no `c.txt` and no `d.fasta.bak`"
    );
}

#[test]
fn an_unreadable_directory_lists_nothing() {
    let dir: PathBuf = PathBuf::from("target/tmp").join("7m5-not-there");
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(
        list(&dir),
        Vec::<Entry>::new(),
        "a directory that is not there gives no entries"
    );
}
