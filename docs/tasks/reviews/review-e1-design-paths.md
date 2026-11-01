# Review refactor E1: share the steps of the three design paths

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review. Non-test lines 1374 → 1270.
Planner: Claude. Parent: `docs/tasks/reviews/review-post-m4.md` finding E1.

**Behavior-preserving.** Every existing test must pass unchanged, and every
design and error payload must be byte-identical: values, the order of
obstacles, and message text.

## 1. The duplication

`crates/core/src/primer_design.rs` has three private design paths. Each one
repeats the same steps:

| Step | `design_inline` | `design_deletion` | `design_tailed` |
| --- | --- | --- | --- |
| smallest-pair check → `NoPrimerDesignPossible` | yes | yes | yes |
| the `fits` closure → `NoPrimerDesignFits` | yes | yes | yes |
| 5′/3′ availability obstacles | yes, needs `MIN_FLANK + first_reverse` and `first_flank` | yes, `first` both sides | yes, `first` both sides |
| ambiguity scan → `AmbiguousTemplateBase` | one range | two ranges | two ranges, same as deletion |
| available-run counting | yes | yes | yes, same as deletion |
| Tm balance and annealing temperature | yes | yes | yes |

`design_deletion` and `design_tailed` are the same procedure, apart from
three differences when there are no tails:
- both primers are built with `construct_primer`;
- `forward_whole_tm` and `reverse_whole_tm` are `None`;
- the warnings inspect the primers as they are, which is the same text,
  because an untailed primer is its annealing portion.

## 2. Required changes, `crates/core/src/primer_design.rs` only

**2.1 Private helpers.** Use the names given, or close equivalents. Each gets
a one-line doc comment that cites the rule it implements.

- `fn require_possible(template: &Sequence, smallest_pair: u64) -> Result<(), DomainError>`
  holds the `smallest_pair > length` check and its `NoPrimerDesignPossible`
  payload. The saturating sums stay at the call sites (B2).
- `fn availability_obstacles(start: u64, end: u64, length: u64, five_needed: u64, three_needed: u64) -> Vec<DesignObstacle>`
  returns the `FivePrimeAvailability` obstacle first, then
  `ThreePrimeAvailability`, with exactly the payloads used today.
- `fn ambiguity_obstacles(wild_type: &[char], ranges: &[(u64, u64)]) -> Vec<DesignObstacle>`
  scans each half-open range in the order given, ascending within each one.
- `fn run_after(wild_type: &[char], from: u64) -> u64` counts ACGT bases from
  `from` up to the template's end. `fn run_before(wild_type: &[char], to: u64) -> u64`
  counts ACGT bases ending just before `to`, going down to 0.
- `fn pair_summary(forward_tm: &MeltingTemperature, reverse_tm: &MeltingTemperature) -> (bool, f64)`
  returns `(tm_balance_met, annealing_temperature)` with today's formulas.

**2.2 Use them in all three paths.** Keep every range exactly as it is
today. In particular, `design_inline`'s ambiguity range stays
`boundary - first_reverse .. end + first_flank`, where
`boundary = start - MIN_FLANK`. That range includes the edit region and the
deletion and tail paths' ranges don't. This difference is finding B14, an open
question: do not unify it.

**2.3 Merge the deletion path into the edge-pair path.**
- Rename `design_tailed` to `design_edge_pair`, and let it take an empty
  `text`.
- When `text` is empty:
  - build both primers with `construct_primer` and
    `request.placement()`, exactly as `design_deletion` does today;
  - set `forward_whole_tm` and `reverse_whole_tm` to `None`;
  - inspect the warnings on `sizer.forward(...)` and `sizer.reverse(...)?`,
    as `design_deletion` does.
- When `text` is non-empty, behave exactly as `design_tailed` does today.
- `design_deletion` keeps its own checks. It reads `start` and `end` from
  the target coordinates, and refuses an empty coordinate list with the same
  `out_of_scope` message. It then calls
  `design_edge_pair(template, result, request, start, end, String::new(), false)`.
- Update `design_edge_pair`'s doc comment to cover all three callers.

**2.4 Comments** move with the code they describe. Delete a comment only if
the code it described is gone.

## 3. Guards

Run both from the repo root. Each must print nothing and exit 0.

- **Public API unchanged**
  (`python3 -I target/api_guard.py crates/core/src/primer_design.rs`).
  The script compares every line starting with `pub ` before the test
  module, against `HEAD`:

  ```python
  import re, subprocess, sys
  def api(text):
      text = text.split('#[cfg(test)]')[0]
      return [l.strip() for l in text.splitlines() if re.match(r'\s*pub\s', l)]
  bad = False
  for f in sys.argv[1:]:
      old = subprocess.run(['git', 'show', 'HEAD:' + f], capture_output=True,
                           text=True, check=True).stdout
      if api(old) != api(open(f).read()):
          print('PUBLIC API CHANGE:', f); bad = True
  sys.exit(1 if bad else 0)
  ```

  The file `target/api_guard.py` exists. If it is missing, recreate it from
  the text above.
- **Tests unchanged:** `git diff --stat` shows no change below the
  `#[cfg(test)]` line of `primer_design.rs`, and no change under
  `crates/core/tests/`.

## 4. Do not

- Edit any other file.
- Change behavior, error payloads or message text, or the order in which
  obstacles are reported.
- Change any existing test.
- Unify the inline ambiguity range (B14).
- Touch `size_pair` or `Sizer`, apart from passing them the same values.
- Add public items.

## 5. Checks

Use `TMPDIR=$PWD/target/tmp` for cargo (`mkdir -p target/tmp`).

```bash
python3 -I target/api_guard.py crates/core/src/primer_design.rs
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

## 6. Handoff

Report:
- the line counts of the non-test part, before and after;
- the helpers added, and which paths use each one;
- the exact check results, including the total test count, which must
  equal 510;
- any deviation.
