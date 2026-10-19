# Milestone 6e1: what a quote shows

Status: **Ready for the worker, 2026-10-19.**
Planner: Claude. Parent: `docs/tasks/milestone-6-nl.md` §5.3a (with its
tightenings), §5.5 (numbering), M6-11, M6-15, §6 (6e, split into 6e1 and
6e2).

## 1. What and why

The model reports a mutation as a canonical token plus a verbatim quote of
the user's words (M6-15). Luria believes the token only as far as the quote
supports it. This contract is that judgement, as pure functions over text:

- is the residue number written in the quote?
- is each residue named in it, and the original before the target?
- do the user's words about numbering mean "from the start of the CDS"?

6e2 calls these to decide between proposing a request and asking a
question. Nothing here sees the model, a record or a request.

## 2. Requirements this contract applies

- **Plan §5.3a**, as tightened 2026-10-19 (read the "Tightenings" block):
  number as a maximal digit run; residues by one-letter code, three-letter
  code or English name; one-letter codes only next to a digit; lower-case
  `his`, `met`, `pro` are not codes; the original named before the target.
- **Plan §5.5:** whether numbering words mean "from the start of the CDS"
  is decided by Rust from a fixed list, not by the model; anything off the
  list is refused.
- **D005:** every rule fails closed: a doubtful case is *not* evidence, so
  luria asks.
- All positions are **char indices**, as in `notation.rs`: never index a
  `&str` by byte, and never panic, whatever the input.

## 3. Files

### 3.1 `crates/cli/src/interpret.rs`

Add `pub mod evidence;` between `pub mod catalogue;` and `pub mod notation;`.
Change nothing else.

### 3.2 `crates/cli/src/interpret/evidence.rs` (new)

Module doc: what the module judges, that it fails closed, and the rules of
§3.3 in prose.

```rust
use crate::interpret::notation::Substitution;
use crate::interpret::notation::RESIDUE_NAMES;
use luria_core::genetic_code::Residue;

/// What a quote shows of one substitution (plan §5.3a).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Evidence {
    /// The residue number is written in the quote.
    pub number: bool,
    /// The original residue is named in the quote.
    pub original: bool,
    /// The target residue is named in the quote.
    pub target: bool,
    /// Some naming of the original ends at or before the start of some
    /// naming of the target. False whenever either is not named.
    pub in_order: bool,
}

/// The phrases, in normalized form (§3.3.4), that state numbering from the
/// start of the CDS.
pub const ORIGIN_PHRASES: [&str; 9] = [
    "from the start of the cds",
    "from the start of the annotated cds",
    "from the start of the coding sequence",
    "from the beginning of the cds",
    "from the beginning of the annotated cds",
    "from the beginning of the coding sequence",
    "from the cds start",
    "from the first codon of the cds",
    "from the start codon",
];

/// Words that, anywhere in the numbering words, make them doubtful: a
/// negation, a contrast, or another origin. `t` is the end of a
/// contraction such as "isn't" once normalized.
pub const NUMBERING_VETO: [&str; 18] = [
    "not", "never", "no", "t", "except", "instead", "rather", "but",
    "mature", "precursor", "signal", "tag", "plasmid", "construct", "vector",
    "uniprot", "pdb", "structure",
];

/// Whether `number` is written in `quote` as a maximal run of ASCII digits.
pub fn number_visible(quote: &str, number: i64) -> bool;

/// Every place `residue` is named in `quote`, as half-open char spans,
/// sorted by start then end, without repeats.
pub fn residue_mentions(quote: &str, residue: &Residue) -> Vec<(usize, usize)>;

/// What `quote` shows of `substitution`.
pub fn evidence(quote: &str, substitution: &Substitution) -> Evidence;

/// Whether the user's numbering words state numbering from the start of
/// the CDS (M6-11).
pub fn numbering_from_cds_start(words: &str) -> bool;
```

### 3.3 The rules

Throughout, *letter* means an ASCII letter (`char::is_ascii_alphabetic`)
and *digit* an ASCII digit (`char::is_ascii_digit`). A non-ASCII character
is neither. Work on `quote.chars().collect::<Vec<char>>()`.

1. **`number_visible`.** True when some maximal run of digits in the quote
   — a run with no digit just before it and none just after it — is exactly
   `number.to_string()`. So `42` is not visible in `420` or `142`. A
   `number` below 1 is never visible.

2. **`residue_mentions`.** Find the `RESIDUE_NAMES` entry whose `letter`
   equals `residue.symbol()`. Its mentions are:
   - **one-letter**: each index `i` where the char is exactly `entry.letter`
     (upper case), the char before `i` (if any) is not a letter, the char
     after `i` (if any) is not a letter, and the char just before `i` or
     just after `i` is a digit. Span `(i, i + 1)`;
   - **three-letter**: each index `i` where the next three chars equal
     `entry.three` ignoring ASCII case, with no letter just before `i` or
     just after `i + 3`. Span `(i, i + 3)`. **Except** that when the three
     chars written are exactly `his`, `met` or `pro` (all lower case), it
     is not a mention;
   - **names**: for each of `entry.names`, each index `i` where the next
     `name.chars().count()` chars equal the name ignoring ASCII case, with
     no letter just before `i` or just after the end. Span `(i, end)`.

   Sort by start then end; remove exact repeats.

3. **`evidence`.** `number` is `number_visible(quote, s.number)`;
   `original` is whether `residue_mentions(quote, &s.from)` is non-empty;
   `target` likewise for `s.to`; `in_order` is whether some original
   mention `o` and some target mention `t` have `o.1 <= t.0`. When `from`
   and `to` are the same residue the same mention cannot serve as both,
   which `o.1 <= t.0` already ensures.

4. **`numbering_from_cds_start`.** Normalize: ASCII-lower-case every char;
   replace every char that is not a letter or digit by a space; split on
   spaces, dropping empty pieces. That gives a list of words. True when
   both hold:
   - some phrase of `ORIGIN_PHRASES`, split on spaces, occurs in the word
     list as consecutive words;
   - no word of the list is in `NUMBERING_VETO`.

   Empty words, or any words off the list, give false.

## 4. Tests (planner-written; add verbatim)

At the end of `crates/cli/src/interpret/evidence.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpret::notation::parse_substitution;

    fn token(text: &str) -> Substitution {
        parse_substitution(text).unwrap_or_else(|error| panic!("{}: {:?}", text, error))
    }

    fn shows(number: bool, original: bool, target: bool, in_order: bool) -> Evidence {
        Evidence {
            number,
            original,
            target,
            in_order,
        }
    }

    #[test]
    fn a_number_is_visible_only_as_a_whole_digit_run() {
        let cases: [(&str, i64, bool); 12] = [
            ("alanine 42 to glycine", 42, true),
            ("A42G", 42, true),
            ("Ala42Gly", 42, true),
            ("V1A", 1, true),
            ("A9223372036854775807G", i64::MAX, true),
            ("alanine 420 to glycine", 42, false),
            ("alanine 142 to glycine", 42, false),
            ("alanine 4 2 to glycine", 42, false),
            ("alanine to glycine", 42, false),
            ("A42G", 4, false),
            ("A\u{ff14}\u{ff12}G", 42, false),
            ("residue 0", 0, false),
        ];
        for case in cases {
            assert_eq!(number_visible(case.0, case.1), case.2, "{:?} {}", case.0, case.1);
        }
    }

    #[test]
    fn each_way_of_naming_a_residue_is_found_where_it_is_written() {
        let cases: [(&str, Residue, Vec<(usize, usize)>); 14] = [
            ("A42G", Residue::A, vec![(0, 1)]),
            ("A42G", Residue::G, vec![(3, 4)]),
            ("p.A42G", Residue::A, vec![(2, 3)]),
            ("Ala42Gly", Residue::A, vec![(0, 3)]),
            ("Ala42Gly", Residue::G, vec![(5, 8)]),
            ("ALA-42-gly", Residue::G, vec![(7, 10)]),
            ("change alanine 42 to glycine", Residue::A, vec![(7, 14)]),
            ("change alanine 42 to glycine", Residue::G, vec![(21, 28)]),
            ("Trp87\u{2192}Ala", Residue::W, vec![(0, 3)]),
            ("Trp87\u{2192}Ala", Residue::A, vec![(6, 9)]),
            ("W87 to alanine", Residue::W, vec![(0, 1)]),
            ("aspartic acid 102 or Aspartate", Residue::D, vec![(0, 13), (21, 30)]),
            ("\u{e9}alanine 42", Residue::A, vec![(1, 8)]),
            ("alanine 42 to alanine", Residue::A, vec![(0, 7), (14, 21)]),
        ];
        for case in cases {
            assert_eq!(residue_mentions(case.0, &case.1), case.2, "{:?} {:?}", case.0, case.1);
        }
    }

    // Ordinary English is not evidence (plan §5.3a, tightenings).
    #[test]
    fn words_that_only_look_like_codes_are_not_mentions() {
        let none: [(&str, Residue); 10] = [
            ("A glycine at 42", Residue::A),
            ("I need it", Residue::I),
            ("In synA42", Residue::A),
            ("his residue 42", Residue::H),
            ("we met at 42", Residue::M),
            ("pro 42", Residue::P),
            ("glycyl 42", Residue::G),
            ("alanine", Residue::L),
            ("Alanine", Residue::N),
            ("AG", Residue::A),
        ];
        for case in none {
            assert_eq!(residue_mentions(case.0, &case.1), vec![], "{:?} {:?}", case.0, case.1);
        }
        assert_eq!(residue_mentions("His57", &Residue::H), vec![(0, 3)]);
        assert_eq!(residue_mentions("HIS 57", &Residue::H), vec![(0, 3)]);
        assert_eq!(residue_mentions("Met1", &Residue::M), vec![(0, 3)]);
        assert_eq!(residue_mentions("Pro12", &Residue::P), vec![(0, 3)]);
        assert_eq!(residue_mentions("histidine 57", &Residue::H), vec![(0, 9)]);
        // "alanine" holds "ala" but not as a code: the letter after it.
        assert_eq!(residue_mentions("alanine", &Residue::A), vec![(0, 7)]);
    }

    #[test]
    fn the_corpus_quotes_show_what_the_corpus_says() {
        let cases: [(&str, &str, Evidence); 11] = [
            ("alanine 42 to glycine", "A42G", shows(true, true, true, true)),
            ("Ala42Gly", "A42G", shows(true, true, true, true)),
            ("A42G", "A42G", shows(true, true, true, true)),
            ("W87 to alanine", "W87A", shows(true, true, true, true)),
            ("valine 1 to alanine", "V1A", shows(true, true, true, true)),
            ("alanine to glycine", "A42G", shows(false, true, true, true)),
            ("alanine 420 to glycine", "A42G", shows(false, true, true, true)),
            ("residue 42 to glycine", "A42G", shows(true, false, true, false)),
            ("alanine 42 to something better", "A42G", shows(true, true, false, false)),
            ("delete K12", "K12A", shows(true, true, false, false)),
            ("the alanine at position 42 of synA to a glycine", "A42G", shows(true, true, true, true)),
        ];
        for case in cases {
            assert_eq!(evidence(case.0, &token(case.1)), case.2, "{:?} {}", case.0, case.1);
        }
    }

    // A swapped token must not pass: synA's residue 42 really is alanine,
    // so core's expected-residue check would not catch it (plan §5.3a).
    #[test]
    fn the_original_must_be_named_before_the_target() {
        let cases: [(&str, &str, Evidence); 4] = [
            ("glycine 42 to alanine", "A42G", shows(true, true, true, false)),
            ("make 42 a glycine instead of the alanine", "A42G", shows(true, true, true, false)),
            ("alanine 42 to alanine", "A42A", shows(true, true, true, true)),
            ("alanine 42", "A42A", shows(true, true, true, false)),
        ];
        for case in cases {
            assert_eq!(evidence(case.0, &token(case.1)), case.2, "{:?} {}", case.0, case.1);
        }
    }

    #[test]
    fn numbering_words_are_accepted_only_from_the_list() {
        let accepted: [&str; 11] = [
            "from the start of the CDS",
            "from the CDS start",
            "from the beginning of the annotated CDS",
            "Number residues from the start of the CDS.",
            "Residues are numbered from the CDS start.",
            "counting from the start of the CDS",
            "numbering from the CDS start",
            "(numbered from the CDS start)",
            "Number from the beginning of the CDS.",
            "numbered from the start codon",
            "FROM THE START OF THE CODING SEQUENCE",
        ];
        for words in accepted {
            assert!(numbering_from_cds_start(words), "{:?} was refused", words);
        }
        let refused: [&str; 13] = [
            "",
            "from the start of the plasmid",
            "from the start",
            "from residue 1",
            "1-based",
            "UniProt numbering",
            "from the CDS start, not the mature protein",
            "from the start of the mature CDS",
            "from the CDS start except the signal peptide",
            "this isn't from the CDS start",
            "from the CDS start in the PDB structure",
            "from the CDS start but after the tag",
            "fromthe CDS start",
        ];
        for words in refused {
            assert!(!numbering_from_cds_start(words), "{:?} was accepted", words);
        }
    }

    #[test]
    fn no_input_panics() {
        let inputs: [&str; 6] = ["", "\u{2192}", "\u{e9}\u{e9}\u{e9}", "4", "A", "\u{1f9ec}42\u{1f9ec}"];
        for input in inputs {
            let _ = number_visible(input, 42);
            let _ = residue_mentions(input, &Residue::A);
            let _ = evidence(input, &token("A42G"));
            let _ = numbering_from_cds_start(input);
        }
    }
}
```

`cargo fmt` may reflow these; do not change content. If a test fails,
stop and report what the function returned. Do not change the test.

### 4.1 Proving the rules are load-bearing (AGENTS.md)

Disable each once, confirm the named test fails, restore, report both:

| Rule, disabled | Test that must fail |
| --- | --- |
| a maximal digit run (accept any substring) | `a_number_is_visible_only_as_a_whole_digit_run` |
| one-letter codes need an adjacent digit | `words_that_only_look_like_codes_are_not_mentions` |
| lower-case `his`, `met`, `pro` are not codes | `words_that_only_look_like_codes_are_not_mentions` |
| `in_order` (set it to `original && target`) | `the_original_must_be_named_before_the_target` |
| `NUMBERING_VETO` (ignore it) | `numbering_words_are_accepted_only_from_the_list` |

## 5. Do not

- Read the model's extraction, a catalogue, a profile or a record, or build
  anything: that is 6e2.
- Change `notation.rs` or any other existing file except the one line in
  `interpret.rs`.
- Add a dependency or a regex crate. Index no `&str` by byte. Panic on no
  input.
- Change `luria-core`, `luria-nl`, `tests/fixtures/` or `docs/`.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the new and changed files, the exact commands and their results,
the §4.1 results, and anything that did not match this contract. Quote
command output or the file for every claim; say plainly what you did not
check.
