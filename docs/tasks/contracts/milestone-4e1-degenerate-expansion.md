# Milestone 4e-1 task contract: expanding IUPAC ambiguity codes

Status: **Implemented 2026-10-15** by the local worker; accepted on planner review (§8).
Date: 2026-10-15
Planner: Claude. Parent: `docs/tasks/contracts/milestone-4-indels.md`, pass 5. This is
the first of four contracts; it adds only a pure function and its tests.
Nothing calls it yet.

## 1. Goal

`sequence.rs` can say which concrete bases an IUPAC code stands for, and can
expand a text containing ambiguity codes into every concrete sequence it
denotes, in D033's order.

## 2. Requirements, inline (D033, approved 2026-10-15)

- Each position's bases are taken in `A`, `C`, `G`, `T` order, and the
  **leftmost position varies fastest**. A replacement of `NS` gives, in order:
  `AC, CC, GC, TC, AG, CG, GG, TG`. That order is the reference tool's, quoted
  by the owner from its help text; it is the authority for this function.
- The IUPAC sets are the standard ones, already implied by
  `DNA_ALPHABET = "ACGTRYSWKMBDHVN"` and the complement table in this module:

  | code | bases | code | bases |
  | --- | --- | --- | --- |
  | `A` `C` `G` `T` | themselves | `B` | `C G T` |
  | `R` | `A G` | `D` | `A G T` |
  | `Y` | `C T` | `H` | `A C T` |
  | `S` | `C G` | `V` | `A C G` |
  | `W` | `A T` | `N` | `A C G T` |
  | `K` | `G T` | | |
  | `M` | `A C` | | |

  Each ambiguity code's set must be the complement-image of its partner's set
  under `A↔T`, `C↔G`: `B` (`CGT`) complements to `V` (`ACG`), `D` (`AGT`) to
  `H` (`ACT`), `R` to `Y`, `K` to `M`, `S` and `W` and `N` to themselves.
  §4's T6 checks this against the module's own `complement_letter`, so the
  table cannot drift from the complement rule.

## 3. Required behaviour, all in `crates/core/src/sequence.rs`

Both functions are additive. Change nothing that exists.

```rust
/// The concrete bases an IUPAC nucleotide code stands for, in `A`, `C`, `G`,
/// `T` order. `None` for any character outside `DNA_ALPHABET`.
pub fn bases_of(letter: char) -> Option<Vec<char>>;

/// Every concrete sequence a text of IUPAC codes denotes, in D033's order.
pub fn expand_degenerate(text: &str) -> Result<Vec<String>, DomainError>;
```

- `bases_of` returns a one-element vector for `A`, `C`, `G`, `T`.
- `expand_degenerate` refuses with `DomainError::InvalidSequenceCharacters`
  (`sequence: "replacement"`, `position` of the first offending character,
  `characters` that character) when a character is outside `DNA_ALPHABET`.
  It does **not** case-fold: callers pass canonical text.
- An empty text gives an empty vector. It is not an error here; the caller
  decides whether an empty replacement is legal.
- A text of only unambiguous bases gives exactly one sequence, equal to the
  text.
- The order is D033's: leftmost position fastest. Implement it by counting —
  for index `i` of `n` total, choose each position's base from its own set
  with the leftmost position's set as the innermost-varying digit — rather
  than by nesting loops, so the order does not depend on text length.
- The count is the product of the set sizes. Do not cap it here; D033 puts
  the 64-design warning at the request layer, not in this function.

## 4. Required tests, in `sequence.rs`'s test module

| # | Test | Expect |
| --- | --- | --- |
| T1 | the quoted expansion | `expand_degenerate("NS")` is exactly `["AC","CC","GC","TC","AG","CG","GG","TG"]`, in that order |
| T2 | plain bases expand to themselves | `expand_degenerate("ACGT")` is `["ACGT"]` |
| T3 | one code | `expand_degenerate("R")` is `["A","G"]`; `expand_degenerate("N")` is `["A","C","G","T"]` |
| T4 | the leftmost position varies fastest | `expand_degenerate("RRA")` is `["AAA","GAA","AGA","GGA"]` |
| T5 | every code's set | for each of the 15 letters, `bases_of` returns the §2 table's set, in `A`,`C`,`G`,`T` order; `bases_of('F')` and `bases_of('a')` are `None` |
| T6 | the sets respect the complement rule | for every letter `x` in `DNA_ALPHABET`, the set of `complement_letter(x)` equals the complements of `bases_of(x)`'s members, as a sorted set |
| T7 | the count is the product | `expand_degenerate("NNN").len() == 64` and all entries are distinct; `expand_degenerate("NNNN").len() == 256` |
| T8 | an empty text | `expand_degenerate("")` is empty |
| T9 | a character outside the alphabet | `expand_degenerate("ACFG")` gives `InvalidSequenceCharacters` with `position == 2` |

## 5. Do not

- Do not change `DNA_ALPHABET`, `complement_letter`, `normalize`,
  `is_dna_base` or anything else that exists in `sequence.rs`.
- Do not touch any other file, any fixture, or `docs/`. No new dependencies
  and no new `DomainError` variants.
- Do not add a cap on the number of expansions.

## 6. Checks

The five in `AGENTS.md`, with `export TMPDIR=$PWD/target/tmp`. Expected: 405
tests (396 + 9), none failing.

## 7. Handoff

Changed files, exact check results, added test names, anything unclear. If
something is ambiguous, stop and report rather than choosing.

## 8. Outcome, 2026-10-15

One worker run, the first of this milestone not cut off by the harness's time
limit — the contract was deliberately small.

Planner-run checks: fmt, clippy `-D warnings` and `git diff --check` pass, and
**405 tests pass (396 + 9), 0 failures**. `git diff` on `sequence.rs` has zero
removal lines, so the change is additive as required.

**Mutation check.** Making the rightmost position the fastest-varying digit —
the order the planner first drafted — fails four tests, including T1, which is
pinned to the owner's quoted list. Breaking the `S` set fails two; truncating
`N` fails five, including T6's complement check.

**Honesty note.** The worker reported, unprompted, that an intermediate edit
had truncated an existing digest assertion and that it had reverted it. The
planner verified this independently: the assertion reads `seq-a@a83f6d82` at
line 654 and the file's diff contains no removal lines.

**Known boundary, not handled:** the stride product is `usize`, so a text with
enough ambiguity codes would overflow before anything was allocated. D033 puts
the 64-design warning at the request layer, which is where a real request is
bounded; a direct caller of `expand_degenerate` is not. Worth a guard when the
request layer is built (4e-4).
