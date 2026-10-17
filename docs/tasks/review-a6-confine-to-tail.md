# Review A6: accept `ConfineToTail` where it changes nothing (D038)

Status: **Implemented 2026-10-16** by the local worker; accepted on planner review.
Planner: Claude. Parent: `docs/tasks/review-post-m4.md` A6. Decision: D038.

## 1. Requirement (D038, inline)

> A `luriaNative` request with placement `ConfineToTail` is designed exactly
> as the same request with `Inline` when the edit is an insertion (D032), or a
> replacement with `max(w, m) > 6`, which already puts every written base in
> the 5′ tails (D033). The design records the placement the caller asked for.
> For every other edit (substitutions, inline replacements and deletions) the
> placement is still refused with `UnsupportedPrimerPlacement`.

## 2. Required changes, `crates/core/src/primer_design.rs` only

1. **`design_primers` placement check** (around line 441). Refuse only when
   both of these hold:
   - the placement is unsupported (`!request.placement().is_supported()`);
   - the edit is not one D038 accepts: neither an insertion, nor a
     replacement with `max(w, m) > MAX_INLINE_REPLACEMENT`. Read `w` and `m`
     with `replacement_claim`.

   Keep the refusal's error payload exactly as it is today.
2. **`design_edge_pair`.** Pass `PrimerPlacement::Inline` to every
   `construct_primer` call, instead of `request.placement()`. Those primers
   are untailed, so they are inline by shape, and `construct_primer` refuses
   `ConfineToTail`. For today's inputs this changes nothing, because the
   placement there is always `Inline`. Leave `design_inline` as it is.
3. **The stored request** is the caller's, so a design reports the placement
   the caller asked for.
4. **Doc comments.** Update the module doc and `design_primers`' doc to state
   D038, wherever they say the placement is refused.

## 3. Required tests, in `primer_design.rs`'s test module

Build each design twice, once with `Inline` and once with `ConfineToTail`,
and use the module's existing helpers. For each accepted case, assert:
- the two designs are equal in `pair()`, `edit_span()`, `forward_tm()`,
  `reverse_tm()`, `forward_whole_tm()`, `reverse_whole_tm()`,
  `annealing_temperature()`, `tm_balance_met()` and both warning lists;
- `request().placement()` is `ConfineToTail` on the second design.

1. `confine_to_tail_designs_a_short_insertion_as_inline`: an insertion of at
   most 6 bases (the whole tail on the forward primer; the reverse primer has
   no tail).
2. `confine_to_tail_designs_a_long_insertion_as_inline`: an insertion of more
   than 6 bases (a split tail).
3. `confine_to_tail_designs_a_tail_path_replacement_as_inline`: a replacement
   with `max(w, m) > 6`.
4. `confine_to_tail_is_refused_for_an_inline_replacement`: a replacement with
   `max(w, m) <= 6` gives `UnsupportedPrimerPlacement`.
5. `confine_to_tail_is_refused_for_a_deletion`: gives
   `UnsupportedPrimerPlacement`.

The existing `confine_to_tail_is_refused_before_the_search` test, a
substitution, must still pass unchanged.

## 4. Do not

- Edit any other file. `PrimerPlacement::is_supported` and `construct_primer`
  keep refusing `ConfineToTail` as a caller's choice.
- Change existing tests or any design for an `Inline` request.

## 5. Checks

Same as `docs/tasks/review-a4-origin-split.md` §6.

## 6. Handoff

Report the changed file, the new tests, the exact check results, and any
deviation.
