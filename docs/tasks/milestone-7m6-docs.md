# Milestone 7m6: the docs for the owner's hand-test changes

Status: **Ready for the worker, 2026-10-22.** Planner: Claude. Parent:
`docs/tasks/milestone-7m-polish.md` §2 item 6. Follows 7m1 to 7m5, all
committed.

This contract authorizes the worker to edit `docs/cli.md` and
`docs/decisions.md` exactly as below and nowhere else. Every old text
must occur exactly once; if one does not, stop and report it
(ask_owner). Replace text verbatim; do not reword.

## 1. `docs/cli.md` §11

1. Replace

   ```
   request and result files stay zero-based (§3). `?` opens a help pop-up
   with the legend and the keys.
   ```

   with

   ```
   request and result files stay zero-based (§3). `?` opens a help pop-up
   with the open screen's keys, and on the variant screen the legend.
   ```

2. In the screen table, the Overview row's keys
   `` `j`/`k` move, `enter` open, `p` provenance, `e` problem, `x` export ``
   become `` `j`/`k` move, `enter` open, `e` problem, `x` export ``.
3. The Details row's text becomes: `Everything the result records for
   the variant: the edit, its warnings by code, and each primer with its
   parts, Tm basis and warnings, or why no pair was designed; then the
   provenance: the package, every algorithm and version, the request and
   template digests, what a GenBank record imported and did not, and the
   buffer`. Its keys stay `` `j`/`k` scroll ``.
4. Delete the whole `| Provenance |` row.
5. The Problem row's keys `` `p` provenance `` become `` `j`/`k` scroll ``.
6. A check, not an edit: after items 1 to 5, `` `p` `` does not occur in
   §11. If it does, stop and report where.
7. Replace

   ```
   With `--export DIR`, `x` on the overview lists the files an export would
   write, and `y` writes them:
   ```

   with

   ```
   With `--export DIR`, `x` on the overview lists the files an export would
   write, and `w` or `o` writes them; `esc` goes back:
   ```

8. In the export file table, `` | `<member>.fa` | `` becomes
   `` | `<template id>_<member>.fa` | ``.
9. Replace

   ```
   `<request>` is the request file's name without `.json`. If any of these
   files exists, nothing is written. Before it writes, luria runs the
   request again and refuses if the result is not the one on screen.
   ```

   with

   ```
   `<request>` is the request file's name without `.json`. If any of these
   files exists, a prompt names them, and `y` replaces them; any other key
   writes nothing. A replaced file is written whole and then moved into
   place, and a name that is a link or a folder is never replaced. Before
   it writes, luria runs the request again and refuses if the result is
   not the one on screen.
   ```

## 2. `docs/cli.md` §12

Replace everything from the line `## 12. Entering a design in the
terminal: \`luria new\`` to the end of the file with the whole of
`docs/tasks/milestone-7m6-cli-section.txt`.

## 3. `docs/decisions.md`

1. In D046's Export bullet, replace

   ```
     IDT order table, written only on `y` into the `--export DIR`
     directory, as `<request>.result.json`, `<member>.fa` and
     `<request>.order.tsv`, and never replacing a file (§8a).
   ```

   with

   ```
     IDT order table, written only on `w` or `o` into the `--export DIR`
     directory, as `<request>.result.json`, `<template id>_<member>.fa`
     (M7-57) and `<request>.order.tsv`; an existing file is replaced only
     on `y` at a prompt that names it (M7-49, D047).
   ```

2. Replace D047, from its heading `## D047:` to the end of the file, with
   the whole of `docs/tasks/milestone-7m6-d047.txt`.

## 4. Test

`crates/cli/tests/documented_new.rs` and `documented_review.rs` must pass
unchanged. If either fails, stop and report the failure; do not change the
test or the text.

## 5. Do not

Change only `docs/cli.md` and `docs/decisions.md`, as above. No code,
test, fixture, other doc, Cargo file or `vendor/`. Do not stage or commit.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result and any old text that did not occur exactly once.
