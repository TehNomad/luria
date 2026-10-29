# Contract 9c1 — core: the GenBank reader keeps what a save writes back

Planner: Claude. Status: draft. Tier: full.
Plan row: `docs/tasks/milestone-9-editor.md` §7, row 9c1.

## 1. What this contract builds

The GenBank reader starts keeping what the writer (9c3) needs to write a
record back:
- the LOCUS line as read, and its molecule type, division and date;
- every header line between LOCUS and FEATURES, as text;
- each feature's own lines, as read.

It also gains public constructors, so that 9e can build a `GenBankRecord`
from an edit buffer. Nothing the reader accepts or refuses changes, and
no existing accessor changes its answer.

Rules that apply, already settled:
- `crates/core/src/genbank.rs:1-70` — the reader's rules 1–9. They stay
  exactly as they are; this contract only keeps more of what they read.
- Plan §5 M9-7, M9-15 — what a save writes back, and why these are kept.

## 2. What exists and must be reused

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `GenBankRecord`, `GenBankFeature`, `GenBankQualifier` | `crates/core/src/genbank.rs:82-165` | Gain the fields and accessors below |
| `read_locus`, `Locus` | `genbank.rs:174`, `:710` | Already splits the LOCUS tokens; the new fields come from the same tokens |
| `read_genbank` | `genbank.rs:775` | The one place the new fields are filled |
| `tests/fixtures/genbank/raw/J01749.1.gb` | — | pBR322 as NCBI wrote it: LOCUS line 1, FEATURES line 231, ORIGIN line 399 |

## 3. Files the worker may touch

1. `crates/core/src/genbank.rs` — change: the fields, accessors and
   constructors of §4, and their unit tests.
2. `crates/core/tests/genbank_header.rs` — new: §5 tests 1–2.

Everything else is out of scope.

## 4. The public surface, exactly

```rust
impl GenBankRecord {
    /// The LOCUS line as read, its trailing `\r` removed.
    pub fn locus_line(&self) -> String;
    /// The token right after the `bp` token, unless it is `linear` or
    /// `circular`: `DNA`, `ds-DNA`, `mRNA` … `None` when there is none.
    pub fn molecule_type(&self) -> Option<String>;
    /// The token right after the topology token, unless it is a date (as
    /// `date` reads one): `SYN`, `BCT` … `None` when there is none.
    pub fn division(&self) -> Option<String>;
    /// The first token after the `bp` token of the form `DD-MMM-YYYY`
    /// (two digits, three ASCII upper-case letters, four digits).
    pub fn date(&self) -> Option<String>;
    /// Every non-blank line after the LOCUS line and before the FEATURES
    /// line (or before ORIGIN when there is no FEATURES section), verbatim
    /// with its trailing `\r` removed, in file order.
    pub fn header_lines(&self) -> Vec<String>;
    /// A record from its parts, as 9e builds one from an edit buffer:
    /// `locus_line` is empty, `molecule_type`, `division` and `date` are
    /// `None`, and `header_lines` is empty.
    pub fn from_parts(
        locus_name: String,
        accession: Option<String>,
        version: Option<String>,
        sequence: Sequence,
        features: Vec<GenBankFeature>,
    ) -> GenBankRecord;
}

impl GenBankFeature {
    /// The feature's lines as read: its key line, then every non-blank line
    /// up to the next key line, the next section line, or ORIGIN, each
    /// verbatim with its trailing `\r` removed. Empty for a feature built
    /// with `new`.
    pub fn lines(&self) -> Vec<String>;
    pub fn new(
        key: String,
        location: String,
        qualifiers: Vec<GenBankQualifier>,
        line: u64,
    ) -> GenBankFeature;
}

impl GenBankQualifier {
    pub fn new(name: String, value: Option<String>) -> GenBankQualifier;
}
```

Edge cases:
- `LOCUS       x 12 bp linear`: molecule, division and date are all `None`.
- `LOCUS       pUC19   2686 bp ds-DNA     circular SYN 12-OCT-2020`:
  `ds-DNA`, `SYN`, `12-OCT-2020`.
- A LOCUS line with the date and no division (`… circular 12-OCT-2020`):
  the division is `None` and the date is read.
- A record whose FEATURES section is empty or missing still has its
  header lines.

## 5. Tests the worker writes

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `genbank_header.rs::pbr322_keeps_its_header` | pBR322: `locus_line()` is the file's line 1. Molecule `DNA`, division `SYN`, date `30-SEP-2008`. `header_lines()` equals the file's non-blank lines 2–230, in order |
| 2 | `genbank_header.rs::pbr322_features_keep_their_lines` | Joining every feature's `lines()`, in order, gives exactly the file's non-blank lines 232–398. The first feature's first line is line 232. Every feature's first line contains its `location()`'s first 10 characters |
| 3 | unit: `locus_tokens` | The four edge cases above, plus pBR322's LOCUS line |
| 4 | unit: `parts_build_a_record` | `from_parts` then every accessor returns what was given, and `lines()` of a `new` feature is empty |

## 6. Checks

`scripts/check`, plus `cargo test -p luria-core --test genbank_header`
(2 tests), and `--test genbank_fixtures` and `--test view_record_fixtures`
passing unchanged, which shows the reader reads every fixture as before.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | `header_lines` skips the line after LOCUS | test 1 |
| 2 | A feature's `lines()` stops at its key line | test 2 |
| 3 | `division` takes the token after `bp` | test 3, pBR322 |

## 8. Do not

- Change any rule, refusal, line number or existing accessor's answer.
- Keep the ORIGIN lines or anything after `//`.
- Give header lines a meaning (no COMMENT parsing here: 9c3 does that).
