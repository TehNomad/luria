---
type: Task Contract
title: "Milestone 5c1: the GenBank location parser"
description: "A GenBank feature location (complement(join(2001..2400,1..150))) has to become luria intervals and a strand before any CDS can be built from it."
status: deprecated
tags: [milestone-5]
---
# Milestone 5c1: the GenBank location parser

Status: **Implemented 2026-10-17** by the local worker; accepted on first review.
Planner: Claude. Parent: [docs/tasks/contracts/milestone-5-cli.md](/tasks/contracts/milestone-5-cli.md) §3.3 (GenBank,
"Locations") and §5 (5c1). Follows 5b (FASTA reader, committed).

## 1. What and why

A GenBank feature location (`complement(join(2001..2400,1..150))`) has to
become luria intervals and a strand before any CDS can be built from it.
This contract adds one pure function that does exactly that, and refuses
every location form luria does not represent, each with its own rule. It
reads no files and knows nothing about records, features or qualifiers;
5c2 and 5c3 build on it.

The parser lives in a new file, `crates/core/src/genbank_location.rs`. (The
plan named one `genbank.rs`; the record reader in 5c2 will be that file, and
keeping the location grammar separate keeps both small.)

## 2. Requirements this contract applies

- **D006 / D041:** luria coordinates are zero-based half-open. GenBank
  positions are one-based and inclusive, so the GenBank range `a..b` is the
  luria interval `[a - 1, b)`.
- **D009:** strand is explicit and never inferred; unsupported compound or
  spliced features produce explicit errors.
- **D036:** a circular feature is one interval, or exactly two that cross the
  origin, `[a, n)` followed by `[0, b)`, in that order **for either strand**
  (`tests/fixtures/core-foundation/cases.tsv` row `cds_c_wrap_minus` is
  `21:24;0:6` on `-`). A linear feature is one interval.
- **Plan §3.3:** accepted are `a..b`, `complement(a..b)`, and, on a circular
  template only, a join crossing the origin, written either
  `join(a..n,1..b)` or with `complement(…)` around it or around each part.
  Refused, each with its own error: fuzzy ends (`<`, `>`), `^`, `order()`,
  any other `join`, single-base locations, references into other records.

## 3. Error variant and rule (in `crates/core/src/error.rs`)

```rust
/// Which rule refused a GenBank location. Stable: `code()` is part of the
/// CLI's output contract (D041), and renaming one needs approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenBankLocationRule {
    Syntax,
    PositionZero,
    RangeEndsBeforeStart,
    PastSequenceEnd,
    FuzzyEnd,
    BetweenBases,
    Order,
    RemoteReference,
    SinglePosition,
    OriginJoinOnLinear,
    UnsupportedJoin,
}
```

`impl GenBankLocationRule` has two methods, each an exhaustive `match` with
no `_` arm:
- `pub fn code(self) -> &'static str`: the name in snake_case
  (`syntax`, `position_zero`, `range_ends_before_start`,
  `past_sequence_end`, `fuzzy_end`, `between_bases`, `order`,
  `remote_reference`, `single_position`, `origin_join_on_linear`,
  `unsupported_join`);
- `pub fn category(self) -> ErrorCategory`:
  - `MalformedInput`: `Syntax`, `PositionZero`, `RangeEndsBeforeStart`;
  - `BiologicalValidation`: `PastSequenceEnd` (well-formed, but contradicts
    the sequence, as `CoordinateOutOfBounds` does);
  - `Unsupported`: all the others.

```rust
/// Payload: a GenBank feature location luria cannot use. `location` is the
/// text as given; `sequence_length` is the length it was read against.
#[derive(Clone, Debug, PartialEq)]
pub struct UnusableGenBankLocationProblem {
    pub location: String,
    pub rule: GenBankLocationRule,
    pub sequence_length: u64,
}
```

Add `DomainError::UnusableGenBankLocation(UnusableGenBankLocationProblem)` as
the last variant, after `MalformedFasta`. Then:
- `category()`: `problem.rule.category()` (the one variant whose category
  depends on its payload; say so in a comment);
- `code()`: `"unusable_genbank_location"`;
- `message()`: `GenBank location '{location}' {reason}`, where `{reason}` is:

| Rule | Reason |
| --- | --- |
| `Syntax` | `is not a location luria can read` |
| `PositionZero` | `uses position 0; GenBank positions count from 1` |
| `RangeEndsBeforeStart` | `has a range that ends before it starts` |
| `PastSequenceEnd` | `reaches past the end of the {sequence_length}-base sequence` |
| `FuzzyEnd` | `has a fuzzy end (< or >), which marks an incomplete feature` |
| `BetweenBases` | `names a site between two bases (^), not a range of bases` |
| `Order` | `uses order(), whose parts have no stated connection` |
| `RemoteReference` | `refers to another record` |
| `SinglePosition` | `is a single base, not a range` |
| `OriginJoinOnLinear` | `joins across the origin, but the sequence is linear` |
| `UnsupportedJoin` | `is a join luria does not represent: only a join that crosses the origin of a circular sequence is read` |

## 4. The parser: `crates/core/src/genbank_location.rs`

Add `pub mod genbank_location;` to `lib.rs` in alphabetical order.

```rust
/// A GenBank location as luria intervals: zero-based half-open `(start, end)`
/// pairs in D036 order, and the strand.
#[derive(Clone, Debug, PartialEq)]
pub struct GenBankLocation {
    intervals: Vec<(u64, u64)>,
    strand: Strand,
}

impl GenBankLocation {
    pub fn intervals(&self) -> Vec<(u64, u64)>;
    pub fn strand(&self) -> Strand;
}

pub fn parse_genbank_location(
    text: &str,
    sequence_length: u64,
    topology: SequenceTopology,
) -> Result<GenBankLocation, DomainError>;
```

The intervals are plain pairs, not `NucleotideInterval`s: the parser has no
`Sequence` to reference. 5c3 builds intervals and the `Cds` from them, and
`Cds::create` keeps its own checks (overlap, length divisible by three, and
so on). The parser does not restate those.

### 4.1 Steps, in this order (the first that applies decides)

Every refusal carries `location: text` (as given, before whitespace removal)
and `sequence_length`.

1. Remove every ASCII whitespace character (space, tab, `\r`, `\n`): a
   location continued over several lines reads as one.
2. Empty after step 1 → `Syntax`.
3. Contains `:` → `RemoteReference`.
4. Contains `<` or `>` → `FuzzyEnd`.
5. Contains `^` → `BetweenBases`.
6. Contains `order(` → `Order`.
7. Parse against this grammar. Anything it does not match exactly, with
   nothing left over, is `Syntax`:

   ```
   location := part
             | "complement(" part ")"
             | "join(" item ("," item)* ")"
             | "complement(join(" part ("," part)* "))"
   item     := part | "complement(" part ")"
   part     := INT | INT ".." INT
   INT      := one or more ASCII digits, fitting in u64
   ```

   Keywords are lower case and matched exactly. A number that overflows
   `u64` is `Syntax`.
8. Check each part, left to right, and within a part in this order:
   - a bare `INT` → `SinglePosition`;
   - either position is 0 → `PositionZero`;
   - `a..b` with `b < a` → `RangeEndsBeforeStart`;
   - `b > sequence_length` → `PastSequenceEnd`.

   `a..a` is a valid one-base range and is not refused here.
9. Shape:
   - `part` → `[(a - 1, b)]`, `Plus`.
   - `complement(part)` → `[(a - 1, b)]`, `Minus`.
   - A join is accepted **only** as an origin join, which means exactly two
     ranges `P1 = a..n` and `P2 = 1..b` in one of these three spellings:
     - `join(P1,P2)`, both items plain → `Plus`;
     - `complement(join(P1,P2))` → `Minus`;
     - `join(complement(P2),complement(P1))`, both items complemented, `P2`
       first → `Minus`;

     where `n == sequence_length` and the start of `P2` is 1. The result is
     `[(a - 1, n), (0, b)]` for all three.
   - If the join matches one of those spellings and positions but the
     topology is linear → `OriginJoinOnLinear`.
   - Every other join → `UnsupportedJoin`. That includes one part, three or
     more parts, mixed plain and complemented items, parts in the other
     order, and two parts that don't meet at the origin (exons).

Write a module doc comment summarizing steps 1–9 and the 1-based to
half-open conversion, citing D006, D009, D036 and D041.

## 5. Tests

### 5.1 Unit tests in `genbank_location.rs` (planner-written; add verbatim)

Expected intervals are derived by hand from §2's conversion: `a..b` becomes
`(a - 1, b)`.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCategory;
    use crate::error::GenBankLocationRule;
    use crate::error::GenBankLocationRule::*;

    const N: u64 = 100;

    fn circular(text: &str) -> Result<GenBankLocation, DomainError> {
        parse_genbank_location(text, N, SequenceTopology::Circular)
    }

    fn linear(text: &str) -> Result<GenBankLocation, DomainError> {
        parse_genbank_location(text, N, SequenceTopology::Linear)
    }

    fn accepted(found: Result<GenBankLocation, DomainError>) -> (Vec<(u64, u64)>, Strand) {
        let location = found.unwrap_or_else(|error| panic!("refused: {}", error.message()));
        (location.intervals(), location.strand())
    }

    fn rule(found: Result<GenBankLocation, DomainError>) -> GenBankLocationRule {
        match found {
            Err(DomainError::UnusableGenBankLocation(problem)) => {
                assert_eq!(problem.sequence_length, N);
                problem.rule
            }
            other => panic!("expected UnusableGenBankLocation, got {:?}", other),
        }
    }

    #[test]
    fn a_range_is_one_based_inclusive_converted_to_half_open() {
        assert_eq!(accepted(linear("1..99")), (vec![(0, 99)], Strand::Plus));
        assert_eq!(accepted(linear("1..100")), (vec![(0, 100)], Strand::Plus));
        assert_eq!(accepted(linear("5..5")), (vec![(4, 5)], Strand::Plus));
        assert_eq!(accepted(circular("10..30")), (vec![(9, 30)], Strand::Plus));
    }

    #[test]
    fn a_complement_is_the_same_interval_on_the_minus_strand() {
        assert_eq!(accepted(linear("complement(10..30)")), (vec![(9, 30)], Strand::Minus));
    }

    // join(90..100,1..12) on a 100-base circle is 90..100 then 1..12:
    // [89, 100) then [0, 12), D036's order, for both strands.
    #[test]
    fn each_origin_join_spelling_gives_the_d036_intervals() {
        let both = vec![(89, 100), (0, 12)];
        assert_eq!(accepted(circular("join(90..100,1..12)")), (both.clone(), Strand::Plus));
        assert_eq!(
            accepted(circular("complement(join(90..100,1..12))")),
            (both.clone(), Strand::Minus)
        );
        assert_eq!(
            accepted(circular("join(complement(1..12),complement(90..100))")),
            (both.clone(), Strand::Minus)
        );
    }

    #[test]
    fn whitespace_from_continuation_lines_is_removed() {
        assert_eq!(
            accepted(circular("join(90..100,\n                     1..12)")),
            (vec![(89, 100), (0, 12)], Strand::Plus)
        );
        assert_eq!(accepted(linear(" complement( 10 .. 30 ) ")), (vec![(9, 30)], Strand::Minus));
    }

    #[test]
    fn a_reference_into_another_record_is_refused() {
        assert_eq!(rule(linear("J00194.1:100..202")), RemoteReference);
        assert_eq!(rule(linear("join(1..10,J00194.1:100..202)")), RemoteReference);
    }

    #[test]
    fn a_fuzzy_end_is_refused() {
        assert_eq!(rule(linear("<1..99")), FuzzyEnd);
        assert_eq!(rule(linear("1..>99")), FuzzyEnd);
        assert_eq!(rule(linear("complement(<1..99)")), FuzzyEnd);
    }

    #[test]
    fn a_site_between_bases_is_refused() {
        assert_eq!(rule(linear("10^11")), BetweenBases);
    }

    #[test]
    fn order_is_refused() {
        assert_eq!(rule(linear("order(1..10,20..30)")), Order);
        assert_eq!(rule(linear("complement(order(1..10,20..30))")), Order);
    }

    #[test]
    fn a_single_position_is_refused() {
        assert_eq!(rule(linear("467")), SinglePosition);
        assert_eq!(rule(linear("complement(46)")), SinglePosition);
        assert_eq!(rule(circular("join(46,1..12)")), SinglePosition);
    }

    #[test]
    fn position_zero_is_refused() {
        assert_eq!(rule(linear("0..10")), PositionZero);
        assert_eq!(rule(linear("complement(0..10)")), PositionZero);
    }

    #[test]
    fn a_reversed_range_is_refused() {
        assert_eq!(rule(linear("30..10")), RangeEndsBeforeStart);
        assert_eq!(rule(circular("95..5")), RangeEndsBeforeStart);
    }

    #[test]
    fn a_range_past_the_sequence_end_is_refused() {
        assert_eq!(rule(linear("1..101")), PastSequenceEnd);
        assert_eq!(rule(circular("join(90..101,1..12)")), PastSequenceEnd);
    }

    #[test]
    fn an_origin_join_on_a_linear_sequence_is_refused() {
        assert_eq!(rule(linear("join(90..100,1..12)")), OriginJoinOnLinear);
        assert_eq!(rule(linear("complement(join(90..100,1..12))")), OriginJoinOnLinear);
        assert_eq!(
            rule(linear("join(complement(1..12),complement(90..100))")),
            OriginJoinOnLinear
        );
    }

    #[test]
    fn every_other_join_is_refused() {
        for text in [
            "join(1..10,20..30)",                          // exons
            "join(1..12,90..100)",                         // origin parts, wrong order
            "join(complement(90..100),complement(1..12))", // minus, wrong order
            "join(90..100,complement(1..12))",             // mixed strands
            "join(80..89,90..100,1..12)",                  // three parts
            "join(1..99)",                                 // one part
            "join(90..99,1..12)",                          // does not reach n
            "join(90..100,2..12)",                         // does not start at 1
        ] {
            assert_eq!(rule(circular(text)), UnsupportedJoin, "{}", text);
        }
        assert_eq!(rule(linear("join(1..10,20..30)")), UnsupportedJoin);
    }

    #[test]
    fn text_outside_the_grammar_is_a_syntax_error() {
        for text in [
            "",
            "   ",
            "1..",
            "..5",
            "1...5",
            "complement(1..5",
            "complement(complement(1..5))",
            "complement(join(complement(1..5),complement(90..100)))",
            "Complement(1..5)",
            "bond(1,5)",
            "gap(10)",
            "1..5x",
            "1..5,6..9",
            "99999999999999999999..100",
        ] {
            assert_eq!(rule(linear(text)), Syntax, "{:?}", text);
        }
    }

    // Steps 3-6 run before the grammar, and step 8 before the shape: the
    // first rule that applies decides.
    #[test]
    fn the_first_rule_that_applies_decides() {
        assert_eq!(rule(linear("J00194.1:<1..10")), RemoteReference);
        assert_eq!(rule(linear("order(<1..10,20..30)")), FuzzyEnd);
        assert_eq!(rule(linear("join(0..10,20..30)")), PositionZero);
        assert_eq!(rule(linear("join(1..101,1..12)")), PastSequenceEnd);
    }

    #[test]
    fn each_rule_has_its_category() {
        let malformed = DomainError::UnusableGenBankLocation(UnusableGenBankLocationProblem {
            location: "0..10".to_string(),
            rule: PositionZero,
            sequence_length: N,
        });
        assert!(malformed.category() == ErrorCategory::MalformedInput);
        assert!(linear("1..101").unwrap_err().category() == ErrorCategory::BiologicalValidation);
        assert!(linear("<1..99").unwrap_err().category() == ErrorCategory::Unsupported);
        assert_eq!(linear("<1..99").unwrap_err().code(), "unusable_genbank_location");
    }

    #[test]
    fn the_message_quotes_the_location_as_given() {
        assert_eq!(
            linear("join(90..100,\n1..12)").unwrap_err().message(),
            "GenBank location 'join(90..100,\n1..12)' joins across the origin, but the sequence is linear"
        );
        assert_eq!(
            linear("1..101").unwrap_err().message(),
            "GenBank location '1..101' reaches past the end of the 100-base sequence"
        );
    }
}
```

If `use crate::error::UnusableGenBankLocationProblem;` is not already in scope
through `super::*`, add that import to the test module. `cargo fmt` may
reflow; do not change content. If a test fails, stop and report the test and
what the parser returned; do not change the test.

### 5.2 `error.rs` tests

- In `every_variant_has_its_pinned_code`, add after the `MalformedFasta` case:
  `DomainError::UnusableGenBankLocation(UnusableGenBankLocationProblem {
  location: s(), rule: GenBankLocationRule::Syntax, sequence_length: 1 })`
  with code `"unusable_genbank_location"`, and change the count from 48 to
  49.
- Add after `every_fasta_rule_has_its_pinned_code`:

```rust
    #[test]
    fn every_genbank_location_rule_has_its_pinned_code_and_category() {
        use ErrorCategory::*;
        use GenBankLocationRule::*;
        let cases: [(GenBankLocationRule, &str, ErrorCategory); 11] = [
            (Syntax, "syntax", MalformedInput),
            (PositionZero, "position_zero", MalformedInput),
            (RangeEndsBeforeStart, "range_ends_before_start", MalformedInput),
            (PastSequenceEnd, "past_sequence_end", BiologicalValidation),
            (FuzzyEnd, "fuzzy_end", Unsupported),
            (BetweenBases, "between_bases", Unsupported),
            (Order, "order", Unsupported),
            (RemoteReference, "remote_reference", Unsupported),
            (SinglePosition, "single_position", Unsupported),
            (OriginJoinOnLinear, "origin_join_on_linear", Unsupported),
            (UnsupportedJoin, "unsupported_join", Unsupported),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (rule, code, category) in cases {
            assert_eq!(rule.code(), code, "{:?}", rule);
            assert!(rule.category() == category, "{:?}", rule);
            assert!(seen.insert(code), "duplicate code {}", code);
        }
    }
```

### 5.3 The three `variant_name` matches

The exhaustive `fn variant_name` in `crates/core/tests/core_foundation.rs`,
`mutation_fixtures.rs` and `mutation_invariants.rs` each gain exactly one
arm, `DomainError::UnusableGenBankLocation(_) => "UnusableGenBankLocation",`,
after the `MalformedFasta` arm. Nothing else in those files.

### 5.4 Proving each rule is load-bearing (AGENTS.md)

For each rule, disable its check in the parser (so it never refuses for that
reason, e.g. `if false && …`), run `cargo test -p luria-core
genbank_location`, confirm the named test fails, and restore. If disabling
a check changes the outcome to a *different* rule rather than acceptance,
that still counts as the named test failing. Report the failing tests for
each.

| Rule | Test that must fail |
| --- | --- |
| `Syntax` | `text_outside_the_grammar_is_a_syntax_error` (disable the "nothing left over" check) |
| `PositionZero` | `position_zero_is_refused` |
| `RangeEndsBeforeStart` | `a_reversed_range_is_refused` |
| `PastSequenceEnd` | `a_range_past_the_sequence_end_is_refused` |
| `FuzzyEnd` | `a_fuzzy_end_is_refused` |
| `BetweenBases` | `a_site_between_bases_is_refused` |
| `Order` | `order_is_refused` |
| `RemoteReference` | `a_reference_into_another_record_is_refused` |
| `SinglePosition` | `a_single_position_is_refused` |
| `OriginJoinOnLinear` | `an_origin_join_on_a_linear_sequence_is_refused` |
| `UnsupportedJoin` | `every_other_join_is_refused` (disable the `n == sequence_length` and start-is-1 checks) |

## 6. Do not

- Edit files other than `crates/core/src/error.rs`,
  `crates/core/src/genbank_location.rs` (new), `crates/core/src/lib.rs`, and
  the three `variant_name` arms in §5.3.
- Change any existing behavior, message or test beyond §5.2 and §5.3.
- Build `NucleotideInterval`, `Feature` or `Cds` values, or check overlap or
  CDS length: those stay in `Cds::create`.
- Accept any form outside §4.1's grammar, or repair one (no lower-casing of
  keywords, no reordering of join parts).
- Add dependencies, or use regular expressions (there is no regex crate);
  write a small hand parser.

## 7. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 8. Handoff

Report the changed and new files, the exact commands and results, the §5.4
disable-and-restore results per rule, and anything that did not match this
contract.
