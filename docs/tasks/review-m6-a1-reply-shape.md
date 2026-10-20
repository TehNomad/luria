# Review M6 A-1, A-3: a reply is an object, and its error is one line

Status: **Implemented 2026-10-20** by the local worker; accepted on planner review. Planner: Claude. Parent:
`docs/tasks/review-post-m6.md` A-1, A-3. Owner, 2026-10-20: "Go ahead and
fix them" (the review recommends refusing an array-form reply).

Files (no others):
- `crates/nl/src/extraction.rs`

## 1. Defects

- A-1: `parse_extraction` accepts a JSON **array** where an object is
  required, at the top and for each mutation, because serde's derived
  struct deserializer also reads a sequence, which skips field names and
  `deny_unknown_fields`. `["q5_residue_substitutions","synA",[["A42G","A42G",null]],null,null,[]]`
  parses as a full `Extraction`. Plan §3 step 1: a wrong type is a
  failure, not a repair.
- A-3: `ExtractionError.message` is serde's error text, which echoes the
  model's own strings: `"operation":"a\nb\nINJECTED"` gives a message with
  two newlines, against the type's own "one line" rule.

## 2. Required change

All in `crates/nl/src/extraction.rs`.

### 2.1 The shape check (A-1)

`parse_extraction` keeps its size check first and its typed parse second,
unchanged (the typed parse is what refuses duplicate keys). When the typed
parse succeeds, it then parses the same `text` as `serde_json::Value`
and refuses:
- a top-level value that is not an object:
  `"the reply is not a JSON object"`;
- any element `i` of the top-level `"mutations"` array that is not an
  object: `format!("mutations[{}] is not a JSON object", i)`.

Only when both pass is the typed `Extraction` returned. A failure to parse
as `Value` after the typed parse succeeded cannot happen; if it does,
return its error text as the message rather than panicking.

### 2.2 One line (A-3)

Every `ExtractionError` message is passed through a new private function

```rust
/// One line of at most `MAX_MESSAGE_CHARS` chars: each control char is a
/// space, and a longer message is cut and ends with `...`.
fn one_line(message: String) -> String
```

with `const MAX_MESSAGE_CHARS: usize = 300;`. A message of more than 300
chars becomes its first 297 chars followed by `...` (300 in all). Chars,
never bytes. The size-cap message and the shape messages go through it
too.

Update the module doc's list of refusals with "an array where an object
is required" and the `ExtractionError` doc with the 300-char limit.

## 3. Tests (write first; confirm each new one fails before the change)

In the test module, existing tests unchanged:

1. `an_array_is_refused_where_an_object_is_required`:
   - `parse_extraction(r#"["q5_residue_substitutions","synA",[["A42G","A42G",null]],null,null,[]]"#)`
     is `Err` with message `"the reply is not a JSON object"`;
   - `parse_extraction(r#"{"operation":"q5_residue_substitutions","cds":"synA","mutations":[["A42G","A42G","GGT"]],"composition":null,"numbering":null,"unsupported":[]}"#)`
     is `Err` with message `"mutations[0] is not a JSON object"`;
   - the same document with `"mutations":[{"token":"A42G","quote":"A42G","codon":null},["W87A","W87A",null]]`
     is `Err` with message `"mutations[1] is not a JSON object"`.
2. `an_error_message_is_one_line`: the reply
   `{"operation":"a\nb\nINJECTED","cds":null,"mutations":[],"composition":null,"numbering":null,"unsupported":[]}`
   (the `\n` written as JSON escapes, so the JSON is valid) is `Err`, and
   its message holds no char for which `char::is_control` is true.
3. `a_long_error_message_is_cut`: a reply whose `operation` is a 1000-char
   string of `x` is `Err`; its message has exactly 300 chars and ends
   with `...`.

Disabling proofs (once each, run, restore, report): skip the mutation
check → test 1's second and third cases fail; skip `one_line` → tests 2
and 3 fail.

## 4. Do not

- Add dependencies, or change any file but `extraction.rs`.
- Change any type, field, existing message text (other than passing it
  through `one_line`) or existing test.

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, each new test and whether it failed before the
change, each disabling proof and its result, the check results, and any
deviation.
