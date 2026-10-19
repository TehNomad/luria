//! Reading the structure of one GenBank flat-file record (D041, D009,
//! D007, D008).
//!
//! The reader takes text, never a file: opening a file belongs to the caller
//! (D041). It reads the record's layout — the LOCUS line, ACCESSION and
//! VERSION, the FEATURES table, the ORIGIN sequence and the `//` terminator —
//! and nothing else. Feature locations are kept as text and no qualifier is
//! given a meaning: 5c3 parses the location of the CDSs a request names, so a
//! problem in a feature nobody asked for never fails a run (D041). Header
//! sections other than ACCESSION and VERSION are source metadata that luria
//! does not interpret.
//!
//! The rules, in the order the contract fixes them:
//!
//! 1. `str::lines()` splits the text; a trailing `\r` is removed from every
//!    line and line numbers count from 1, as an editor numbers them: they are
//!    not sequence coordinates (§4.1).
//! 2. A line is blank when it is empty or holds only spaces, tabs and `\r`;
//!    blank lines are skipped before, inside and after the record. Columns
//!    count spaces only, so a tab is never indentation (§4.1).
//! 3. The first non-blank line must start with `LOCUS` followed by a space or
//!    the end of the line, or the text is `NoLocus` with that line; a text
//!    with no non-blank line is `NoLocus` with no line (§4.2).
//! 4. The LOCUS line gives the length in bp — a digits token before a `bp`
//!    token at index 2 or later, `LocusLength` otherwise — exactly one of
//!    `linear` or `circular` among the tokens after that `bp` token
//!    (`TopologyMissing`, `TopologyConflict`) and the record's name, which
//!    states no topology of its own. Every refusal carries the LOCUS line's
//!    number, and the topology is the sequence's, never inferred (§4.3 step
//!    2, D009, D041).
//! 5. Each later non-blank line is `//`, a section line (first character an
//!    ASCII upper-case letter), a continuation line (first character a space)
//!    or `LineLayout`; `LOCUS`, `ACCESSION`, `VERSION`, `FEATURES` and
//!    `ORIGIN` appear once each, and a repeat before `//` is
//!    `RepeatedSection` (§4.4). ACCESSION and VERSION keep their first token
//!    exactly, `.` included; other sections and their continuation lines are
//!    skipped.
//! 6. FEATURES (§4.5): exactly 5 spaces starts a feature key, at least 21
//!    spaces is the qualifier area, and any other indentation is
//!    `LineLayout`. A qualifier-area line before the first key line is
//!    `LineLayout`. Each qualifier-area line then takes the first of five
//!    cases: continuing an open quoted value, starting a new qualifier,
//!    continuing a location, continuing an unquoted value, or
//!    `QualifierSyntax`.
//! 7. A qualifier value starting with `"` is quoted: its text lines are
//!    joined with one space, it stays open while the quote count is odd, and
//!    a closed value must end at its closing quote and hold every inner `"`
//!    doubled, so `"a"b` and `"a" "b"` are `QualifierSyntax`; its stored
//!    text is inside the outer quotes with each `""` made a `"`. An unquoted
//!    value may not contain a `"` at all, and `/name=` is the empty value
//!    (§4.6, amended in review). Every qualifier is read the same way, so a
//!    wrapped `/translation` keeps its spaces: 5c3 removes them there.
//! 8. Each ORIGIN sequence line is a position number then its bases: a line
//!    whose first token is not a number is `OriginLine`, a base token holding
//!    `;` is the same refusal — ORIGIN text never reaches `normalize`'s
//!    comment-line rule — and a number other than one past the bases before
//!    it is `OriginNumbering` (§4.7, §4.8).
//! 9. The end: `//` with no ORIGIN section is `NoOrigin`; text that ends
//!    before `//` is `MissingEnd`; after `//` a `LOCUS` line is
//!    `SecondRecord` and any other non-blank line is `TextAfterEnd`; the
//!    ORIGIN letters go to `sequence::normalize` unchanged, whose errors are
//!    returned unchanged, and last the LOCUS length must equal the
//!    normalized length or the result is `LengthMismatch` with the LOCUS
//!    line (§4.8, D008).
//!
//! The caller's identifier is the sequence's identity: the LOCUS name,
//! ACCESSION and VERSION are source metadata, never identity (D007). One
//! record per file, and nothing is repaired: keywords are not re-cased, tabs
//! are not expanded, a misnumbered ORIGIN line is not skipped, and nothing
//! is read past `//` (D041, D008).

use crate::error::DomainError;
use crate::error::GenBankRule;
use crate::error::MalformedGenBankProblem;
use crate::identity::Identifier;
use crate::sequence::normalize;
use crate::sequence::Sequence;
use crate::sequence::SequenceTopology;

/// One qualifier of a feature: `/name` (value `None`) or `/name=value`.
#[derive(Clone, Debug, PartialEq)]
pub struct GenBankQualifier {
    name: String,
    value: Option<String>,
}

impl GenBankQualifier {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }
}

/// One entry of the FEATURES table. The location is text: 5c3 parses it, and
/// only for the CDSs a request names.
#[derive(Clone, Debug, PartialEq)]
pub struct GenBankFeature {
    key: String,
    location: String,
    qualifiers: Vec<GenBankQualifier>,
    line: u64,
}

impl GenBankFeature {
    pub fn key(&self) -> String {
        self.key.clone()
    }

    /// The location text, its continuation lines appended with no separator.
    pub fn location(&self) -> String {
        self.location.clone()
    }

    /// In the order written.
    pub fn qualifiers(&self) -> Vec<GenBankQualifier> {
        self.qualifiers.clone()
    }

    /// The text line of the feature key, counted from 1.
    pub fn line(&self) -> u64 {
        self.line
    }
}

/// One GenBank record: source metadata, the sequence, and the feature table.
#[derive(Clone, Debug, PartialEq)]
pub struct GenBankRecord {
    locus_name: String,
    accession: Option<String>,
    version: Option<String>,
    sequence: Sequence,
    features: Vec<GenBankFeature>,
}

impl GenBankRecord {
    pub fn locus_name(&self) -> String {
        self.locus_name.clone()
    }

    pub fn accession(&self) -> Option<String> {
        self.accession.clone()
    }

    pub fn version(&self) -> Option<String> {
        self.version.clone()
    }

    pub fn sequence(&self) -> Sequence {
        self.sequence.clone()
    }

    /// The feature table, in the order written.
    pub fn features(&self) -> Vec<GenBankFeature> {
        self.features.clone()
    }
}

/// What a LOCUS line states: the length in bp, the record's name, and the
/// topology the sequence carries (§4.3).
struct Locus {
    length: u64,
    name: String,
    topology: SequenceTopology,
}

/// What the qualifier the last FEATURES line touched left behind, and so
/// what the next qualifier-area line may do with it (§4.5, §4.6).
#[derive(Clone, Debug, PartialEq)]
enum ValueState {
    /// No qualifier yet, so a plain text line continues the location.
    Nothing,
    /// A `/name` with no value, which no text line may continue.
    Flag,
    /// An unquoted value, whose text so far the reader holds.
    Unquoted,
    /// A quoted value, complete.
    Quoted,
    /// A quoted value still open, its joined text the reader holds.
    OpenValue,
}

/// The FEATURES section as read so far: its completed features, and the one
/// still open with the qualifier its last line left. An empty `key` means no
/// feature has started yet (§4.5, §4.6).
struct FeaturesRead {
    done: Vec<GenBankFeature>,
    key: String,
    location: String,
    line: u64,
    qualifiers: Vec<GenBankQualifier>,
    qualifier_name: String,
    value_text: String,
    qualifier_line: u64,
    value: ValueState,
}

/// The refusal of a GenBank text: the rule, and the line to point at (§4.1).
fn refusal(sequence: &str, rule: GenBankRule, line: Option<u64>) -> DomainError {
    DomainError::MalformedGenBank(MalformedGenBankProblem {
        sequence: sequence.to_string(),
        rule,
        line,
    })
}

/// True for a blank character: a space, a tab or a carriage return (§4.1).
fn is_blank_char(ch: char) -> bool {
    ch == ' ' || ch == '\t' || ch == '\r'
}

/// True for an empty line, or one made only of spaces, tabs and `\r` (§4.1).
fn is_blank(line: &str) -> bool {
    for ch in line.chars() {
        if !is_blank_char(ch) {
            return false;
        }
    }
    true
}

/// The text's lines, each with a trailing `\r` removed (§4.1). `str::lines()`
/// drops the `\r` of a `\r\n`; a line that ends with `\r` for another reason
/// is trimmed here too, so a `\r` never reaches the rules below.
fn record_lines(text: String) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        let chars: Vec<char> = line.chars().collect();
        let mut to: usize = chars.len();
        while to > 0 && chars[to - 1] == '\r' {
            to -= 1
        }
        lines.push(chars[0..to].iter().collect());
    }
    lines
}

/// The count of spaces at the start of a line: columns count spaces only, and
/// a tab is never indentation (§4.1, §4.4, §4.5).
fn space_indent(line: &str) -> usize {
    let mut count: usize = 0;
    for ch in line.chars() {
        if ch != ' ' {
            break;
        }
        count += 1
    }
    count
}

/// True for the record's terminator: `//` with its trailing spaces removed
/// (§4.4).
fn terminated(line: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    let mut to: usize = chars.len();
    while to > 0 && chars[to - 1] == ' ' {
        to -= 1
    }
    to == 2 && chars[0] == '/' && chars[1] == '/'
}

/// True for the LOCUS keyword: `LOCUS` followed by a space or the end of the
/// line (§4.2, §4.8).
fn locus_line(line: &str) -> bool {
    line == "LOCUS" || line.starts_with("LOCUS ")
}

/// True when the first character is an ASCII upper-case letter, which starts
/// a section (§4.4).
fn section_line(line: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    !chars.is_empty() && chars[0].is_ascii_uppercase()
}

/// The text up to the first space, or the whole line (§4.4, §4.5).
fn first_word(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut to: usize = 0;
    while to < chars.len() && chars[to] != ' ' {
        to += 1
    }
    chars[0..to].iter().collect()
}

/// The text after the first `count` characters, spaces removed from both ends
/// (§4.4 value lines, §4.5 key lines).
fn after_start(line: &str, count: usize) -> String {
    let chars: Vec<char> = line.chars().collect();
    let from: usize = if count < chars.len() {
        count
    } else {
        chars.len()
    };
    trim_spaces(chars[from..chars.len()].iter().collect())
}

/// `text` with spaces removed from both ends, internal spaces kept (§4.5).
fn trim_spaces(text: String) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut from: usize = 0;
    while from < chars.len() && chars[from] == ' ' {
        from += 1
    }
    let mut to: usize = chars.len();
    while to > from && chars[to - 1] == ' ' {
        to -= 1
    }
    chars[from..to].iter().collect()
}

/// The line's tokens, split on runs of spaces (§4.3, §4.7).
fn space_tokens(line: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current: String = "".to_string();
    for ch in line.chars() {
        if ch == ' ' {
            if !current.is_empty() {
                tokens.push(current);
                current = "".to_string();
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// True for a non-empty run of ASCII digits (§4.3, §4.7).
fn ascii_digits(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|ch| ch.is_ascii_digit())
}

/// Two text lines joined with one space (§4.6: wrapped prose is joined with a
/// space).
fn join_with_space(before: String, text: String) -> String {
    [before, text].join(" ")
}

/// The count of `"` in a text (§4.6).
fn quote_count(text: &str) -> usize {
    let mut count: usize = 0;
    for ch in text.chars() {
        if ch == '"' {
            count += 1
        }
    }
    count
}

/// A closed quoted value's text: the characters inside the outer quotes, with
/// each `""` made a `"` (§4.6).
fn decode_quoted(text: String) -> String {
    let chars: Vec<char> = text.chars().collect();
    let inner: Vec<char> = chars[1..chars.len() - 1].to_vec();
    let mut out: Vec<char> = Vec::new();
    let mut at: usize = 0;
    while at < inner.len() {
        if at + 1 < inner.len() && inner[at] == '"' && inner[at + 1] == '"' {
            out.push('"');
            at += 2
        } else {
            out.push(inner[at]);
            at += 1
        }
    }
    out.iter().collect()
}

/// The FEATURES section before any of its lines has been read (§4.5).
fn new_features_read() -> FeaturesRead {
    FeaturesRead {
        done: Vec::new(),
        key: "".to_string(),
        location: "".to_string(),
        line: 0,
        qualifiers: Vec::new(),
        qualifier_name: "".to_string(),
        value_text: "".to_string(),
        qualifier_line: 0,
        value: ValueState::Nothing,
    }
}

/// The feature a key line starts: its key, the location text the key line
/// gives after it, and the line the key sits on (§4.5).
fn new_feature(
    mut state: FeaturesRead,
    key: String,
    location: String,
    number: u64,
) -> FeaturesRead {
    state.key = key;
    state.location = location;
    state.line = number;
    state
}

/// The pending qualifier, complete: `/name` holds `None`, and `/name=` an
/// empty value (§4.6). An open quoted value is not complete and stays out
/// until it closes.
fn close_qualifier(mut state: FeaturesRead) -> FeaturesRead {
    match state.value {
        ValueState::Flag => state.qualifiers.push(GenBankQualifier {
            name: state.qualifier_name,
            value: None,
        }),
        ValueState::Unquoted | ValueState::Quoted => state.qualifiers.push(GenBankQualifier {
            name: state.qualifier_name,
            value: Some(state.value_text.to_string()),
        }),
        ValueState::Nothing | ValueState::OpenValue => (),
    }
    state.qualifier_name = "".to_string();
    state.value_text = "".to_string();
    state.qualifier_line = 0;
    state.value = ValueState::Nothing;
    state
}

/// The end of the feature in progress: its pending qualifier is complete
/// first, and a quoted value still open at that point is refused at the line
/// of its `/name=` (§4.5).
fn end_feature(sequence: &str, state: FeaturesRead) -> Result<FeaturesRead, DomainError> {
    if state.value == ValueState::OpenValue {
        return Err(refusal(
            sequence,
            GenBankRule::QualifierSyntax,
            Some(state.qualifier_line),
        ));
    }
    let mut state: FeaturesRead = close_qualifier(state);
    if !state.key.is_empty() {
        state.done.push(GenBankFeature {
            key: state.key,
            location: state.location,
            qualifiers: state.qualifiers,
            line: state.line,
        });
    }
    state.key = "".to_string();
    state.location = "".to_string();
    state.line = 0;
    state.qualifiers = Vec::new();
    Ok(state)
}

/// True when the text of a closed quoted value holds a `"` inside its outer
/// quotes that is not part of a `""` pair, read left to right: an inner quote
/// is literal only when doubled, so `/note="a" "b"` is not one value (§4.6,
/// amended in review A-5).
fn lone_inner_quote(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let mut at: usize = 1;
    while at + 1 < chars.len() {
        if chars[at] != '"' {
            at += 1
        } else if at + 2 < chars.len() && chars[at + 1] == '"' {
            at += 2
        } else {
            return true;
        }
    }
    false
}

/// A quoted value whose joined text has become complete, or stayed open: the
/// text inside the outer quotes with each `""` made a `"`, or a refusal when
/// the joined text does not end at its closing quote, or when an inner quote
/// is not doubled (§4.5, §4.6).
fn close_quoted(sequence: &str, mut state: FeaturesRead) -> Result<FeaturesRead, DomainError> {
    if quote_count(&state.value_text) % 2 == 1 {
        return Ok(state);
    }
    if !state.value_text.ends_with("\"") {
        return Err(refusal(
            sequence,
            GenBankRule::QualifierSyntax,
            Some(state.qualifier_line),
        ));
    }
    if lone_inner_quote(&state.value_text) {
        return Err(refusal(
            sequence,
            GenBankRule::QualifierSyntax,
            Some(state.qualifier_line),
        ));
    }
    state.value_text = decode_quoted(state.value_text);
    state.value = ValueState::Quoted;
    Ok(state)
}

/// Case 1 of the qualifier area: a text line inside an open quoted value,
/// joined with one space and closed when its quote count becomes even
/// (§4.5, §4.6).
fn continue_quoted(
    sequence: &str,
    text: String,
    mut state: FeaturesRead,
) -> Result<FeaturesRead, DomainError> {
    state.value_text = join_with_space(state.value_text, text);
    close_quoted(sequence, state)
}

/// Case 2 of the qualifier area: a new qualifier, `/name` with no value or
/// `/name=value`. An empty name, or one holding a `"` or any character for
/// which `char::is_whitespace` is true, is refused (§4.5 case 2, §4.6).
fn start_qualifier(
    sequence: &str,
    number: u64,
    text: String,
    mut state: FeaturesRead,
) -> Result<FeaturesRead, DomainError> {
    state = close_qualifier(state);
    let chars: Vec<char> = text.chars().collect();
    // The leading `/` is part of the syntax, never of the name.
    let rest: Vec<char> = chars[1..chars.len()].to_vec();
    let mut at: usize = 0;
    while at < rest.len() && rest[at] != '=' {
        at += 1
    }
    let name: String = rest[0..at].iter().collect();
    let has_value: bool = at < rest.len();
    let value: String = if has_value {
        rest[at + 1..rest.len()].iter().collect()
    } else {
        "".to_string()
    };
    if name.is_empty() || name.contains("\"") || name.chars().any(|ch| ch.is_whitespace()) {
        return Err(refusal(
            sequence,
            GenBankRule::QualifierSyntax,
            Some(number),
        ));
    }
    state.qualifier_name = name;
    state.qualifier_line = number;
    if !has_value {
        state.value = ValueState::Flag;
        return Ok(state);
    }
    if value.starts_with("\"") {
        state.value_text = value;
        state.value = ValueState::OpenValue;
        return close_quoted(sequence, state);
    }
    if value.contains("\"") {
        return Err(refusal(
            sequence,
            GenBankRule::QualifierSyntax,
            Some(number),
        ));
    }
    state.value_text = value;
    state.value = ValueState::Unquoted;
    Ok(state)
}

/// One line of the qualifier area: the first of the five cases that applies
/// decides, in the order §4.5 fixes them. `text` is the line with its spaces
/// removed from both ends.
fn qualifier_area(
    sequence: &str,
    number: u64,
    text: String,
    mut state: FeaturesRead,
) -> Result<FeaturesRead, DomainError> {
    if state.value == ValueState::OpenValue {
        // Case 1: a continuation of an open quoted value, a `/` included.
        return continue_quoted(sequence, text, state);
    }
    if text.starts_with("/") {
        // Case 2: a new qualifier.
        return start_qualifier(sequence, number, text, state);
    }
    if state.value == ValueState::Nothing && state.qualifiers.is_empty() {
        // Case 3: a continuation of the location, appended with no separator.
        state.location = [state.location, text].join("");
        return Ok(state);
    }
    if state.value == ValueState::Unquoted {
        // Case 4: a continuation of an unquoted value, one space apart. An
        // unquoted value may not hold a quote character, and neither may a
        // line joined onto it (§4.6): refused at the continuation line's
        // number.
        if text.contains("\"") {
            return Err(refusal(
                sequence,
                GenBankRule::QualifierSyntax,
                Some(number),
            ));
        }
        state.value_text = join_with_space(state.value_text, text);
        return Ok(state);
    }
    // Case 5: a closed quoted value, or a `/name` with no value: nothing
    // left to continue.
    Err(refusal(
        sequence,
        GenBankRule::QualifierSyntax,
        Some(number),
    ))
}

/// One continuation line of the FEATURES section (§4.5): a key line at
/// exactly 5 spaces, the qualifier area at 21 or more, and any other
/// indentation as `LineLayout`.
fn read_features_line(
    sequence: &str,
    number: u64,
    line: &str,
    state: FeaturesRead,
) -> Result<FeaturesRead, DomainError> {
    let indent: usize = space_indent(line);
    if indent == 5 {
        // A key line ends the feature before it (§4.5).
        let closed: FeaturesRead = end_feature(sequence, state)?;
        let body: String = after_start(line, 5);
        let key: String = first_word(&body);
        let location: String = after_start(&body, key.chars().count());
        if location.is_empty() {
            return Err(refusal(sequence, GenBankRule::LineLayout, Some(number)));
        }
        Ok(new_feature(closed, key, location, number))
    } else if indent >= 21 {
        if state.key.is_empty() {
            // A qualifier-area line before the first key line (§4.5).
            return Err(refusal(sequence, GenBankRule::LineLayout, Some(number)));
        }
        qualifier_area(sequence, number, trim_spaces(line.to_string()), state)
    } else {
        Err(refusal(sequence, GenBankRule::LineLayout, Some(number)))
    }
}

/// The value of an ACCESSION or VERSION keyword line: the first token after
/// the keyword, kept exactly, or `None` when the line holds none (§4.4).
fn keyword_value(line: &str) -> Option<String> {
    let rest: String = after_start(line, first_word(line).chars().count());
    let tokens: Vec<String> = space_tokens(&rest);
    if tokens.is_empty() {
        None
    } else {
        Some(tokens[0].to_string())
    }
}

/// One sequence line of the ORIGIN section (§4.7): its first token is the
/// position one past the bases read before it, `before`, and its remaining
/// tokens are the line's bases. A base token holding `;` is an ORIGIN line
/// that is not one, refused at this line's number.
fn read_origin_line(
    sequence: &str,
    number: u64,
    before: u64,
    line: &str,
) -> Result<String, DomainError> {
    let tokens: Vec<String> = space_tokens(line);
    if tokens.is_empty() {
        return Err(refusal(sequence, GenBankRule::OriginLine, Some(number)));
    }
    if !ascii_digits(&tokens[0]) {
        return Err(refusal(sequence, GenBankRule::OriginLine, Some(number)));
    }
    let position: u64 = match tokens[0].parse::<u64>() {
        Ok(value) => value,
        Err(_) => return Err(refusal(sequence, GenBankRule::OriginLine, Some(number))),
    };
    if position != before + 1 {
        return Err(refusal(
            sequence,
            GenBankRule::OriginNumbering,
            Some(number),
        ));
    }
    // A `;` in a base token is not a sequence character: refused here, so
    // ORIGIN text never reaches `normalize`'s comment-line rule (§4.8).
    for token in &tokens[1..tokens.len()] {
        if token.contains(";") {
            return Err(refusal(sequence, GenBankRule::OriginLine, Some(number)));
        }
    }
    Ok(tokens[1..tokens.len()].join(""))
}

/// The LOCUS line's length in bp, the record's name and the sequence's
/// topology (§4.3, amended in review): the length is a digits token before a
/// `bp` token at index 2 or later, the topology exactly one of `linear` or
/// `circular` among the tokens after that `bp` token, matched lower case, and
/// the name is the token after `LOCUS` unless the length token is that token.
/// A `linear` or `circular` name states no topology, so zero matches is
/// `TopologyMissing` and more than one is `TopologyConflict`. Every refusal
/// carries the LOCUS line's `number`.
fn read_locus(sequence: &str, line: &str, number: u64) -> Result<Locus, DomainError> {
    let tokens: Vec<String> = space_tokens(line);
    let mut at: usize = 2;
    while at < tokens.len() && tokens[at] != "bp" {
        at += 1
    }
    if at >= tokens.len() {
        return Err(refusal(sequence, GenBankRule::LocusLength, Some(number)));
    }
    let length_token: String = tokens[at - 1].to_string();
    if !ascii_digits(&length_token) {
        return Err(refusal(sequence, GenBankRule::LocusLength, Some(number)));
    }
    let length: u64 = match length_token.parse::<u64>() {
        Ok(value) => value,
        Err(_) => return Err(refusal(sequence, GenBankRule::LocusLength, Some(number))),
    };
    // The topology tokens are the ones after the `bp` token (§4.3 step 2,
    // amended in review): a token before it belongs to the record's name, and
    // a name such as `linear` states no topology.
    let topology_tokens: Vec<String> = tokens[at + 1..tokens.len()].to_vec();
    let mut topologies: usize = 0;
    for token in &topology_tokens {
        if token == "linear" || token == "circular" {
            topologies += 1
        }
    }
    if topologies == 0 {
        return Err(refusal(
            sequence,
            GenBankRule::TopologyMissing,
            Some(number),
        ));
    }
    if topologies > 1 {
        return Err(refusal(
            sequence,
            GenBankRule::TopologyConflict,
            Some(number),
        ));
    }
    let topology: SequenceTopology = if topology_tokens.iter().any(|token| token == "circular") {
        SequenceTopology::Circular
    } else {
        SequenceTopology::Linear
    };
    // The name is the token after `LOCUS`, except when the length token is
    // that token, as a line that states no name does (§4.3).
    let name: String = if at == 2 {
        "".to_string()
    } else {
        tokens[1].to_string()
    };
    Ok(Locus {
        length,
        name,
        topology,
    })
}

/// Read a text that holds exactly one GenBank record: its source metadata,
/// its sequence, and its feature table with each location kept as text. The
/// identifier is the sequence's, supplied by the caller (D007); the topology
/// comes from the LOCUS line (D009, D041) and the ORIGIN letters go to
/// `normalize` unchanged (D008).
pub fn read_genbank(name: Identifier, text: String) -> Result<GenBankRecord, DomainError> {
    let identifier: String = name.value().to_string();
    let lines: Vec<String> = record_lines(text);

    // §4.2: the first non-blank line must state LOCUS.
    let mut locus_number: u64 = 0;
    let mut locus_text: String = "".to_string();
    let mut found: u64 = 0;
    for line in &lines {
        found += 1;
        if !is_blank(line) {
            locus_number = found;
            locus_text = line.to_string();
            break;
        }
    }
    if locus_number == 0 {
        // A text with no non-blank line has no line to point at.
        return Err(refusal(&identifier, GenBankRule::NoLocus, None));
    }
    if !locus_line(&locus_text) {
        return Err(refusal(
            &identifier,
            GenBankRule::NoLocus,
            Some(locus_number),
        ));
    }

    // §4.3: the LOCUS line's length, name and topology.
    let locus: Locus = read_locus(&identifier, &locus_text, locus_number)?;

    // §4.4: the sections after LOCUS, in line order.
    let mut accession: Option<String> = None;
    let mut version: Option<String> = None;
    let mut seen_accession: bool = false;
    let mut seen_version: bool = false;
    let mut seen_features: bool = false;
    let mut seen_origin: bool = false;
    let mut in_features: bool = false;
    let mut in_origin: bool = false;
    let mut end_line: Option<u64> = None;
    let mut features: FeaturesRead = new_features_read();
    let mut origin_lines: Vec<String> = Vec::new();
    let mut origin_length: u64 = 0;

    let mut number: u64 = 0;
    for line in &lines {
        number += 1;
        if number <= locus_number || is_blank(line) {
            continue;
        }

        if end_line.is_some() {
            // §4.8: after the terminator only blank lines may follow.
            if locus_line(line) {
                return Err(refusal(
                    &identifier,
                    GenBankRule::SecondRecord,
                    Some(number),
                ));
            }
            return Err(refusal(
                &identifier,
                GenBankRule::TextAfterEnd,
                Some(number),
            ));
        }

        if terminated(line) {
            // §4.5 and §4.8: the record ends at its terminator.
            features = end_feature(&identifier, features)?;
            if !seen_origin {
                return Err(refusal(&identifier, GenBankRule::NoOrigin, Some(number)));
            }
            end_line = Some(number);
            continue;
        }

        if space_indent(line) > 0 {
            // A continuation line, which belongs to the current section
            // (§4.4, §4.5, §4.7).
            if in_features {
                features = read_features_line(&identifier, number, line, features)?;
            } else if in_origin {
                let bases: String = read_origin_line(&identifier, number, origin_length, line)?;
                let added: u64 = bases.chars().count() as u64;
                origin_lines.push(bases);
                origin_length += added;
            }
            continue;
        }

        if !section_line(line) {
            // §4.7 and §4.4: in ORIGIN any line that is neither a
            // continuation nor the terminator is an ORIGIN line that is not
            // one; elsewhere such a line fits no part of the layout.
            if in_origin {
                return Err(refusal(&identifier, GenBankRule::OriginLine, Some(number)));
            }
            return Err(refusal(&identifier, GenBankRule::LineLayout, Some(number)));
        }

        // §4.4: a section line, which ends the feature in progress (§4.5).
        features = end_feature(&identifier, features)?;
        if in_origin {
            // §4.7: ORIGIN runs to its `//`, so a section line inside it is a
            // sequence line that is not one, whichever keyword it carries. It
            // outranks `RepeatedSection`: ORIGIN may not be re-opened.
            return Err(refusal(&identifier, GenBankRule::OriginLine, Some(number)));
        }
        // The first LOCUS is the line §4.2 read, so a LOCUS here repeats it.
        let keyword: String = first_word(line);
        let repeated: bool = keyword == "LOCUS"
            || (keyword == "ACCESSION" && seen_accession)
            || (keyword == "VERSION" && seen_version)
            || (keyword == "FEATURES" && seen_features)
            || (keyword == "ORIGIN" && seen_origin);
        if repeated {
            return Err(refusal(
                &identifier,
                GenBankRule::RepeatedSection,
                Some(number),
            ));
        }
        if keyword == "ACCESSION" {
            seen_accession = true;
            accession = keyword_value(line);
        } else if keyword == "VERSION" {
            seen_version = true;
            version = keyword_value(line);
        } else if keyword == "FEATURES" {
            seen_features = true;
            in_features = true;
            in_origin = false;
        } else if keyword == "ORIGIN" {
            seen_origin = true;
            in_origin = true;
            in_features = false;
        } else {
            // Any other keyword, and its continuation lines, are skipped. A
            // line in ORIGIN never reaches here (§4.7 refused it above).
            in_features = false;
            in_origin = false;
        }
    }

    // §4.5: a quoted value still open at the end of the text is refused at
    // the line of its `/name=`, as that line came first (§4.5, §4.8).
    features = end_feature(&identifier, features)?;

    if end_line.is_none() {
        // §4.8: the text ended before its terminator.
        return Err(refusal(&identifier, GenBankRule::MissingEnd, None));
    }

    // §4.8: the ORIGIN letters go to normalize unchanged, so case folding and
    // the alphabet check stay there, and so are its errors (D008).
    let sequence: Sequence = normalize(name, origin_lines.join("\n"), locus.topology)?;
    if sequence.length() != locus.length {
        return Err(refusal(
            &identifier,
            GenBankRule::LengthMismatch,
            Some(locus_number),
        ));
    }

    Ok(GenBankRecord {
        locus_name: locus.name,
        accession,
        version,
        sequence,
        features: features.done,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCategory;
    use crate::error::GenBankRule;
    use crate::error::GenBankRule::*;
    use crate::sequence::normalize;

    fn s(text: &str) -> String {
        text.to_string()
    }

    fn key(name: &str, location: &str) -> String {
        format!("     {:<16}{}", name, location)
    }

    fn q(text: &str) -> String {
        format!("{:21}{}", "", text)
    }

    fn locus(rest: &str) -> String {
        format!("LOCUS       test1                     {}", rest)
    }

    fn sample() -> Vec<String> {
        vec![
            locus("30 bp    DNA     circular SYN 17-OCT-2026"),
            s("DEFINITION  A synthetic test record."),
            s("ACCESSION   TEST0001"),
            s("VERSION     TEST0001.1"),
            s("KEYWORDS    ."),
            s("SOURCE      synthetic"),
            s("  ORGANISM  synthetic"),
            s("FEATURES             Location/Qualifiers"),
            key("source", "1..30"),
            q("/organism=\"synthetic\""),
            q("/mol_type=\"other DNA\""),
            key("CDS", "1..12"),
            q("/locus_tag=\"tst_0001\""),
            q("/codon_start=1"),
            q("/note=\"a note that wraps onto"),
            q("a second line\""),
            q("/translation=\"MAWN\""),
            key("misc_feature", "join(25..30,"),
            q("1..3)"),
            q("/label=wrap"),
            q("/pseudo"),
            s("ORIGIN"),
            s("        1 atggcttgga"),
            s("       11 atcagcagca"),
            s("       21 gcagcagcag"),
            s("//"),
        ]
    }

    fn name() -> Identifier {
        Identifier::create("tmpl".to_string()).unwrap()
    }

    fn read(lines: &[String]) -> Result<GenBankRecord, DomainError> {
        read_genbank(name(), lines.join("\n") + "\n")
    }

    fn pairs(feature: &GenBankFeature) -> Vec<(String, Option<String>)> {
        feature
            .qualifiers()
            .iter()
            .map(|qualifier| (qualifier.name(), qualifier.value()))
            .collect()
    }

    fn some(text: &str) -> Option<String> {
        Some(text.to_string())
    }

    fn refusal(found: Result<GenBankRecord, DomainError>) -> (GenBankRule, Option<u64>) {
        match found {
            Err(error) => {
                assert!(error.category() == ErrorCategory::MalformedInput);
                assert_eq!(error.code(), "malformed_genbank");
                match error {
                    DomainError::MalformedGenBank(problem) => {
                        assert_eq!(problem.sequence, "tmpl");
                        (problem.rule, problem.line)
                    }
                    other => panic!("expected MalformedGenBank, got {:?}", other),
                }
            }
            Ok(record) => panic!("accepted: {:?}", record),
        }
    }

    #[test]
    fn the_sample_gives_its_metadata_and_sequence() {
        let record = read(&sample()).unwrap();
        assert_eq!(record.locus_name(), "test1");
        assert_eq!(record.accession(), some("TEST0001"));
        assert_eq!(record.version(), some("TEST0001.1"));
        let expected = normalize(
            name(),
            "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Circular,
        )
        .unwrap();
        assert_eq!(record.sequence(), expected);
    }

    #[test]
    fn the_sample_gives_its_features_in_order() {
        let features = read(&sample()).unwrap().features();
        let keys: Vec<String> = features.iter().map(|feature| feature.key()).collect();
        assert_eq!(keys, vec![s("source"), s("CDS"), s("misc_feature")]);
        let lines: Vec<u64> = features.iter().map(|feature| feature.line()).collect();
        assert_eq!(lines, vec![9, 12, 18]);
        assert_eq!(features[0].location(), "1..30");
        assert_eq!(features[1].location(), "1..12");
        assert_eq!(features[2].location(), "join(25..30,1..3)");
        assert_eq!(
            pairs(&features[0]),
            vec![
                (s("organism"), some("synthetic")),
                (s("mol_type"), some("other DNA"))
            ]
        );
        assert_eq!(
            pairs(&features[1]),
            vec![
                (s("locus_tag"), some("tst_0001")),
                (s("codon_start"), some("1")),
                (s("note"), some("a note that wraps onto a second line")),
                (s("translation"), some("MAWN")),
            ]
        );
        assert_eq!(
            pairs(&features[2]),
            vec![(s("label"), some("wrap")), (s("pseudo"), None)]
        );
    }

    #[test]
    fn the_topology_comes_from_the_locus_line() {
        let mut lines = sample();
        lines[0] = locus("30 bp    DNA     linear   SYN 17-OCT-2026");
        assert_eq!(
            read(&lines).unwrap().sequence().topology(),
            SequenceTopology::Linear
        );
    }

    #[test]
    fn crlf_blank_lines_and_a_missing_final_newline_read_the_same() {
        let unix = read(&sample()).unwrap();
        let mut spaced: Vec<String> = vec![s(""), s("   ")];
        for line in sample() {
            spaced.push(line);
            spaced.push(s(""));
        }
        let windows = read_genbank(name(), spaced.join("\r\n")).unwrap();
        assert_eq!(unix.locus_name(), windows.locus_name());
        assert_eq!(unix.accession(), windows.accession());
        assert_eq!(unix.version(), windows.version());
        assert_eq!(unix.sequence(), windows.sequence());
        // Feature line numbers differ, because blank lines were added; every
        // other part of each feature is the same.
        type FeatureShape = Vec<(String, String, Vec<(String, Option<String>)>)>;
        let shape = |record: &GenBankRecord| -> FeatureShape {
            record
                .features()
                .iter()
                .map(|feature| (feature.key(), feature.location(), pairs(feature)))
                .collect()
        };
        assert_eq!(shape(&unix), shape(&windows));
        assert_eq!(windows.features()[1].line(), 25);
    }

    #[test]
    fn quoted_values_unescape_and_may_continue_with_a_slash() {
        let mut lines = sample();
        lines[14] = q("/note=\"say \"\"hi\"\"\"");
        lines[15] = q("/note=\"see");
        lines.insert(16, q("/path/to/thing\""));
        let features = read(&lines).unwrap().features();
        let notes: Vec<(String, Option<String>)> = pairs(&features[1])
            .into_iter()
            .filter(|(name, _)| name == "note")
            .collect();
        assert_eq!(
            notes,
            vec![
                (s("note"), some("say \"hi\"")),
                (s("note"), some("see /path/to/thing"))
            ]
        );
    }

    #[test]
    fn a_wrapped_translation_is_joined_with_a_space() {
        let mut lines = sample();
        lines[16] = q("/translation=\"MAWN");
        lines.insert(17, q("QQ\""));
        let features = read(&lines).unwrap().features();
        assert_eq!(pairs(&features[1])[3], (s("translation"), some("MAWN QQ")));
    }

    #[test]
    fn an_unquoted_value_continues_with_a_space_and_an_empty_value_is_kept() {
        let mut lines = sample();
        lines[19] = q("/label=wrap");
        lines.insert(20, q("around"));
        lines.insert(21, q("/note="));
        let features = read(&lines).unwrap().features();
        assert_eq!(
            pairs(&features[2]),
            vec![
                (s("label"), some("wrap around")),
                (s("note"), some("")),
                (s("pseudo"), None)
            ]
        );
    }

    #[test]
    fn a_record_without_features_or_accession_reads() {
        let mut lines = sample();
        lines.drain(7..21);
        lines.drain(2..4);
        let record = read(&lines).unwrap();
        assert!(record.features().is_empty());
        assert_eq!(record.accession(), None);
        assert_eq!(record.version(), None);
        assert_eq!(record.sequence().length(), 30);
    }

    #[test]
    fn an_accession_line_without_a_value_is_none_and_a_dot_is_kept() {
        let mut lines = sample();
        lines[2] = s("ACCESSION");
        lines[3] = s("VERSION     .");
        let record = read(&lines).unwrap();
        assert_eq!(record.accession(), None);
        assert_eq!(record.version(), some("."));
    }

    #[test]
    fn a_text_that_does_not_start_with_locus_is_refused() {
        assert_eq!(refusal(read_genbank(name(), s(""))), (NoLocus, None));
        assert_eq!(refusal(read_genbank(name(), s(" \n\t\n"))), (NoLocus, None));
        let mut lines = sample();
        lines.insert(0, s("DEFINITION  first"));
        assert_eq!(refusal(read(&lines)), (NoLocus, Some(1)));
        let mut lower = sample();
        lower[0] = lower[0].replace("LOCUS", "locus");
        assert_eq!(refusal(read(&lower)), (NoLocus, Some(1)));
        let mut joined = sample();
        joined[0] = s("LOCUSX      test1   30 bp    DNA     circular");
        assert_eq!(refusal(read(&joined)), (NoLocus, Some(1)));
    }

    #[test]
    fn a_locus_line_without_a_length_in_bp_is_refused() {
        for rest in [
            "DNA     circular SYN 17-OCT-2026",
            "30 aa    circular",
            "x30 bp    DNA     circular",
            "99999999999999999999 bp    DNA     circular",
        ] {
            let mut lines = sample();
            lines[0] = locus(rest);
            assert_eq!(refusal(read(&lines)), (LocusLength, Some(1)), "{}", rest);
        }
    }

    #[test]
    fn a_locus_line_without_one_topology_is_refused() {
        let mut lines = sample();
        lines[0] = locus("30 bp    DNA     SYN 17-OCT-2026");
        assert_eq!(refusal(read(&lines)), (TopologyMissing, Some(1)));
        lines[0] = locus("30 bp    DNA     Circular SYN 17-OCT-2026");
        assert_eq!(refusal(read(&lines)), (TopologyMissing, Some(1)));
        lines[0] = locus("30 bp    DNA     linear circular SYN 17-OCT-2026");
        assert_eq!(refusal(read(&lines)), (TopologyConflict, Some(1)));
        lines[0] = locus("30 bp    DNA     circular circular SYN 17-OCT-2026");
        assert_eq!(refusal(read(&lines)), (TopologyConflict, Some(1)));
    }

    #[test]
    fn a_section_that_appears_once_may_not_repeat() {
        let mut version = sample();
        version.insert(4, s("VERSION     TEST0001.2"));
        assert_eq!(refusal(read(&version)), (RepeatedSection, Some(5)));
        let mut locus_again = sample();
        locus_again.insert(21, locus("30 bp    DNA     circular"));
        assert_eq!(refusal(read(&locus_again)), (RepeatedSection, Some(22)));
        let mut features = sample();
        features.insert(21, s("FEATURES             Location/Qualifiers"));
        assert_eq!(refusal(read(&features)), (RepeatedSection, Some(22)));
    }

    #[test]
    fn a_line_that_fits_no_part_of_the_layout_is_refused() {
        let cases: Vec<(usize, String)> = vec![
            (11, s("    gene            1..3")),
            (11, s("          gene      1..3")),
            (11, s("     gene")),
            (8, q("/note=\"x\"")),
            (11, s("\tgene\t1..3")),
            (2, s("definition  lower case")),
            (2, s("1 stray")),
        ];
        for (index, line) in cases {
            let mut lines = sample();
            lines.insert(index, line.clone());
            assert_eq!(
                refusal(read(&lines)),
                (LineLayout, Some(index as u64 + 1)),
                "{:?}",
                line
            );
        }
    }

    #[test]
    fn a_malformed_qualifier_is_refused() {
        let mut unterminated = sample();
        unterminated[14] = q("/note=\"never closed");
        unterminated.remove(15);
        assert_eq!(refusal(read(&unterminated)), (QualifierSyntax, Some(15)));

        let mut trailing = sample();
        trailing[16] = q("/translation=\"MAWN\"x");
        assert_eq!(refusal(read(&trailing)), (QualifierSyntax, Some(17)));

        let cases: [(usize, &str); 4] = [
            (17, "extra"),
            (17, "/=x"),
            (17, "/bad name=x"),
            (21, "after a flag"),
        ];
        for (index, text) in cases {
            let mut lines = sample();
            lines.insert(index, q(text));
            assert_eq!(
                refusal(read(&lines)),
                (QualifierSyntax, Some(index as u64 + 1)),
                "{:?}",
                text
            );
        }

        let mut quote_in_bare = sample();
        quote_in_bare[19] = q("/label=wr\"ap");
        assert_eq!(refusal(read(&quote_in_bare)), (QualifierSyntax, Some(20)));
    }

    #[test]
    fn a_record_without_origin_is_refused() {
        let mut lines = sample();
        lines.drain(21..25);
        assert_eq!(refusal(read(&lines)), (NoOrigin, Some(22)));
    }

    #[test]
    fn an_origin_line_without_its_position_is_refused() {
        for line in [
            "          atcagcagca",
            "       1x atcagcagca",
            "BASE COUNT 10 a",
        ] {
            let mut lines = sample();
            lines[23] = s(line);
            assert_eq!(refusal(read(&lines)), (OriginLine, Some(24)), "{:?}", line);
        }
    }

    #[test]
    fn an_origin_line_numbered_wrongly_is_refused() {
        let mut lines = sample();
        lines[23] = s("       12 atcagcagca");
        assert_eq!(refusal(read(&lines)), (OriginNumbering, Some(24)));
        let mut merged = sample();
        merged[22] = s("        1 atggcttgga atcagcagca");
        assert_eq!(refusal(read(&merged)), (OriginNumbering, Some(24)));
    }

    #[test]
    fn a_locus_length_other_than_the_sequence_is_refused() {
        let mut lines = sample();
        lines[0] = locus("31 bp    DNA     circular SYN 17-OCT-2026");
        assert_eq!(refusal(read(&lines)), (LengthMismatch, Some(1)));
    }

    #[test]
    fn a_record_must_end_with_its_terminator_and_nothing_after() {
        let mut missing = sample();
        missing.pop();
        assert_eq!(refusal(read(&missing)), (MissingEnd, None));

        let mut second = sample();
        second.push(locus("30 bp    DNA     circular"));
        assert_eq!(refusal(read(&second)), (SecondRecord, Some(27)));

        let mut junk = sample();
        junk.push(s(""));
        junk.push(s("junk"));
        assert_eq!(refusal(read(&junk)), (TextAfterEnd, Some(28)));

        let mut blank = sample();
        blank.push(s(""));
        blank.push(s("   "));
        assert!(read(&blank).is_ok());
    }

    #[test]
    fn the_origin_letters_go_through_normalize() {
        let mut lines = sample();
        lines[24] = s("       21 gcagcagcax");
        match read(&lines).unwrap_err() {
            DomainError::InvalidSequenceCharacters(problem) => {
                assert_eq!(problem.position, 29);
                assert_eq!(problem.characters, "X");
            }
            other => panic!("expected InvalidSequenceCharacters, got {:?}", other),
        }
        let mut empty = sample();
        empty.drain(22..25);
        empty[0] = locus("0 bp    DNA     circular");
        assert!(matches!(
            read(&empty).unwrap_err(),
            DomainError::EmptySequenceContent(_)
        ));
    }

    // Added in review: ORIGIN runs to `//`, so a section line inside it is
    // an ORIGIN line that is not one, whether or not its keyword was seen.
    // `drain(2..4)` removes ACCESSION and VERSION, so ORIGIN is index 19
    // and index 21 sits between its first and second sequence lines.
    #[test]
    fn a_section_line_inside_origin_is_an_origin_line() {
        for line in [
            "ACCESSION   TEST0002",
            "VERSION     TEST0002.1",
            "FEATURES             Location/Qualifiers",
            "ORIGIN",
            "LOCUS       x 30 bp DNA circular",
        ] {
            let mut lines = sample();
            lines.drain(2..4);
            lines.insert(21, s(line));
            assert_eq!(refusal(read(&lines)), (OriginLine, Some(22)), "{:?}", line);
        }
    }

    // Added in review (A-1): an ORIGIN line's bases never reach `normalize`'s
    // comment-line rule, so a `;` in any base token is an ORIGIN line that is
    // not one, refused at that line's number.
    #[test]
    fn an_origin_line_holding_a_semicolon_is_refused() {
        let mut comment = sample();
        comment[0] = locus("20 bp    DNA circular SYN 17-OCT-2026");
        comment[23] = s("       11 ;tcagcagca");
        assert_eq!(refusal(read(&comment)), (OriginLine, Some(24)));

        let mut inner = sample();
        inner[23] = s("       11 atcag;agca");
        assert_eq!(refusal(read(&inner)), (OriginLine, Some(24)));
    }

    // Added in review (A-2): a qualifier name may hold no whitespace, not only
    // no space.
    #[test]
    fn a_qualifier_name_may_not_hold_a_tab() {
        let mut tabbed = sample();
        tabbed[20] = q("/pseudo\t");
        assert_eq!(refusal(read(&tabbed)), (QualifierSyntax, Some(21)));
    }

    // Added in review (A-4): the topology tokens are the ones after the `bp`
    // token, so a LOCUS name that reads `linear` or `circular` states nothing
    // about topology (D009, D041 G4).
    #[test]
    fn a_locus_topology_token_before_the_bp_token_is_not_a_topology() {
        let mut unnamed_topology = sample();
        unnamed_topology[0] = s("LOCUS       linear   30 bp    DNA     SYN 17-OCT-2026");
        assert_eq!(refusal(read(&unnamed_topology)), (TopologyMissing, Some(1)));

        let mut named_topology = sample();
        named_topology[0] = s("LOCUS       circular   30 bp    DNA     linear   SYN 17-OCT-2026");
        let record = read(&named_topology).unwrap();
        assert_eq!(record.locus_name(), "circular");
        assert_eq!(record.sequence().topology(), SequenceTopology::Linear);
    }

    // Added in review (A-5): inside a closed quoted value every `"` must be
    // part of a `""` pair, read left to right; a lone `"` is refused.
    #[test]
    fn a_closed_quoted_value_may_not_hold_a_lone_inner_quote() {
        let mut lone = sample();
        lone[14] = q("/note=\"a\" \"b\"");
        lone.remove(15);
        assert_eq!(refusal(read(&lone)), (QualifierSyntax, Some(15)));

        let mut doubled = sample();
        doubled[14] = q("/note=\"say \"\"hi\"\"\"");
        doubled.remove(15);
        let features = read(&doubled).unwrap().features();
        assert_eq!(
            pairs(&features[1]),
            vec![
                (s("locus_tag"), some("tst_0001")),
                (s("codon_start"), some("1")),
                (s("note"), some("say \"hi\"")),
                (s("translation"), some("MAWN")),
            ]
        );
    }

    // Added in review: `/label=wrap` is at index 19, so its continuation is
    // line 21.
    #[test]
    fn an_unquoted_continuation_may_not_hold_a_quote() {
        let mut lines = sample();
        lines.insert(20, q("ar\"ound"));
        assert_eq!(refusal(read(&lines)), (QualifierSyntax, Some(21)));
    }

    #[test]
    fn each_message_names_the_line_when_there_is_one() {
        assert_eq!(
            read_genbank(name(), s("")).unwrap_err().message(),
            "GenBank text for 'tmpl': it does not start with a LOCUS line"
        );
        let mut lines = sample();
        lines[0] = locus("31 bp    DNA     circular");
        assert_eq!(
            read(&lines).unwrap_err().message(),
            "GenBank text for 'tmpl': the LOCUS line's length differs from the ORIGIN sequence's (line 1)"
        );
    }
}
