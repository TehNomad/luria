# Contract 8i2 — cli: `luria find` (§5.5)

Planner: Claude. Status: **Implemented 2026-10-26** by worker `w-8i2` in two
rounds, reviewed and accepted. Tier: light on wording, full on behaviour (plan
§7). Plan row: **8i2** (D056). Findings from review:

* Round 1 built §4 as written; all 14 tests and G1–G7 behaved as §7 says,
  except that G5 also fails 13 (its pattern `N` is IUPAC but not `ACGT`).
* The planner ran nine mutations; four survived because §5 named no test
  for them: `--aa` ignored, a linear GenBank record reported `circular`, a
  non-UTF-8 file read lossily, and spaces kept in the record name (which
  panics in `Identifier::create`). Round 2 added tests 15–18 and guards
  G8–G11 (below) without touching the source; all nine are now caught.
* The `--cds` range is refused in `find::run` before the file is read, not
  in `parse_find`; it still prints the usage text and exits 2.
* `--cds` on a FASTA (no CDSs) gives the "no CDS … is named" refusal; no
  test has one.

## 1. What this contract builds

`luria find FILE PATTERN [--aa] [--cds VALUE] [--circular]`: read a sequence
file the way the view will, search it for a DNA or protein pattern, and print
the hits as tab-separated rows. It adds the `find` dispatch arm, the byte
ceiling, and the file reader 8h's view will share.

Rules that apply, already settled:
- Plan §5.5 (`docs/tasks/contracts/milestone-8-view.md:429-477`) and §5.4's mode rule
  (`:409-416`).
- D049 (M8-15 `--cds`, M8-20 `--circular`: FASTA only; a GenBank record's
  topology is authoritative), D050 (residue numbers, no cap in core), D052
  (exit 0 on no hits; the output names what it searched), D054 (hit order),
  D056 (`MAX_RECORD_BYTES` = 64 MiB; `--cds` answers and their exit codes).

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `parse`, `USAGE`, `UsageError`, `report_usage` | `crates/cli/src/args.rs:62-143`, `main.rs:490` | the `find` arm; usage errors print `luria: {message}\n\n{USAGE}`, exit 2 |
| `ExitStatus::for_failure(category)` | `crates/cli/src/exit.rs:44` | every `DomainError`'s exit code |
| `open_record(name, text)` | `crates/core/src/view_record.rs:346` | the only way the record is read |
| `match_cds`, `cds_span`, `NameSource` | `crates/core/src/name_match.rs` (8i1) | `--cds` |
| `find_dna`, `DnaHit`, `HitStrand` | `crates/core/src/search.rs:84-188` | DNA mode |
| `find_protein`, `ProteinTarget`, `ProteinHit` | `crates/core/src/protein_search.rs` | protein mode |
| `sequence::normalize(name, text, Circular)` | `crates/core/src/sequence.rs:323` | `--circular`: a circular copy of a FASTA's sequence |
| test runner idiom | `crates/cli/tests/cli.rs:11`, `end_to_end.rs:40` | run the binary; write temporary inputs the way `end_to_end.rs` does |

## 3. Files the worker may touch

1. `crates/cli/src/args.rs`: `Command::Find { file, pattern, aa, cds, circular }`,
   its parser, and one `USAGE` line, `luria find FILE PATTERN [--aa] [--cds
   NAME] [--circular]`.
2. `crates/cli/src/main.rs`: the `Command::Find` arm.
3. `crates/cli/src/record_file.rs`: new. `pub const MAX_RECORD_BYTES: u64 =
   67_108_864;` and `pub fn read_record_file(path: &Path) -> Result<ViewRecord,
   RecordFileProblem>`.
4. `crates/cli/src/find.rs`: new. Everything else in §4, with the output built
   by a pure function and printed by `run`.
5. `crates/cli/src/lib.rs`: `pub mod find;` and `pub mod record_file;`.
6. `crates/cli/tests/find_cli.rs`: new.

## 4. Behaviour, in order

1. **Arguments.** Exactly two positionals after `find`. Each of `--aa`,
   `--cds VALUE` and `--circular` may appear once, anywhere after `find`. A
   missing positional, an extra one, an unknown option, a repeated option or
   `--cds` with no value is a usage error.
2. **A range for `--cds`.** A value that is entirely `c?DIGITS..DIGITS` (with
   spaces trimmed) is a usage error: `searching a range of bases comes in
   milestone 9` (D049).
3. **The file** (`read_record_file`). Its byte length comes from metadata,
   before reading. Over `MAX_RECORD_BYTES` → `luria: FILE is N bytes; luria
   reads files of at most 67108864 bytes (64 MiB)`, exit 6. Unreadable →
   `luria: cannot read FILE: {cause}`, exit 4. Not UTF-8 → `luria: FILE is not
   UTF-8 text`, exit 4. The record's name is the file name with every character
   outside printable ASCII, and every space, replaced by `_` (`record` if that
   leaves nothing). Then `open_record`. A refusal prints `luria: {message}` and
   exits `for_failure(category)`.
4. **`--circular`.** On a GenBank record → `luria: '--circular' is for FASTA
   files; FILE states its own topology`, exit 2, with no usage text. On FASTA,
   the search uses a circular copy of the sequence.
5. **Mode.** Protein when `--aa` or `--cds` is given. Otherwise DNA, unless the
   pattern holds a character that is not an IUPAC nucleotide letter
   (`ACGTURYSWKMBDHVN`, either case): then protein (§5.4). `MKSNNA` is all
   IUPAC letters, so it is DNA.
6. **`--cds`.** `match_cds(record, VALUE)`:
   - `Nothing` → `luria: no CDS in FILE is named 'VALUE'`, exit 2.
   - `Several` → `luria: 'VALUE' names N CDSs in FILE; give one of them:`,
     then one line per candidate, `  {display name} ({a}..{b}, {strand})
     matched by {source} '{name}'`. `{source}` is `name`, `/gene`,
     `/locus_tag`, `/protein_id` or `list form`; a CDS not drawn shows
     `(not drawn)` for its span. Exit 2.
   - `One` of an unusable CDS → `luria: CDS 'NAME' cannot be searched:
     {refusal message}`, exit `for_failure` of that refusal's category.
   These three print no usage text.
7. **Search.** DNA mode uses `find_dna`; protein mode uses `find_protein` with
   the matched CDS or `SixFrames`. A refusal (an empty pattern, a bad letter)
   prints `luria: {message}` and exits `for_failure(category)`.
8. **Output**, on stdout, exit 0, also when nothing is found (D052):

```text
# file: FILE
# record: {GenBank|FASTA}, {length} bp, {topology}
# searched: PATTERN as {DNA on both strands|protein in six frames|protein in CDS {name} ({a}..{b}, {strand})}
start	end	strand                                   (DNA)
start	end	strand	frame	first_residue	last_residue   (protein)
...rows...
showing the first 200 of N hits                          (only when N > 200)
N hits                                                   (1 hit when N is 1)
linear: hits do not cross the end                        (when the sequence searched is linear)
frames do not cross the origin                           (six frames on a circular sequence)
```

   `{topology}` is `circular` or `linear` for GenBank, `linear (assumed: a FASTA
   file states no topology)`, or `circular (--circular)`. PATTERN, FILE and
   VALUE are printed as given. Rows are in core's order (D054). Positions are
   1-based inclusive, and `end = (start0 + length - 1) mod L + 1`, so a
   wrapping hit has `end < start`. DNA strand is `+`, `-` or `both`. Protein
   strand is `+` or `-`; `frame` is `1`, `2`, `3`, or `-` for a CDS; residue
   numbers are numbers or `stop` (D050). At most 200 rows print (M8-12).

## 5. Tests the worker writes (`find_cli.rs`, through the binary)

`pbr` is `tests/fixtures/genbank/raw/J01749.1.gb`. Hit values come from
`search-dna-hits.tsv` and `search-protein-hits.tsv` (zero-based there, plus one
here). Temporary FASTA files are written as `end_to_end.rs` writes its inputs.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `the_ecori_site_wraps_the_origin` | `find pbr GAATTC`: the three `#` lines (`GenBank, 4361 bp, circular`; `GAATTC as DNA on both strands`), header `start\tend\tstrand`, one row `4359\t3\tboth`, `1 hit`, nothing after it; exit 0 |
| 2 | `an_internal_site` | `find pbr GGATCC`: row `375\t380\tboth`, `1 hit` |
| 3 | `nothing_found_is_success` | FASTA `>w\nTTCAAAAAAAAAAGAA\n`, `GAATTC`: record line `FASTA, 16 bp, linear (assumed: a FASTA file states no topology)`, no rows, `0 hits`, then `linear: hits do not cross the end`; exit 0 |
| 4 | `circular_finds_the_site_across_the_end` | same file, `--circular`: `circular (--circular)`, row `14\t3\tboth`, `1 hit`, no linear line |
| 5 | `a_cds_search_names_the_cds` | `find pbr MKSNNA --cds tet`: `MKSNNA as protein in CDS tet (86..1276, +)`, header with six columns, row `86\t103\t+\t-\t1\t6` |
| 6 | `the_stop_is_a_word` | `find pbr TST* --cds tet`: row `1265\t1276\t+\t-\t394\tstop` |
| 7 | `a_protein_letter_switches_the_mode` | `find pbr MSIQHF`: `as protein in six frames`, row `4136\t4153\t-\t2\t70\t75`, last line `frames do not cross the origin`. And `find pbr MKSNNA` (no `--aa`) says `as DNA on both strands` |
| 8 | `several_cds_candidates_are_listed` | `--cds AAB5973`: exit 2, stdout empty, stderr exactly the three candidate lines for `tet (86..1276, +)`, `ROP protein (1915..2106, +)` and `bla (3293..4153, -)`, each `matched by /protein_id '…'` |
| 9 | `an_unknown_cds_and_an_unusable_one` | `--cds lactamase` → exit 2, `no CDS in … is named 'lactamase'`. `--cds rop` → exit 6, message names ROP and its `no_id` refusal's message |
| 10 | `a_range_is_refused_for_now` | `--cds 86..1276` and `--cds c345..43` → exit 2, stderr contains `searching a range of bases comes in milestone 9` and the usage text |
| 11 | `the_byte_ceiling_is_checked_before_reading` | a file of exactly 67 108 864 newlines → exit 4 with the unknown-format message (it got past the byte check); one byte more → exit 6 with the byte message. `MAX_RECORD_BYTES == 67_108_864` |
| 12 | `circular_is_refused_for_genbank` | `find pbr GAATTC --circular` → exit 2, the §4.4 message, no usage text |
| 13 | `the_cap_is_200_rows` | FASTA of 300 `N`, pattern `N`: 200 rows, `showing the first 200 of 300 hits`, `300 hits` |
| 14 | `usage_and_input_errors` | no pattern, an extra word, `--bogus`, `--aa --aa`, `--cds` last → exit 2 with usage text. A missing file → exit 4, `cannot read`. Pattern `""` → exit 4 (`EmptySearchPattern`); `GAA1` → exit 4 naming `1` |
| 15 | `the_aa_flag_makes_a_dna_looking_pattern_protein` | round 2: `find pbr MKSNNA --aa` → six frames, row `86\t103\t+\t2\t29\t34` |
| 16 | `a_linear_genbank_record_states_its_topology` | round 2: `synthetic-linear.gb` → `GenBank, 393 bp, linear`, `0 hits`, the linear line |
| 17 | `a_file_that_is_not_utf8_is_refused` | round 2: bytes `>x\n\xff\n` → exit 4, `is not UTF-8 text` |
| 18 | `the_record_name_is_the_file_name_made_printable` | round 2: `my plasmid.txt` → record `my_plasmid.txt`; `pé.gb` → `p_.gb` |

## 6. Checks

`scripts/check`, plus `cargo test -p luria-cli --test find_cli` and
`python3 -I tests/fixtures/verify.py` (still `all 25 checks passed`).

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | `end` printed as `start + length` (no wrap) | 1, 4 |
| G2 | No byte check (read, then let `open_record` decide) | 11 |
| G3 | Exit 1 when there are no hits | 3 |
| G4 | `--cds` takes the first of several candidates | 8 |
| G5 | Mode is protein whenever a letter is outside `ACGT` | 7 |
| G6 | No cap | 13 |
| G7 | `--circular` accepted for GenBank | 12 |
| G8 | `--aa` ignored by the mode rule (round 2) | 15 |
| G9 | a stated topology always printed `circular` (round 2) | 16 |
| G10 | non-UTF-8 bytes read lossily (round 2) | 17 |
| G11 | spaces kept in the record name (round 2) | 18 (a panic, exit 101) |

## 8. Do not

- No new dependency; no change to `crates/core`, `review`, `new`, or any
  existing test. The `USAGE` change is the one line above.
- No `luria FILE` (bare file) dispatch: that is 8h, after 8k1 (D056).
- No JSON output, no `--cds` range form, no flag or environment variable for
  the cap or the ceilings (plan §8).

## 9. Report

Every guard experiment with its failure line, and the exact stdout of tests 1,
5 and 8.
