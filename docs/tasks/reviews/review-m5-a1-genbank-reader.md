# Review M5 A-1, A-2, A-4, A-5, A-6: the readers drop or misread input

Status: **Implemented 2026-10-18** by the local worker; accepted on planner review. Planner: Claude. Parent:
[docs/tasks/reviews/review-post-m5.md](/tasks/reviews/review-post-m5.md) A-1, A-2, A-4, A-5 and A-6.
Requirement: D008, which says normalization must never silently "delete
unexpected symbols", and D009/D041 (G4), under which topology comes from the
LOCUS line and is never inferred.
This contract amends [docs/tasks/contracts/milestone-5c2-genbank-record.md](/tasks/contracts/milestone-5c2-genbank-record.md) §4.3, §4.5
and §4.8 as described below. 5c2 stays a historical record: do not edit it.
Files: `crates/core/src/sequence.rs`, `crates/core/src/genbank.rs`.

## 1. Defects (each reproduced by the reviewer)

- **A-1.** `read_origin_line` counts an ORIGIN line's characters before
  `normalize` sees them. `normalize` skips any line whose first character
  after trimming is `;`, so an ORIGIN line `11 ;tcagcagca` disappears. With
  LOCUS `20 bp`, a 30-character ORIGIN reads as 20 bases without error.
- **A-6 (and the second half of A-1).** `normalize` trims each line with
  `str::trim`, which removes Unicode whitespace. A non-breaking space
  (U+00A0) or an ideographic space (U+3000) at either end of a line is
  silently dropped, in FASTA and GenBank alike. Inside a line only `' '`,
  `'\t'` and `'\r'` are skipped, so the two rules disagree.
- **A-2.** A qualifier name is refused only when it holds a space or a `"`.
  `/pseudo\t` is read as a qualifier named `"pseudo\t"`, so a `/pseudo` CDS
  passes the pseudo rule.
- **A-4.** LOCUS topology is matched against every token, including the
  name. `LOCUS linear 30 bp DNA SYN …`, which states no topology, is read as
  linear. `LOCUS circular 30 bp DNA linear …` is refused as
  `TopologyConflict`, although its topology is linear.
- **A-5.** A quoted value closes when its quote count is even, so
  `/note="a" "b"` is accepted and stored as `a" "b`. An inner quote must be
  doubled (`""`).

## 2. Required change

### 2.1 `sequence.rs`, `normalize`

Replace `line.trim()` with a trim that removes only `' '`, `'\t'` and
`'\r'` from both ends of the line, the same three characters the inner loop
skips. Every other character, Unicode whitespace included, reaches the
alphabet check and is refused there with `InvalidSequenceCharacters`, as any
other unexpected symbol is. The `;` comment-line rule is unchanged for
`normalize`. Update the doc comment's "whitespace is ignored" bullet to name
the three characters.

### 2.2 `genbank.rs`

1. **`read_origin_line` (A-1).** After the number check, if any base token
   contains `;`, return `refusal(sequence, GenBankRule::OriginLine,
   Some(number))`. This amends 5c2 §4.8: ORIGIN text never reaches
   `normalize`'s comment-line rule.
2. **`start_qualifier` (A-2).** Refuse a name that contains any character
   for which `char::is_whitespace` is true, in addition to the existing
   empty and `"` rules. Same error: `QualifierSyntax` at the qualifier's
   line. Update the doc comment. This amends 5c2 §4.5 case 2.
3. **`read_locus` (A-4).** Count `linear` and `circular` only among the
   tokens **after** the `bp` token found in step 1. The length and name
   rules are unchanged. The refusals are unchanged: zero matches is
   `TopologyMissing`, more than one is `TopologyConflict`. The topology is
   taken from those same tokens. Update the doc comment. This amends 5c2
   §4.3 step 2.
4. **`close_quoted` (A-5).** When a quoted value is about to close (its quote
   count is even and it ends in `"`), check the text between the opening
   and closing quote: every `"` in it must be part of a `""` pair, read left
   to right. If a lone `"` is found, return `QualifierSyntax` at
   `state.qualifier_line`. Otherwise decode as now.

## 3. Tests (write first; confirm each fails before the change)

Use the existing helpers in `genbank.rs`'s test module (`sample`, `s`, `q`,
`key`, `locus`, `read`) and the style of the existing refusal tests there.
Line numbers are 1-based indexes into `sample()`.

In `sequence.rs`'s test module:
1. `normalize` of `"\u{a0}ACGT"` is `InvalidSequenceCharacters` with
   position 0 and characters `"\u{a0}"`.
2. `normalize` of `"ACGT\u{3000}"` is `InvalidSequenceCharacters` with
   position 4 and characters `"\u{3000}"`.
3. `normalize` of `"  ACGT\t\r"` still gives `ACGT`.

In `genbank.rs`'s test module:
4. **A-1:** `sample()` with line 1 replaced by `locus("20 bp    DNA
   circular SYN 17-OCT-2026")` and line 24 (index 23) replaced by
   `s("       11 ;tcagcagca")`, with line 25 (index 24) kept as
   `       21 gcagcagcag`, is refused `OriginLine` at line 24.
5. **A-1:** `sample()` with line 24 (index 23) replaced by
   `s("       11 atcag;agca")` is refused `OriginLine` at line 24.
6. **A-2:** `sample()` with line 21 (`/pseudo`, index 20) replaced by
   `q("/pseudo\t")` is refused `QualifierSyntax` at line 21.
7. **A-4:** `sample()` with line 1 replaced by
   `s("LOCUS       linear   30 bp    DNA     SYN 17-OCT-2026")` is refused
   `TopologyMissing` at line 1.
8. **A-4:** `sample()` with line 1 replaced by
   `s("LOCUS       circular   30 bp    DNA     linear   SYN 17-OCT-2026")`
   reads, with topology linear and LOCUS name `circular`.
9. **A-5:** `sample()` with line 15 (index 14) replaced by
   `q("/note=\"a\" \"b\"")` and line 16 (index 15, the note's continuation)
   removed is refused `QualifierSyntax` at line 15.
10. **A-5:** the same, but with `q("/note=\"say \"\"hi\"\"\"")`, reads, and
    that qualifier's value is `say "hi"`.

Before writing tests 4–10, print `sample()` with indexes and confirm the line
numbers above. If any is wrong, stop and report rather than adjusting.

## 4. Do not

- Add dependencies.
- Change any file other than the two named.
- Change any existing test or fixture. If an existing test fails after the
  change, stop and report it with the failure output.
- Change `GenBankRule` or any error payload.
- Change `fasta.rs`. It reaches the fix through `normalize`.

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, each new test and whether it failed before the
change, the check results, and any deviation.
