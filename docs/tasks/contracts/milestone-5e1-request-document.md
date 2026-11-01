# Milestone 5e1: the strict request document

Status: **Implemented 2026-10-17** by the local worker, across two runs (the first hit the turn limit with `json.rs` done and the readers unwritten); accepted on review.
Planner: Claude. Parent: [docs/tasks/contracts/milestone-5-cli.md](/tasks/contracts/milestone-5-cli.md) §3.4 and §5 (5e,
split into 5e1 and 5e2). Follows 5d (the `luria-cli` crate).

## 1. What and why

The request file is the typed surface that `luria q5 design` reads, and that
the Milestone 6 adapter will write (D004, D016). This contract reads its
JSON text into a plain Rust model, `RequestDocument`, refusing anything the
schema doesn't allow. Each refusal names the JSON path of the problem.

5e1 does no file I/O and calls no core constructor. Values that core
validates, such as identifiers, residue letters, coordinates, completeness,
genetic code and numbering, are kept as given, so that core's own errors
report them in 5e2. This contract only checks the document's shape: which
fields exist, their JSON types, and the closed sets of keyword values.

## 2. Requirements this contract applies

- **Plan §3.4:** the request is JSON and strict. Unknown fields, missing
  required fields and wrong types are each refused, with the JSON path.
  Nothing has a default that changes the science: policy, style, minimum
  overlap, placement, the coordinate convention, and (for FASTA) topology
  are all required. The template is FASTA or GenBank, not both. There is
  exactly one of three edit forms (`edit`, `variants`, `degenerate`), and
  five edit kinds. `residue` comes with or without `cds`, with `codon` or
  `"policy": "default"`. `combined_construct` is accepted by the parser
  (the workflow refuses it).
- **D041:** `"coordinates": "zero-based-half-open"` is required, and any
  other value is refused. Intervals are `[start, end]` pairs. Residue
  numbers are 1-based.
- **D041 (G6):** the buffer is required. It is either the preset
  `"q5_reaction"` or an object with `monovalentMillimolar`,
  `primerNanomolar` and `dmsoPercent`.
- **Decided here:**
  - duplicate object keys are refused. serde_json's own `Value` keeps the
    last one silently, which would be a hidden repair;
  - `template.sha256` is checked as 64 lower-case hex digits here; 5e2
    compares it with the sequence's content digest;
  - `null` is never a stand-in for an absent field.

## 3. Files

- `crates/cli/src/json.rs` (new): the JSON tree and its parser.
- `crates/cli/src/request.rs` (new): `RequestError`, the document model and
  `parse_request`.
- `crates/cli/src/lib.rs`: add `pub mod json;` and `pub mod request;`,
  keeping the module list alphabetical.

### 3.1 `json.rs`

```rust
/// A JSON value whose objects keep their members in document order. Built
/// only by `parse_json`, which refuses duplicate keys.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

/// Parse one JSON document. The error text is serde_json's, which ends with
/// ` at line L column C`.
pub fn parse_json(text: &str) -> Result<Json, String>;
```

Implement it with serde only:
- `impl<'de> serde::Deserialize<'de> for Json`, calling `deserialize_any`
  with a `Visitor` that handles unit, bool, `i64`, `u64`, `f64` (via
  `serde_json::Number::from_f64`; a non-finite value is an error), str,
  string, seq and map;
- in `visit_map`, collect the members in order. A key already seen is
  `serde::de::Error::custom(format!("duplicate key '{}'", key))`;
- `parse_json` makes a `serde_json::Deserializer::from_str(text)`, calls
  `Json::deserialize`, then `deserializer.end()` so trailing text is
  refused, and maps any error to its `to_string()`.

### 3.2 `request.rs`: the error

```rust
/// What went wrong with a request before core saw it. Every one is
/// `MalformedInput` (plan §3.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestErrorCode {
    /// The request file could not be read (5e2, 5g).
    Unreadable,
    /// The request is not valid JSON or breaks the schema.
    Invalid,
    /// The template file named by the request could not be read (5e2).
    TemplateUnreadable,
    /// `template.sha256` is not the template sequence's digest (5e2).
    TemplateDigestMismatch,
}

impl RequestErrorCode {
    /// `request_unreadable`, `request_invalid`, `template_unreadable`,
    /// `template_digest_mismatch`.
    pub fn code(self) -> &'static str;
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestError {
    pub code: RequestErrorCode,
    /// The JSON path of the problem: `$` for the whole document, then
    /// `.key` for an object member and `[i]` for an array element, e.g.
    /// `$.variants.members[2].residue.number`.
    pub path: String,
    /// One line, no trailing newline.
    pub message: String,
}
```

5e1 produces only `Invalid`. The others are declared now so the codes are
pinned before any JSON exposes them.

### 3.3 `request.rs`: the document model

All fields are `pub`, and every type derives `Clone, Debug, PartialEq`.

```rust
pub struct RequestDocument {
    pub template: TemplateSpec,
    pub scope: Option<String>,
    pub operation: OperationSpec,
    pub design: DesignSpec,
}

pub enum TemplateSpec {
    Fasta { id: String, path: String, topology: TopologySpec, sha256: Option<String>, cds: Vec<CdsSpec> },
    GenBank { id: String, path: String, sha256: Option<String>, cds: Vec<String> },
}

pub enum TopologySpec { Linear, Circular }

pub struct CdsSpec {
    pub id: String,
    pub intervals: Vec<(i64, i64)>,
    pub strand: StrandSpec,
    pub completeness: String,
    pub genetic_code: String,
    pub numbering: String,
}

pub enum StrandSpec { Plus, Minus }

pub enum OperationSpec {
    Edit(MemberSpec),
    Variants { composition: CompositionSpec, members: Vec<MemberSpec> },
    Degenerate { name: String, interval: (i64, i64), from: String, to: String },
}

pub enum CompositionSpec { SeparateVariants, CombinedConstruct }

pub struct MemberSpec { pub name: String, pub edit: EditSpec }

pub enum EditSpec {
    Residue { cds: Option<String>, number: i64, from: String, to: String, codon: CodonSpec },
    Substitution { interval: (i64, i64), from: String, to: String },
    Deletion { interval: (i64, i64), from: String },
    Insertion { point: i64, before: String, after: String, insert: String },
    Replacement { interval: (i64, i64), from: String, to: String },
}

pub enum CodonSpec { Supplied(String), Default }

pub struct DesignSpec {
    pub policy: PolicySpec,
    pub style: StyleSpec,
    pub min_annealing_overlap: i64,
    pub placement: PlacementSpec,
    pub buffer: BufferSpec,
}

pub enum PolicySpec { LuriaNative }
pub enum StyleSpec { Q5Polymerase }
pub enum PlacementSpec { Inline, ConfineToTail }
pub enum BufferSpec {
    Q5Reaction,
    Values { monovalent_millimolar: f64, primer_nanomolar: f64, dmso_percent: f64 },
}

pub fn parse_request(text: &str) -> Result<RequestDocument, RequestError>;
```

### 3.4 The schema

Each object below lists its fields in **schema order**. A field marked
`(opt)` may be absent. A group marked `one of` must hold exactly one of the
named fields.

| Object (path) | Fields |
| --- | --- |
| request (`$`) | `schema` = `"luria.q5-request/1"`; `coordinates` = `"zero-based-half-open"`; `template`; `scope` string (opt); one of `edit`, `variants`, `degenerate`; `design` |
| template (`$.template`) | `id` string; one of `fasta` string, `genbank` string; with `fasta`: `topology` one of `"linear"`, `"circular"`, then `sha256` (opt), then `cds`: array of cds objects; with `genbank`: `sha256` (opt), then `cds`: array of strings |
| cds object (`$.template.cds[i]`) | `id` string; `intervals` array of pairs; `strand` one of `"+"`, `"-"`; `completeness` string; `geneticCode` string; `numbering` string |
| edit and each member (`$.edit`, `$.variants.members[i]`) | `name` string; one of `residue`, `substitution`, `deletion`, `insertion`, `replacement` |
| `residue` | `cds` string (opt); `number` integer; `from` string; `to` string; one of `codon` string, `policy` = `"default"` |
| `substitution`, `replacement` | `interval` pair; `from` string; `to` string |
| `deletion` | `interval` pair; `from` string |
| `insertion` | `point` integer; `before` string; `after` string; `insert` string |
| variants (`$.variants`) | `composition` one of `"separate_variants"`, `"combined_construct"`; `members` array of member objects |
| degenerate (`$.degenerate`) | `name` string; `interval` pair; `from` string; `to` string |
| design (`$.design`) | `policy` = `"luriaNative"`; `style` = `"q5_polymerase"`; `minAnnealingOverlap` integer; `placement` one of `"inline"`, `"confine_to_tail"`; `buffer`: `"q5_reaction"` or a buffer object |
| buffer object (`$.design.buffer`) | `monovalentMillimolar` number; `primerNanomolar` number; `dmsoPercent` number |

Types:
- **integer:** a JSON number that `serde_json::Number::as_i64` accepts. A
  fraction, an exponent form, or a value outside `i64` is not one.
- **number:** any JSON number, read with `as_f64`.
- **pair:** an array of exactly two integers, read as `(start, end)`.
- **sha256:** a string of exactly 64 characters, each `0-9` or `a-f`.

### 3.5 Checking order and messages

For each object:
1. if the value is not an object → `expected an object`, at the value's
   path;
2. members in document order: an unknown key → `unknown field 'K'`, at the
   member's path (`$.template.K`). One exception: `topology` in a template
   that has `genbank` → `'topology' is not allowed with 'genbank': a GenBank
   record states its topology on its LOCUS line`, at `$.template.topology`.
   Which keys are allowed in a template depends on whether it has `fasta`
   or `genbank`. A template with both or neither is reported by step 3, not
   here, so check the `fasta`/`genbank` group first in that object only;
3. fields in schema order, each checked completely, including everything
   nested in it, before the next:
   - a required field that is absent → `missing required field 'F'`, at the
     object's path;
   - a `one of` group with none present → `exactly one of 'A', 'B' is
     required`, and with more than one → `only one of 'A', 'B' may be
     given`, at the object's path. The names are quoted and listed in
     schema order, joined by `, `.

Value messages, at the value's own path:

| Problem | Message |
| --- | --- |
| not a string | `expected a string` |
| not an integer | `expected an integer` |
| not a number | `expected a number` |
| not an array | `expected an array` |
| not a pair (not an array, or not two elements) | `expected [start, end]` |
| a pair element that is not an integer | `expected an integer` (path `...[0]` or `...[1]`) |
| a fixed value differs | `expected "V"` |
| a value outside a closed set | `expected one of "A", "B"` (in the table's order) |
| a bad sha256 | `expected 64 lower-case hexadecimal digits` |
| a bad buffer | `expected "q5_reaction" or an object with monovalentMillimolar, primerNanomolar and dmsoPercent` (when the value is neither a string equal to `q5_reaction` nor an object) |

A JSON syntax error, a duplicate key, or trailing text → path `$`, message
`invalid JSON: ` followed by `parse_json`'s error.

The first problem found is the one reported.

Write module doc comments for both files summarizing §2 and §3.4–§3.5.

## 4. Tests (planner-written; add verbatim)

### 4.1 `json.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn members_keep_their_document_order() {
        let parsed = parse_json(r#"{"b": 1, "a": [true, null, "x", 2.5]}"#).unwrap();
        match parsed {
            Json::Object(members) => {
                let keys: Vec<&str> = members.iter().map(|(key, _)| key.as_str()).collect();
                assert_eq!(keys, vec!["b", "a"]);
                assert_eq!(
                    members[1].1,
                    Json::Array(vec![
                        Json::Bool(true),
                        Json::Null,
                        Json::String("x".to_string()),
                        Json::Number(serde_json::Number::from_f64(2.5).unwrap()),
                    ])
                );
            }
            other => panic!("expected an object, got {:?}", other),
        }
    }

    #[test]
    fn a_duplicate_key_is_refused_with_its_place() {
        let error = parse_json("{\"a\": 1,\n \"a\": 2}").unwrap_err();
        assert!(error.starts_with("duplicate key 'a'"), "{}", error);
        assert!(error.contains("line 2"), "{}", error);
        let nested = parse_json(r#"{"x": {"k": 1, "k": 1}}"#).unwrap_err();
        assert!(nested.starts_with("duplicate key 'k'"), "{}", nested);
    }

    #[test]
    fn trailing_text_and_bad_syntax_are_refused() {
        assert!(parse_json("{} {}").unwrap_err().starts_with("trailing characters"));
        assert!(parse_json("").is_err());
        assert!(parse_json("{\"a\": 1,}").is_err());
    }
}
```

### 4.2 `request.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn valid() -> Value {
        json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": {
                "id": "sid-1",
                "fasta": "sid-1_cDNA.fa",
                "topology": "linear",
                "cds": [{
                    "id": "sid-1",
                    "intervals": [[0, 3153]],
                    "strand": "+",
                    "completeness": "complete",
                    "geneticCode": "standard",
                    "numbering": "from-cds-start"
                }]
            },
            "scope": "sid-1",
            "edit": {
                "name": "D404N",
                "residue": { "cds": "sid-1", "number": 404, "from": "D", "to": "N", "codon": "AAC" }
            },
            "design": {
                "policy": "luriaNative",
                "style": "q5_polymerase",
                "minAnnealingOverlap": 15,
                "placement": "inline",
                "buffer": "q5_reaction"
            }
        })
    }

    fn genbank_variants() -> Value {
        json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": {
                "id": "pbr322",
                "genbank": "J01749.1.gb",
                "sha256": "536042a7485bdabdf49c4d21a306213a098d77d19bdba57a1c6adc76ce2b8372",
                "cds": ["tet", "bla"]
            },
            "variants": {
                "composition": "separate_variants",
                "members": [
                    { "name": "s1", "substitution": { "interval": [99, 100], "from": "A", "to": "G" } },
                    { "name": "d1", "deletion": { "interval": [99, 102], "from": "ACG" } },
                    { "name": "i1", "insertion": { "point": 100, "before": "A", "after": "C", "insert": "GGG" } },
                    { "name": "r1", "replacement": { "interval": [99, 102], "from": "ACG", "to": "TT" } },
                    { "name": "x1", "residue": { "number": 42, "from": "A", "to": "G", "policy": "default" } }
                ]
            },
            "design": {
                "policy": "luriaNative",
                "style": "q5_polymerase",
                "minAnnealingOverlap": 15,
                "placement": "confine_to_tail",
                "buffer": { "monovalentMillimolar": 50, "primerNanomolar": 200.5, "dmsoPercent": 0 }
            }
        })
    }

    fn parse(value: &Value) -> Result<RequestDocument, RequestError> {
        parse_request(&value.to_string())
    }

    fn refused(value: &Value) -> (String, String) {
        match parse(value) {
            Err(error) => {
                assert_eq!(error.code, RequestErrorCode::Invalid);
                (error.path, error.message)
            }
            Ok(document) => panic!("accepted: {:?}", document),
        }
    }

    fn set(value: &mut Value, pointer: &str, new: Value) {
        *value.pointer_mut(pointer).unwrap() = new;
    }

    fn remove(value: &mut Value, pointer: &str, key: &str) {
        value.pointer_mut(pointer).unwrap().as_object_mut().unwrap().remove(key);
    }

    fn insert(value: &mut Value, pointer: &str, key: &str, new: Value) {
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(key.to_string(), new);
    }

    fn s(text: &str) -> String {
        text.to_string()
    }

    fn pair(path: &str, message: &str) -> (String, String) {
        (path.to_string(), message.to_string())
    }

    #[test]
    fn the_sid1_request_parses_to_its_document() {
        let expected = RequestDocument {
            template: TemplateSpec::Fasta {
                id: s("sid-1"),
                path: s("sid-1_cDNA.fa"),
                topology: TopologySpec::Linear,
                sha256: None,
                cds: vec![CdsSpec {
                    id: s("sid-1"),
                    intervals: vec![(0, 3153)],
                    strand: StrandSpec::Plus,
                    completeness: s("complete"),
                    genetic_code: s("standard"),
                    numbering: s("from-cds-start"),
                }],
            },
            scope: Some(s("sid-1")),
            operation: OperationSpec::Edit(MemberSpec {
                name: s("D404N"),
                edit: EditSpec::Residue {
                    cds: Some(s("sid-1")),
                    number: 404,
                    from: s("D"),
                    to: s("N"),
                    codon: CodonSpec::Supplied(s("AAC")),
                },
            }),
            design: DesignSpec {
                policy: PolicySpec::LuriaNative,
                style: StyleSpec::Q5Polymerase,
                min_annealing_overlap: 15,
                placement: PlacementSpec::Inline,
                buffer: BufferSpec::Q5Reaction,
            },
        };
        assert_eq!(parse(&valid()).unwrap(), expected);
    }

    #[test]
    fn a_genbank_variant_set_parses_every_edit_kind() {
        let expected = RequestDocument {
            template: TemplateSpec::GenBank {
                id: s("pbr322"),
                path: s("J01749.1.gb"),
                sha256: Some(s(
                    "536042a7485bdabdf49c4d21a306213a098d77d19bdba57a1c6adc76ce2b8372",
                )),
                cds: vec![s("tet"), s("bla")],
            },
            scope: None,
            operation: OperationSpec::Variants {
                composition: CompositionSpec::SeparateVariants,
                members: vec![
                    MemberSpec {
                        name: s("s1"),
                        edit: EditSpec::Substitution { interval: (99, 100), from: s("A"), to: s("G") },
                    },
                    MemberSpec {
                        name: s("d1"),
                        edit: EditSpec::Deletion { interval: (99, 102), from: s("ACG") },
                    },
                    MemberSpec {
                        name: s("i1"),
                        edit: EditSpec::Insertion {
                            point: 100,
                            before: s("A"),
                            after: s("C"),
                            insert: s("GGG"),
                        },
                    },
                    MemberSpec {
                        name: s("r1"),
                        edit: EditSpec::Replacement { interval: (99, 102), from: s("ACG"), to: s("TT") },
                    },
                    MemberSpec {
                        name: s("x1"),
                        edit: EditSpec::Residue {
                            cds: None,
                            number: 42,
                            from: s("A"),
                            to: s("G"),
                            codon: CodonSpec::Default,
                        },
                    },
                ],
            },
            design: DesignSpec {
                policy: PolicySpec::LuriaNative,
                style: StyleSpec::Q5Polymerase,
                min_annealing_overlap: 15,
                placement: PlacementSpec::ConfineToTail,
                buffer: BufferSpec::Values {
                    monovalent_millimolar: 50.0,
                    primer_nanomolar: 200.5,
                    dmso_percent: 0.0,
                },
            },
        };
        assert_eq!(parse(&genbank_variants()).unwrap(), expected);
    }

    #[test]
    fn a_degenerate_replacement_and_a_combined_construct_parse() {
        let mut value = valid();
        remove(&mut value, "", "edit");
        insert(
            &mut value,
            "",
            "degenerate",
            json!({ "name": "lib", "interval": [9, 12], "from": "GAT", "to": "NNK" }),
        );
        assert_eq!(
            parse(&value).unwrap().operation,
            OperationSpec::Degenerate { name: s("lib"), interval: (9, 12), from: s("GAT"), to: s("NNK") }
        );
        let mut combined = genbank_variants();
        set(&mut combined, "/variants/composition", json!("combined_construct"));
        match parse(&combined).unwrap().operation {
            OperationSpec::Variants { composition, .. } => {
                assert_eq!(composition, CompositionSpec::CombinedConstruct)
            }
            other => panic!("expected variants, got {:?}", other),
        }
    }

    #[test]
    fn the_top_level_is_strict() {
        let mut wrong_schema = valid();
        set(&mut wrong_schema, "/schema", json!("luria.q5-request/2"));
        assert_eq!(refused(&wrong_schema), pair("$.schema", "expected \"luria.q5-request/1\""));

        let mut coordinates = valid();
        set(&mut coordinates, "/coordinates", json!("one-based"));
        assert_eq!(
            refused(&coordinates),
            pair("$.coordinates", "expected \"zero-based-half-open\"")
        );

        let mut missing = valid();
        remove(&mut missing, "", "design");
        assert_eq!(refused(&missing), pair("$", "missing required field 'design'"));

        let mut unknown = valid();
        insert(&mut unknown, "", "extra", json!(1));
        assert_eq!(refused(&unknown), pair("$.extra", "unknown field 'extra'"));

        let mut none = valid();
        remove(&mut none, "", "edit");
        assert_eq!(
            refused(&none),
            pair("$", "exactly one of 'edit', 'variants', 'degenerate' is required")
        );

        let mut two = valid();
        insert(
            &mut two,
            "",
            "degenerate",
            json!({ "name": "lib", "interval": [9, 12], "from": "GAT", "to": "NNK" }),
        );
        assert_eq!(
            refused(&two),
            pair("$", "only one of 'edit', 'variants', 'degenerate' may be given")
        );

        let mut scope = valid();
        set(&mut scope, "/scope", json!(5));
        assert_eq!(refused(&scope), pair("$.scope", "expected a string"));

        let mut null_scope = valid();
        set(&mut null_scope, "/scope", Value::Null);
        assert_eq!(refused(&null_scope), pair("$.scope", "expected a string"));

        assert_eq!(refused(&json!([1, 2])), pair("$", "expected an object"));
    }

    #[test]
    fn the_template_is_one_form_with_its_own_fields() {
        let mut both = valid();
        insert(&mut both, "/template", "genbank", json!("x.gb"));
        assert_eq!(refused(&both), pair("$.template", "only one of 'fasta', 'genbank' may be given"));

        let mut neither = valid();
        remove(&mut neither, "/template", "fasta");
        assert_eq!(
            refused(&neither),
            pair("$.template", "exactly one of 'fasta', 'genbank' is required")
        );

        let mut no_topology = valid();
        remove(&mut no_topology, "/template", "topology");
        assert_eq!(refused(&no_topology), pair("$.template", "missing required field 'topology'"));

        let mut genbank_topology = genbank_variants();
        insert(&mut genbank_topology, "/template", "topology", json!("circular"));
        assert_eq!(
            refused(&genbank_topology),
            pair(
                "$.template.topology",
                "'topology' is not allowed with 'genbank': a GenBank record states its topology on its LOCUS line"
            )
        );

        let mut capital = valid();
        set(&mut capital, "/template/topology", json!("Linear"));
        assert_eq!(
            refused(&capital),
            pair("$.template.topology", "expected one of \"linear\", \"circular\"")
        );

        let mut digest = valid();
        insert(&mut digest, "/template", "sha256", json!("ABC"));
        assert_eq!(
            refused(&digest),
            pair("$.template.sha256", "expected 64 lower-case hexadecimal digits")
        );

        let mut upper_digest = genbank_variants();
        set(
            &mut upper_digest,
            "/template/sha256",
            json!("536042A7485BDABDF49C4D21A306213A098D77D19BDBA57A1C6ADC76CE2B8372"),
        );
        assert_eq!(
            refused(&upper_digest),
            pair("$.template.sha256", "expected 64 lower-case hexadecimal digits")
        );

        let mut genbank_object = genbank_variants();
        set(&mut genbank_object, "/template/cds/0", json!({ "id": "tet" }));
        assert_eq!(refused(&genbank_object), pair("$.template.cds[0]", "expected a string"));

        let mut no_cds = valid();
        remove(&mut no_cds, "/template", "cds");
        assert_eq!(refused(&no_cds), pair("$.template", "missing required field 'cds'"));
    }

    #[test]
    fn a_declared_cds_is_checked_field_by_field() {
        let mut short = valid();
        set(&mut short, "/template/cds/0/intervals/0", json!([0]));
        assert_eq!(refused(&short), pair("$.template.cds[0].intervals[0]", "expected [start, end]"));

        let mut fraction = valid();
        set(&mut fraction, "/template/cds/0/intervals/0", json!([0, 3153.0]));
        assert_eq!(
            refused(&fraction),
            pair("$.template.cds[0].intervals[0][1]", "expected an integer")
        );

        let mut strand = valid();
        set(&mut strand, "/template/cds/0/strand", json!("plus"));
        assert_eq!(
            refused(&strand),
            pair("$.template.cds[0].strand", "expected one of \"+\", \"-\"")
        );

        let mut unknown = valid();
        insert(&mut unknown, "/template/cds/0", "name", json!("x"));
        assert_eq!(refused(&unknown), pair("$.template.cds[0].name", "unknown field 'name'"));

        let mut missing = valid();
        remove(&mut missing, "/template/cds/0", "geneticCode");
        assert_eq!(
            refused(&missing),
            pair("$.template.cds[0]", "missing required field 'geneticCode'")
        );
    }

    #[test]
    fn each_edit_is_checked_where_it_sits() {
        let mut both = valid();
        insert(&mut both, "/edit/residue", "policy", json!("default"));
        assert_eq!(refused(&both), pair("$.edit.residue", "only one of 'codon', 'policy' may be given"));

        let mut neither = valid();
        remove(&mut neither, "/edit/residue", "codon");
        assert_eq!(
            refused(&neither),
            pair("$.edit.residue", "exactly one of 'codon', 'policy' is required")
        );

        let mut policy = valid();
        remove(&mut policy, "/edit/residue", "codon");
        insert(&mut policy, "/edit/residue", "policy", json!("minimal"));
        assert_eq!(refused(&policy), pair("$.edit.residue.policy", "expected \"default\""));

        let mut two_kinds = valid();
        insert(
            &mut two_kinds,
            "/edit",
            "deletion",
            json!({ "interval": [1, 2], "from": "A" }),
        );
        assert_eq!(
            refused(&two_kinds),
            pair(
                "$.edit",
                "only one of 'residue', 'substitution', 'deletion', 'insertion', 'replacement' may be given"
            )
        );

        let mut nameless = valid();
        remove(&mut nameless, "/edit", "name");
        assert_eq!(refused(&nameless), pair("$.edit", "missing required field 'name'"));

        let mut text_number = valid();
        set(&mut text_number, "/edit/residue/number", json!("404"));
        assert_eq!(refused(&text_number), pair("$.edit.residue.number", "expected an integer"));

        let mut huge = valid();
        set(&mut huge, "/edit/residue/number", json!(18446744073709551615u64));
        assert_eq!(refused(&huge), pair("$.edit.residue.number", "expected an integer"));

        let mut member = genbank_variants();
        set(&mut member, "/variants/members/1/deletion/interval", json!([3]));
        assert_eq!(
            refused(&member),
            pair("$.variants.members[1].deletion.interval", "expected [start, end]")
        );

        let mut insertion = genbank_variants();
        remove(&mut insertion, "/variants/members/2/insertion", "after");
        assert_eq!(
            refused(&insertion),
            pair("$.variants.members[2].insertion", "missing required field 'after'")
        );

        let mut members = genbank_variants();
        set(&mut members, "/variants/members", json!({}));
        assert_eq!(refused(&members), pair("$.variants.members", "expected an array"));

        let mut composition = genbank_variants();
        set(&mut composition, "/variants/composition", json!("together"));
        assert_eq!(
            refused(&composition),
            pair(
                "$.variants.composition",
                "expected one of \"separate_variants\", \"combined_construct\""
            )
        );
    }

    #[test]
    fn the_design_names_every_choice() {
        let mut preset = valid();
        set(&mut preset, "/design/buffer", json!("q5"));
        assert_eq!(
            refused(&preset),
            pair(
                "$.design.buffer",
                "expected \"q5_reaction\" or an object with monovalentMillimolar, primerNanomolar and dmsoPercent"
            )
        );

        let mut partial = genbank_variants();
        remove(&mut partial, "/design/buffer", "dmsoPercent");
        assert_eq!(refused(&partial), pair("$.design.buffer", "missing required field 'dmsoPercent'"));

        let mut text_salt = genbank_variants();
        set(&mut text_salt, "/design/buffer/monovalentMillimolar", json!("150"));
        assert_eq!(
            refused(&text_salt),
            pair("$.design.buffer.monovalentMillimolar", "expected a number")
        );

        let mut policy = valid();
        set(&mut policy, "/design/policy", json!("nebasechanger"));
        assert_eq!(refused(&policy), pair("$.design.policy", "expected \"luriaNative\""));

        let mut placement = valid();
        set(&mut placement, "/design/placement", json!("tail"));
        assert_eq!(
            refused(&placement),
            pair("$.design.placement", "expected one of \"inline\", \"confine_to_tail\"")
        );

        let mut overlap = valid();
        set(&mut overlap, "/design/minAnnealingOverlap", json!(1.5));
        assert_eq!(refused(&overlap), pair("$.design.minAnnealingOverlap", "expected an integer"));

        let mut missing = valid();
        remove(&mut missing, "/design", "style");
        assert_eq!(refused(&missing), pair("$.design", "missing required field 'style'"));
    }

    #[test]
    fn bad_json_is_refused_at_the_root() {
        for text in ["", "{\"schema\": 1,}", "{} {}", "{\"a\": 1, \"a\": 2}"] {
            match parse_request(text) {
                Err(error) => {
                    assert_eq!(error.code, RequestErrorCode::Invalid, "{:?}", text);
                    assert_eq!(error.path, "$", "{:?}", text);
                    assert!(error.message.starts_with("invalid JSON: "), "{:?}", error.message);
                }
                Ok(document) => panic!("accepted {:?}: {:?}", text, document),
            }
        }
    }

    #[test]
    fn every_request_error_code_is_pinned() {
        assert_eq!(RequestErrorCode::Unreadable.code(), "request_unreadable");
        assert_eq!(RequestErrorCode::Invalid.code(), "request_invalid");
        assert_eq!(RequestErrorCode::TemplateUnreadable.code(), "template_unreadable");
        assert_eq!(RequestErrorCode::TemplateDigestMismatch.code(), "template_digest_mismatch");
    }
}
```

`cargo fmt` may reflow; do not change content. If a test fails, stop and
report the test, the expected `(path, message)` and what the parser
returned. Do not change the test.

`serde_json::json!` and `Value` appear only in tests. 5d's rule against them
covers output documents, and the parser itself must use `Json` (§3.1).

### 4.3 Proving the rules are load-bearing (AGENTS.md)

Disable each rule once, confirm the named test fails, and restore:

| Rule | Test that must fail |
| --- | --- |
| duplicate keys are refused | `a_duplicate_key_is_refused_with_its_place` |
| trailing text is refused (`end()`) | `trailing_text_and_bad_syntax_are_refused` |
| unknown fields are refused | `the_top_level_is_strict` |
| `one of` with two present is refused | `the_top_level_is_strict` |
| an integer must fit `as_i64` | `each_edit_is_checked_where_it_sits` |
| `topology` with `genbank` is refused | `the_template_is_one_form_with_its_own_fields` |
| sha256 is lower-case hex | `the_template_is_one_form_with_its_own_fields` |
| `null` is not an absent field | `the_top_level_is_strict` |

## 5. Do not

- Edit files other than `crates/cli/src/json.rs`, `crates/cli/src/request.rs`
  and `crates/cli/src/lib.rs`.
- Add dependencies, read files, or call any `luria-core` function.
- Use `#[derive(Deserialize)]` on the document types, or
  `serde_json::Value`, outside tests. The path-located checks walk `Json`.
- Accept anything §3.4 does not list, or default any field.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the changed and new files, the exact commands and results, the §4.3
results, and anything that did not match this contract.
