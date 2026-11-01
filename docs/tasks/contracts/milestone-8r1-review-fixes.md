# Contract 8r1 — cli: three bugs from the milestone 8 code review

Planner: Claude. Status: **Implemented 2026-10-28** by worker `w-8r1` in
one round, reviewed and accepted; G1 to G3 each failed their test. The
planner corrected one test comment (`/dev/zero`'s metadata length is 0). Tier: full for item 2 (D048's ceiling),
presentation (D051) for items 1 and 3.
Plan row: none. The owner asked, on 2026-10-28, for the review's three
bugs to be fixed before milestone 9.

## 1. What this contract builds

Three separate fixes, each with a test that fails without it:

1. The feature list draws a row DIM when `enter` refuses it. For a
   minus-strand feature it tests the wrong end today.
2. The 64 MiB byte ceiling holds for a file whose metadata gives no
   length (a pipe, a FIFO, `/dev/zero`).
3. In the view, a protein hit in a CDS that ends on the stop codon says
   so in its label.

Rules that apply, already settled:
- D053, M8-21 (`milestone-8m-polish.md` §1): the 5′ end is the start of
  the first interval on the plus strand and the end of the last interval
  on the minus strand; a `<` (plus) or `>` (minus) there is "start not
  stated".
- D048 (M8-14) and D056: `MAX_RECORD_BYTES` is 64 MiB, and over it the
  file is refused, exit 6, before it is parsed.
- M8-17 (`milestone-8-view.md` §9) and D050: a CDS hit that ends on the
  stop has no last residue number; `luria find` prints `stop`, and the
  view says "to the stop codon".

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `refused` | `crates/cli/src/sequence_view/list.rs:231` | Item 1: the `Drawn` arm |
| `five_prime_is_unstated` | `crates/cli/src/sequence_view/state.rs:2012` | Item 1: the rule `enter` already uses. Make it `pub(super)` and call it from `refused` |
| `only_a_refused_row_is_dim` | `list.rs`, test module | Model for item 1's test; `record(path, name)` and `open_record` build records |
| `read_record_file` | `crates/cli/src/record_file.rs:135` | Item 2 |
| `RecordFileProblem::TooLarge` and its message | `record_file.rs:42`, `:60` | Item 2 |
| `the_byte_ceiling_is_checked_before_reading` | `crates/cli/tests/find_cli.rs:348` | Must pass unchanged |
| `protein_label` | `crates/cli/src/sequence_view/search.rs:348` | Item 3 |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/list.rs` — `refused`'s `Drawn` arm, and a
   new test in its test module.
2. `crates/cli/src/sequence_view/state.rs` — `five_prime_is_unstated`'s
   visibility only.
3. `crates/cli/src/record_file.rs` — `read_record_file`, `TooLarge` and its
   message, and new unit tests.
4. `crates/cli/src/sequence_view/search.rs` — `protein_label`, its doc
   comment, and a new test in its test module.

## 4. The changes, exactly

**Item 1.** In `refused`, `FeatureShape::Drawn(drawn)` gives
`five_prime_is_unstated(drawn)` instead of `drawn.open_before.is_some()`.
Fix `refused`'s doc comment if it names `open_before`.

**Item 2.** `read_record_file` keeps the metadata check first, as it is.
It then reads through `File::open(path)` and `Read::take(MAX_RECORD_BYTES
+ 1)` into a `Vec<u8>`. If more than `MAX_RECORD_BYTES` bytes arrive,
it returns `TooLarge` without parsing. `TooLarge` becomes
`TooLarge { bytes: Option<u64> }`:
- `Some(n)`: the metadata's length, with today's message unchanged
  (`find_cli.rs:348` pins it).
- `None`: the stream ran past the ceiling. The message is
  `{file} is more than {MAX_RECORD_BYTES} bytes; luria reads files of at
  most {MAX_RECORD_BYTES} bytes (64 MiB)`, with the same exit status.

An open or read error is `Unreadable`, as today.

**Item 3.** In `protein_label`, the `(Some(first), None)` case gives
`"{first} to the stop codon"` (for example `284 to the stop codon`). The
other cases are unchanged. Update the doc comment to match.

## 5. Tests the worker writes

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `list.rs::a_minus_strand_row_is_dim_by_its_5_prime_end` | A GenBank record built in the test from text (LOCUS, a FEATURES table, ORIGIN; 120 bases, linear) with four `misc_feature`s: `<1..100`, `1..>100`, `complement(<1..100)` and `complement(1..>100)`. `refused` is true for the 1st and the 4th and false for the 2nd and the 3rd. The two minus-strand rows fail today |
| 2 | `record_file.rs::a_stream_past_the_ceiling_is_refused` | `read_record_file(Path::new("/dev/zero"))` is `Err(TooLarge { bytes: None })`. Gate it on `cfg(target_os = "linux")` |
| 3 | `record_file.rs::the_stream_message_says_more_than` | `report` on `TooLarge { bytes: None }` gives the §4 item 2 message and exit status 6 |
| 4 | `search.rs::a_cds_hit_ending_on_the_stop_says_so` | `protein_label` on a CDS hit (`frame: None`) with `first: Some(284)`, `last: None` is `284 to the stop codon` |

## 6. Checks

`scripts/check`, plus `cargo test -p luria-cli --test find_cli` passing
with `the_byte_ceiling_is_checked_before_reading` unchanged.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | Restore `drawn.open_before.is_some()` in `refused` | test 1 |
| G2 | Keep the `take`, but drop the check that more than `MAX_RECORD_BYTES` bytes arrived, so the bytes go on to be parsed | test 2. **Never** test by removing the `take`: an unbounded read of `/dev/zero` exhausts the machine's memory |
| G3 | Restore `(Some(first), _) => first.to_string()` | test 4 |

## 8. Do not

- Touch any other rule in `refused`, the list's drawing, or `enter`.
- Read a file twice, or read it before the metadata check.
- Change `luria find`'s output, `MAX_RECORD_BYTES` or any exit status.
- Edit existing tests or fixtures.

## 9. Report

G1 to G3, each with its failure line.
