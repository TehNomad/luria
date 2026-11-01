# Contract 9r — core, cli: the review's memory and speed fixes

Planner: Claude. Status: implemented. Tier: light, with rule 2 full (the edit
buffer, M9-10).
Plan row: `docs/tasks/milestone-9-editor.md` §7, row 9r.

## 1. What this contract fixes

These are findings 6 to 9 of the milestone 9 review (2026-10-31). The
owner asked for them to be fixed before the merge. None of them changes
what the user sees:
1. Once ORFs have been shown, every edit computes them again, even while
   their lanes are hidden.
2. Every edit stores a full copy of all features in its `Undo`, and
   `apply_edit` and `undo_group` copy the whole buffer. On a genome with
   thousands of features, memory grows by megabytes per typed base.
3. A rebuilt record keeps the `GenBankRecord` it was built from, a
   second full copy of the sequence that nothing reads.
4. A save matches each feature against every opened feature, cloning as
   it goes.

## 2. Files the worker may touch

1. `crates/cli/src/sequence_view/state.rs`.
2. `crates/core/src/edit_buffer.rs`.
3. `crates/core/src/view_record.rs`: `rebuild` and `genbank_view` only.
4. `crates/core/src/genbank_write.rs`: `feature_lines` and
   `unchanged_feature` only.
5. `crates/core/tests/view_rebuild.rs`: test 3 only.
6. `crates/cli/tests/sequence_view_orf_lanes.rs`: in
   `o_shows_and_hides_the_orfs`, only the assertion after the second `O`
   that `orfs()` is `Some`. It becomes `is_none()`, with a message citing
   9r §3.1. It encodes the 8m18 rule this contract reverses.

## 3. Rules

1. **ORFs while hidden.** Hiding the ORF lanes (`toggle_orf_lanes`) sets
   `orfs` to `None`. Showing them computes them, as today. `rebuild_view`
   keeps its rule: it computes them again only when `orfs` is `Some`.
2. **Undo memory.**
   - `Undo.features` becomes `Option<Vec<EditFeature>>`. `undo` restores
     the features only when they are `Some`, and otherwise leaves them
     as they are.
   - Add `pub fn without_features(self) -> Undo`, which drops them.
   - In `state.rs`, every `Undo` pushed to a group after the group's
     first is stored `without_features()`. A group is undone newest
     first, so its first `Undo`, undone last, restores the features as
     they were before the group. Redo replays the edits and needs no
     snapshot.
   - `debug_assert!` in `undo_group` that the group's first `Undo` holds
     features.
3. **No whole-buffer copy per edit.** `apply_edit` takes the buffer out
   of `self.edit` and applies the edit to it in place. If `rebuild_view`
   then fails, it calls `buffer.undo(undo)` and puts the buffer back. The
   state after a failed edit is as today: nothing changed. `undo_group`
   and `redo_group` work the same way.
4. **No spare sequence.** `rebuild` returns `genbank: None` for every
   record. The sequence goes into the `ViewRecord` by move, not by
   `clone`. `genbank_view` may change to take what it needs by value;
   `open_record` must return exactly what it returns today. The save
   keeps reading `self.opened.genbank`, which is unchanged.
5. **Matching features at a save.** `feature_lines` finds a buffer
   feature's opened feature by its `line`, through one map from line to
   opened index built once per save. A feature counts as unchanged under
   the same comparison `unchanged_feature` makes today: key, location
   text and qualifiers. Compare by reference, with no clones. A buffer
   feature whose line is in no opened feature is new, as today.

## 4. Tests

| # | Test | Asserts |
| --- | --- | --- |
| 1 | unit, `state.rs`: `hidden_orfs_are_not_recomputed` | pBR322: `O`, `O` (shown, then hidden): `orfs` is `None`. One `x` leaves it `None`. `O` again: `Some`, and equal to the ORFs a fresh view of the edited record computes |
| 2 | unit, `state.rs`: `a_group_keeps_one_feature_snapshot` | pBR322: `i`, `ACG`, `esc`. The closed group has 3 `Undo`s, and only the first holds features. `u`: the sequence and every feature (key, location, qualifiers) equal the opened record's. `ctrl-r`: equal to the view before the `u` |
| 3 | `view_rebuild.rs::an_unedited_rebuild_is_the_opened_record` | Add one assertion: the rebuilt record's `genbank` is `None`. Change nothing else in the test |
| 4 | unit, `edit_buffer.rs`: `undo_without_features_keeps_the_features` | A buffer, one `Insert` before a feature (the feature shifts), and its `Undo` passed through `without_features`: after `undo`, the letters are back and the features are still shifted. With features: both are back |
| 5 | existing | Apart from §2.6, every 9e, 9g, 9h, 9l, 9n and 9o test, `genbank_write_fixtures.rs`, `feature_edit_fixtures.rs` and `view_rebuild.rs` pass with no other change |
| 6 | measurement | `recompute_profile` in release: report the table. Step 1 (the clone) should be gone or near 0 |

## 5. Checks

`scripts/check`; then test 6's release run.

## 6. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| 1 | Hiding the lanes keeps `orfs` | test 1 |
| 2 | Every `Undo` in the group is stored `without_features`, the first too | test 2 |
| 3 | `undo` ignores `Some` features | test 4 |

Disable each rule once, run the tests, restore it, check with `cmp`
against a backup in `target/tmp/`, and report what failed.

## 7. Do not

- Change what any key does, what is drawn, or what a save writes.
- Change any fixture, or any test not named in §2.
- Add dependencies.

## 8. Result (planner)

Release, `recompute_profile`: the buffer copy (step 1) is gone. One edit:
A 92 ms (was 107), B 327 ms (ORF scan, varies), C 1.0 ms. Each group now
keeps one feature snapshot, not one per base. One temporary sequence copy
stays inside `rebuild`; removing it means changing `view_cdss`, which is
outside this contract.
