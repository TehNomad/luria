//! The plain model of a Q5 request document and the one function that reads
//! JSON into it (milestone 5e1 §3.3-§3.5). 5e1 checks only the document's
//! shape: which fields exist, their JSON types and the closed sets of
//! keywords. Values core validates - identifiers, residue letters,
//! coordinates, completeness, genetic code and numbering - carry biological
//! meaning, so they are kept as given and core reports them (§1, §2). The
//! same goes for `template.sha256`: here it is only checked as a digest, and
//! 5e2 compares it with the sequence's content digest (§2).
//!
//! # The rules this module enforces
//!
//! - one JSON document per request: a JSON syntax error, a duplicate key or
//!   trailing text is reported at `$` as `invalid JSON: ` plus serde_json's
//!   message (§3.5);
//! - for each object, in order: a non-object gives `expected an object` at
//!   the value's path; then members in document order give `unknown field
//!   'K'` at the member's path; then the fields in schema order, each checked
//!   completely including everything nested in it (§3.5);
//! - a template resolves its `fasta`/`genbank` group before that scan, so the
//!   allowed keys are known; both or neither is a group error at the
//!   template's path, and `topology` with `genbank` names the LOCUS rule
//!   (§3.5);
//! - a required field that is absent gives `missing required field 'F'` at
//!   the object's path; a `one of` group with none present gives `exactly one
//!   of 'A', 'B' is required` and with more than one `only one of 'A', 'B'
//!   may be given`, the names quoted and in schema order (§3.5);
//! - `null` is a value of its own, never an absent field: it is refused where
//!   a string, integer or number is expected (§2);
//! - an integer must fit `as_i64`, so a fraction, an exponent form or a value
//!   outside `i64` is refused; a number is read with `as_f64`; a pair is
//!   exactly two integers; and a sha256 is 64 lower-case hexadecimal digits
//!   (§3.4);
//! - a value outside a closed set gives `expected one of "A", "B"` in the
//!   table's order, and a fixed value that differs gives `expected "V"`
//!   (§3.5).
//!
//! The first problem found is the one reported, and every problem 5e1 finds
//! is `request_invalid` (§3.2).

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
    /// A file `--write-edited` would write already exists (5g2). Not produced
    /// by `parse_request`: it is here so the CLI's own codes are one list
    /// (plan §3.6).
    OutputExists,
    /// A result name cannot be a file name: it holds a path separator, so the
    /// file would sit outside the output directory (5g2 §4.1). Not produced
    /// by `parse_request`: it is here so the CLI's own codes are one list
    /// (plan §3.6).
    OutputNameUnusable,
}

impl RequestErrorCode {
    /// `request_unreadable`, `request_invalid`, `template_unreadable`,
    /// `template_digest_mismatch`, `output_exists`, `output_name_unusable`.
    pub fn code(self) -> &'static str {
        match self {
            RequestErrorCode::Unreadable => "request_unreadable",
            RequestErrorCode::Invalid => "request_invalid",
            RequestErrorCode::TemplateUnreadable => "template_unreadable",
            RequestErrorCode::TemplateDigestMismatch => "template_digest_mismatch",
            RequestErrorCode::OutputExists => "output_exists",
            RequestErrorCode::OutputNameUnusable => "output_name_unusable",
        }
    }
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

/// The request document as the schema describes it: one template, the
/// operation to carry out and one design.
#[derive(Clone, Debug, PartialEq)]
pub struct RequestDocument {
    pub template: TemplateSpec,
    pub scope: Option<String>,
    pub operation: OperationSpec,
    pub design: DesignSpec,
}

/// The template, in one of its two forms: a FASTA sequence named by a path
/// and stated topology, or a GenBank record.
#[derive(Clone, Debug, PartialEq)]
pub enum TemplateSpec {
    Fasta {
        id: String,
        path: String,
        topology: TopologySpec,
        sha256: Option<String>,
        cds: Vec<CdsSpec>,
    },
    GenBank {
        id: String,
        path: String,
        sha256: Option<String>,
        cds: Vec<String>,
    },
}

/// A FASTA sequence's topology, which a GenBank record states on its LOCUS
/// line instead.
#[derive(Clone, Debug, PartialEq)]
pub enum TopologySpec {
    Linear,
    Circular,
}

/// One declared CDS of a FASTA template.
#[derive(Clone, Debug, PartialEq)]
pub struct CdsSpec {
    pub id: String,
    pub intervals: Vec<(i64, i64)>,
    pub strand: StrandSpec,
    pub completeness: String,
    pub genetic_code: String,
    pub numbering: String,
}

/// Which side of the sequence a CDS runs on.
#[derive(Clone, Debug, PartialEq)]
pub enum StrandSpec {
    Plus,
    Minus,
}

/// The operation the request asks for: one edit, a set of variants, or a
/// degenerate replacement.
#[derive(Clone, Debug, PartialEq)]
pub enum OperationSpec {
    Edit(MemberSpec),
    Variants {
        composition: CompositionSpec,
        members: Vec<MemberSpec>,
    },
    Degenerate {
        name: String,
        interval: (i64, i64),
        from: String,
        to: String,
    },
}

/// Whether variants are built one at a time or as one construct.
#[derive(Clone, Debug, PartialEq)]
pub enum CompositionSpec {
    SeparateVariants,
    CombinedConstruct,
}

/// One named edit inside a variant set.
#[derive(Clone, Debug, PartialEq)]
pub struct MemberSpec {
    pub name: String,
    pub edit: EditSpec,
}

/// One edit: a residue letter, a substitution, a deletion, an insertion, or
/// a replacement.
#[derive(Clone, Debug, PartialEq)]
pub enum EditSpec {
    Residue {
        cds: Option<String>,
        number: i64,
        from: String,
        to: String,
        codon: CodonSpec,
    },
    Substitution {
        interval: (i64, i64),
        from: String,
        to: String,
    },
    Deletion {
        interval: (i64, i64),
        from: String,
    },
    Insertion {
        point: i64,
        before: String,
        after: String,
        insert: String,
    },
    Replacement {
        interval: (i64, i64),
        from: String,
        to: String,
    },
}

/// A residue letter's codon, or the default one.
#[derive(Clone, Debug, PartialEq)]
pub enum CodonSpec {
    Supplied(String),
    Default,
}

/// The design to run: which planner, which style, how much annealing a
/// primer must share, where placements may sit and the buffer to leave.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignSpec {
    pub policy: PolicySpec,
    pub style: StyleSpec,
    pub min_annealing_overlap: i64,
    pub placement: PlacementSpec,
    pub buffer: BufferSpec,
}

/// The planner that runs the design.
#[derive(Clone, Debug, PartialEq)]
pub enum PolicySpec {
    LuriaNative,
}

/// The style the planner designs in.
#[derive(Clone, Debug, PartialEq)]
pub enum StyleSpec {
    Q5Polymerase,
}

/// Where a placement may sit.
#[derive(Clone, Debug, PartialEq)]
pub enum PlacementSpec {
    Inline,
    ConfineToTail,
}

/// The annealing buffer: Q5's own reaction rates, or the values to use.
#[derive(Clone, Debug, PartialEq)]
pub enum BufferSpec {
    Q5Reaction,
    Values {
        monovalent_millimolar: f64,
        primer_nanomolar: f64,
        dmso_percent: f64,
    },
}

/// One step of a JSON path: an object member or an array element.
#[derive(Clone, Debug)]
enum Token {
    MemberKey(String),
    ElementIndex(usize),
}

/// The path taken so far, each step with the JSON value it names.
type Tokens = Vec<(Token, crate::json::Json)>;

/// The JSON path of a problem: `$`, then `.key` per member and `[i]` per
/// element.
fn path_of(steps: Tokens) -> String {
    let suffix: String = steps
        .iter()
        .map(|step| match step {
            (Token::MemberKey(key), _) => {
                format!(".{}", key)
            }
            (Token::ElementIndex(index), _) => {
                format!("[{}]", index)
            }
        })
        .collect::<Vec<String>>()
        .join("");
    format!("${}", suffix)
}

/// The path of one more member, for an error about an unknown field.
fn member_path(steps: Tokens, key: String) -> String {
    let mut longer: Tokens = steps.clone();
    longer.push((Token::MemberKey(key), crate::json::Json::Null));
    path_of(longer)
}

/// The value of a member by name, or `null` when the object has no such
/// member.
fn member_value(members: Vec<(String, crate::json::Json)>, key: String) -> crate::json::Json {
    for (name, value) in members {
        if name == key {
            return value.clone();
        }
    }
    crate::json::Json::Null
}

/// The message for a value that is not one of a closed set, in the table's
/// order (§3.5).
fn one_of(names: Vec<String>) -> String {
    let listed: String = names
        .iter()
        .map(|name| format!("\"{}\"", name))
        .collect::<Vec<String>>()
        .join(", ");
    format!("expected one of {}", listed)
}

/// The message for a value that must be one fixed string (§3.5).
fn expected(value: String) -> String {
    format!("expected \"{}\"", value)
}

/// An object's members, in document order.
type Members = Vec<(String, crate::json::Json)>;

/// The path of one more member.
fn step_key(steps: Tokens, key: String) -> Tokens {
    let mut longer: Tokens = steps.clone();
    longer.push((Token::MemberKey(key), crate::json::Json::Null));
    longer
}

/// The path of one more array element.
fn step_index(steps: Tokens, index: usize) -> Tokens {
    let mut longer: Tokens = steps.clone();
    longer.push((Token::ElementIndex(index), crate::json::Json::Null));
    longer
}

/// The members of an object value, refusing anything else (§3.5).
fn want_object(steps: Tokens, value: crate::json::Json) -> Result<Members, RequestError> {
    match value {
        crate::json::Json::Object(members) => Ok(members),
        _ => Err(RequestError {
            code: RequestErrorCode::Invalid,
            path: path_of(steps),
            message: "expected an object".to_string(),
        }),
    }
}

/// The elements of an array value, refusing anything else (§3.5).
fn want_array(
    steps: Tokens,
    value: crate::json::Json,
) -> Result<Vec<crate::json::Json>, RequestError> {
    match value {
        crate::json::Json::Array(items) => Ok(items),
        _ => Err(RequestError {
            code: RequestErrorCode::Invalid,
            path: path_of(steps),
            message: "expected an array".to_string(),
        }),
    }
}

/// A string value, refusing anything else (§3.5).
fn want_string(steps: Tokens, value: crate::json::Json) -> Result<String, RequestError> {
    match value {
        crate::json::Json::String(text) => Ok(text),
        _ => Err(RequestError {
            code: RequestErrorCode::Invalid,
            path: path_of(steps),
            message: "expected a string".to_string(),
        }),
    }
}

/// An integer value: a JSON number that fits `as_i64` (§3.4).
fn want_integer(steps: Tokens, value: crate::json::Json) -> Result<i64, RequestError> {
    match value {
        crate::json::Json::Number(number) => match number.as_i64() {
            Some(found) => Ok(found),
            None => Err(RequestError {
                code: RequestErrorCode::Invalid,
                path: path_of(steps),
                message: "expected an integer".to_string(),
            }),
        },
        _ => Err(RequestError {
            code: RequestErrorCode::Invalid,
            path: path_of(steps),
            message: "expected an integer".to_string(),
        }),
    }
}

/// A number value: any JSON number (§3.4).
fn want_number(steps: Tokens, value: crate::json::Json) -> Result<f64, RequestError> {
    match value {
        crate::json::Json::Number(number) => match number.as_f64() {
            Some(found) => Ok(found),
            None => Err(RequestError {
                code: RequestErrorCode::Invalid,
                path: path_of(steps),
                message: "expected a number".to_string(),
            }),
        },
        _ => Err(RequestError {
            code: RequestErrorCode::Invalid,
            path: path_of(steps),
            message: "expected a number".to_string(),
        }),
    }
}

/// A string value that is one of a closed set, refusing the rest with the
/// set's own message, in table order (§3.5).
fn want_one_of(
    steps: Tokens,
    value: crate::json::Json,
    names: Vec<String>,
) -> Result<String, RequestError> {
    match want_string(steps.clone(), value) {
        Ok(text) => {
            if names.iter().any(|name| name == text.as_str()) {
                Ok(text)
            } else {
                Err(RequestError {
                    code: RequestErrorCode::Invalid,
                    path: path_of(steps),
                    message: one_of(names),
                })
            }
        }
        Err(error) => Err(error),
    }
}

/// A string value that is one fixed string (§3.5).
fn want_fixed(steps: Tokens, value: crate::json::Json, text: String) -> Result<(), RequestError> {
    match want_string(steps.clone(), value) {
        Ok(given) => {
            if given == text {
                Ok(())
            } else {
                Err(RequestError {
                    code: RequestErrorCode::Invalid,
                    path: path_of(steps),
                    message: expected(text),
                })
            }
        }
        Err(error) => Err(error),
    }
}

/// A pair: an array of exactly two integers, each with its own path (§3.4).
fn want_pair(steps: Tokens, value: crate::json::Json) -> Result<(i64, i64), RequestError> {
    let items = want_array(steps.clone(), value)?;
    if items.len() != 2 {
        return Err(RequestError {
            code: RequestErrorCode::Invalid,
            path: path_of(steps),
            message: "expected [start, end]".to_string(),
        });
    }
    let start = want_integer(step_index(steps.clone(), 0), items[0].clone())?;
    let finish = want_integer(step_index(steps, 1), items[1].clone())?;
    Ok((start, finish))
}

/// A `sha256` value: exactly 64 lower-case hexadecimal digits (§3.4).
fn want_digest(steps: Tokens, value: crate::json::Json) -> Result<String, RequestError> {
    let text = want_string(steps.clone(), value)?;
    if text.len() == 64 && text.chars().all(is_hex_digit) {
        Ok(text)
    } else {
        Err(RequestError {
            code: RequestErrorCode::Invalid,
            path: path_of(steps),
            message: "expected 64 lower-case hexadecimal digits".to_string(),
        })
    }
}

/// A character of a `sha256` digest (§3.4).
fn is_hex_digit(character: char) -> bool {
    character.is_ascii_digit() || ('a'..='f').contains(&character)
}

/// Whether an object holds a member of that name.
fn has_field(members: Vec<(String, crate::json::Json)>, key: String) -> bool {
    for member in members.clone() {
        let (name, _) = member;
        if name == key {
            return true;
        }
    }
    false
}

/// A member an object must hold, refused at the object's path when it is
/// absent (§3.5).
fn require(
    steps: Tokens,
    members: Vec<(String, crate::json::Json)>,
    field: String,
) -> Result<crate::json::Json, RequestError> {
    if !has_field(members.clone(), field.clone()) {
        return Err(RequestError {
            code: RequestErrorCode::Invalid,
            path: path_of(steps.clone()),
            message: format!("missing required field '{}'", field),
        });
    }
    Ok(member_value(members.clone(), field.clone()))
}

/// The value of a member that may be absent, or `null` when it is not there.
fn optional(members: Vec<(String, crate::json::Json)>, field: String) -> crate::json::Json {
    member_value(members.clone(), field.clone())
}

/// The message when none of a one-of group is present (§3.5).
fn exactly_one(names: Vec<String>) -> String {
    let mut quoted: Vec<String> = [].to_vec();
    for name in names.clone() {
        quoted.push(format!("'{}'", name));
    }
    format!("exactly one of {} is required", quoted.join(", "))
}

/// The message when more than one member of a one-of group is present (§3.5).
fn only_one(names: Vec<String>) -> String {
    let mut quoted: Vec<String> = [].to_vec();
    for name in names.clone() {
        quoted.push(format!("'{}'", name));
    }
    format!("only one of {} may be given", quoted.join(", "))
}

/// The one member of a one-of group that is present (§3.5).
fn one_group(
    steps: Tokens,
    members: Vec<(String, crate::json::Json)>,
    names: Vec<String>,
) -> Result<String, RequestError> {
    let mut given: Vec<String> = [].to_vec();
    for name in names.clone() {
        if has_field(members.clone(), name.clone()) {
            given.push(name.clone());
        }
    }
    if given.len() == 1 {
        Ok(given[0].clone())
    } else if given.is_empty() {
        Err(RequestError {
            code: RequestErrorCode::Invalid,
            path: path_of(steps.clone()),
            message: exactly_one(names.clone()),
        })
    } else {
        Err(RequestError {
            code: RequestErrorCode::Invalid,
            path: path_of(steps.clone()),
            message: only_one(names.clone()),
        })
    }
}

/// Whether a name is one of a closed set.
fn has_key(names: Vec<String>, key: String) -> bool {
    for name in names.clone() {
        if name == key {
            return true;
        }
    }
    false
}

/// Any member an object holds that the schema does not name (§3.5).
fn check_unknown(
    steps: Tokens,
    members: Vec<(String, crate::json::Json)>,
    allowed: Vec<String>,
) -> Result<(), RequestError> {
    for member in members.clone() {
        let (key, _) = member;
        if !has_key(allowed.clone(), key.clone()) {
            return Err(RequestError {
                code: RequestErrorCode::Invalid,
                path: member_path(steps.clone(), key.clone()),
                message: format!("unknown field '{}'", key),
            });
        }
    }
    Ok(())
}

/// A required string field (§3.5).
fn field_string(
    steps: Tokens,
    members: Vec<(String, crate::json::Json)>,
    field: String,
) -> Result<String, RequestError> {
    let value = require(steps.clone(), members.clone(), field.clone())?;
    want_string(step_key(steps.clone(), field.clone()), value)
}

/// A required integer field (§3.4).
fn field_integer(
    steps: Tokens,
    members: Vec<(String, crate::json::Json)>,
    field: String,
) -> Result<i64, RequestError> {
    let value = require(steps.clone(), members.clone(), field.clone())?;
    want_integer(step_key(steps.clone(), field.clone()), value)
}

/// A required number field (§3.4).
fn field_number(
    steps: Tokens,
    members: Vec<(String, crate::json::Json)>,
    field: String,
) -> Result<f64, RequestError> {
    let value = require(steps.clone(), members.clone(), field.clone())?;
    want_number(step_key(steps.clone(), field.clone()), value)
}

/// A required pair field, its elements with their own paths (§3.4).
fn field_pair(
    steps: Tokens,
    members: Vec<(String, crate::json::Json)>,
    field: String,
) -> Result<(i64, i64), RequestError> {
    let value = require(steps.clone(), members.clone(), field.clone())?;
    want_pair(step_key(steps.clone(), field.clone()), value)
}

/// A required field whose value is one of a closed set (§3.5).
fn field_one_of(
    steps: Tokens,
    members: Vec<(String, crate::json::Json)>,
    field: String,
    names: Vec<String>,
) -> Result<String, RequestError> {
    let value = require(steps.clone(), members.clone(), field.clone())?;
    want_one_of(step_key(steps.clone(), field.clone()), value, names.clone())
}

/// A required field whose value must be one fixed string (§3.5).
fn field_fixed(
    steps: Tokens,
    members: Vec<(String, crate::json::Json)>,
    field: String,
    text: String,
) -> Result<(), RequestError> {
    let value = require(steps.clone(), members.clone(), field.clone())?;
    want_fixed(step_key(steps.clone(), field.clone()), value, text)
}

/// An optional string field: a value of its own is refused, an absent one is
/// `None` (§2, §3.5).
fn field_string_opt(
    steps: Tokens,
    members: Vec<(String, crate::json::Json)>,
    field: String,
) -> Result<Option<String>, RequestError> {
    if !has_field(members.clone(), field.clone()) {
        return Ok(None);
    }
    let value = optional(members.clone(), field.clone());
    let text = want_string(step_key(steps.clone(), field.clone()), value)?;
    Ok(Some(text))
}

/// An optional `sha256` field (§3.4).
fn field_digest_opt(
    steps: Tokens,
    members: Vec<(String, crate::json::Json)>,
    field: String,
) -> Result<Option<String>, RequestError> {
    if !has_field(members.clone(), field.clone()) {
        return Ok(None);
    }
    let value = optional(members.clone(), field.clone());
    let text = want_digest(step_key(steps.clone(), field.clone()), value)?;
    Ok(Some(text))
}

/// A required field that holds an array (§3.5).
fn field_array(
    steps: Tokens,
    members: Vec<(String, crate::json::Json)>,
    field: String,
) -> Result<Vec<crate::json::Json>, RequestError> {
    let value = require(steps.clone(), members.clone(), field.clone())?;
    want_array(step_key(steps.clone(), field.clone()), value)
}

/// A required field that holds an object, with its members and their path
/// (§3.5).
fn field_object(
    steps: Tokens,
    members: Vec<(String, crate::json::Json)>,
    field: String,
) -> Result<(Tokens, Vec<(String, crate::json::Json)>), RequestError> {
    let value = require(steps.clone(), members.clone(), field.clone())?;
    let deeper: Tokens = step_key(steps.clone(), field.clone());
    let nested: Vec<(String, crate::json::Json)> = want_object(deeper.clone(), value)?;
    Ok((deeper.clone(), nested))
}

/// A schema's field names and closed values, given as text, in the order the
/// schema lists them (§3.4).
fn names_of(words: Vec<&str>) -> Vec<String> {
    words
        .iter()
        .map(|word| word.to_string())
        .collect::<Vec<String>>()
}

/// A member a template's form does not allow: an unknown field, or `topology`
/// beside `genbank`, whose topology its LOCUS line states (§3.5).
fn check_template_members(
    steps: Tokens,
    members: Members,
    allowed: Vec<String>,
) -> Result<(), RequestError> {
    for member in members.clone() {
        let (key, _) = member;
        if has_key(allowed.clone(), key.clone()) {
            continue;
        }
        if key == "topology" {
            return Err(RequestError {
                code: RequestErrorCode::Invalid,
                path: member_path(steps.clone(), key.clone()),
                message: "'topology' is not allowed with 'genbank': a GenBank record states its topology on its LOCUS line".to_string(),
            });
        }
        return Err(RequestError {
            code: RequestErrorCode::Invalid,
            path: member_path(steps.clone(), key.clone()),
            message: format!("unknown field '{}'", key),
        });
    }
    Ok(())
}

/// The message for a value that is neither the preset buffer nor a buffer
/// object (§3.5).
fn buffer_expected(steps: Tokens) -> Result<BufferSpec, RequestError> {
    Err(RequestError {
        code: RequestErrorCode::Invalid,
        path: path_of(steps.clone()),
        message: "expected \"q5_reaction\" or an object with monovalentMillimolar, primerNanomolar and dmsoPercent".to_string(),
    })
}

/// The buffer: the preset `q5_reaction`, or an object of its three numbers
/// (§3.4, §3.5). It has no default: one of the two is required.
fn read_buffer(steps: Tokens, members: Members) -> Result<BufferSpec, RequestError> {
    let value: crate::json::Json = require(steps.clone(), members.clone(), "buffer".to_string())?;
    let deeper: Tokens = step_key(steps.clone(), "buffer".to_string());
    match value {
        crate::json::Json::String(text) => match text.as_str() {
            "q5_reaction" => Ok(BufferSpec::Q5Reaction),
            _ => buffer_expected(deeper.clone()),
        },
        crate::json::Json::Object(nested) => read_buffer_values(deeper.clone(), nested),
        _ => buffer_expected(deeper),
    }
}

/// A buffer object: three numbers, in the order the schema lists them (§3.4).
fn read_buffer_values(steps: Tokens, members: Members) -> Result<BufferSpec, RequestError> {
    check_unknown(
        steps.clone(),
        members.clone(),
        names_of(vec![
            "monovalentMillimolar",
            "primerNanomolar",
            "dmsoPercent",
        ]),
    )?;
    let monovalent_millimolar: f64 = field_number(
        steps.clone(),
        members.clone(),
        "monovalentMillimolar".to_string(),
    )?;
    let primer_nanomolar: f64 = field_number(
        steps.clone(),
        members.clone(),
        "primerNanomolar".to_string(),
    )?;
    let dmso_percent: f64 =
        field_number(steps.clone(), members.clone(), "dmsoPercent".to_string())?;
    Ok(BufferSpec::Values {
        monovalent_millimolar,
        primer_nanomolar,
        dmso_percent,
    })
}

/// The design: which planner runs it, which style it designs in, how much
/// annealing a primer must share, where placements may sit and the buffer to
/// leave. Every one of the five is required (§3.4).
fn read_design(steps: Tokens, members: Members) -> Result<DesignSpec, RequestError> {
    check_unknown(
        steps.clone(),
        members.clone(),
        names_of(vec![
            "policy",
            "style",
            "minAnnealingOverlap",
            "placement",
            "buffer",
        ]),
    )?;
    field_fixed(
        steps.clone(),
        members.clone(),
        "policy".to_string(),
        "luriaNative".to_string(),
    )?;
    field_fixed(
        steps.clone(),
        members.clone(),
        "style".to_string(),
        "q5_polymerase".to_string(),
    )?;
    let min_annealing_overlap: i64 = field_integer(
        steps.clone(),
        members.clone(),
        "minAnnealingOverlap".to_string(),
    )?;
    let placement: PlacementSpec = match field_one_of(
        steps.clone(),
        members.clone(),
        "placement".to_string(),
        names_of(vec!["inline", "confine_to_tail"]),
    )?
    .as_str()
    {
        "inline" => PlacementSpec::Inline,
        _ => PlacementSpec::ConfineToTail,
    };
    let buffer: BufferSpec = read_buffer(steps.clone(), members.clone())?;
    Ok(DesignSpec {
        policy: PolicySpec::LuriaNative,
        style: StyleSpec::Q5Polymerase,
        min_annealing_overlap,
        placement,
        buffer,
    })
}

/// A set of variants: how they are built and the members that make them up
/// (§3.4). `combined_construct` is parsed here; the workflow refuses it (§2).
fn read_variants(steps: Tokens, members: Members) -> Result<OperationSpec, RequestError> {
    check_unknown(
        steps.clone(),
        members.clone(),
        names_of(vec!["composition", "members"]),
    )?;
    let composition: CompositionSpec = match field_one_of(
        steps.clone(),
        members.clone(),
        "composition".to_string(),
        names_of(vec!["separate_variants", "combined_construct"]),
    )?
    .as_str()
    {
        "separate_variants" => CompositionSpec::SeparateVariants,
        _ => CompositionSpec::CombinedConstruct,
    };
    let items: Vec<crate::json::Json> =
        field_array(steps.clone(), members.clone(), "members".to_string())?;
    let array_steps: Tokens = step_key(steps.clone(), "members".to_string());
    let mut found: Vec<MemberSpec> = [].to_vec();
    for (index, item) in items.clone().iter().enumerate() {
        let deeper: Tokens = step_index(array_steps.clone(), index);
        found.push(read_member(
            deeper.clone(),
            want_object(deeper.clone(), item.clone())?,
        )?);
    }
    Ok(OperationSpec::Variants {
        composition,
        members: found,
    })
}

/// A degenerate replacement: an interval, the letters it holds and the
/// pattern it becomes (§3.4).
fn read_degenerate(steps: Tokens, members: Members) -> Result<OperationSpec, RequestError> {
    check_unknown(
        steps.clone(),
        members.clone(),
        names_of(vec!["name", "interval", "from", "to"]),
    )?;
    let name: String = field_string(steps.clone(), members.clone(), "name".to_string())?;
    let interval: (i64, i64) = field_pair(steps.clone(), members.clone(), "interval".to_string())?;
    let from: String = field_string(steps.clone(), members.clone(), "from".to_string())?;
    let to: String = field_string(steps.clone(), members.clone(), "to".to_string())?;
    Ok(OperationSpec::Degenerate {
        name,
        interval,
        from,
        to,
    })
}

/// The operation, in whichever of its three forms the one-of group named. The
/// group has only these three names, so the last arm is `degenerate` (§3.4).
fn read_operation(
    steps: Tokens,
    members: Members,
    kind: String,
) -> Result<OperationSpec, RequestError> {
    match kind.as_str() {
        "edit" => {
            let member: MemberSpec = read_member(steps.clone(), members.clone())?;
            Ok(OperationSpec::Edit(member))
        }
        "variants" => read_variants(steps.clone(), members.clone()),
        _ => read_degenerate(steps.clone(), members.clone()),
    }
}

/// One named edit: its name and one of the five kinds of edit (§3.4).
fn read_member(steps: Tokens, members: Members) -> Result<MemberSpec, RequestError> {
    check_unknown(
        steps.clone(),
        members.clone(),
        names_of(vec![
            "name",
            "residue",
            "substitution",
            "deletion",
            "insertion",
            "replacement",
        ]),
    )?;
    let name: String = field_string(steps.clone(), members.clone(), "name".to_string())?;
    let kind: String = one_group(
        steps.clone(),
        members.clone(),
        names_of(vec![
            "residue",
            "substitution",
            "deletion",
            "insertion",
            "replacement",
        ]),
    )?;
    let (edit_steps, edit_members) = field_object(steps.clone(), members.clone(), kind.clone())?;
    let edit: EditSpec = read_edit(edit_steps.clone(), edit_members, kind)?;
    Ok(MemberSpec { name, edit })
}

/// The edit a member gives, of the kind its one-of group named. The group has
/// only these five names, so the last arm is `replacement` (§3.4).
fn read_edit(steps: Tokens, members: Members, kind: String) -> Result<EditSpec, RequestError> {
    match kind.as_str() {
        "residue" => read_residue(steps.clone(), members.clone()),
        "substitution" => read_substitution(steps.clone(), members.clone()),
        "deletion" => read_deletion(steps.clone(), members.clone()),
        "insertion" => read_insertion(steps.clone(), members.clone()),
        _ => read_replacement(steps.clone(), members.clone()),
    }
}

/// A residue letter: which CDS it sits in, its number, the letters it changes
/// between and how its codon is given (§3.4, §3.5).
fn read_residue(steps: Tokens, members: Members) -> Result<EditSpec, RequestError> {
    check_unknown(
        steps.clone(),
        members.clone(),
        names_of(vec!["cds", "number", "from", "to", "codon", "policy"]),
    )?;
    let cds: Option<String> = field_string_opt(steps.clone(), members.clone(), "cds".to_string())?;
    let number: i64 = field_integer(steps.clone(), members.clone(), "number".to_string())?;
    let from: String = field_string(steps.clone(), members.clone(), "from".to_string())?;
    let to: String = field_string(steps.clone(), members.clone(), "to".to_string())?;
    let kind: String = one_group(
        steps.clone(),
        members.clone(),
        names_of(vec!["codon", "policy"]),
    )?;
    let codon: CodonSpec = match kind.as_str() {
        "codon" => CodonSpec::Supplied(field_string(
            steps.clone(),
            members.clone(),
            "codon".to_string(),
        )?),
        _ => {
            field_fixed(
                steps.clone(),
                members.clone(),
                "policy".to_string(),
                "default".to_string(),
            )?;
            CodonSpec::Default
        }
    };
    Ok(EditSpec::Residue {
        cds,
        number,
        from,
        to,
        codon,
    })
}

/// A substitution: an interval, and the letters that replace what it holds
/// (§3.4).
fn read_substitution(steps: Tokens, members: Members) -> Result<EditSpec, RequestError> {
    check_unknown(
        steps.clone(),
        members.clone(),
        names_of(vec!["interval", "from", "to"]),
    )?;
    let interval: (i64, i64) = field_pair(steps.clone(), members.clone(), "interval".to_string())?;
    let from: String = field_string(steps.clone(), members.clone(), "from".to_string())?;
    let to: String = field_string(steps.clone(), members.clone(), "to".to_string())?;
    Ok(EditSpec::Substitution { interval, from, to })
}

/// A deletion: an interval, and the letters it removes (§3.4).
fn read_deletion(steps: Tokens, members: Members) -> Result<EditSpec, RequestError> {
    check_unknown(
        steps.clone(),
        members.clone(),
        names_of(vec!["interval", "from"]),
    )?;
    let interval: (i64, i64) = field_pair(steps.clone(), members.clone(), "interval".to_string())?;
    let from: String = field_string(steps.clone(), members.clone(), "from".to_string())?;
    Ok(EditSpec::Deletion { interval, from })
}

/// An insertion: a point, the letters either side of it and the letters to
/// put in (§3.4).
fn read_insertion(steps: Tokens, members: Members) -> Result<EditSpec, RequestError> {
    check_unknown(
        steps.clone(),
        members.clone(),
        names_of(vec!["point", "before", "after", "insert"]),
    )?;
    let point: i64 = field_integer(steps.clone(), members.clone(), "point".to_string())?;
    let before: String = field_string(steps.clone(), members.clone(), "before".to_string())?;
    let after: String = field_string(steps.clone(), members.clone(), "after".to_string())?;
    let insert: String = field_string(steps.clone(), members.clone(), "insert".to_string())?;
    Ok(EditSpec::Insertion {
        point,
        before,
        after,
        insert,
    })
}

/// A replacement: an interval, and the letters that take its place (§3.4).
fn read_replacement(steps: Tokens, members: Members) -> Result<EditSpec, RequestError> {
    check_unknown(
        steps.clone(),
        members.clone(),
        names_of(vec!["interval", "from", "to"]),
    )?;
    let interval: (i64, i64) = field_pair(steps.clone(), members.clone(), "interval".to_string())?;
    let from: String = field_string(steps.clone(), members.clone(), "from".to_string())?;
    let to: String = field_string(steps.clone(), members.clone(), "to".to_string())?;
    Ok(EditSpec::Replacement { interval, from, to })
}

/// One declared CDS of a FASTA template, checked field by field (§3.4). Its
/// values are kept as core states them, so core reports them (§1).
fn read_cds(steps: Tokens, members: Members) -> Result<CdsSpec, RequestError> {
    check_unknown(
        steps.clone(),
        members.clone(),
        names_of(vec![
            "id",
            "intervals",
            "strand",
            "completeness",
            "geneticCode",
            "numbering",
        ]),
    )?;
    let id: String = field_string(steps.clone(), members.clone(), "id".to_string())?;
    let intervals: Vec<(i64, i64)> = read_intervals(steps.clone(), members.clone())?;
    let strand: StrandSpec = match field_one_of(
        steps.clone(),
        members.clone(),
        "strand".to_string(),
        names_of(vec!["+", "-"]),
    )?
    .as_str()
    {
        "+" => StrandSpec::Plus,
        _ => StrandSpec::Minus,
    };
    let completeness: String =
        field_string(steps.clone(), members.clone(), "completeness".to_string())?;
    let genetic_code: String =
        field_string(steps.clone(), members.clone(), "geneticCode".to_string())?;
    let numbering: String = field_string(steps.clone(), members.clone(), "numbering".to_string())?;
    Ok(CdsSpec {
        id,
        intervals,
        strand,
        completeness,
        genetic_code,
        numbering,
    })
}

/// A CDS's `intervals`: an array of [start, end] pairs, each element with its
/// own path (§3.4).
fn read_intervals(steps: Tokens, members: Members) -> Result<Vec<(i64, i64)>, RequestError> {
    let items: Vec<crate::json::Json> =
        field_array(steps.clone(), members.clone(), "intervals".to_string())?;
    let field_steps: Tokens = step_key(steps.clone(), "intervals".to_string());
    let mut found: Vec<(i64, i64)> = [].to_vec();
    for (index, item) in items.clone().iter().enumerate() {
        found.push(want_pair(
            step_index(field_steps.clone(), index),
            item.clone(),
        )?);
    }
    Ok(found)
}

/// The template, in one of its two forms. Its `fasta`/`genbank` group is
/// resolved first, because the form decides which members the schema allows,
/// and a `topology` beside `genbank` names the LOCUS rule (§3.4, §3.5).
fn read_template(steps: Tokens, members: Members) -> Result<TemplateSpec, RequestError> {
    let kind: String = one_group(
        steps.clone(),
        members.clone(),
        names_of(vec!["fasta", "genbank"]),
    )?;
    let allowed: Vec<String> = match kind.as_str() {
        "fasta" => names_of(vec!["id", "fasta", "topology", "sha256", "cds"]),
        _ => names_of(vec!["id", "genbank", "sha256", "cds"]),
    };
    check_template_members(steps.clone(), members.clone(), allowed)?;
    let id: String = field_string(steps.clone(), members.clone(), "id".to_string())?;
    let path: String = field_string(steps.clone(), members.clone(), kind.clone())?;
    match kind.as_str() {
        "fasta" => {
            let topology: TopologySpec = match field_one_of(
                steps.clone(),
                members.clone(),
                "topology".to_string(),
                names_of(vec!["linear", "circular"]),
            )?
            .as_str()
            {
                "linear" => TopologySpec::Linear,
                _ => TopologySpec::Circular,
            };
            let sha256: Option<String> =
                field_digest_opt(steps.clone(), members.clone(), "sha256".to_string())?;
            let items: Vec<crate::json::Json> =
                field_array(steps.clone(), members.clone(), "cds".to_string())?;
            let array_steps: Tokens = step_key(steps.clone(), "cds".to_string());
            let mut found: Vec<CdsSpec> = [].to_vec();
            for (index, item) in items.clone().iter().enumerate() {
                let deeper: Tokens = step_index(array_steps.clone(), index);
                found.push(read_cds(
                    deeper.clone(),
                    want_object(deeper.clone(), item.clone())?,
                )?);
            }
            Ok(TemplateSpec::Fasta {
                id,
                path,
                topology,
                sha256,
                cds: found,
            })
        }
        _ => {
            let sha256: Option<String> =
                field_digest_opt(steps.clone(), members.clone(), "sha256".to_string())?;
            let items: Vec<crate::json::Json> =
                field_array(steps.clone(), members.clone(), "cds".to_string())?;
            let array_steps: Tokens = step_key(steps.clone(), "cds".to_string());
            let mut found: Vec<String> = [].to_vec();
            for (index, item) in items.clone().iter().enumerate() {
                found.push(want_string(
                    step_index(array_steps.clone(), index),
                    item.clone(),
                )?);
            }
            Ok(TemplateSpec::GenBank {
                id,
                path,
                sha256,
                cds: found,
            })
        }
    }
}

/// A request document at `$`: the schema and coordinate convention it must
/// state, one template, an optional scope, one operation and one design (§3.4,
/// §3.5). Nothing has a default that changes the science.
fn read_root(steps: Tokens, members: Members) -> Result<RequestDocument, RequestError> {
    check_unknown(
        steps.clone(),
        members.clone(),
        names_of(vec![
            "schema",
            "coordinates",
            "template",
            "scope",
            "edit",
            "variants",
            "degenerate",
            "design",
        ]),
    )?;
    field_fixed(
        steps.clone(),
        members.clone(),
        "schema".to_string(),
        crate::REQUEST_SCHEMA.to_string(),
    )?;
    field_fixed(
        steps.clone(),
        members.clone(),
        "coordinates".to_string(),
        "zero-based-half-open".to_string(),
    )?;
    let (template_steps, template_members) =
        field_object(steps.clone(), members.clone(), "template".to_string())?;
    let template: TemplateSpec = read_template(template_steps.clone(), template_members)?;
    let scope: Option<String> =
        field_string_opt(steps.clone(), members.clone(), "scope".to_string())?;
    let kind: String = one_group(
        steps.clone(),
        members.clone(),
        names_of(vec!["edit", "variants", "degenerate"]),
    )?;
    let (operation_steps, operation_members) =
        field_object(steps.clone(), members.clone(), kind.clone())?;
    let operation: OperationSpec =
        read_operation(operation_steps.clone(), operation_members, kind)?;
    let (design_steps, design_members) =
        field_object(steps.clone(), members.clone(), "design".to_string())?;
    let design: DesignSpec = read_design(design_steps.clone(), design_members)?;
    Ok(RequestDocument {
        template,
        scope,
        operation,
        design,
    })
}

/// Read a request document from JSON text, refusing anything the schema does
/// not allow and naming the JSON path of the first problem (§3.4, §3.5). A
/// JSON syntax error, a duplicate key or trailing text is reported at `$`
/// (§3.5).
pub fn parse_request(text: &str) -> Result<RequestDocument, RequestError> {
    let value: crate::json::Json =
        crate::json::parse_json(text).map_err(|problem| RequestError {
            code: RequestErrorCode::Invalid,
            path: "$".to_string(),
            message: format!("invalid JSON: {}", problem),
        })?;
    let steps: Tokens = [].to_vec();
    let members: Members = want_object(steps.clone(), value)?;
    let document: RequestDocument = read_root(steps.clone(), members)?;
    Ok(document)
}

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
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(key);
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
                        edit: EditSpec::Substitution {
                            interval: (99, 100),
                            from: s("A"),
                            to: s("G"),
                        },
                    },
                    MemberSpec {
                        name: s("d1"),
                        edit: EditSpec::Deletion {
                            interval: (99, 102),
                            from: s("ACG"),
                        },
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
                        edit: EditSpec::Replacement {
                            interval: (99, 102),
                            from: s("ACG"),
                            to: s("TT"),
                        },
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
            OperationSpec::Degenerate {
                name: s("lib"),
                interval: (9, 12),
                from: s("GAT"),
                to: s("NNK")
            }
        );
        let mut combined = genbank_variants();
        set(
            &mut combined,
            "/variants/composition",
            json!("combined_construct"),
        );
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
        assert_eq!(
            refused(&wrong_schema),
            pair("$.schema", "expected \"luria.q5-request/1\"")
        );

        let mut coordinates = valid();
        set(&mut coordinates, "/coordinates", json!("one-based"));
        assert_eq!(
            refused(&coordinates),
            pair("$.coordinates", "expected \"zero-based-half-open\"")
        );

        let mut missing = valid();
        remove(&mut missing, "", "design");
        assert_eq!(
            refused(&missing),
            pair("$", "missing required field 'design'")
        );

        let mut unknown = valid();
        insert(&mut unknown, "", "extra", json!(1));
        assert_eq!(refused(&unknown), pair("$.extra", "unknown field 'extra'"));

        let mut none = valid();
        remove(&mut none, "", "edit");
        assert_eq!(
            refused(&none),
            pair(
                "$",
                "exactly one of 'edit', 'variants', 'degenerate' is required"
            )
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
            pair(
                "$",
                "only one of 'edit', 'variants', 'degenerate' may be given"
            )
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
        assert_eq!(
            refused(&both),
            pair("$.template", "only one of 'fasta', 'genbank' may be given")
        );

        let mut neither = valid();
        remove(&mut neither, "/template", "fasta");
        assert_eq!(
            refused(&neither),
            pair(
                "$.template",
                "exactly one of 'fasta', 'genbank' is required"
            )
        );

        let mut no_topology = valid();
        remove(&mut no_topology, "/template", "topology");
        assert_eq!(
            refused(&no_topology),
            pair("$.template", "missing required field 'topology'")
        );

        let mut genbank_topology = genbank_variants();
        insert(
            &mut genbank_topology,
            "/template",
            "topology",
            json!("circular"),
        );
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
            pair(
                "$.template.topology",
                "expected one of \"linear\", \"circular\""
            )
        );

        let mut digest = valid();
        insert(&mut digest, "/template", "sha256", json!("ABC"));
        assert_eq!(
            refused(&digest),
            pair(
                "$.template.sha256",
                "expected 64 lower-case hexadecimal digits"
            )
        );

        let mut upper_digest = genbank_variants();
        set(
            &mut upper_digest,
            "/template/sha256",
            json!("536042A7485BDABDF49C4D21A306213A098D77D19BDBA57A1C6ADC76CE2B8372"),
        );
        assert_eq!(
            refused(&upper_digest),
            pair(
                "$.template.sha256",
                "expected 64 lower-case hexadecimal digits"
            )
        );

        let mut genbank_object = genbank_variants();
        set(
            &mut genbank_object,
            "/template/cds/0",
            json!({ "id": "tet" }),
        );
        assert_eq!(
            refused(&genbank_object),
            pair("$.template.cds[0]", "expected a string")
        );

        let mut no_cds = valid();
        remove(&mut no_cds, "/template", "cds");
        assert_eq!(
            refused(&no_cds),
            pair("$.template", "missing required field 'cds'")
        );
    }

    #[test]
    fn a_declared_cds_is_checked_field_by_field() {
        let mut short = valid();
        set(&mut short, "/template/cds/0/intervals/0", json!([0]));
        assert_eq!(
            refused(&short),
            pair("$.template.cds[0].intervals[0]", "expected [start, end]")
        );

        let mut fraction = valid();
        set(
            &mut fraction,
            "/template/cds/0/intervals/0",
            json!([0, 3153.0]),
        );
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
        assert_eq!(
            refused(&unknown),
            pair("$.template.cds[0].name", "unknown field 'name'")
        );

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
        assert_eq!(
            refused(&both),
            pair(
                "$.edit.residue",
                "only one of 'codon', 'policy' may be given"
            )
        );

        let mut neither = valid();
        remove(&mut neither, "/edit/residue", "codon");
        assert_eq!(
            refused(&neither),
            pair(
                "$.edit.residue",
                "exactly one of 'codon', 'policy' is required"
            )
        );

        let mut policy = valid();
        remove(&mut policy, "/edit/residue", "codon");
        insert(&mut policy, "/edit/residue", "policy", json!("minimal"));
        assert_eq!(
            refused(&policy),
            pair("$.edit.residue.policy", "expected \"default\"")
        );

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
        assert_eq!(
            refused(&nameless),
            pair("$.edit", "missing required field 'name'")
        );

        let mut text_number = valid();
        set(&mut text_number, "/edit/residue/number", json!("404"));
        assert_eq!(
            refused(&text_number),
            pair("$.edit.residue.number", "expected an integer")
        );

        let mut huge = valid();
        set(
            &mut huge,
            "/edit/residue/number",
            json!(18446744073709551615u64),
        );
        assert_eq!(
            refused(&huge),
            pair("$.edit.residue.number", "expected an integer")
        );

        let mut member = genbank_variants();
        set(
            &mut member,
            "/variants/members/1/deletion/interval",
            json!([3]),
        );
        assert_eq!(
            refused(&member),
            pair(
                "$.variants.members[1].deletion.interval",
                "expected [start, end]"
            )
        );

        let mut insertion = genbank_variants();
        remove(&mut insertion, "/variants/members/2/insertion", "after");
        assert_eq!(
            refused(&insertion),
            pair(
                "$.variants.members[2].insertion",
                "missing required field 'after'"
            )
        );

        let mut members = genbank_variants();
        set(&mut members, "/variants/members", json!({}));
        assert_eq!(
            refused(&members),
            pair("$.variants.members", "expected an array")
        );

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
        assert_eq!(
            refused(&partial),
            pair("$.design.buffer", "missing required field 'dmsoPercent'")
        );

        let mut text_salt = genbank_variants();
        set(
            &mut text_salt,
            "/design/buffer/monovalentMillimolar",
            json!("150"),
        );
        assert_eq!(
            refused(&text_salt),
            pair("$.design.buffer.monovalentMillimolar", "expected a number")
        );

        let mut policy = valid();
        set(&mut policy, "/design/policy", json!("nebasechanger"));
        assert_eq!(
            refused(&policy),
            pair("$.design.policy", "expected \"luriaNative\"")
        );

        let mut placement = valid();
        set(&mut placement, "/design/placement", json!("tail"));
        assert_eq!(
            refused(&placement),
            pair(
                "$.design.placement",
                "expected one of \"inline\", \"confine_to_tail\""
            )
        );

        let mut overlap = valid();
        set(&mut overlap, "/design/minAnnealingOverlap", json!(1.5));
        assert_eq!(
            refused(&overlap),
            pair("$.design.minAnnealingOverlap", "expected an integer")
        );

        let mut missing = valid();
        remove(&mut missing, "/design", "style");
        assert_eq!(
            refused(&missing),
            pair("$.design", "missing required field 'style'")
        );
    }

    #[test]
    fn bad_json_is_refused_at_the_root() {
        for text in ["", "{\"schema\": 1,}", "{} {}", "{\"a\": 1, \"a\": 2}"] {
            match parse_request(text) {
                Err(error) => {
                    assert_eq!(error.code, RequestErrorCode::Invalid, "{:?}", text);
                    assert_eq!(error.path, "$", "{:?}", text);
                    assert!(
                        error.message.starts_with("invalid JSON: "),
                        "{:?}",
                        error.message
                    );
                }
                Ok(document) => panic!("accepted {:?}: {:?}", text, document),
            }
        }
    }

    #[test]
    fn every_request_error_code_is_pinned() {
        assert_eq!(RequestErrorCode::Unreadable.code(), "request_unreadable");
        assert_eq!(RequestErrorCode::Invalid.code(), "request_invalid");
        assert_eq!(
            RequestErrorCode::TemplateUnreadable.code(),
            "template_unreadable"
        );
        assert_eq!(
            RequestErrorCode::TemplateDigestMismatch.code(),
            "template_digest_mismatch"
        );
    }
}
