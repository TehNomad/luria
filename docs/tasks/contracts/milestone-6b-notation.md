# Milestone 6b: mutation notation and the residue-name table

Status: **Implemented 2026-10-19** by the local worker, accepted after one
review round. The round added `the_table_holds_the_iupac_codes_and_names`:
the §4 tests read `RESIDUE_NAMES` back to itself, so nothing pinned its sourced
values (a planner gap; the worker proved the new test fails on "Glu" → "Glq").
Planner: Claude. Parent: `docs/tasks/contracts/milestone-6-nl.md` §5.3, §5.3a, M6-8,
M6-15, §6 (6b).

## 1. What and why

The model reports each mutation as a `token` such as `A42G` (plan §5.2).
This contract reads that token into a residue substitution, or says
precisely why it can't. It is the first code in `luria-cli`'s new
`interpret` module, and it is pure: text in, value or refusal out.

It also adds the table of residue names (`A`, `Ala`, `alanine`) that the
notation needs for three-letter codes and that 6e's evidence check will
need for English names (plan §5.3a).

## 2. Requirements this contract applies

- **M6-8:** milestone 6 accepts residue substitutions only, in one-letter
  (`A42G`) and three-letter (`Ala42Gly`) notation, each optionally
  prefixed `p.`. Every other form — a stop, a deletion, an insertion, a
  frameshift, a DNA-level change — is refused, never guessed.
- **Plan §5.3a, `?` clarification:** a `?` in place of the original
  residue, the number or the target means the user did not state that part.
  The parser reports which parts, in order, so 6e can ask the specific
  question. A `?` never becomes a value.
- **Review input 4:** residue letters are normalized. The parser accepts
  any case and reports core's `Residue`; the canonical token is upper-case.
- **D003, D002:** no second residue representation. Results are
  `luria_core::genetic_code::Residue`, obtained through
  `residue_for_symbol`, whose doc comment says it exists for exactly this.
- **Not this parser's business:** whether the residue exists in a CDS,
  whether `from` equals `to`, or whether the number is in range. Those are
  core's, at `luria q5 design` (plan §5.5). `A42A` and `A9999G` parse.

## 3. Files

### 3.1 `crates/cli/src/lib.rs`

Add `pub mod interpret;` after `pub mod exit;`. Change nothing else.

### 3.2 `crates/cli/src/interpret.rs` (new)

```rust
//! `luria q5 interpret` (milestone 6): from what a model says the user
//! asked for, to a typed request luria can run. Everything here is
//! deterministic and holds no model I/O; the model is spoken to only in
//! `luria-nl` (milestone 6 plan §3, §5.1).

pub mod notation;
```

### 3.3 `crates/cli/src/interpret/notation.rs` (new)

Module doc: the accepted forms, the refusals, the `?` rule, and what the
parser does not judge (§2's last bullet).

```rust
use luria_core::genetic_code::residue_for_symbol;
use luria_core::genetic_code::Residue;

/// One of the twenty residues: its one-letter code, its three-letter code
/// and the English names a user may write for it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResidueName {
    pub letter: char,
    pub three: &'static str,
    pub names: &'static [&'static str],
}

/// The twenty residues of the standard genetic code, in the order of their
/// three-letter codes. Codes and names: IUPAC-IUB Joint Commission on
/// Biochemical Nomenclature, "Nomenclature and Symbolism for Amino Acids
/// and Peptides" (1983), 3AA-1. Checked against `luria-core` by test, so
/// the two cannot drift.
pub const RESIDUE_NAMES: [ResidueName; 20] = [
    ResidueName { letter: 'A', three: "Ala", names: &["alanine"] },
    ResidueName { letter: 'R', three: "Arg", names: &["arginine"] },
    ResidueName { letter: 'N', three: "Asn", names: &["asparagine"] },
    ResidueName { letter: 'D', three: "Asp", names: &["aspartate", "aspartic acid"] },
    ResidueName { letter: 'C', three: "Cys", names: &["cysteine"] },
    ResidueName { letter: 'Q', three: "Gln", names: &["glutamine"] },
    ResidueName { letter: 'E', three: "Glu", names: &["glutamate", "glutamic acid"] },
    ResidueName { letter: 'G', three: "Gly", names: &["glycine"] },
    ResidueName { letter: 'H', three: "His", names: &["histidine"] },
    ResidueName { letter: 'I', three: "Ile", names: &["isoleucine"] },
    ResidueName { letter: 'L', three: "Leu", names: &["leucine"] },
    ResidueName { letter: 'K', three: "Lys", names: &["lysine"] },
    ResidueName { letter: 'M', three: "Met", names: &["methionine"] },
    ResidueName { letter: 'F', three: "Phe", names: &["phenylalanine"] },
    ResidueName { letter: 'P', three: "Pro", names: &["proline"] },
    ResidueName { letter: 'S', three: "Ser", names: &["serine"] },
    ResidueName { letter: 'T', three: "Thr", names: &["threonine"] },
    ResidueName { letter: 'W', three: "Trp", names: &["tryptophan"] },
    ResidueName { letter: 'Y', three: "Tyr", names: &["tyrosine"] },
    ResidueName { letter: 'V', three: "Val", names: &["valine"] },
];

/// A residue substitution read from a token.
#[derive(Clone, Debug, PartialEq)]
pub struct Substitution {
    pub from: Residue,
    /// 1-based, as written; at least 1.
    pub number: i64,
    pub to: Residue,
}

impl Substitution {
    /// The canonical one-letter token, upper-case: `A42G`. It is the
    /// member name a proposed request uses (plan §5.5).
    pub fn canonical(&self) -> String;
}

/// A part of a substitution a `?` stood for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Original,
    Number,
    Target,
}

/// Why a token is not a substitution luria can make.
#[derive(Clone, Debug, PartialEq)]
pub enum NotationError {
    /// No form milestone 6 accepts. `token` is the text exactly as given.
    Unsupported { token: String },
    /// A well-formed substitution with `?` in place of these parts, in the
    /// order original, number, target. Never empty.
    Unstated { parts: Vec<Part> },
}

/// Read one mutation token.
pub fn parse_substitution(token: &str) -> Result<Substitution, NotationError>;
```

### 3.4 The grammar `parse_substitution` applies

Work on `char`s throughout. Never index a `&str` by byte, so that no
input, non-ASCII included, can panic.

1. If the token starts with exactly `p.` (lower-case `p`, then `.`),
   remove those two characters. No other prefix is removed, and nothing is
   trimmed: whitespace anywhere is refused.
2. **Original side.** If the next character is `?`, it is the side.
   Otherwise the side is the longest run of ASCII letters; it must be 1 or
   3 letters long.
3. **Number.** If the next character is `?`, it is the number. Otherwise
   it is the longest run of ASCII digits (`'0'..='9'`): non-empty, no
   leading `0`, and a value from 1 to `i64::MAX`.
4. **Target side.** Everything left: exactly `?`, or 1 or 3 ASCII letters.
5. **Letters.** A 1-letter side is a one-letter code, matched against
   `RESIDUE_NAMES[..].letter` ignoring ASCII case. A 3-letter side is a
   three-letter code, matched against `RESIDUE_NAMES[..].three` ignoring
   ASCII case. A side that matches nothing is refused. The matched letter
   becomes a `Residue` through `residue_for_symbol`.
6. **One notation.** When both sides are letters, both must be one-letter
   or both three-letter. `Ala42G` and `A42Gly` are refused.
7. Any failure in steps 2 to 6 is `Unsupported { token }`, with `token`
   the whole original text, prefix included.
8. Only a token that passed steps 1 to 6 can be `Unstated`: if any side or
   the number is `?`, return `Unstated` with those parts in order.
   Malformed beats unstated: `A42G?` and `Ala?G` are `Unsupported`.

`residue_for_symbol` cannot fail for a letter of `RESIDUE_NAMES` (the
drift test proves it). If it ever returns `None`, return `Unsupported`; do
not panic.

## 4. Tests (planner-written; add verbatim)

At the end of `crates/cli/src/interpret/notation.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(token: &str) -> Substitution {
        parse_substitution(token)
            .unwrap_or_else(|error| panic!("{:?} was refused: {:?}", token, error))
    }

    fn refused(token: &str) -> NotationError {
        match parse_substitution(token) {
            Err(error) => error,
            Ok(found) => panic!("{:?} parsed as {:?}", token, found),
        }
    }

    #[test]
    fn every_accepted_form_reads_the_same_substitution() {
        let tokens: [&str; 8] = [
            "A42G", "a42g", "p.A42G", "p.a42g", "Ala42Gly", "ala42gly", "ALA42GLY", "p.Ala42Gly",
        ];
        for token in tokens {
            let found: Substitution = parsed(token);
            assert_eq!(found.from, Residue::A, "{}", token);
            assert_eq!(found.number, 42, "{}", token);
            assert_eq!(found.to, Residue::G, "{}", token);
            assert_eq!(found.canonical(), "A42G", "{}", token);
        }
    }

    #[test]
    fn the_corpus_tokens_read_as_written() {
        let cases: [(&str, Residue, i64, Residue); 3] = [
            ("W87A", Residue::W, 87, Residue::A),
            ("V1A", Residue::V, 1, Residue::A),
            ("A420G", Residue::A, 420, Residue::G),
        ];
        for case in cases {
            let found: Substitution = parsed(case.0);
            assert_eq!(found.from, case.1, "{}", case.0);
            assert_eq!(found.number, case.2, "{}", case.0);
            assert_eq!(found.to, case.3, "{}", case.0);
            assert_eq!(found.canonical(), case.0, "{}", case.0);
        }
    }

    // Not this parser's business (contract §2): a same-residue token and a
    // number past any CDS parse, and core judges them at design.
    #[test]
    fn biology_is_not_judged_here() {
        assert_eq!(parsed("A42A").canonical(), "A42A");
        assert_eq!(parsed("A9999G").number, 9999);
        assert_eq!(parsed("A9223372036854775807G").number, i64::MAX);
    }

    #[test]
    fn every_residue_reads_in_both_notations() {
        for entry in RESIDUE_NAMES.iter() {
            let expected: Residue = residue_for_symbol(entry.letter)
                .unwrap_or_else(|| panic!("core has no residue {}", entry.letter));
            let one: String = format!("{}7{}", entry.letter, entry.letter);
            let three: String = format!("{}7{}", entry.three, entry.three);
            for token in [one.clone(), three.clone(), three.to_ascii_lowercase()] {
                let found: Substitution = parsed(&token);
                assert_eq!(found.from, expected.clone(), "{}", token);
                assert_eq!(found.to, expected.clone(), "{}", token);
                assert_eq!(found.canonical(), one, "{}", token);
            }
        }
    }

    #[test]
    fn a_question_mark_reports_each_unstated_part_in_order() {
        let cases: [(&str, Vec<Part>); 7] = [
            ("?42G", vec![Part::Original]),
            ("A?G", vec![Part::Number]),
            ("A42?", vec![Part::Target]),
            ("??G", vec![Part::Original, Part::Number]),
            ("???", vec![Part::Original, Part::Number, Part::Target]),
            ("p.?42Gly", vec![Part::Original]),
            ("Ala?Gly", vec![Part::Number]),
        ];
        for case in cases {
            assert_eq!(
                refused(case.0),
                NotationError::Unstated { parts: case.1 },
                "{}",
                case.0
            );
        }
    }

    #[test]
    fn every_other_form_is_unsupported_and_named() {
        let tokens: [&str; 37] = [
            "",
            "A42",
            "42G",
            "AG",
            "A42*",
            "Ala42Ter",
            "*101Q",
            "K12del",
            "A42_G43insK",
            "A42fs",
            "A42=",
            "c.123A>G",
            "P.A42G",
            "pA42G",
            " A42G",
            "A42G ",
            "A 42 G",
            "A042G",
            "A0G",
            "A-4G",
            "A9223372036854775808G",
            "Ala42G",
            "A42Gly",
            "Xaa42Gly",
            "B42G",
            "Z42G",
            "X42G",
            "U42G",
            "O42G",
            "AL42G",
            "Alan42G",
            "\u{c4}42G",
            "A42\u{11e}",
            "A4\u{ff12}G",
            "A42G?",
            "Ala?G",
            "A?42G",
        ];
        for token in tokens {
            assert_eq!(
                refused(token),
                NotationError::Unsupported {
                    token: token.to_string()
                },
                "{:?}",
                token
            );
        }
    }

    // The table and core must name the same twenty residues (plan §5.3a).
    #[test]
    fn the_table_holds_exactly_cores_twenty_residues() {
        let mut letters: Vec<char> = RESIDUE_NAMES.iter().map(|entry| entry.letter).collect();
        letters.sort();
        letters.dedup();
        assert_eq!(letters.len(), 20, "a letter repeats in RESIDUE_NAMES");
        for entry in RESIDUE_NAMES.iter() {
            let residue: Residue = residue_for_symbol(entry.letter)
                .unwrap_or_else(|| panic!("core has no residue {}", entry.letter));
            assert_eq!(residue.symbol(), entry.letter.to_string());
        }
        for letter in 'A'..='Z' {
            assert_eq!(
                residue_for_symbol(letter).is_some(),
                letters.contains(&letter),
                "core and the table disagree about {}",
                letter
            );
        }
    }

    #[test]
    fn three_letter_codes_and_names_are_distinct() {
        let mut threes: Vec<String> = RESIDUE_NAMES
            .iter()
            .map(|entry| entry.three.to_ascii_lowercase())
            .collect();
        threes.sort();
        threes.dedup();
        assert_eq!(threes.len(), 20, "a three-letter code repeats");
        let mut names: Vec<&str> = RESIDUE_NAMES
            .iter()
            .flat_map(|entry| entry.names.iter().copied())
            .collect();
        let count: usize = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), count, "a residue name repeats");
        assert_eq!(count, 22, "20 residues, two with a second name");
    }
}
```

`cargo fmt` may reflow these, and it will reflow the `RESIDUE_NAMES`
literal; do not change content. If a test fails, stop and report what
`parse_substitution` returned. Do not change the test.

### 4.1 Proving the rules are load-bearing (AGENTS.md)

Disable each once, confirm the named test fails, restore, and report both
outcomes:

| Rule, disabled | Test that must fail |
| --- | --- |
| the no-leading-zero check (step 3) | `every_other_form_is_unsupported_and_named` |
| the one-notation check (step 6) | `every_other_form_is_unsupported_and_named` |
| case-insensitive three-letter matching (match exactly instead) | `every_accepted_form_reads_the_same_substitution` |
| `?` recognition (treat `?` like any other character) | `a_question_mark_reports_each_unstated_part_in_order` |
| "malformed beats unstated" (return `Unstated` as soon as a `?` is seen) | `every_other_form_is_unsupported_and_named` |

## 5. Do not

- Add a dependency, or make `luria-cli` depend on `luria-nl`. 6b needs
  neither.
- Change `luria-core`, any existing file under `crates/cli/` other than the
  one line in `lib.rs`, `tests/fixtures/` or `docs/`.
- Add grounding, the evidence check against a quote, a `name_of` lookup, or
  anything that builds a request: those are 6e.
- Panic on any input. `parse_substitution` is total.
- Use a regex crate or `serde_json` here.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the new and changed files, the exact commands and their results,
the §4.1 results, and anything that did not match this contract.
