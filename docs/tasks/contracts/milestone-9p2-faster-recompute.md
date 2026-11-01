# Contract 9p2 — core: the two slow steps of an edit, made fast

Planner: Claude. Status: implemented. Tier: full (core: sequence and genetic
code, M9-10).
Plan row: [docs/tasks/contracts/milestone-9-editor.md](/tasks/contracts/milestone-9-editor.md) §7, row 9p (second half).

## 1. What this contract builds

9p1 §8 measured one edit on a 10 000 000-base record: 2.0 s, or 5.4 s
with ORFs shown. Two steps cost almost all of it:
- `EditBuffer::to_sequence` takes 1.9 s. Its bases are already valid and
  upper case, but it goes through `normalize`, which checks and copies
  them again.
- `find_orfs` takes 3.7 s. It allocates a `String` per codon position, and
  `lookup` rebuilds the 64-letter table on every call.

This contract makes both fast **without changing any result**. Targets
in a release build: view A's step 3 under 150 ms, and view B's step 6
under 600 ms.

## 2. What exists

| Item | Where (`file:line`) | Use here |
| --- | --- | --- |
| `Sequence`, `normalize`, `canonical_text`, `sha256_hex` | `crates/core/src/sequence.rs:78`, `:316-363`, `:193`; `crates/core/src/identity.rs` | A new constructor beside `normalize` |
| `EditBuffer::to_sequence` | `crates/core/src/edit_buffer.rs:771` | Calls the new constructor |
| `lookup`, `base_place`, `residue_for_symbol`, `STANDARD_GENETIC_CODE` | `crates/core/src/genetic_code.rs:186-210`, `:25` | A non-allocating form; `lookup` delegates to it |
| `reading_of`, `codon_text`, the walk | `crates/core/src/orf.rs:120-149`, `:200-215` | Codons as `[char; 3]`, no `String` |
| `recompute_profile` | `crates/cli/src/sequence_view/state.rs` (9p1) | The measurement, unchanged |
| `orf_fixtures.rs`, `genetic_code` tests, `edit_buffer` tests | `crates/core/tests/`, `crates/core/src/` | Must pass unchanged |

## 3. Files the worker may touch

1. `crates/core/src/sequence.rs`: the new constructor and its tests.
2. `crates/core/src/edit_buffer.rs`: `to_sequence` only.
3. `crates/core/src/genetic_code.rs`: `lookup_letters`, `lookup`'s body,
   and tests.
4. `crates/core/src/orf.rs`: `codon_text` and its caller.

## 4. Behaviour

1. **`Sequence::from_canonical`** (`pub(crate)`):
   ```rust
   pub(crate) fn from_canonical(name: Identifier, letters: Vec<char>,
                                topology: SequenceTopology) -> Result<Sequence, DomainError>
   ```
   - It is for letters that are already canonical: every one an
     upper-case IUPAC letter (`is_dna_letter`). A `debug_assert!` checks
     this.
   - Empty letters give the same `EmptySequenceContent` error
     `normalize` gives.
   - It builds the canonical text once, with
     `String::with_capacity(letters.len())`, hashes it with `sha256_hex`,
     and moves `letters` into the `Sequence` with no clone.
   - The result must equal what `normalize` gives for the same letters.
2. **`EditBuffer::to_sequence`** calls `from_canonical` with a clone of
   its letters. Its doc comment says why that is safe: every edit stores
   only upper-case IUPAC letters (M9-4).
3. **`lookup_letters(codon: [char; 3]) -> CodonOutcome`** (`pub`). It
   gives the same answer as `lookup` for every three letters, and
   allocates nothing. Index the table through
   `STANDARD_GENETIC_CODE.as_bytes()`, or a `const` array. `lookup`
   keeps its signature: a string of length 3 goes to `lookup_letters`,
   and any other length stays `Invalid`, as today.
4. **ORFs.** `codon_text` becomes a function returning `[char; 3]`,
   wrapping as today, and the walk calls `lookup_letters`. The walk is
   otherwise unchanged: the same phases, limits and order.

## 5. Tests

| # | Test | Asserts |
| --- | --- | --- |
| 1 | unit, `sequence.rs`: `from_canonical_equals_normalize` | For four letter sets: pBR322's bases (read from the fixture as `normalize` reads it), the 15 IUPAC letters in order, one base, and a 1000-base run of `ACGTRYKMSWBDHVN` repeated. `from_canonical` gives a `Sequence` equal (`==`) to `normalize` of the same letters as text, with the same name and topology |
| 2 | unit: `from_canonical_refuses_empty` | Empty letters give the same error as `normalize` of an empty text: the same `code()` and `message()` |
| 3 | unit, `genetic_code.rs`: `lookup_letters_matches_the_old_lookup` | Copy today's `lookup` body into the test module as `reference_lookup(String)`. For every triple over the 15 IUPAC letters plus `X` and `a` (17³ triples), `lookup_letters` equals `reference_lookup` of the same three letters |
| 4 | existing: `orf_fixtures.rs`, `sequence_view_orf_lanes.rs`, all `genetic_code`, `edit_buffer` and `view_rebuild` tests | Pass unchanged |
| 5 | measurement | `cargo test -p luria-cli --release --lib recompute_profile -- --ignored --nocapture`: report the table. A3 under 150 ms and B6 under 600 ms. If not, report the profile and stop; do not change the algorithm |

## 6. Checks

`scripts/check`; then test 5's release run.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | `from_canonical` hashes the text with one letter changed (for example the last made `N`) | test 1 |
| 2 | `lookup_letters` computes the index with the first and last bases swapped | test 3 |

Disable each rule once, run the tests, restore it, check with `cmp`
against a backup in `target/tmp/`, and report what failed.

## 8. Do not

- Change any result: no fixture, no existing test, no public signature
  except the two additions (`from_canonical`, `lookup_letters`).
- Change `normalize`, the ORF algorithm, or anything in `crates/cli/`.
- Add dependencies.

## 9. Result (planner)

Release, median of three (`recompute_profile`): view A's step 3 went from
1929 ms to 69 ms, and view B's step 6 from 3669 ms to 210 ms. One edit in
all: A 110 ms (was 2044), B 340 ms (was 5438), C 1.0 ms (was 6.4).
[docs/specs/cli.md](/specs/cli.md) §13.7 states the new times.
