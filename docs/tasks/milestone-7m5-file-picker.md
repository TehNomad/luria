# Milestone 7m5: a filtered file list under `File`

Status: **Implemented, 2026-10-22.** Planner: Claude. Parent:
`docs/tasks/milestone-7m-polish.md`, M7-50. Builds on 7m4 (committed
`7848c46`).

## 1. The owner's answer this contract applies

M7-50: a filtered list under `File`: `.fa .fasta .fna .gb .gbk .genbank`
files and folders in the directory being typed, filtered as you type,
`↑`/`↓` to pick, `enter` to choose. `tab` still moves on.

## 2. Changes

### 2.1 Listing, `crates/cli/src/new/picker.rs` (new; add `pub mod picker;` to `new.rs`)

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry { pub name: String, pub folder: bool }

pub const TEMPLATE_EXTENSIONS: [&str; 6] = ["fa", "fasta", "fna", "gb", "gbk", "genbank"];

pub fn list(dir: &Path) -> Vec<Entry>
```

`list` reads `dir` once with `fs::read_dir`. It keeps folders, and files
whose extension is in `TEMPLATE_EXTENSIONS` (compared case-insensitively).
It leaves out names starting with `.` and names that are not UTF-8. The
order is folders first, then files, each sorted by name. A directory
that cannot be read gives an empty list. A symlink counts as what it
points at (`fs::metadata`), and one that is broken is left out. This is
the only file reading the picker does.

### 2.2 The form, `crates/cli/src/new/form.rs` (pure, as now)

- `pub fn file_dir(&self) -> String`: the typed `File` text up to and
  including its last `/`, or `""` when it has none. This is the
  directory the loop lists, relative to `cwd`.
- `pub fn set_listing(&mut self, dir: String, entries: Vec<Entry>)` stores
  the listing for `dir`. `listing_dir() -> Option<String>` returns it,
  so the loop lists a directory only when `file_dir()` changes.
- `pub fn choices(&self) -> Vec<Entry>`: the stored entries whose name
  starts with the typed text after `file_dir()` (case-insensitive). Empty
  when focus is not `File`, or when the stored listing is for another
  directory.
- `pub fn chosen(&self) -> Option<usize>`: the highlighted choice, `None`
  until `↑` or `↓` is pressed. Any edit of `File` resets it to `None`.
- On `File`, when `choices()` is not empty: `Down` highlights the next
  choice (from `None`, the first), and `Up` the previous (from the
  first, back to `None`), with no wrapping. When it is empty, `Up` and
  `Down` move focus, as now.
- `Enter` on `File` with a highlighted choice:
  - a folder: `File` becomes `file_dir() + name + "/"`, focus stays,
    `chosen` resets, and the result is `Continue`;
  - a file: `File` becomes `file_dir() + name`, then it behaves exactly
    as `Tab` does now (focus moves on, and `Load(path)` is returned).
- `Enter` with nothing highlighted, `Tab` and `BackTab` behave as now.

### 2.3 Text and drawing, `crates/cli/src/new/text.rs` and `crates/cli/src/new/view.rs`

- Under the `File` row, and before its `<name>, FASTA` note, one continued
  row per choice, at most 8: `> name` for the highlighted one and
  `  name` for the rest. A folder's name ends with `/`. With more than 8
  choices, a 9th row reads `  … <n> more`, and the window scrolls so
  the highlighted choice is shown. With no choices, there are no rows.
- `view.rs`: the highlighted row is reversed, folder rows are bold, and
  the `… more` row is dim. A row starting `> ` or `  ` under `File` is
  told apart from other continued rows by its position, so read it from
  `FormText`. Add `pub picker_rows: Option<(usize, usize)>` to
  `FormText`: the first line index and the count, or `None`.
- The form's key line, while focus is `File` and `choices()` is not
  empty: `↑/↓ pick  enter choose  tab next  esc quit`.

### 2.4 The loop, `crates/cli/src/new/terminal.rs`

Before every draw, while the proposal is not shown: when `form.focus()`
is `File` and `form.listing_dir()` is not `Some(form.file_dir())`, call
`form.set_listing(dir, picker::list(&cwd.join(dir_or_dot)))`. Here
`dir_or_dot` is `form.file_dir()`, or `"."` when that is empty.

## 3. Tests

1. `crates/cli/tests/new_picker.rs` (new), `list_keeps_templates_and_folders`:
   in a fresh `target/tmp` directory, create `a.fa`, `B.GB`, `c.txt`,
   `.hidden.fa`, `d.fasta.bak` and folder `sub`. `list` gives `sub/`
   (folder), then `B.GB` and `a.fa` in sorted order, and nothing else.
2. `new_picker.rs`, `an_unreadable_directory_lists_nothing`: a path that
   does not exist gives an empty list.
3. `new_form.rs`, `typing_filters_the_choices`: a listing with `sid-1_cDNA.fa`,
   `syn.fa` and folder `genbank`; typing `s` keeps the two files, `si`
   keeps one, `x` keeps none.
4. `new_form.rs`, `down_and_enter_choose_a_file_and_load_it`: from an
   empty `File`, `Down` then `Enter` on `sid-1_cDNA.fa` (sorted first
   among files, after the folder) gives `Load("sid-1_cDNA.fa")`.
   Choose the entry by its own position in `choices()`; do not assume an
   index.
5. `new_form.rs`, `a_folder_is_entered_not_loaded`: choosing `genbank/`
   leaves focus on `File`, sets the text to `genbank/`, and gives
   `Continue`. `file_dir()` is then `genbank/`.
6. `new_form.rs`, `up_and_down_move_focus_without_choices`: with no
   listing, `Down` on `File` moves focus as now.
7. `new_text.rs`, `the_choices_are_drawn_under_file`: the rows and the
   `> ` marker; 10 entries show 8 rows and `  … 2 more`.

Existing tests change only if they press `Up`/`Down` on `File` with a
listing set, and none do today. No existing assertion is expected to
change. If one must, stop and report it.

### 3.1 Rules that must be proven

1. Template files only (drop the extension filter): test 1 fails.
2. A folder is entered, not loaded (treat folders as files): test 5
   fails.

## 4. Do not

- Change only `new.rs` (one `pub mod` line), the new `new/picker.rs`,
  `new/form.rs`, `new/text.rs`, `new/view.rs`, `new/terminal.rs`, and the
  tests named in §3.
- No change to the review screens, `interpret`, fixtures, docs, Cargo
  files or `vendor/`. No new dependency.
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the §3.1 proofs, and any existing assertion changed.

## 6. Implementation notes (planner, 2026-10-22)

- 1140 tests passed, 0 failed, 1 ignored; fmt, clippy, diff check clean;
  fixtures 19 of 19. Both §3.1 proofs failed as required. No existing
  assertion changed; the tests diff is insertions only.
- Untested: the loop's listing (reads the directory whenever `File`'s
  directory part changes) and the picker's styles on a real terminal.
