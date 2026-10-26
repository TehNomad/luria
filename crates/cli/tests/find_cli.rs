//! Contract 8i2 §5: `luria find`, run as a process.
//!
//! Every hit row here comes from `tests/fixtures/view/search-dna-hits.tsv` and
//! `tests/fixtures/view/search-protein-hits.tsv`, which Biopython produced and no
//! `crates/` program touched (see that directory's `README.md`). `find` prints
//! the same hits in 1-based inclusive coordinates, so a table row's `start` and
//! `length` become `start + 1` and `(start + length - 1) mod L + 1` (§4.8). The
//! length `4361 bp` and the three CDS spans come from
//! `tests/fixtures/genbank/raw/J01749.1.gb` as written, and the ROP CDS's
//! `no_id` refusal is what `tests/fixtures/README.md` and D053 record for it.

use luria_cli::args::USAGE;
use luria_cli::record_file::MAX_RECORD_BYTES;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};

/// Run one `luria find` command line, as `cli.rs` runs the executable.
fn luria(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(arguments)
        .output()
        .expect("the luria executable runs")
}

/// The test's own directory, the way `end_to_end.rs` makes one: under `target/`,
/// which is gitignored — a 64 MiB input belongs there, not in `/tmp`.
fn workspace(test: &str) -> PathBuf {
    let dir: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("find-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("the test directory is created");
    dir
}

/// pBR322 copied into that directory, and its absolute path: the `# file:` line
/// prints the path as it was given, so an absolute path makes an absolute
/// expectation.
fn pbr(test: &str) -> PathBuf {
    let path: PathBuf = workspace(test).join("J01749.1.gb");
    fs::copy("../../tests/fixtures/genbank/raw/J01749.1.gb", &path)
        .expect("the pBR322 fixture is copied");
    path
}

/// One FASTA file, written into the test's directory as `end_to_end.rs` writes
/// its inputs.
fn fasta(test: &str, name: &str, text: &str) -> PathBuf {
    let path: PathBuf = workspace(test).join(name);
    fs::write(&path, text).expect("the FASTA file is written");
    path
}

/// stdout or stderr as text; every case here prints text.
fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("the output is UTF-8")
}

/// The lines of `printed`, the empty one after its final newline dropped.
fn lines(printed: &str) -> Vec<String> {
    let mut parts: Vec<&str> = printed.split('\n').collect();
    parts.pop();
    parts.iter().map(|line| line.to_string()).collect()
}

/// 1: the EcoRI site at the end of pBR322 wraps the origin: the table's
/// zero-based 4358 with `wraps_origin yes` prints as `4359` and `3`, one hit,
/// and nothing is claimed after the count (D052).
#[test]
fn the_ecori_site_wraps_the_origin() {
    let record: PathBuf = pbr("ecori-wraps");
    let output: Output = luria(&["find", record.to_str().unwrap(), "GAATTC"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        text(&output.stdout),
        format!(
            "# file: {}\n\
             # record: GenBank, 4361 bp, circular\n\
             # searched: GAATTC as DNA on both strands\n\
             start\tend\tstrand\n\
             4359\t3\tboth\n\
             1 hit\n",
            record.display()
        )
    );
    assert!(output.stderr.is_empty());
}

/// 2: an internal site does not wrap: the table's zero-based 374, length 6,
/// `wraps_origin no` prints as `375` and `380`.
#[test]
fn an_internal_site() {
    let record: PathBuf = pbr("internal-site");
    let output: Output = luria(&["find", record.to_str().unwrap(), "GGATCC"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        text(&output.stdout),
        format!(
            "# file: {}\n\
             # record: GenBank, 4361 bp, circular\n\
             # searched: GGATCC as DNA on both strands\n\
             start\tend\tstrand\n\
             375\t380\tboth\n\
             1 hit\n",
            record.display()
        )
    );
    assert!(output.stderr.is_empty());
}

/// 3: a search that finds nothing is a successful run (D052's M8-9), the record
/// line says what was read and that its linearity was only assumed, and the last
/// line says what the run did not look at.
#[test]
fn nothing_found_is_success() {
    let record: PathBuf = fasta("no-hits", "w.fa", ">w\nTTCAAAAAAAAAAGAA\n");
    let output: Output = luria(&["find", record.to_str().unwrap(), "GAATTC"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        text(&output.stdout),
        format!(
            "# file: {}\n\
             # record: FASTA, 16 bp, linear (assumed: a FASTA file states no topology)\n\
             # searched: GAATTC as DNA on both strands\n\
             start\tend\tstrand\n\
             0 hits\n\
             linear: hits do not cross the end\n",
            record.display()
        )
    );
    assert!(output.stderr.is_empty());
}

/// 4: `--circular` on that same record finds the site that runs across its end:
/// the table's own wrap rule gives `14` and `3`, and the record line says the
/// circularity came from the flag, not from the file (§4.4, D049's M8-20).
#[test]
fn circular_finds_the_site_across_the_end() {
    let record: PathBuf = fasta("circular-wrap", "w.fa", ">w\nTTCAAAAAAAAAAGAA\n");
    let output: Output = luria(&["find", record.to_str().unwrap(), "GAATTC", "--circular"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        text(&output.stdout),
        format!(
            "# file: {}\n\
             # record: FASTA, 16 bp, circular (--circular)\n\
             # searched: GAATTC as DNA on both strands\n\
             start\tend\tstrand\n\
             14\t3\tboth\n\
             1 hit\n",
            record.display()
        )
    );
    assert!(output.stderr.is_empty());
}

/// 5: a CDS search names the CDS whose translation it read, in the list form the
/// plan gives, and its rows have six columns: the table's `tet` hit at zero-based
/// 85, length 18, frame `-`, residues 1 and 6.
#[test]
fn a_cds_search_names_the_cds() {
    let record: PathBuf = pbr("cds-names");
    let output: Output = luria(&["find", record.to_str().unwrap(), "MKSNNA", "--cds", "tet"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        text(&output.stdout),
        format!(
            "# file: {}\n\
             # record: GenBank, 4361 bp, circular\n\
             # searched: MKSNNA as protein in CDS tet (86..1276, +)\n\
             start\tend\tstrand\tframe\tfirst_residue\tlast_residue\n\
             86\t103\t+\t-\t1\t6\n\
             1 hit\n",
            record.display()
        )
    );
    assert!(output.stderr.is_empty());
}

/// 6: a hit that runs onto the CDS's terminal stop codon has no last residue
/// there, and says so with the word `stop` (D010, D050's M8-17): the table's
/// `TST*` at zero-based 1264, length 12, first residue 394 and `stop`. `*` is no
/// nucleotide, so the mode is protein without `--aa` (§4.5).
#[test]
fn the_stop_is_a_word() {
    let record: PathBuf = pbr("stop-word");
    let output: Output = luria(&["find", record.to_str().unwrap(), "TST*", "--cds", "tet"]);

    assert_eq!(output.status.code(), Some(0));
    let rows: Vec<String> = lines(&text(&output.stdout));
    assert_eq!(
        rows[2],
        "# searched: TST* as protein in CDS tet (86..1276, +)"
    );
    assert_eq!(
        rows[3],
        "start\tend\tstrand\tframe\tfirst_residue\tlast_residue"
    );
    assert_eq!(rows[4], "1265\t1276\t+\t-\t394\tstop");
    assert_eq!(rows[5], "1 hit");
    assert_eq!(
        rows.len(),
        6,
        "a CDS search on a circular record claims nothing else"
    );
    assert!(output.stderr.is_empty());
}

/// 7: the mode comes from the pattern's letters and not from a list of names:
/// `MSIQHF` holds none and is searched as protein in six frames — the table's hit
/// at zero-based 4135, length 18, minus strand, frame 2, codons 70 and 75 — while
/// `MKSNNA`, every letter an IUPAC nucleotide code, is searched as DNA (§4.5,
/// plan §5.4). Six frames never read across the origin, circular or not, and that
/// claim comes after the count (D052's M8-2).
#[test]
fn a_protein_letter_switches_the_mode() {
    let record: PathBuf = pbr("mode-letters");
    let path: &str = record.to_str().unwrap();

    let protein: Output = luria(&["find", path, "MSIQHF"]);
    assert_eq!(protein.status.code(), Some(0));
    let rows: Vec<String> = lines(&text(&protein.stdout));
    assert_eq!(rows[2], "# searched: MSIQHF as protein in six frames");
    assert_eq!(
        rows[3],
        "start\tend\tstrand\tframe\tfirst_residue\tlast_residue"
    );
    assert_eq!(rows[4], "4136\t4153\t-\t2\t70\t75");
    assert_eq!(rows[5], "1 hit");
    assert_eq!(rows[6], "frames do not cross the origin");
    assert_eq!(rows.len(), 7, "that claim is the last line");
    assert!(protein.stderr.is_empty());

    let dna: Output = luria(&["find", path, "MKSNNA"]);
    assert_eq!(dna.status.code(), Some(0));
    assert_eq!(
        lines(&text(&dna.stdout))[2],
        "# searched: MKSNNA as DNA on both strands",
        "MKSNNA is IUPAC letters, not a protein"
    );
}

/// 8: a value naming several CDSs is refused with every candidate — its span, its
/// strand, and the id source that matched it — and never with a choice (D049,
/// D005). pBR322's three `/protein_id`s are `AAB59735.1`, `AAB59736.1` and
/// `AAB59737.1`, on the CDSs `tet (86..1276, +)`, `ROP protein (1915..2106, +)`
/// and `bla (3293..4153, -)`, in the order the file writes them.
#[test]
fn several_cds_candidates_are_listed() {
    let record: PathBuf = pbr("cds-several");
    let output: Output = luria(&[
        "find",
        record.to_str().unwrap(),
        "MKSNNA",
        "--cds",
        "AAB5973",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        text(&output.stderr),
        format!(
            "luria: 'AAB5973' names 3 CDSs in {}; give one of them:\n  \
             tet (86..1276, +) matched by /protein_id 'AAB59735.1'\n  \
             ROP protein (1915..2106, +) matched by /protein_id 'AAB59736.1'\n  \
             bla (3293..4153, -) matched by /protein_id 'AAB59737.1'\n",
            record.display()
        )
    );
}

/// 9: a value naming nothing is refused, and a value naming one CDS that luria
/// cannot translate exits with that CDS's own refusal and its category's code:
/// pBR322's ROP CDS has none of `/locus_tag`, `/gene` or `/label`, so `import_cds`
/// refuses it as `no_id`, whose category is `Unsupported` (D041, D042, D053). The
/// value `rop` still reaches `ROP protein`, through core's prefix rule and not a
/// name rule written here (§4.6).
#[test]
fn an_unknown_cds_and_an_unusable_one() {
    let record: PathBuf = pbr("cds-unknown");
    let path: &str = record.to_str().unwrap();

    let nothing: Output = luria(&["find", path, "MKSNNA", "--cds", "lactamase"]);
    assert_eq!(nothing.status.code(), Some(2));
    assert!(nothing.stdout.is_empty());
    assert_eq!(
        text(&nothing.stderr),
        format!(
            "luria: no CDS in {} is named 'lactamase'\n",
            record.display()
        )
    );

    let unusable: Output = luria(&["find", path, "MKSNNA", "--cds", "rop"]);
    assert_eq!(unusable.status.code(), Some(6));
    assert!(unusable.stdout.is_empty());
    let stderr: String = text(&unusable.stderr);
    assert!(
        stderr.starts_with(
            "luria: CDS 'ROP protein' cannot be searched: a GenBank CDS has none of \
             /locus_tag, /gene or /label with a value, so it cannot be named"
        ),
        "{stderr}"
    );
    assert!(
        !stderr.contains(USAGE),
        "a refusal of a CDS name prints no usage text"
    );
}

/// 10: a range for `--cds` is refused before the file is read, with the message
/// that says when ranges come, and with the usage text, because it is the command
/// line that got the flag's value wrong (D049's M8-15, plan §5.5).
#[test]
fn a_range_is_refused_for_now() {
    for value in ["86..1276", "c345..43", "  86..1276  "] {
        let output: Output = luria(&[
            "find",
            "../../tests/fixtures/genbank/raw/J01749.1.gb",
            "MKSNNA",
            "--cds",
            value,
        ]);
        assert_eq!(output.status.code(), Some(2), "{value}");
        assert!(output.stdout.is_empty(), "{value}");
        let stderr: String = text(&output.stderr);
        assert!(
            stderr.starts_with("luria: searching a range of bases comes in milestone 9"),
            "{value}: {stderr}"
        );
        assert!(stderr.contains(USAGE), "{value}: the usage text follows");
    }
}

/// 11: the byte ceiling is the cli's and it comes from the file's own metadata,
/// before a byte of it is read: a file of exactly `MAX_RECORD_BYTES` gets as far
/// as the record reader, which cannot tell what format a page of blank lines is
/// written in (exit 4, §4.3), and one byte more is refused by the ceiling
/// (exit 6, D056, D048's M8-14).
#[test]
fn the_byte_ceiling_is_checked_before_reading() {
    assert_eq!(MAX_RECORD_BYTES, 67_108_864, "D056's ceiling");

    let dir: PathBuf = workspace("byte-ceiling");
    let at: PathBuf = dir.join("at-the-ceiling.gb");
    let over: PathBuf = dir.join("one-byte-over.gb");
    let blank: Vec<u8> = vec![b'\n'; MAX_RECORD_BYTES as usize];
    fs::write(&at, &blank).expect("the file of exactly the ceiling is written");
    fs::write(&over, &blank).expect("the file of exactly the ceiling is written");
    OpenOptions::new()
        .append(true)
        .open(&over)
        .and_then(|mut handle| handle.write_all(b"\n"))
        .expect("one byte more is appended");

    let inside: Output = luria(&["find", at.to_str().unwrap(), "GAATTC"]);
    assert_eq!(inside.status.code(), Some(4));
    assert_eq!(
        text(&inside.stderr),
        "luria: cannot tell what format record 'at-the-ceiling.gb' is written in: its \
         first non-blank line starts with neither 'LOCUS' (GenBank) nor '>' (FASTA)\n"
    );

    let outside: Output = luria(&["find", over.to_str().unwrap(), "GAATTC"]);
    assert_eq!(outside.status.code(), Some(6));
    assert_eq!(
        text(&outside.stderr),
        format!(
            "luria: {} is 67108865 bytes; luria reads files of at most 67108864 bytes \
             (64 MiB)\n",
            over.display()
        )
    );

    let _ = fs::remove_file(&at);
    let _ = fs::remove_file(&over);
}

/// 12: a GenBank record states its own topology (D036), so `--circular` is
/// refused rather than obeyed (D049's M8-20), with no usage text (§4.4).
#[test]
fn circular_is_refused_for_genbank() {
    let record: PathBuf = pbr("circular-genbank");
    let output: Output = luria(&["find", record.to_str().unwrap(), "GAATTC", "--circular"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        text(&output.stderr),
        format!(
            "luria: '--circular' is for FASTA files; {} states its own topology\n",
            record.display()
        )
    );
}

/// 13: the cap is presentation and never a rule of core (M8-12, D050's M8-16):
/// 300 hits come back from 300 `N`s, 200 print, and the count says what was cut
/// off. `N` matches every letter a record can hold, and its complement is itself,
/// so each base is one `both` hit, in ascending order — as the table's 4361 hits
/// for `N` on pBR322 show.
#[test]
fn the_cap_is_200_rows() {
    let record: PathBuf = fasta("cap", "n300.fa", &format!(">n300\n{}\n", "N".repeat(300)));
    let output: Output = luria(&["find", record.to_str().unwrap(), "N"]);

    assert_eq!(output.status.code(), Some(0));
    let rows: Vec<String> = lines(&text(&output.stdout));
    assert_eq!(rows[0], format!("# file: {}", record.display()));
    assert_eq!(
        rows[1],
        "# record: FASTA, 300 bp, linear (assumed: a FASTA file states no topology)"
    );
    assert_eq!(rows[2], "# searched: N as DNA on both strands");
    assert_eq!(rows[3], "start\tend\tstrand");

    let printed: Vec<String> = (1..=200)
        .map(|position| format!("{}\t{}\tboth", position, position))
        .collect();
    assert_eq!(rows[4..204], printed[..], "200 rows, and no more");
    assert_eq!(rows[204], "showing the first 200 of 300 hits");
    assert_eq!(rows[205], "300 hits");
    assert_eq!(rows[206], "linear: hits do not cross the end");
    assert_eq!(rows.len(), 207, "and nothing after the claim");
    assert!(output.stderr.is_empty());
}

/// 14: what the command line got wrong is exit 2 with the usage text (§4.1); what
/// the file or the pattern is, is exit 4 with core's own message and none of the
/// usage text (§4.3, §4.7).
#[test]
fn usage_and_input_errors() {
    let record: PathBuf = pbr("usage-errors");
    let path: &str = record.to_str().unwrap();

    let cases: [Vec<&str>; 6] = [
        vec!["find"],
        vec!["find", path],
        vec!["find", path, "GAATTC", "extra"],
        vec!["find", path, "GAATTC", "--bogus"],
        vec!["find", path, "GAATTC", "--aa", "--aa"],
        vec!["find", path, "GAATTC", "--cds"],
    ];
    for arguments in cases {
        let output: Output = luria(&arguments);
        assert_eq!(output.status.code(), Some(2), "{arguments:?}");
        assert!(output.stdout.is_empty(), "{arguments:?}");
        let stderr: String = text(&output.stderr);
        assert!(stderr.starts_with("luria: "), "{arguments:?}: {stderr}");
        assert!(
            stderr.contains(USAGE),
            "{arguments:?}: the usage text follows"
        );
    }

    let missing: PathBuf = record
        .parent()
        .expect("the fixture copy has a directory")
        .join("absent.gb");
    let absent: Output = luria(&["find", missing.to_str().unwrap(), "GAATTC"]);
    assert_eq!(absent.status.code(), Some(4));
    assert!(
        text(&absent.stderr).starts_with(&format!("luria: cannot read {}: ", missing.display())),
        "the message names the file it could not read"
    );
    assert!(
        !text(&absent.stderr).contains(USAGE),
        "a file that is not there prints no usage text"
    );

    let empty: Output = luria(&["find", path, ""]);
    assert_eq!(empty.status.code(), Some(4));
    assert_eq!(
        text(&empty.stderr),
        "luria: a search pattern was not supplied\n"
    );

    // `1` is no nucleotide, so the mode is protein, and `1` is no residue
    // either: core refuses the pattern and names the letter (§4.5, §4.7).
    let bad: Output = luria(&["find", path, "GAA1"]);
    assert_eq!(bad.status.code(), Some(4));
    assert_eq!(
        text(&bad.stderr),
        "luria: search pattern 'GAA1' contains the unsupported pattern letter '1' at \
         zero-based position 3\n"
    );
}

/// 15: `--aa` decides the mode on its own, whatever the pattern looks like: a
/// pattern of nothing but IUPAC nucleotide codes is searched as protein because
/// the flag says so (§4.5, plan §5.4). The row is the table's pbr322/`frames`
/// `MKSNNA` hit — strand `+`, frame 2, zero-based 85, length 18, residues 29 and
/// 34, `wraps_origin no` — printed as `86`, `103`, and the frame's own number;
/// and since six frames never read across the origin, the run says that last
/// (D052's M8-2).
#[test]
fn the_aa_flag_makes_a_dna_looking_pattern_protein() {
    let record: PathBuf = pbr("aa-flag");
    let output: Output = luria(&["find", record.to_str().unwrap(), "MKSNNA", "--aa"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        text(&output.stdout),
        format!(
            "# file: {}\n\
             # record: GenBank, 4361 bp, circular\n\
             # searched: MKSNNA as protein in six frames\n\
             start\tend\tstrand\tframe\tfirst_residue\tlast_residue\n\
             86\t103\t+\t2\t29\t34\n\
             1 hit\n\
             frames do not cross the origin\n",
            record.display()
        )
    );
    assert!(output.stderr.is_empty());
}

/// 16: a GenBank record states its own topology (D036, D049's M8-20), so the
/// record line reports `linear` plain — the assumption note belongs to FASTA
/// alone (§4.4) — and the run still says what it did not look at. The length and
/// the word `linear` are the fixture's LOCUS line as written, and
/// `tests/fixtures/genbank/synthetic-linear-record.tsv` pins them again; and
/// `GAATTC` occurs nowhere in those 393 bases, in either orientation, which is
/// what makes this a run that finds nothing (D052's M8-9).
#[test]
fn a_linear_genbank_record_states_its_topology() {
    let record: PathBuf = workspace("linear-genbank").join("synthetic-linear.gb");
    fs::copy("../../tests/fixtures/genbank/synthetic-linear.gb", &record)
        .expect("the linear fixture is copied");
    let output: Output = luria(&["find", record.to_str().unwrap(), "GAATTC"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        text(&output.stdout),
        format!(
            "# file: {}\n\
             # record: GenBank, 393 bp, linear\n\
             # searched: GAATTC as DNA on both strands\n\
             start\tend\tstrand\n\
             0 hits\n\
             linear: hits do not cross the end\n",
            record.display()
        )
    );
    assert!(output.stderr.is_empty());
}

/// 17: bytes that are no UTF-8 text are refused before any format is guessed,
/// with the file named and the input category's exit code (§4.3): the record
/// reader is given text, and `0xff` is no UTF-8 byte.
#[test]
fn a_file_that_is_not_utf8_is_refused() {
    let record: PathBuf = workspace("not-utf8").join("bad.fa");
    fs::write(&record, [b'>', b'x', b'\n', 0xff, b'\n']).expect("the non-UTF-8 file is written");
    let output: Output = luria(&["find", record.to_str().unwrap(), "GAATTC"]);

    assert_eq!(output.status.code(), Some(4));
    assert!(output.stdout.is_empty());
    assert_eq!(
        text(&output.stderr),
        format!("luria: {} is not UTF-8 text\n", record.display())
    );
}

/// 18: a record is named by its file name made printable (§4.3), and the format
/// refusal names that record: a space is not printable and becomes `_`, and a
/// character outside ASCII becomes one `_` for the one character it is — not one
/// for each of its two UTF-8 bytes, because the rule reads characters.
#[test]
fn the_record_name_is_the_file_name_made_printable() {
    for (name, clean) in [("my plasmid.txt", "my_plasmid.txt"), ("pé.gb", "p_.gb")] {
        let record: PathBuf = fasta("record-name", name, "hello\n");
        assert_eq!(
            record
                .file_name()
                .expect("the file has a name")
                .to_str()
                .unwrap(),
            name,
            "the file is written under the name the case is about"
        );
        let output: Output = luria(&["find", record.to_str().unwrap(), "GAATTC"]);

        assert_eq!(output.status.code(), Some(4), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
        assert_eq!(
            text(&output.stderr),
            format!(
                "luria: cannot tell what format record '{}' is written in: its first \
                 non-blank line starts with neither 'LOCUS' (GenBank) nor '>' (FASTA)\n",
                clean
            ),
            "{name}"
        );
    }
}
