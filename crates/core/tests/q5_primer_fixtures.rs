//! Integration tests driven by the fixtures in `tests/fixtures/q5-primer`.
//!
//! The reference designs in `reference/designs.json` were produced by
//! `reference/neb-q5-sdm-reference.py`, an independent transcription of the
//! reference tool's primer construction. These tests feed each reference design
//! through `primer.rs` and check that luria derives the same quantities the
//! generator recorded: the decomposition, the annealing overlap, the
//! whole-primer match count, the pair relationship, and the H1 outcome.
//!
//! The reference designs are inputs to the validation layer, not expected
//! luria output. luria's own designs (D029, luriaNative) are checked separately
//! against `native.tsv`, written by `reference/native-design.py`, which restates
//! D029's closed form without sharing code with this crate. Where a native row
//! names a reference design, the native forward primer must equal the
//! reference's: D029 and the reference tool agree whenever the reference
//! tool's Tm extension does not fire.
//!
//! Every reference edit replaces one three-base codon. The mutation layer
//! represents that as a residue substitution, so each edit is applied through a
//! one-codon CDS over exactly the edited codon, with the replacement codon
//! supplied. A complete CDS needs no start or stop codon, so this states the
//! edit without assuming a reading frame for the rest of the template.
//!
//! Panic messages name the case and the field. They do not print template or
//! primer text; run the generator script to inspect those.

use luria_core::cds::Cds;
use luria_core::cds::Completeness;
use luria_core::cds::Numbering;
use luria_core::cds::ResiduePosition;
use luria_core::error::DesignObstacle;
use luria_core::error::DomainError;
use luria_core::genetic_code::lookup;
use luria_core::genetic_code::CodonOutcome;
use luria_core::identity::Identifier;
use luria_core::interval::create;
use luria_core::interval::create_point;
use luria_core::interval::NucleotideInterval;
use luria_core::mutation::apply_deletion;
use luria_core::mutation::apply_insertion;
use luria_core::mutation::apply_nucleotide;
use luria_core::mutation::apply_replacement;
use luria_core::mutation::apply_residue;
use luria_core::mutation::verify;
use luria_core::mutation::Codon;
use luria_core::mutation::CodonSelectionPolicy;
use luria_core::mutation::MutationResult;
use luria_core::mutation::NucleotideDeletionIntent;
use luria_core::mutation::NucleotideInsertionIntent;
use luria_core::mutation::NucleotideMutationIntent;
use luria_core::mutation::NucleotideReplacementIntent;
use luria_core::mutation::ResidueMutationIntent;
use luria_core::primer::assemble_pair;
use luria_core::primer::construct_primer;
use luria_core::primer::require_design_constraints;
use luria_core::primer::Primer;
use luria_core::primer::PrimerPair;
use luria_core::primer::PrimerPlacement;
use luria_core::primer::PrimerRole;
use luria_core::primer_design::design_primers;
use luria_core::primer_design::DesignPolicy;
use luria_core::primer_design::DesignStyle;
use luria_core::primer_design::PrimerDesign;
use luria_core::primer_design::PrimerDesignRequest;
use luria_core::primer_design::MAX_WHOLE_TAIL;
use luria_core::primer_design::TA_CAP;
use luria_core::primer_design::TA_OFFSET;
use luria_core::primer_design::TM_FLOOR;
use luria_core::primer_quality::PrimerWarning;
use luria_core::sequence::normalize;
use luria_core::sequence::reverse_complement;
use luria_core::sequence::Sequence;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;
use luria_core::tm::melting_temperature;
use luria_core::tm::BufferModel;
use luria_core::tm::MeltingTemperature;
use luria_core::tm::TmBasis;

const FIXTURE_DIR: &str = "../../tests/fixtures/q5-primer";

/// The number of reference designs: q5p-01, -02, -03, -04, and q5p-07's two
/// variants.
const REFERENCE_DESIGN_COUNT: usize = 6;

// ---------------------------------------------------------------------------
// A minimal JSON reader. The crate has no dependencies, and this test needs
// only objects, arrays, strings, numbers, booleans and null.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn opt(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn get(&self, key: &str) -> &Json {
        self.opt(key)
            .unwrap_or_else(|| panic!("JSON object has no key {:?}", key))
    }

    fn fields(&self) -> &[(String, Json)] {
        match self {
            Json::Object(fields) => fields,
            other => panic!("expected a JSON object, found {:?}", kind(other)),
        }
    }

    fn items(&self) -> &[Json] {
        match self {
            Json::Array(items) => items,
            other => panic!("expected a JSON array, found {:?}", kind(other)),
        }
    }

    fn text(&self) -> &str {
        match self {
            Json::Text(text) => text,
            other => panic!("expected a JSON string, found {:?}", kind(other)),
        }
    }

    fn flag(&self) -> bool {
        match self {
            Json::Bool(flag) => *flag,
            other => panic!("expected a JSON boolean, found {:?}", kind(other)),
        }
    }

    fn integer(&self) -> i64 {
        match self {
            Json::Number(value) if value.fract() == 0.0 => *value as i64,
            other => panic!("expected a JSON integer, found {:?}", other),
        }
    }

    /// A JSON number that may carry a fraction, such as a recorded Tm.
    fn decimal(&self) -> f64 {
        match self {
            Json::Number(value) => *value,
            other => panic!("expected a JSON number, found {:?}", other),
        }
    }

    fn count(&self) -> u64 {
        let value: i64 = self.integer();
        assert!(
            value >= 0,
            "expected a non-negative integer, found {}",
            value
        );
        value as u64
    }
}

fn kind(value: &Json) -> &'static str {
    match value {
        Json::Null => "null",
        Json::Bool(_) => "boolean",
        Json::Number(_) => "number",
        Json::Text(_) => "string",
        Json::Array(_) => "array",
        Json::Object(_) => "object",
    }
}

struct Reader {
    chars: Vec<char>,
    at: usize,
}

impl Reader {
    fn parse(text: &str) -> Json {
        let mut reader = Reader {
            chars: text.chars().collect(),
            at: 0,
        };
        let value: Json = reader.value();
        reader.space();
        assert!(
            reader.at == reader.chars.len(),
            "trailing characters after JSON value at offset {}",
            reader.at
        );
        value
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn next(&mut self) -> char {
        let ch: char = self
            .peek()
            .unwrap_or_else(|| panic!("unexpected end of JSON at offset {}", self.at));
        self.at += 1;
        ch
    }

    fn expect(&mut self, wanted: char) {
        let found: char = self.next();
        assert!(
            found == wanted,
            "expected {:?} at offset {}, found {:?}",
            wanted,
            self.at - 1,
            found
        );
    }

    fn space(&mut self) {
        while matches!(self.peek(), Some(' ' | '\n' | '\r' | '\t')) {
            self.at += 1;
        }
    }

    fn value(&mut self) -> Json {
        self.space();
        match self.peek() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => Json::Text(self.string()),
            Some('t') => self.literal("true", Json::Bool(true)),
            Some('f') => self.literal("false", Json::Bool(false)),
            Some('n') => self.literal("null", Json::Null),
            Some(ch) if ch == '-' || ch == '.' || ch.is_ascii_digit() => self.number(),
            other => panic!("unexpected {:?} at offset {}", other, self.at),
        }
    }

    fn object(&mut self) -> Json {
        self.expect('{');
        let mut fields: Vec<(String, Json)> = Vec::new();
        self.space();
        if self.peek() == Some('}') {
            self.at += 1;
            return Json::Object(fields);
        }
        loop {
            self.space();
            let key: String = self.string();
            self.space();
            self.expect(':');
            let value: Json = self.value();
            fields.push((key, value));
            self.space();
            match self.next() {
                ',' => continue,
                '}' => return Json::Object(fields),
                other => panic!(
                    "expected ',' or '}}' at offset {}, found {:?}",
                    self.at - 1,
                    other
                ),
            }
        }
    }

    fn array(&mut self) -> Json {
        self.expect('[');
        let mut items: Vec<Json> = Vec::new();
        self.space();
        if self.peek() == Some(']') {
            self.at += 1;
            return Json::Array(items);
        }
        loop {
            items.push(self.value());
            self.space();
            match self.next() {
                ',' => continue,
                ']' => return Json::Array(items),
                other => panic!(
                    "expected ',' or ']' at offset {}, found {:?}",
                    self.at - 1,
                    other
                ),
            }
        }
    }

    fn string(&mut self) -> String {
        self.expect('"');
        let mut out: String = String::new();
        loop {
            match self.next() {
                '"' => return out,
                '\\' => {
                    let escaped: char = match self.next() {
                        '"' => '"',
                        '\\' => '\\',
                        '/' => '/',
                        'b' => '\u{8}',
                        'f' => '\u{c}',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'u' => self.unicode(),
                        other => panic!("unknown JSON escape \\{} at offset {}", other, self.at),
                    };
                    out.push(escaped);
                }
                ch => out.push(ch),
            }
        }
    }

    // A \uXXXX escape, including a UTF-16 surrogate pair.
    fn unicode(&mut self) -> char {
        let first: u32 = self.hex4();
        let code: u32 = if (0xD800..0xDC00).contains(&first) {
            self.expect('\\');
            self.expect('u');
            let second: u32 = self.hex4();
            0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00)
        } else {
            first
        };
        char::from_u32(code).unwrap_or_else(|| panic!("invalid code point {:#x}", code))
    }

    fn hex4(&mut self) -> u32 {
        let mut value: u32 = 0;
        for _ in 0..4 {
            let digit: u32 = self
                .next()
                .to_digit(16)
                .unwrap_or_else(|| panic!("bad \\u escape at offset {}", self.at - 1));
            value = value * 16 + digit;
        }
        value
    }

    fn number(&mut self) -> Json {
        let start: usize = self.at;
        while matches!(self.peek(), Some(ch) if ch.is_ascii_digit() || "+-.eE".contains(ch)) {
            self.at += 1;
        }
        let text: String = self.chars[start..self.at].iter().collect();
        Json::Number(
            text.parse::<f64>()
                .unwrap_or_else(|_| panic!("bad JSON number {:?} at offset {}", text, start)),
        )
    }

    fn literal(&mut self, word: &str, value: Json) -> Json {
        for wanted in word.chars() {
            self.expect(wanted);
        }
        value
    }
}

// ---------------------------------------------------------------------------
// Fixture loading.
// ---------------------------------------------------------------------------

fn read(relative: &str) -> String {
    let path: String = format!("{}/{}", FIXTURE_DIR, relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

/// One row of `cases.tsv`, all 14 columns. The three descriptive columns are
/// carried as `_strand`, `_cds` and `_note`: no test reads them, and the leading
/// underscore is how the compiler records that (`docs/tasks/review-c4-fixture-columns.md`
/// §2.1).
struct CaseRow {
    case_id: String,
    kind: String,
    sequence_file: String,
    sequence_digest_16: String,
    edits: String,
    min_overlap: String,
    target_tm: String,
    _strand: String,
    _cds: String,
    reference_design: String,
    expected: String,
    invariants: String,
    reference_check: String,
    _note: String,
}

fn case_rows() -> Vec<CaseRow> {
    let text: String = read("cases.tsv");
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header: Vec<&str> = lines
        .next()
        .expect("cases.tsv has a header")
        .split('\t')
        .collect();
    lines
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert!(
                fields.len() == header.len(),
                "cases.tsv row has {} fields, header has {}",
                fields.len(),
                header.len()
            );
            let field = |name: &str| -> String {
                let index: usize = header
                    .iter()
                    .position(|column| *column == name)
                    .unwrap_or_else(|| panic!("cases.tsv has no column {:?}", name));
                fields[index].to_string()
            };
            CaseRow {
                case_id: field("case_id"),
                kind: field("kind"),
                sequence_file: field("sequence_file"),
                sequence_digest_16: field("sequence_digest_16"),
                edits: field("edits"),
                min_overlap: field("min_overlap"),
                target_tm: field("target_tm"),
                _strand: field("strand"),
                _cds: field("cds"),
                reference_design: field("reference_design"),
                expected: field("expected"),
                invariants: field("invariants"),
                reference_check: field("reference_check"),
                _note: field("note"),
            }
        })
        .collect()
}

/// `cases.tsv` names a case `q5p-NN-description`; `designs.json` keys it by
/// the `q5p-NN` prefix alone.
fn short_id(case_id: &str) -> String {
    case_id
        .splitn(3, '-')
        .take(2)
        .collect::<Vec<&str>>()
        .join("-")
}

fn case_row(case_id: &str) -> CaseRow {
    let rows: Vec<CaseRow> = case_rows()
        .into_iter()
        .filter(|row| short_id(&row.case_id) == case_id)
        .collect();
    assert!(
        rows.len() == 1,
        "cases.tsv has {} rows for {}",
        rows.len(),
        case_id
    );
    rows.into_iter().next().unwrap()
}

/// The template a case names. The SID-1 file is FASTA, whose `>` header line
/// `normalize` does not skip (it skips only `;` comment lines), so the header
/// is dropped here; the remaining text is normalized as usual.
fn template_of(row: &CaseRow) -> Sequence {
    template_from(&row.case_id, &row.sequence_file)
}

fn template_from(label: &str, sequence_file: &str) -> Sequence {
    let raw: String = read(sequence_file);
    let body: String = raw
        .lines()
        .filter(|line| !line.starts_with('>'))
        .collect::<Vec<&str>>()
        .join("\n");
    normalize(
        Identifier::create("q5p-template".to_string()).unwrap(),
        body,
        SequenceTopology::Linear,
    )
    .unwrap_or_else(|error| panic!("{}: template does not normalize: {:?}", label, error))
}

/// One reference design, labelled by its case and, for q5p-07, its variant.
struct Design {
    case_id: String,
    label: String,
    record: Json,
}

fn designs_json() -> Json {
    Reader::parse(&read("reference/designs.json"))
}

/// q5p-01..03 store the design directly, q5p-04 under `design`, and q5p-07
/// one design per `variant-*` key.
fn reference_designs() -> Vec<Design> {
    let mut out: Vec<Design> = Vec::new();
    for (case_id, value) in designs_json().fields() {
        if value.opt("forward").is_some() {
            out.push(Design {
                case_id: case_id.clone(),
                label: case_id.clone(),
                record: value.clone(),
            });
        } else if let Some(design) = value.opt("design") {
            out.push(Design {
                case_id: case_id.clone(),
                label: case_id.clone(),
                record: design.clone(),
            });
        } else {
            let variants: Vec<&(String, Json)> = value
                .fields()
                .iter()
                .filter(|(key, _)| key.starts_with("variant-"))
                .collect();
            assert!(!variants.is_empty(), "{}: no design record found", case_id);
            for (key, design) in variants {
                out.push(Design {
                    case_id: case_id.clone(),
                    label: format!("{}/{}", case_id, key),
                    record: design.clone(),
                });
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Rebuilding a reference design through the mutation and primer layers.
// ---------------------------------------------------------------------------

fn residue_symbol(label: &str, field: &str, codon: &str) -> String {
    match lookup(codon.to_uppercase()) {
        CodonOutcome::Residue(residue) => residue.symbol(),
        other => panic!(
            "{}: edit.{} is not a sense codon ({:?})",
            label, field, other
        ),
    }
}

/// The reference edit as a `MutationResult`: a one-codon CDS over the edited
/// codon, residue 1, the replacement codon supplied.
fn edit_of(design: &Design, template: &Sequence) -> MutationResult {
    let edit: &Json = design.record.get("edit");
    let start: i64 = edit.get("start").integer();
    let end: i64 = edit.get("end").integer();
    let original: String = edit.get("original").text().to_uppercase();
    let replacement: String = edit.get("replacement").text().to_uppercase();
    assert!(end - start == 3, "{}: edit is not one codon", design.label);

    let cds_id: Identifier = Identifier::create("edited-codon".to_string()).unwrap();
    let cds: Cds = Cds::create(
        template,
        cds_id.clone(),
        vec![create(&template.reference(), start, end).unwrap()],
        Strand::Plus,
        Completeness::Complete,
        "standard".to_string(),
        Numbering::FromCdsStart,
    )
    .unwrap_or_else(|error| panic!("{}: one-codon CDS rejected: {:?}", design.label, error));
    let intent: ResidueMutationIntent = ResidueMutationIntent::create(
        ResiduePosition::create(cds_id, 1).unwrap(),
        residue_symbol(&design.label, "original", &original),
        residue_symbol(&design.label, "replacement", &replacement),
        CodonSelectionPolicy::Supplied(Codon::create(replacement.clone()).unwrap()),
    )
    .unwrap();
    apply_residue(
        template,
        cds,
        Identifier::create("q5p-edited".to_string()).unwrap(),
        intent,
    )
    .unwrap_or_else(|error| {
        panic!(
            "{}: edit rejected by the mutation layer: {:?}",
            design.label, error
        )
    })
}

fn span_of(design: &Design, template: &Sequence, primer: &Json) -> NucleotideInterval {
    let span: &[Json] = primer.get("span").items();
    create(&template.reference(), span[0].integer(), span[1].integer())
        .unwrap_or_else(|error| panic!("{}: bad primer span: {:?}", design.label, error))
}

fn primer_of(
    design: &Design,
    template: &Sequence,
    result: &MutationResult,
    role: PrimerRole,
    key: &str,
) -> Primer {
    let record: &Json = design.record.get(key);
    construct_primer(
        template,
        result.clone(),
        role,
        PrimerPlacement::Inline,
        record.get("sequence").text().to_string(),
        span_of(design, template, record),
    )
    .unwrap_or_else(|error| panic!("{}: {} primer rejected: {:?}", design.label, key, error))
}

struct Rebuilt {
    forward: Primer,
    reverse: Primer,
    pair: PrimerPair,
}

fn rebuild(design: &Design) -> Rebuilt {
    let template: Sequence = template_of(&case_row(&design.case_id));
    let result: MutationResult = edit_of(design, &template);
    let forward: Primer = primer_of(design, &template, &result, PrimerRole::Forward, "forward");
    let reverse: Primer = primer_of(design, &template, &result, PrimerRole::Reverse, "reverse");
    let pair: PrimerPair = assemble_pair(forward.clone(), reverse.clone(), result)
        .unwrap_or_else(|error| panic!("{}: pair rejected: {:?}", design.label, error));
    Rebuilt {
        forward,
        reverse,
        pair,
    }
}

fn integers(value: &Json) -> Vec<u64> {
    value.items().iter().map(Json::count).collect()
}

fn span_pairs(value: &Json) -> Vec<(u64, u64)> {
    value
        .items()
        .iter()
        .map(|span| {
            let ends: &[Json] = span.items();
            (ends[0].count(), ends[1].count())
        })
        .collect()
}

fn check_primer(label: &str, key: &str, primer: Primer, record: &Json) {
    let field = |name: &str| format!("{}: {}.{}", label, key, name);
    // Text fields are compared without printing them.
    assert!(
        primer.clone().sequence() == record.get("sequence").text(),
        "{} differs",
        field("sequence")
    );
    assert!(
        primer.clone().annealing_sequence() == record.get("annealing").get("sequence").text(),
        "{} differs",
        field("annealing.sequence")
    );
    let pairs: Vec<(&str, u64, u64)> = vec![
        (
            "length",
            primer.clone().length(),
            record.get("length").count(),
        ),
        (
            "annealing.length",
            primer.clone().annealing_length(),
            record.get("annealing").get("length").count(),
        ),
        (
            "annealingOverlap",
            primer.clone().annealing_overlap(),
            record.get("annealingOverlap").count(),
        ),
        (
            "wholePrimerMatchCount",
            primer.clone().whole_primer_match_count(),
            record.get("wholePrimerMatchCount").count(),
        ),
        (
            "parts (count)",
            primer.clone().parts().len() as u64,
            record.get("parts").items().len() as u64,
        ),
    ];
    for (name, luria, reference) in pairs {
        assert!(
            luria == reference,
            "{}: luria {} vs reference {}",
            field(name),
            luria,
            reference
        );
    }
    let one_based: Vec<u64> = integers(record.get("oneBasedSpan"));
    let (first, last) = primer.clone().one_based_span();
    assert!(
        vec![first, last] == one_based,
        "{}: luria {:?} vs reference {:?}",
        field("oneBasedSpan"),
        (first, last),
        one_based
    );
    let changed: Vec<u64> = integers(record.get("changedPositions"));
    assert!(
        primer.clone().changed_positions() == changed,
        "{}: luria {:?} vs reference {:?}",
        field("changedPositions"),
        primer.clone().changed_positions(),
        changed
    );
    let luria_spans: Vec<(u64, u64)> = primer
        .clone()
        .annealing_spans()
        .iter()
        .map(|span| (span.start(), span.end()))
        .collect();
    let reference_spans: Vec<(u64, u64)> = span_pairs(record.get("annealing").get("spans"));
    assert!(
        luria_spans == reference_spans,
        "{}: luria {:?} vs reference {:?}",
        field("annealing.spans"),
        luria_spans,
        reference_spans
    );
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[test]
fn the_json_reader_handles_every_value_kind_and_escape() {
    let parsed: Json = Reader::parse(
        r#" { "a" : "q\"b\\s\/\b\f\n\r\t\u0041\ud83d\ude00", "b" : [1, -2.5e1, true, false, null, {}, []] } "#,
    );
    assert!(parsed.get("a").text() == "q\"b\\s/\u{8}\u{c}\n\r\tA\u{1F600}");
    let items: &[Json] = parsed.get("b").items();
    assert!(items[0] == Json::Number(1.0));
    assert!(items[1] == Json::Number(-25.0));
    assert!(items[2..5] == [Json::Bool(true), Json::Bool(false), Json::Null]);
    assert!(items[5] == Json::Object(Vec::new()));
    assert!(items[6] == Json::Array(Vec::new()));
}

#[test]
fn every_reference_design_case_has_a_record_and_no_record_lacks_a_case() {
    let flagged: Vec<String> = case_rows()
        .into_iter()
        .filter(|row| row.reference_design == "yes")
        .map(|row| short_id(&row.case_id))
        .collect();
    let recorded: Vec<String> = designs_json()
        .fields()
        .iter()
        .map(|(case_id, _)| case_id.clone())
        .collect();
    assert!(
        flagged == recorded,
        "cases.tsv flags {:?}, designs.json records {:?}",
        flagged,
        recorded
    );
    assert!(reference_designs().len() == REFERENCE_DESIGN_COUNT);
}

#[test]
fn each_reference_design_is_built_against_the_template_its_case_names() {
    for design in reference_designs() {
        let row: CaseRow = case_row(&design.case_id);
        let digest: String = template_of(&row).content_hash().hex_digest()[..16].to_string();
        assert!(
            digest == row.sequence_digest_16,
            "{}: template digest {} vs cases.tsv {}",
            design.label,
            digest,
            row.sequence_digest_16
        );
        let built_against: &str = design.record.get("builtAgainst").text();
        assert!(
            digest == built_against,
            "{}: template digest {} vs builtAgainst {}",
            design.label,
            digest,
            built_against
        );
    }
}

#[test]
fn each_reference_edit_is_reproduced_by_the_mutation_layer() {
    for design in reference_designs() {
        let template: Sequence = template_of(&case_row(&design.case_id));
        let result: MutationResult = edit_of(&design, &template);
        let edit: &Json = design.record.get("edit");
        let start: usize = edit.get("start").count() as usize;
        let end: usize = edit.get("end").count() as usize;
        let edited: String = result.edited().canonical_text();
        assert!(
            edited[start..end] == edit.get("replacement").text().to_uppercase(),
            "{}: edited codon differs from edit.replacement",
            design.label
        );
        assert!(
            result.target_coordinates() == (start as u64..end as u64).collect::<Vec<u64>>(),
            "{}: the result claims {:?}",
            design.label,
            result.target_coordinates()
        );
    }
}

#[test]
fn luria_derives_the_same_primer_decomposition_as_each_reference_design() {
    for design in reference_designs() {
        let rebuilt: Rebuilt = rebuild(&design);
        check_primer(
            &design.label,
            "forward",
            rebuilt.forward,
            design.record.get("forward"),
        );
        check_primer(
            &design.label,
            "reverse",
            rebuilt.reverse,
            design.record.get("reverse"),
        );
    }
}

#[test]
fn luria_derives_the_same_pair_relationship_as_each_reference_design() {
    for design in reference_designs() {
        let pair: PrimerPair = rebuild(&design).pair;
        let relationship: &Json = design.record.get("relationship");
        assert!(
            pair.clone().abuts() == relationship.get("abut").flag(),
            "{}: relationship.abut: luria {}",
            design.label,
            pair.clone().abuts()
        );
        let boundary: u64 = relationship.get("boundary").count();
        assert!(
            pair.clone().boundary() == Some(boundary),
            "{}: relationship.boundary: luria {:?} vs reference {}",
            design.label,
            pair.clone().boundary(),
            boundary
        );
        let gap: i64 = relationship.get("gap").integer();
        assert!(
            pair.clone().separation() == gap,
            "{}: relationship.gap: luria {} vs reference {}",
            design.label,
            pair.clone().separation(),
            gap
        );
    }
}

#[test]
fn luria_reaches_the_same_h1_outcome_as_each_reference_design() {
    for design in reference_designs() {
        let h1: &Json = design.record.get("h1");
        let bound: u64 = h1.get("minAnnealingOverlapSupplied").count();
        assert!(
            bound
                == design
                    .record
                    .get("request")
                    .get("minOverlapSupplied")
                    .count(),
            "{}: h1 bound differs from request.minOverlapSupplied",
            design.label
        );
        let outcome: Result<(), DomainError> =
            require_design_constraints(rebuild(&design).pair, bound);
        let satisfied: bool = h1.get("satisfied").flag();
        match outcome {
            Ok(()) => assert!(
                satisfied,
                "{}: luria satisfies H1, reference does not",
                design.label
            ),
            Err(DomainError::AnnealingOverlapBelowBound(_)) => assert!(
                !satisfied,
                "{}: luria fails H1, reference satisfies it",
                design.label
            ),
            Err(other) => panic!("{}: unexpected H1 error {:?}", design.label, other),
        }
    }
}

#[test]
fn the_separate_variants_disjointness_claim_follows_from_their_spans() {
    let json: Json = designs_json();
    let case: &Json = json.get("q5p-07");
    let claimed: bool = case.get("spansDisjoint").flag();
    let covered: Vec<(u64, u64)> = reference_designs()
        .into_iter()
        .filter(|design| design.case_id == "q5p-07")
        .map(|design| {
            let rebuilt: Rebuilt = rebuild(&design);
            let (forward, reverse) = (rebuilt.forward.span(), rebuilt.reverse.span());
            (
                forward.start().min(reverse.start()),
                forward.end().max(reverse.end()),
            )
        })
        .collect();
    assert!(covered.len() == 2, "q5p-07 has {} variants", covered.len());
    let disjoint: bool = covered[0].1 <= covered[1].0 || covered[1].1 <= covered[0].0;
    assert!(
        disjoint == claimed,
        "q5p-07: spansDisjoint is {}, the pair spans {:?} give {}",
        claimed,
        covered,
        disjoint
    );
}

// ---------------------------------------------------------------------------
// luriaNative designs (D029) against `native.tsv`.
// ---------------------------------------------------------------------------

/// The number of rows in `native.tsv`.
const NATIVE_CASE_COUNT: usize = 8;

struct NativeRow {
    case_id: String,
    sequence_file: String,
    start: i64,
    end: i64,
    original: String,
    replacement: String,
    min_overlap: i64,
    expected: String,
    forward_span: String,
    reverse_span: String,
    flanks: String,
    reverse_length: String,
    forward_tm: String,
    forward_tm_basis: String,
    reverse_tm: String,
    annealing_temp: String,
    balance_met: String,
    reference_design: String,
    failure: String,
}

fn native_rows() -> Vec<NativeRow> {
    let text: String = read("native.tsv");
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header: Vec<&str> = lines
        .next()
        .expect("native.tsv has a header")
        .split('\t')
        .collect();
    lines
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert!(
                fields.len() == header.len(),
                "native.tsv row has {} fields, header has {}",
                fields.len(),
                header.len()
            );
            let field = |name: &str| -> String {
                let index: usize = header
                    .iter()
                    .position(|column| *column == name)
                    .unwrap_or_else(|| panic!("native.tsv has no column {:?}", name));
                fields[index].to_string()
            };
            let edit: Vec<String> = field("edit").split(':').map(str::to_string).collect();
            assert!(
                edit.len() == 4,
                "{}: edit is not start:end:original:replacement",
                field("case_id")
            );
            NativeRow {
                case_id: field("case_id"),
                sequence_file: field("sequence_file"),
                start: edit[0].parse().unwrap(),
                end: edit[1].parse().unwrap(),
                original: edit[2].clone(),
                replacement: edit[3].clone(),
                min_overlap: field("min_overlap").parse().unwrap(),
                expected: field("expected"),
                forward_span: field("forward_span"),
                reverse_span: field("reverse_span"),
                flanks: field("flanks"),
                reverse_length: field("reverse_length"),
                forward_tm: field("forward_tm"),
                forward_tm_basis: field("forward_tm_basis"),
                reverse_tm: field("reverse_tm"),
                annealing_temp: field("annealing_temp"),
                balance_met: field("balance_met"),
                reference_design: field("reference_design"),
                failure: field("failure"),
            }
        })
        .collect()
}

/// The row's edit as a verified `MutationResult`: a nucleotide substitution for
/// one base, a residue substitution on a one-codon CDS for three.
fn native_edit(row: &NativeRow, template: &Sequence) -> MutationResult {
    let edited: Identifier = Identifier::create("native-edited".to_string()).unwrap();
    let (made, cds): (MutationResult, Option<Cds>) = match row.end - row.start {
        1 => {
            let intent: NucleotideMutationIntent = NucleotideMutationIntent::create(
                template.reference(),
                create(&template.reference(), row.start, row.end).unwrap(),
                row.original.clone(),
                row.replacement.clone(),
            )
            .unwrap();
            let made = apply_nucleotide(template, None, edited, intent)
                .unwrap_or_else(|error| panic!("{}: edit rejected: {:?}", row.case_id, error));
            (made, None)
        }
        3 => {
            let cds_id: Identifier = Identifier::create("edited-codon".to_string()).unwrap();
            let cds: Cds = Cds::create(
                template,
                cds_id.clone(),
                vec![create(&template.reference(), row.start, row.end).unwrap()],
                Strand::Plus,
                Completeness::Complete,
                "standard".to_string(),
                Numbering::FromCdsStart,
            )
            .unwrap_or_else(|error| panic!("{}: one-codon CDS rejected: {:?}", row.case_id, error));
            let intent: ResidueMutationIntent = ResidueMutationIntent::create(
                ResiduePosition::create(cds_id, 1).unwrap(),
                residue_symbol(&row.case_id, "original", &row.original),
                residue_symbol(&row.case_id, "replacement", &row.replacement),
                CodonSelectionPolicy::Supplied(Codon::create(row.replacement.clone()).unwrap()),
            )
            .unwrap();
            let made = apply_residue(template, cds.clone(), edited, intent)
                .unwrap_or_else(|error| panic!("{}: edit rejected: {:?}", row.case_id, error));
            (made, Some(cds))
        }
        other => panic!("{}: a {}-base edit has no row form", row.case_id, other),
    };
    verify(template, cds, made)
        .unwrap_or_else(|error| panic!("{}: edit does not verify: {:?}", row.case_id, error))
}

fn native_design(row: &NativeRow) -> Result<PrimerDesign, DomainError> {
    let template: Sequence = template_from(&row.case_id, &row.sequence_file);
    let request: PrimerDesignRequest = PrimerDesignRequest::create(
        DesignStyle::Q5Polymerase,
        DesignPolicy::LuriaNative,
        row.min_overlap,
        PrimerPlacement::Inline,
        BufferModel::q5_reaction(),
    )
    .unwrap();
    design_primers(&template, native_edit(row, &template), request)
}

fn span_text(span: NucleotideInterval) -> String {
    format!("{}:{}", span.start(), span.end())
}

/// `5':20<25` or `3':4<10`, one per side, space separated; a Tm-floor
/// failure is `tm-floor 5':<available>:<reached °C to two decimals>`.
fn obstacle_text(obstacle: &DesignObstacle) -> String {
    match obstacle {
        DesignObstacle::FivePrimeAvailability { available, needed } => {
            format!("5':{}<{}", available, needed)
        }
        DesignObstacle::ThreePrimeAvailability { available, needed } => {
            format!("3':{}<{}", available, needed)
        }
        DesignObstacle::AmbiguousTemplateBase { position, base } => {
            format!("ambiguous:{}={}", position, base)
        }
        DesignObstacle::FivePrimeTmFloorUnreachable {
            available,
            reached_celsius,
        } => format!("tm-floor 5':{}:{:.2}", available, reached_celsius),
        DesignObstacle::ThreePrimeTmFloorUnreachable {
            available,
            reached_celsius,
        } => format!("tm-floor 3':{}:{:.2}", available, reached_celsius),
    }
}

#[test]
fn native_tsv_has_every_case_once() {
    let rows: Vec<NativeRow> = native_rows();
    assert!(
        rows.len() == NATIVE_CASE_COUNT,
        "native.tsv has {} rows",
        rows.len()
    );
    let mut ids: Vec<String> = rows.iter().map(|row| row.case_id.clone()).collect();
    ids.sort();
    ids.dedup();
    assert!(
        ids.len() == NATIVE_CASE_COUNT,
        "native.tsv repeats a case id"
    );
}

#[test]
fn each_native_case_gets_the_design_or_failure_d030_predicts() {
    for row in native_rows() {
        let outcome: Result<PrimerDesign, DomainError> = native_design(&row);
        match row.expected.as_str() {
            "design" => {
                let design: PrimerDesign = outcome.unwrap_or_else(|error| {
                    panic!("{}: expected a design, got {:?}", row.case_id, error)
                });
                let pair: PrimerPair = design.pair();
                let actual: Vec<(&str, String, &String)> = vec![
                    (
                        "forward_span",
                        span_text(pair.clone().forward().span()),
                        &row.forward_span,
                    ),
                    (
                        "reverse_span",
                        span_text(pair.clone().reverse().span()),
                        &row.reverse_span,
                    ),
                    (
                        "flanks",
                        format!(
                            "{}:{}",
                            design.five_prime_flank(),
                            design.three_prime_flank()
                        ),
                        &row.flanks,
                    ),
                    (
                        "reverse_length",
                        design.reverse_length().to_string(),
                        &row.reverse_length,
                    ),
                ];
                for (field, luria, expected) in actual {
                    assert!(
                        luria == *expected,
                        "{}: {}: luria {} vs native.tsv {}",
                        row.case_id,
                        field,
                        luria,
                        expected
                    );
                }
                assert!(pair.abuts(), "{}: the pair does not abut", row.case_id);

                // Tms are recorded to two decimals by the generating script.
                let temperatures: Vec<(&str, f64, &String)> = vec![
                    ("forward_tm", design.forward_tm().celsius(), &row.forward_tm),
                    ("reverse_tm", design.reverse_tm().celsius(), &row.reverse_tm),
                    (
                        "annealing_temp",
                        design.annealing_temperature(),
                        &row.annealing_temp,
                    ),
                ];
                for (field, luria, expected) in temperatures {
                    let wanted: f64 = expected.parse().unwrap();
                    assert!(
                        (luria - wanted).abs() < 0.006,
                        "{}: {}: luria {:.3} vs native.tsv {}",
                        row.case_id,
                        field,
                        luria,
                        expected
                    );
                }
                assert!(
                    design.forward_tm().basis().name() == row.forward_tm_basis,
                    "{}: forward Tm basis {} vs native.tsv {}",
                    row.case_id,
                    design.forward_tm().basis().name(),
                    row.forward_tm_basis
                );
                let met: &str = if design.tm_balance_met() { "yes" } else { "no" };
                assert!(
                    met == row.balance_met,
                    "{}: balance_met {} vs native.tsv {}",
                    row.case_id,
                    met,
                    row.balance_met
                );
            }
            "error:NoPrimerDesignPossible" => match outcome {
                Err(DomainError::NoPrimerDesignPossible(problem)) => {
                    let luria: String =
                        format!("pair:{}>{}", problem.smallest_pair, problem.template_length);
                    assert!(
                        luria == row.failure,
                        "{}: luria {} vs native.tsv {}",
                        row.case_id,
                        luria,
                        row.failure
                    );
                }
                other => panic!(
                    "{}: expected NoPrimerDesignPossible, got {:?}",
                    row.case_id,
                    other.map(|_| "a design")
                ),
            },
            "error:NoPrimerDesignFits" => match outcome {
                Err(DomainError::NoPrimerDesignFits(problem)) => {
                    let luria: String = problem
                        .obstacles
                        .iter()
                        .map(obstacle_text)
                        .collect::<Vec<String>>()
                        .join(" ");
                    assert!(
                        luria == row.failure,
                        "{}: luria {} vs native.tsv {}",
                        row.case_id,
                        luria,
                        row.failure
                    );
                }
                other => panic!(
                    "{}: expected NoPrimerDesignFits, got {:?}",
                    row.case_id,
                    other.map(|_| "a design")
                ),
            },
            other => panic!("{}: unknown expectation {:?}", row.case_id, other),
        }
    }
}

/// Under D030 the forward primer keeps a 10-base 5′ flank and grows its 3′
/// flank for Tm, as the reference tool's does. Where neither needed to grow,
/// the two forward primers must be identical. The reverse primers differ: the
/// reference tool balances against its own mismatch-aware forward Tm, which
/// runs several degrees above the published model's (`tests/fixtures/tm`).
#[test]
fn native_forward_primers_equal_the_reference_where_a_row_names_one() {
    let references: Vec<Design> = reference_designs();
    let mut compared: usize = 0;
    for row in native_rows()
        .into_iter()
        .filter(|row| row.reference_design != "none")
    {
        let reference: &Design = references
            .iter()
            .find(|design| design.label == row.reference_design)
            .unwrap_or_else(|| {
                panic!(
                    "{}: no reference design {}",
                    row.case_id, row.reference_design
                )
            });
        let design: PrimerDesign = native_design(&row)
            .unwrap_or_else(|error| panic!("{}: expected a design, got {:?}", row.case_id, error));
        let forward: Primer = design.pair().forward();
        let span: Vec<u64> = integers(reference.record.get("forward").get("span"));
        assert!(
            vec![forward.clone().span().start(), forward.clone().span().end()] == span,
            "{}: forward span differs from {}",
            row.case_id,
            row.reference_design
        );
        assert!(
            forward.clone().sequence()
                == reference
                    .record
                    .get("forward")
                    .get("sequence")
                    .text()
                    .to_uppercase(),
            "{}: forward text differs from {}",
            row.case_id,
            row.reference_design
        );
        compared += 1;
    }
    assert!(
        compared == 2,
        "compared {} native rows with a reference",
        compared
    );
}

// ---------------------------------------------------------------------------
// luriaNative deletion designs (D031) against `native-deletion.tsv`.
// ---------------------------------------------------------------------------

/// The number of rows in `native-deletion.tsv`.
const DELETION_CASE_COUNT: usize = 13;

/// The number of `native-deletion.tsv` rows that expect a design.
const DELETION_DESIGN_COUNT: usize = 7;

struct DeletionRow {
    case_id: String,
    template_spec: String,
    start: i64,
    end: i64,
    deleted: String,
    min_overlap: i64,
    expected: String,
    forward_span: String,
    reverse_span: String,
    forward_tm: String,
    reverse_tm: String,
    annealing_temp: String,
    balance_met: String,
    forward: String,
    reverse: String,
    failure: String,
}

fn deletion_rows() -> Vec<DeletionRow> {
    let text: String = read("native-deletion.tsv");
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header: Vec<&str> = lines
        .next()
        .expect("native-deletion.tsv has a header")
        .split('\t')
        .collect();
    lines
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert!(
                fields.len() == header.len(),
                "native-deletion.tsv row has {} fields, header has {}",
                fields.len(),
                header.len()
            );
            let field = |name: &str| -> String {
                let index: usize = header
                    .iter()
                    .position(|column| *column == name)
                    .unwrap_or_else(|| panic!("native-deletion.tsv has no column {:?}", name));
                fields[index].to_string()
            };
            let region: Vec<String> = field("deletion").split(':').map(str::to_string).collect();
            assert!(
                region.len() == 3,
                "{}: deletion is not start:end:bases",
                field("case_id")
            );
            DeletionRow {
                case_id: field("case_id"),
                template_spec: field("template"),
                start: region[0].parse().unwrap(),
                end: region[1].parse().unwrap(),
                deleted: region[2].clone(),
                min_overlap: field("min_overlap").parse().unwrap(),
                expected: field("expected"),
                forward_span: field("forward_span"),
                reverse_span: field("reverse_span"),
                forward_tm: field("forward_tm"),
                reverse_tm: field("reverse_tm"),
                annealing_temp: field("annealing_temp"),
                balance_met: field("balance_met"),
                forward: field("forward"),
                reverse: field("reverse"),
                failure: field("failure"),
            }
        })
        .collect()
}

/// The row's template: `seq:<text>` is a synthetic linear sequence, otherwise
/// a fixture path relative to this directory.
fn deletion_template(row: &DeletionRow) -> Sequence {
    if let Some(text) = row.template_spec.strip_prefix("seq:") {
        return normalize(
            Identifier::create("native-deletion-template".to_string()).unwrap(),
            text.to_string(),
            SequenceTopology::Linear,
        )
        .unwrap_or_else(|error| {
            panic!("{}: template does not normalize: {:?}", row.case_id, error)
        });
    }
    template_from(&row.case_id, &row.template_spec)
}

/// The row's deletion as a verified `MutationResult`, asserting the bases the
/// row names as removed.
fn deletion_edit(row: &DeletionRow, template: &Sequence) -> MutationResult {
    let intent: NucleotideDeletionIntent = NucleotideDeletionIntent::create(
        template.reference(),
        create(&template.reference(), row.start, row.end).unwrap(),
        row.deleted.clone(),
    )
    .unwrap_or_else(|error| panic!("{}: deletion rejected: {:?}", row.case_id, error));
    let made: MutationResult = apply_deletion(
        template,
        None,
        Identifier::create("native-deletion".to_string()).unwrap(),
        intent,
    )
    .unwrap_or_else(|error| panic!("{}: deletion rejected: {:?}", row.case_id, error));
    verify(template, None, made)
        .unwrap_or_else(|error| panic!("{}: deletion does not verify: {:?}", row.case_id, error))
}

fn deletion_design(row: &DeletionRow) -> Result<PrimerDesign, DomainError> {
    let template: Sequence = deletion_template(row);
    let request: PrimerDesignRequest = PrimerDesignRequest::create(
        DesignStyle::Q5Polymerase,
        DesignPolicy::LuriaNative,
        row.min_overlap,
        PrimerPlacement::Inline,
        BufferModel::q5_reaction(),
    )
    .unwrap();
    design_primers(&template, deletion_edit(row, &template), request)
}

#[test]
fn native_deletion_tsv_has_every_case_once() {
    let rows: Vec<DeletionRow> = deletion_rows();
    assert!(
        rows.len() == DELETION_CASE_COUNT,
        "native-deletion.tsv has {} rows",
        rows.len()
    );
    let mut ids: Vec<String> = rows.iter().map(|row| row.case_id.clone()).collect();
    ids.sort();
    ids.dedup();
    assert!(
        ids.len() == DELETION_CASE_COUNT,
        "native-deletion.tsv repeats a case id"
    );
}

#[test]
fn each_native_deletion_case_gets_the_design_or_failure_d031_predicts() {
    for row in deletion_rows() {
        let outcome: Result<PrimerDesign, DomainError> = deletion_design(&row);
        match row.expected.as_str() {
            "design" => {
                let design: PrimerDesign = outcome.unwrap_or_else(|error| {
                    panic!("{}: expected a design, got {:?}", row.case_id, error)
                });
                let pair: PrimerPair = design.pair();
                let actual: Vec<(&str, String, &String)> = vec![
                    (
                        "forward_span",
                        span_text(pair.clone().forward().span()),
                        &row.forward_span,
                    ),
                    (
                        "reverse_span",
                        span_text(pair.clone().reverse().span()),
                        &row.reverse_span,
                    ),
                ];
                for (field, luria, expected) in actual {
                    assert!(
                        luria == *expected,
                        "{}: {}: luria {} vs native-deletion.tsv {}",
                        row.case_id,
                        field,
                        luria,
                        expected
                    );
                }
                // Primer text is compared without printing it.
                assert!(
                    pair.clone().forward().sequence() == row.forward,
                    "{}: forward primer text differs from native-deletion.tsv",
                    row.case_id
                );
                assert!(
                    pair.clone().reverse().sequence() == row.reverse,
                    "{}: reverse primer text differs from native-deletion.tsv",
                    row.case_id
                );

                // Tms are recorded to two decimals by the generating script.
                let temperatures: Vec<(&str, f64, &String)> = vec![
                    ("forward_tm", design.forward_tm().celsius(), &row.forward_tm),
                    ("reverse_tm", design.reverse_tm().celsius(), &row.reverse_tm),
                    (
                        "annealing_temp",
                        design.annealing_temperature(),
                        &row.annealing_temp,
                    ),
                ];
                for (field, luria, expected) in temperatures {
                    let wanted: f64 = expected.parse().unwrap();
                    assert!(
                        (luria - wanted).abs() < 0.006,
                        "{}: {}: luria {:.3} vs native-deletion.tsv {}",
                        row.case_id,
                        field,
                        luria,
                        expected
                    );
                }
                let met: &str = if design.tm_balance_met() { "yes" } else { "no" };
                assert!(
                    met == row.balance_met,
                    "{}: balance_met {} vs native-deletion.tsv {}",
                    row.case_id,
                    met,
                    row.balance_met
                );
            }
            "error:NoPrimerDesignPossible" => match outcome {
                Err(DomainError::NoPrimerDesignPossible(problem)) => {
                    let luria: String =
                        format!("pair:{}>{}", problem.smallest_pair, problem.template_length);
                    assert!(
                        luria == row.failure,
                        "{}: luria {} vs native-deletion.tsv {}",
                        row.case_id,
                        luria,
                        row.failure
                    );
                }
                other => panic!(
                    "{}: expected NoPrimerDesignPossible, got {:?}",
                    row.case_id,
                    other.map(|_| "a design")
                ),
            },
            "error:NoPrimerDesignFits" => match outcome {
                Err(DomainError::NoPrimerDesignFits(problem)) => {
                    let luria: String = problem
                        .obstacles
                        .iter()
                        .map(obstacle_text)
                        .collect::<Vec<String>>()
                        .join(" ");
                    assert!(
                        luria == row.failure,
                        "{}: luria {} vs native-deletion.tsv {}",
                        row.case_id,
                        luria,
                        row.failure
                    );
                }
                other => panic!(
                    "{}: expected NoPrimerDesignFits, got {:?}",
                    row.case_id,
                    other.map(|_| "a design")
                ),
            },
            other => panic!("{}: unknown expectation {:?}", row.case_id, other),
        }
    }
}

/// D031's invariants, over every deletion design: both primers meet the Tm
/// floor, the annealing temperature follows the Q5 "Tm+3" rule with its cap,
/// the pair abuts across the removed region, and the forward primer carries no
/// 5′ flank. Both Tms are on the matched basis, since neither primer has a
/// mismatch.
#[test]
fn every_native_deletion_design_meets_the_d031_invariants() {
    let mut designs: usize = 0;
    for row in deletion_rows()
        .into_iter()
        .filter(|row| row.expected == "design")
    {
        let design: PrimerDesign =
            deletion_design(&row).unwrap_or_else(|error| panic!("{}: {:?}", row.case_id, error));
        let forward: f64 = design.forward_tm().celsius();
        let reverse: f64 = design.reverse_tm().celsius();
        assert!(
            forward >= TM_FLOOR,
            "{}: forward {:.2}",
            row.case_id,
            forward
        );
        assert!(
            reverse >= TM_FLOOR,
            "{}: reverse {:.2}",
            row.case_id,
            reverse
        );
        let annealing: f64 = (forward.min(reverse) + TA_OFFSET).min(TA_CAP);
        assert!(
            (design.annealing_temperature() - annealing).abs() < 1e-9,
            "{}: annealing temperature",
            row.case_id
        );
        let pair: PrimerPair = design.pair();
        let forward_start: u64 = pair.clone().forward().span().start();
        let reverse_end: u64 = pair.clone().reverse().span().end();
        let width: u64 = design.deleted_width();
        assert!(
            reverse_end + width == forward_start,
            "{}: reverse end {} + width {} != forward start {}",
            row.case_id,
            reverse_end,
            width,
            forward_start
        );
        assert!(design.five_prime_flank() == 0, "{}", row.case_id);
        assert!(pair.abuts(), "{}: the pair does not abut", row.case_id);
        assert!(
            design.forward_tm().basis() == TmBasis::MatchedComplement,
            "{}: forward Tm basis",
            row.case_id
        );
        assert!(
            design.reverse_tm().basis() == TmBasis::MatchedComplement,
            "{}: reverse Tm basis",
            row.case_id
        );
        designs += 1;
    }
    assert!(
        designs == DELETION_DESIGN_COUNT,
        "checked {} deletion designs",
        designs
    );
}

// ---------------------------------------------------------------------------
// luriaNative insertion designs (D032) against `native-insertion.tsv`.
// ---------------------------------------------------------------------------

/// The number of rows in `native-insertion.tsv`.
const INSERTION_CASE_COUNT: usize = 15;

/// The number of `native-insertion.tsv` rows that expect a design.
const INSERTION_DESIGN_COUNT: usize = 11;

struct InsertionRow {
    case_id: String,
    template_spec: String,
    point: i64,
    inserted: String,
    min_overlap: i64,
    expected: String,
    forward_span: String,
    reverse_span: String,
    forward_tail: String,
    reverse_tail: String,
    forward: String,
    reverse: String,
    forward_tm: String,
    reverse_tm: String,
    forward_whole_tm: String,
    reverse_whole_tm: String,
    annealing_temp: String,
    balance_met: String,
    failure: String,
}

fn insertion_rows() -> Vec<InsertionRow> {
    let text: String = read("native-insertion.tsv");
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header: Vec<&str> = lines
        .next()
        .expect("native-insertion.tsv has a header")
        .split('\t')
        .collect();
    lines
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert!(
                fields.len() == header.len(),
                "native-insertion.tsv row has {} fields, header has {}",
                fields.len(),
                header.len()
            );
            let field = |name: &str| -> String {
                let index: usize = header
                    .iter()
                    .position(|column| *column == name)
                    .unwrap_or_else(|| panic!("native-insertion.tsv has no column {:?}", name));
                fields[index].to_string()
            };
            InsertionRow {
                case_id: field("case_id"),
                template_spec: field("template"),
                point: field("point").parse().unwrap(),
                inserted: field("inserted"),
                min_overlap: field("min_overlap").parse().unwrap(),
                expected: field("expected"),
                forward_span: field("forward_anneal_span"),
                reverse_span: field("reverse_anneal_span"),
                forward_tail: field("forward_tail"),
                reverse_tail: field("reverse_tail"),
                forward: field("forward"),
                reverse: field("reverse"),
                forward_tm: field("forward_tm"),
                reverse_tm: field("reverse_tm"),
                forward_whole_tm: field("forward_whole_tm"),
                reverse_whole_tm: field("reverse_whole_tm"),
                annealing_temp: field("annealing_temp"),
                balance_met: field("balance_met"),
                failure: field("failure"),
            }
        })
        .collect()
}

/// The row's template: `seq:<text>` is a synthetic linear sequence, otherwise
/// a fixture path relative to this directory.
fn insertion_template(row: &InsertionRow) -> Sequence {
    if let Some(text) = row.template_spec.strip_prefix("seq:") {
        return normalize(
            Identifier::create("native-insertion-template".to_string()).unwrap(),
            text.to_string(),
            SequenceTopology::Linear,
        )
        .unwrap_or_else(|error| {
            panic!("{}: template does not normalize: {:?}", row.case_id, error)
        });
    }
    template_from(&row.case_id, &row.template_spec)
}

/// The row's insertion as a verified `MutationResult`, asserting the flanking
/// bases read back from the template: an insertion point selects no base, so
/// the base on each side of it is what identifies where it sits.
fn insertion_edit(row: &InsertionRow, template: &Sequence) -> MutationResult {
    let letters: Vec<char> = template.letters();
    let at: usize = row.point as usize;
    assert!(
        at > 0 && at < letters.len(),
        "{}: the point {} has no base on each side of it",
        row.case_id,
        row.point
    );
    let intent: NucleotideInsertionIntent = NucleotideInsertionIntent::create(
        template.reference(),
        create_point(&template.reference(), row.point)
            .unwrap_or_else(|error| panic!("{}: point rejected: {:?}", row.case_id, error)),
        letters[at - 1].to_string(),
        letters[at].to_string(),
        row.inserted.clone(),
    )
    .unwrap_or_else(|error| panic!("{}: insertion rejected: {:?}", row.case_id, error));
    let made: MutationResult = apply_insertion(
        template,
        None,
        Identifier::create("native-insertion".to_string()).unwrap(),
        intent,
    )
    .unwrap_or_else(|error| panic!("{}: insertion rejected: {:?}", row.case_id, error));
    verify(template, None, made)
        .unwrap_or_else(|error| panic!("{}: insertion does not verify: {:?}", row.case_id, error))
}

fn insertion_design(row: &InsertionRow) -> Result<PrimerDesign, DomainError> {
    let template: Sequence = insertion_template(row);
    let request: PrimerDesignRequest = PrimerDesignRequest::create(
        DesignStyle::Q5Polymerase,
        DesignPolicy::LuriaNative,
        row.min_overlap,
        PrimerPlacement::Inline,
        BufferModel::q5_reaction(),
    )
    .unwrap();
    design_primers(&template, insertion_edit(row, &template), request)
}

/// A primer's 5′ tail text, or `none` when it has no tail: the fixture writes
/// `none` for the reverse primer of an insertion short enough to sit wholly on
/// the forward primer's tail.
fn tail_text(primer: Primer) -> String {
    match primer.clone().tail() {
        Some(part) => part.clone().sequence(),
        None => "none".to_string(),
    }
}

#[test]
fn native_insertion_tsv_has_every_case_once() {
    let rows: Vec<InsertionRow> = insertion_rows();
    assert!(
        rows.len() == INSERTION_CASE_COUNT,
        "native-insertion.tsv has {} rows",
        rows.len()
    );
    let mut ids: Vec<String> = rows.iter().map(|row| row.case_id.clone()).collect();
    ids.sort();
    ids.dedup();
    assert!(
        ids.len() == INSERTION_CASE_COUNT,
        "native-insertion.tsv repeats a case id"
    );
}

#[test]
fn each_native_insertion_case_gets_the_design_or_failure_d032_predicts() {
    for row in insertion_rows() {
        let outcome: Result<PrimerDesign, DomainError> = insertion_design(&row);
        match row.expected.as_str() {
            "design" => {
                let design: PrimerDesign = outcome.unwrap_or_else(|error| {
                    panic!("{}: expected a design, got {:?}", row.case_id, error)
                });
                let pair: PrimerPair = design.pair();
                let actual: Vec<(&str, String, &String)> = vec![
                    (
                        "forward_anneal_span",
                        span_text(pair.clone().forward().span()),
                        &row.forward_span,
                    ),
                    (
                        "reverse_anneal_span",
                        span_text(pair.clone().reverse().span()),
                        &row.reverse_span,
                    ),
                ];
                for (field, luria, expected) in actual {
                    assert!(
                        luria == *expected,
                        "{}: {}: luria {} vs native-insertion.tsv {}",
                        row.case_id,
                        field,
                        luria,
                        expected
                    );
                }
                // Primer and tail text is compared without printing it.
                assert!(
                    pair.clone().forward().sequence() == row.forward,
                    "{}: forward primer text differs from native-insertion.tsv",
                    row.case_id
                );
                assert!(
                    pair.clone().reverse().sequence() == row.reverse,
                    "{}: reverse primer text differs from native-insertion.tsv",
                    row.case_id
                );
                assert!(
                    tail_text(pair.clone().forward()) == row.forward_tail,
                    "{}: forward tail differs from native-insertion.tsv",
                    row.case_id
                );
                assert!(
                    tail_text(pair.clone().reverse()) == row.reverse_tail,
                    "{}: reverse tail differs from native-insertion.tsv",
                    row.case_id
                );

                // Tms are recorded to two decimals by the generating script.
                // `forward_tm` and `reverse_tm` are the annealing portions',
                // which is what D032 sizes and balances on; the whole primers'
                // Tms are reported alongside and never used for sizing.
                let whole_forward: f64 = design
                    .forward_whole_tm()
                    .unwrap_or_else(|| panic!("{}: no forward whole-primer Tm", row.case_id))
                    .celsius();
                let whole_reverse: f64 = design
                    .reverse_whole_tm()
                    .unwrap_or_else(|| panic!("{}: no reverse whole-primer Tm", row.case_id))
                    .celsius();
                let temperatures: Vec<(&str, f64, &String)> = vec![
                    ("forward_tm", design.forward_tm().celsius(), &row.forward_tm),
                    ("reverse_tm", design.reverse_tm().celsius(), &row.reverse_tm),
                    ("forward_whole_tm", whole_forward, &row.forward_whole_tm),
                    ("reverse_whole_tm", whole_reverse, &row.reverse_whole_tm),
                    (
                        "annealing_temp",
                        design.annealing_temperature(),
                        &row.annealing_temp,
                    ),
                ];
                for (field, luria, expected) in temperatures {
                    let wanted: f64 = expected.parse().unwrap();
                    assert!(
                        (luria - wanted).abs() < 0.006,
                        "{}: {}: luria {:.3} vs native-insertion.tsv {}",
                        row.case_id,
                        field,
                        luria,
                        expected
                    );
                }
                let met: &str = if design.tm_balance_met() { "yes" } else { "no" };
                assert!(
                    met == row.balance_met,
                    "{}: balance_met {} vs native-insertion.tsv {}",
                    row.case_id,
                    met,
                    row.balance_met
                );
            }
            "error:NoPrimerDesignPossible" => match outcome {
                Err(DomainError::NoPrimerDesignPossible(problem)) => {
                    let luria: String =
                        format!("pair:{}>{}", problem.smallest_pair, problem.template_length);
                    assert!(
                        luria == row.failure,
                        "{}: luria {} vs native-insertion.tsv {}",
                        row.case_id,
                        luria,
                        row.failure
                    );
                }
                other => panic!(
                    "{}: expected NoPrimerDesignPossible, got {:?}",
                    row.case_id,
                    other.map(|_| "a design")
                ),
            },
            "error:NoPrimerDesignFits" => match outcome {
                Err(DomainError::NoPrimerDesignFits(problem)) => {
                    let luria: String = problem
                        .obstacles
                        .iter()
                        .map(obstacle_text)
                        .collect::<Vec<String>>()
                        .join(" ");
                    assert!(
                        luria == row.failure,
                        "{}: luria {} vs native-insertion.tsv {}",
                        row.case_id,
                        luria,
                        row.failure
                    );
                }
                other => panic!(
                    "{}: expected NoPrimerDesignFits, got {:?}",
                    row.case_id,
                    other.map(|_| "a design")
                ),
            },
            other => panic!("{}: unknown expectation {:?}", row.case_id, other),
        }
    }
}

/// A primer's 5′ tail letters, empty when it has no tail.
fn tail_letters(primer: Primer) -> Vec<char> {
    match primer.clone().tail() {
        Some(part) => part.clone().sequence().chars().collect(),
        None => Vec::new(),
    }
}

/// D032's invariants, over every insertion design: both annealing portions
/// meet the Tm floor, the annealing temperature follows the Q5 "Tm+3" rule
/// with its cap, both annealing portions abut at the point itself — an
/// insertion occupies no template position — and the two 5′ tails rebuild the
/// inserted text exactly. Both Tms are on the matched basis, since an
/// insertion's annealing portions are wild-type template.
#[test]
fn every_native_insertion_design_meets_the_d032_invariants() {
    let mut designs: usize = 0;
    for row in insertion_rows()
        .into_iter()
        .filter(|row| row.expected == "design")
    {
        let design: PrimerDesign =
            insertion_design(&row).unwrap_or_else(|error| panic!("{}: {:?}", row.case_id, error));
        let forward: f64 = design.forward_tm().celsius();
        let reverse: f64 = design.reverse_tm().celsius();
        assert!(
            forward >= TM_FLOOR,
            "{}: forward {:.2}",
            row.case_id,
            forward
        );
        assert!(
            reverse >= TM_FLOOR,
            "{}: reverse {:.2}",
            row.case_id,
            reverse
        );
        let annealing: f64 = (forward.min(reverse) + TA_OFFSET).min(TA_CAP);
        assert!(
            (design.annealing_temperature() - annealing).abs() < 1e-9,
            "{}: annealing temperature",
            row.case_id
        );
        let pair: PrimerPair = design.pair();
        let point: u64 = row.point as u64;
        assert!(
            pair.clone().reverse().span().end() == point
                && pair.clone().forward().span().start() == point,
            "{}: the annealing portions do not abut at the point {}",
            row.case_id,
            point
        );

        // The reverse tail is the reverse complement of the first `floor(k/2)`
        // inserted bases and the forward tail the rest, so complementing the
        // reverse tail back and concatenating rebuilds the insertion.
        let forward_tail: Vec<char> = tail_letters(pair.clone().forward());
        let reverse_tail: Vec<char> = tail_letters(pair.clone().reverse());
        let forward_text: String = forward_tail.iter().collect::<String>();
        let restored: String = reverse_complement(reverse_tail)
            .unwrap_or_else(|error| {
                panic!("{}: tail does not complement: {:?}", row.case_id, error)
            })
            .iter()
            .collect::<String>();
        let rebuilt: String = format!("{}{}", restored, forward_text);
        assert!(
            rebuilt == row.inserted,
            "{}: the tails rebuild {:?}, not the inserted {:?}",
            row.case_id,
            rebuilt,
            row.inserted
        );

        // At or below `MAX_WHOLE_TAIL` the whole insertion sits on the forward
        // primer's tail, so the reverse primer has none.
        let width: u64 = row.inserted.len() as u64;
        if width <= MAX_WHOLE_TAIL {
            assert!(
                pair.clone().reverse().tail().is_none(),
                "{}: a {}-base insertion split the reverse primer",
                row.case_id,
                width
            );
        }

        assert!(
            design.forward_tm().basis() == TmBasis::MatchedComplement,
            "{}: forward Tm basis",
            row.case_id
        );
        assert!(
            design.reverse_tm().basis() == TmBasis::MatchedComplement,
            "{}: reverse Tm basis",
            row.case_id
        );
        designs += 1;
    }
    assert!(
        designs == INSERTION_DESIGN_COUNT,
        "checked {} insertion designs",
        designs
    );
}

/// D032 puts no cap on insertion length, so a 210-base insertion is designed:
/// both primers come out at 120 nt, past the owner's 100-nt practical limit
/// for ordering a standard oligo. That limit is a quality warning on the
/// emitted design, not a refusal.
#[test]
fn a_primer_past_the_ordering_limit_is_warned_about_and_still_emitted() {
    let row: InsertionRow = insertion_rows()
        .into_iter()
        .find(|row| row.case_id == "ins-13")
        .unwrap_or_else(|| panic!("native-insertion.tsv has no ins-13 row"));
    let design: PrimerDesign = insertion_design(&row)
        .unwrap_or_else(|error| panic!("ins-13: expected a design, got {:?}", error));
    let pair: PrimerPair = design.pair();
    assert!(
        pair.clone().forward().length() == 120,
        "ins-13: forward primer length {}, expected 120",
        pair.clone().forward().length()
    );
    assert!(
        pair.clone().reverse().length() == 120,
        "ins-13: reverse primer length {}, expected 120",
        pair.clone().reverse().length()
    );
    let over: PrimerWarning = PrimerWarning::AboveOrderingLimit { length: 120 };
    assert!(
        design.forward_warnings().contains(&over),
        "ins-13: forward warnings {:?}",
        design.forward_warnings()
    );
    assert!(
        design.reverse_warnings().contains(&over),
        "ins-13: reverse warnings {:?}",
        design.reverse_warnings()
    );
}

// ---------------------------------------------------------------------------
// luriaNative replacement designs (D033) against `native-replacement.tsv`.
// ---------------------------------------------------------------------------

/// The number of rows in `native-replacement.tsv`.
const REPLACEMENT_CASE_COUNT: usize = 15;

/// The number of `native-replacement.tsv` rows that expect a design.
const REPLACEMENT_DESIGN_COUNT: usize = 11;

struct ReplacementRow {
    case_id: String,
    template_spec: String,
    start: i64,
    end: i64,
    original: String,
    replacement: String,
    min_overlap: i64,
    path: String,
    expected: String,
    forward_span: String,
    reverse_span: String,
    forward_tail: String,
    reverse_tail: String,
    forward: String,
    reverse: String,
    forward_tm: String,
    forward_tm_basis: String,
    reverse_tm: String,
    forward_whole_tm: String,
    reverse_whole_tm: String,
    annealing_temp: String,
    balance_met: String,
    failure: String,
}

fn replacement_rows() -> Vec<ReplacementRow> {
    let text: String = read("native-replacement.tsv");
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header: Vec<&str> = lines
        .next()
        .expect("native-replacement.tsv has a header")
        .split('\t')
        .collect();
    lines
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert!(
                fields.len() == header.len(),
                "native-replacement.tsv row has {} fields, header has {}",
                fields.len(),
                header.len()
            );
            let field = |name: &str| -> String {
                let index: usize = header
                    .iter()
                    .position(|column| *column == name)
                    .unwrap_or_else(|| panic!("native-replacement.tsv has no column {:?}", name));
                fields[index].to_string()
            };
            let region: Vec<String> = field("region").split(':').map(str::to_string).collect();
            assert!(
                region.len() == 3,
                "{}: region is not start:end:bases",
                field("case_id")
            );
            ReplacementRow {
                case_id: field("case_id"),
                template_spec: field("template"),
                start: region[0].parse().unwrap(),
                end: region[1].parse().unwrap(),
                original: region[2].clone(),
                replacement: field("replacement"),
                min_overlap: field("min_overlap").parse().unwrap(),
                path: field("path"),
                expected: field("expected"),
                forward_span: field("forward_span"),
                reverse_span: field("reverse_span"),
                forward_tail: field("forward_tail"),
                reverse_tail: field("reverse_tail"),
                forward: field("forward"),
                reverse: field("reverse"),
                forward_tm: field("forward_tm"),
                forward_tm_basis: field("forward_tm_basis"),
                reverse_tm: field("reverse_tm"),
                forward_whole_tm: field("forward_whole_tm"),
                reverse_whole_tm: field("reverse_whole_tm"),
                annealing_temp: field("annealing_temp"),
                balance_met: field("balance_met"),
                failure: field("failure"),
            }
        })
        .collect()
}

/// The row's template: `seq:<text>` is a synthetic linear sequence, otherwise
/// a fixture path relative to this directory.
fn replacement_template(row: &ReplacementRow) -> Sequence {
    if let Some(text) = row.template_spec.strip_prefix("seq:") {
        return normalize(
            Identifier::create("native-replacement-template".to_string()).unwrap(),
            text.to_string(),
            SequenceTopology::Linear,
        )
        .unwrap_or_else(|error| {
            panic!("{}: template does not normalize: {:?}", row.case_id, error)
        });
    }
    template_from(&row.case_id, &row.template_spec)
}

/// The row's replacement as a verified `MutationResult`, asserting the bases the
/// row names as replaced: a replacement overwrites a region, so the text in it
/// is what identifies where it sits, and the text written need not match its
/// width.
fn replacement_edit(row: &ReplacementRow, template: &Sequence) -> MutationResult {
    let intent: NucleotideReplacementIntent = NucleotideReplacementIntent::create(
        template.reference(),
        create(&template.reference(), row.start, row.end).unwrap(),
        row.original.clone(),
        row.replacement.clone(),
    )
    .unwrap_or_else(|error| panic!("{}: replacement rejected: {:?}", row.case_id, error));
    let made: MutationResult = apply_replacement(
        template,
        None,
        Identifier::create("native-replacement".to_string()).unwrap(),
        intent,
    )
    .unwrap_or_else(|error| panic!("{}: replacement rejected: {:?}", row.case_id, error));
    verify(template, None, made)
        .unwrap_or_else(|error| panic!("{}: replacement does not verify: {:?}", row.case_id, error))
}

fn replacement_design(row: &ReplacementRow) -> Result<PrimerDesign, DomainError> {
    let template: Sequence = replacement_template(row);
    let request: PrimerDesignRequest = PrimerDesignRequest::create(
        DesignStyle::Q5Polymerase,
        DesignPolicy::LuriaNative,
        row.min_overlap,
        PrimerPlacement::Inline,
        BufferModel::q5_reaction(),
    )
    .unwrap();
    design_primers(&template, replacement_edit(row, &template), request)
}

#[test]
fn native_replacement_tsv_has_every_case_once() {
    let rows: Vec<ReplacementRow> = replacement_rows();
    assert!(
        rows.len() == REPLACEMENT_CASE_COUNT,
        "native-replacement.tsv has {} rows",
        rows.len()
    );
    let designs: usize = rows.iter().filter(|row| row.expected == "design").count();
    assert!(
        designs == REPLACEMENT_DESIGN_COUNT,
        "native-replacement.tsv expects {} designs, not {}",
        designs,
        REPLACEMENT_DESIGN_COUNT
    );
    let mut ids: Vec<String> = rows.iter().map(|row| row.case_id.clone()).collect();
    ids.sort();
    ids.dedup();
    assert!(
        ids.len() == REPLACEMENT_CASE_COUNT,
        "native-replacement.tsv repeats a case id"
    );
}

#[test]
fn each_native_replacement_case_gets_the_design_or_failure_d033_predicts() {
    for row in replacement_rows() {
        let outcome: Result<PrimerDesign, DomainError> = replacement_design(&row);
        match row.expected.as_str() {
            "design" => {
                let design: PrimerDesign = outcome.unwrap_or_else(|error| {
                    panic!("{}: expected a design, got {:?}", row.case_id, error)
                });
                let pair: PrimerPair = design.pair();
                let actual: Vec<(&str, String, &String)> = vec![
                    (
                        "forward_span",
                        span_text(pair.clone().forward().span()),
                        &row.forward_span,
                    ),
                    (
                        "reverse_span",
                        span_text(pair.clone().reverse().span()),
                        &row.reverse_span,
                    ),
                ];
                for (field, luria, expected) in actual {
                    assert!(
                        luria == *expected,
                        "{}: {}: luria {} vs native-replacement.tsv {}",
                        row.case_id,
                        field,
                        luria,
                        expected
                    );
                }
                // Primer and tail text is compared without printing it.
                assert!(
                    pair.clone().forward().sequence() == row.forward,
                    "{}: forward primer text differs from native-replacement.tsv",
                    row.case_id
                );
                assert!(
                    pair.clone().reverse().sequence() == row.reverse,
                    "{}: reverse primer text differs from native-replacement.tsv",
                    row.case_id
                );
                assert!(
                    tail_text(pair.clone().forward()) == row.forward_tail,
                    "{}: forward tail differs from native-replacement.tsv",
                    row.case_id
                );
                assert!(
                    tail_text(pair.clone().reverse()) == row.reverse_tail,
                    "{}: reverse tail differs from native-replacement.tsv",
                    row.case_id
                );

                // Tms are recorded to two decimals by the generating script.
                // `forward_tm` and `reverse_tm` are the annealing portions',
                // which is what D033 sizes and balances on; the whole primers'
                // Tms are reported alongside and never used for sizing.
                let temperatures: Vec<(&str, f64, &String)> = vec![
                    ("forward_tm", design.forward_tm().celsius(), &row.forward_tm),
                    ("reverse_tm", design.reverse_tm().celsius(), &row.reverse_tm),
                    (
                        "annealing_temp",
                        design.annealing_temperature(),
                        &row.annealing_temp,
                    ),
                ];
                for (field, luria, expected) in temperatures {
                    let wanted: f64 = expected.parse().unwrap();
                    assert!(
                        (luria - wanted).abs() < 0.006,
                        "{}: {}: luria {:.3} vs native-replacement.tsv {}",
                        row.case_id,
                        field,
                        luria,
                        expected
                    );
                }

                // The whole primers' Tms are reported only on the tails path:
                // an inline primer has no tail, so its Tm is already the whole
                // primer's and the fixture writes `none`.
                let whole: Vec<(&str, Option<MeltingTemperature>, String)> = vec![
                    (
                        "forward_whole_tm",
                        design.forward_whole_tm(),
                        row.forward_whole_tm,
                    ),
                    (
                        "reverse_whole_tm",
                        design.reverse_whole_tm(),
                        row.reverse_whole_tm,
                    ),
                ];
                for (field, value, text) in whole {
                    match (value, text == "none") {
                        (None, true) => {}
                        (None, false) => panic!(
                            "{}: {}: luria reports no whole-primer Tm, native-replacement.tsv has {}",
                            row.case_id,
                            field,
                            text
                        ),
                        (Some(found), true) => panic!(
                            "{}: {}: luria {:.3}, native-replacement.tsv has none",
                            row.case_id,
                            field,
                            found.celsius()
                        ),
                        (Some(found), false) => {
                            let wanted: f64 = text.parse().unwrap();
                            assert!(
                                (found.celsius() - wanted).abs() < 0.006,
                                "{}: {}: luria {:.3} vs native-replacement.tsv {}",
                                row.case_id,
                                field,
                                found.celsius(),
                                text
                            );
                        }
                    }
                }

                // D033 keeps D030's mismatch-aware basis only where the primer
                // and its site align, that is where `m == w`; otherwise the
                // forward Tm is on the matched basis.
                assert!(
                    design.forward_tm().basis().name() == row.forward_tm_basis,
                    "{}: forward Tm basis {} vs native-replacement.tsv {}",
                    row.case_id,
                    design.forward_tm().basis().name(),
                    row.forward_tm_basis
                );

                // Where `m != w` the matched basis is the one D033 chose, not
                // one the mismatch-aware sizing fell back to: a fallback
                // records a note on the Tm, a chosen basis records none.
                if row.replacement.chars().count() as u64 != (row.end - row.start) as u64 {
                    assert!(
                        design.forward_tm().substituted_because().is_none(),
                        "{}: the forward Tm fell back to the matched basis with {:?}",
                        row.case_id,
                        design.forward_tm().substituted_because()
                    );
                }
                let met: &str = if design.tm_balance_met() { "yes" } else { "no" };
                assert!(
                    met == row.balance_met,
                    "{}: balance_met {} vs native-replacement.tsv {}",
                    row.case_id,
                    met,
                    row.balance_met
                );
            }
            "error:NoPrimerDesignPossible" => match outcome {
                Err(DomainError::NoPrimerDesignPossible(problem)) => {
                    let luria: String =
                        format!("pair:{}>{}", problem.smallest_pair, problem.template_length);
                    assert!(
                        luria == row.failure,
                        "{}: luria {} vs native-replacement.tsv {}",
                        row.case_id,
                        luria,
                        row.failure
                    );
                }
                other => panic!(
                    "{}: expected NoPrimerDesignPossible, got {:?}",
                    row.case_id,
                    other.map(|_| "a design")
                ),
            },
            "error:NoPrimerDesignFits" => match outcome {
                Err(DomainError::NoPrimerDesignFits(problem)) => {
                    let luria: String = problem
                        .obstacles
                        .iter()
                        .map(obstacle_text)
                        .collect::<Vec<String>>()
                        .join(" ");
                    assert!(
                        luria == row.failure,
                        "{}: luria {} vs native-replacement.tsv {}",
                        row.case_id,
                        luria,
                        row.failure
                    );
                }
                other => panic!(
                    "{}: expected NoPrimerDesignFits, got {:?}",
                    row.case_id,
                    other.map(|_| "a design")
                ),
            },
            other => panic!("{}: unknown expectation {:?}", row.case_id, other),
        }
    }
}

/// D033's invariants, over every replacement design: both primers meet the Tm
/// floor and the annealing temperature follows the Q5 "Tm+3" rule with its cap,
/// on both paths. Inline, the pair abuts across the region itself — the forward
/// primer's span covers `[s, e)` and the reverse ends where it starts — and
/// neither primer carries a 5′ tail. On the tails path the annealing portions
/// meet the two edges of the region, the two 5′ tails rebuild the replacement
/// exactly, and the pair's footprint is the region's own width.
#[test]
fn every_native_replacement_design_meets_the_d033_invariants() {
    let mut designs: usize = 0;
    for row in replacement_rows()
        .into_iter()
        .filter(|row| row.expected == "design")
    {
        let design: PrimerDesign =
            replacement_design(&row).unwrap_or_else(|error| panic!("{}: {:?}", row.case_id, error));
        let forward: f64 = design.forward_tm().celsius();
        let reverse: f64 = design.reverse_tm().celsius();
        assert!(
            forward >= TM_FLOOR,
            "{}: forward {:.2}",
            row.case_id,
            forward
        );
        assert!(
            reverse >= TM_FLOOR,
            "{}: reverse {:.2}",
            row.case_id,
            reverse
        );
        let annealing: f64 = (forward.min(reverse) + TA_OFFSET).min(TA_CAP);
        assert!(
            (design.annealing_temperature() - annealing).abs() < 1e-9,
            "{}: annealing temperature",
            row.case_id
        );
        let pair: PrimerPair = design.pair();
        let start: u64 = row.start as u64;
        let end: u64 = row.end as u64;
        match row.path.as_str() {
            "inline" => {
                let forward_span: NucleotideInterval = pair.clone().forward().span();
                assert!(
                    forward_span.start() <= start && forward_span.end() >= end,
                    "{}: the forward span {} does not cover the region {}:{}",
                    row.case_id,
                    span_text(forward_span),
                    start,
                    end
                );
                assert!(
                    pair.clone().reverse().span().end() == forward_span.start(),
                    "{}: the annealing portions do not abut across the region",
                    row.case_id
                );
                assert!(
                    pair.clone().forward().tail().is_none()
                        && pair.clone().reverse().tail().is_none(),
                    "{}: an inline pair carries a 5′ tail",
                    row.case_id
                );
            }
            "tails" => {
                assert!(
                    pair.clone().reverse().span().end() == start,
                    "{}: the reverse primer does not anneal up to the region start {}",
                    row.case_id,
                    start
                );
                assert!(
                    pair.clone().forward().span().start() == end,
                    "{}: the forward primer does not anneal from the region end {}",
                    row.case_id,
                    end
                );

                // The reverse tail is the reverse complement of the first
                // `floor(m/2)` written bases and the forward tail the rest,
                // so complementing the reverse tail back and concatenating
                // rebuilds the replacement.
                let forward_tail: Vec<char> = tail_letters(pair.clone().forward());
                let reverse_tail: Vec<char> = tail_letters(pair.clone().reverse());
                let restored: String = reverse_complement(reverse_tail)
                    .unwrap_or_else(|error| {
                        panic!("{}: tail does not complement: {:?}", row.case_id, error)
                    })
                    .iter()
                    .collect::<String>();
                let rebuilt: String =
                    format!("{}{}", restored, forward_tail.iter().collect::<String>());
                assert!(
                    rebuilt == row.replacement,
                    "{}: the tails rebuild {:?}, not the replacement {:?}",
                    row.case_id,
                    rebuilt,
                    row.replacement
                );
                assert!(
                    pair.clone().deleted_width() == end - start,
                    "{}: the pair removes {}, not the region's {}",
                    row.case_id,
                    pair.clone().deleted_width(),
                    end - start
                );
            }
            other => panic!("{}: unknown path {:?}", row.case_id, other),
        }
        designs += 1;
    }
    assert!(
        designs == REPLACEMENT_DESIGN_COUNT,
        "checked {} replacement designs",
        designs
    );
}

// ---------------------------------------------------------------------------
// Tm: luria's function against the values the reference designs record.
// ---------------------------------------------------------------------------

/// Letters of the template over a span, as the primer reads them.
fn site_letters(template: &Sequence, span: (u64, u64), reverse: bool) -> Vec<char> {
    let slice: Vec<char> = template.letters()[span.0 as usize..span.1 as usize].to_vec();
    if reverse {
        reverse_complement(slice).unwrap()
    } else {
        slice
    }
}

/// luria's Tm against the value each reference design records.
///
/// The recorded `tm` is a **matched-duplex** value: the fixture generator
/// computed it without mismatch parameters, and the research notes record the
/// same figure as a hand reconstruction over the pure Watson-Crick table
/// (`docs/research/nebasechanger-client-investigation.md` §4.2, which gives
/// 74.18 for this primer against 74.2397 for the tool's own no-mismatch path).
/// So the matched basis is the one comparable to it, and it is asserted here.
///
/// The mismatch-aware basis is **not** asserted against these values, because
/// it is a different quantity. It is measured and printed instead: it comes out
/// 6-7 °C below the matched basis on these primers, whereas the research notes
/// record the reference tool's own mismatch-aware figure as only 0.57 °C below
/// its matched one (73.7841 against 74.3490). luria's figure is the one the
/// published mismatch parameters give, confirmed against Biopython in
/// `tm_crosscheck.rs`; the reference tool's mismatch handling therefore differs
/// from the published model, and no claim of agreement with it is made.
#[test]
fn luria_tm_matches_the_recorded_reference_tm_on_the_matched_basis() {
    let buffer: BufferModel = BufferModel::q5_reaction();
    let mut worst: f64 = 0.0;
    let mut compared: usize = 0;
    let mut aware_drop_seen: bool = false;
    for design in reference_designs() {
        let row: CaseRow = case_row(&design.case_id);
        let template: Sequence = template_of(&row);
        for (role, reverse) in [("forward", false), ("reverse", true)] {
            let record: &Json = design.record.get(role);
            let primer: Vec<char> = record
                .get("sequence")
                .text()
                .to_uppercase()
                .chars()
                .collect();
            let span: Vec<u64> = integers(record.get("span"));
            let site: Vec<char> = site_letters(&template, (span[0], span[1]), reverse);
            let recorded: f64 = record.get("tm").decimal();

            let matched: MeltingTemperature =
                melting_temperature(&primer, &primer, &buffer).unwrap();
            let difference: f64 = (matched.celsius() - recorded).abs();
            assert!(
                difference < 0.25,
                "{} {}: luria {:.3} °C on the matched basis, the fixture records {:.2}, difference {:.3}",
                design.label,
                role,
                matched.celsius(),
                recorded,
                difference
            );
            worst = worst.max(difference);
            compared += 1;

            // The mismatch-aware basis, where the published parameters reach.
            // Measured, not asserted against `recorded`; see the comment above.
            let aware: Result<MeltingTemperature, DomainError> =
                melting_temperature(&primer, &site, &buffer);
            let aware_text: String = match &aware {
                Ok(value) => {
                    assert!(
                        value.celsius() <= matched.celsius(),
                        "{} {}: a mismatch raised the Tm",
                        design.label,
                        role
                    );
                    if value.celsius() < matched.celsius() {
                        aware_drop_seen = true;
                    }
                    format!("{:7.3}", value.celsius())
                }
                Err(DomainError::TmParametersUnavailable(_)) => "tandem ".to_string(),
                Err(other) => panic!("{} {}: {:?}", design.label, role, other),
            };
            println!(
                "{:20} {:7} recorded {:6.2}  matched {:7.3}  aware {}",
                design.label,
                role,
                recorded,
                matched.celsius(),
                aware_text
            );
        }
    }
    assert!(compared == 12, "compared {} primers", compared);
    assert!(
        aware_drop_seen,
        "no primer showed a mismatch-aware Tm below its matched one"
    );
    println!("\nworst matched-basis difference: {:.3} °C", worst);
}

/// D030's invariants, over every native design: both primers meet the Tm
/// floor, the annealing temperature follows the Q5 "Tm+3" rule with its cap,
/// and the forward primer's 5′ flank is the fixed 10 bases.
#[test]
fn every_native_design_meets_the_d030_invariants() {
    let mut designs: usize = 0;
    for row in native_rows()
        .into_iter()
        .filter(|row| row.expected == "design")
    {
        let design: PrimerDesign =
            native_design(&row).unwrap_or_else(|error| panic!("{}: {:?}", row.case_id, error));
        let forward: f64 = design.forward_tm().celsius();
        let reverse: f64 = design.reverse_tm().celsius();
        assert!(
            forward >= TM_FLOOR,
            "{}: forward {:.2}",
            row.case_id,
            forward
        );
        assert!(
            reverse >= TM_FLOOR,
            "{}: reverse {:.2}",
            row.case_id,
            reverse
        );
        let annealing: f64 = (forward.min(reverse) + TA_OFFSET).min(TA_CAP);
        assert!(
            (design.annealing_temperature() - annealing).abs() < 1e-9,
            "{}: annealing temperature",
            row.case_id
        );
        assert!(design.five_prime_flank() == 10, "{}", row.case_id);
        assert!(design.reverse_tm().basis() == TmBasis::MatchedComplement);
        designs += 1;
    }
    assert!(designs == 4, "checked {} native designs", designs);
}

// ---------------------------------------------------------------------------
// Every column of `cases.tsv`, and the NEBaseChanger files of
// `tests/fixtures/example-construct/` (`docs/tasks/review-post-m4.md` C4, C6,
// C9). Each check below compares two files that already exist, or luria
// against external NEBaseChanger output; none copies luria's own output.
// ---------------------------------------------------------------------------

/// The number of rows in `cases.tsv`.
const CASE_COUNT: usize = 10;

/// One edit of a `cases.tsv` `edits` cell: `start:end:original:replacement`,
/// zero-based half-open (D004). An empty `original` marks an insertion, an
/// empty `replacement` a deletion.
struct Edit {
    start: i64,
    end: i64,
    original: String,
    replacement: String,
}

/// A cell that must hold an integer, naming the case and column in the failure.
fn integer_of(label: &str, name: &str, text: String) -> i64 {
    text.parse::<i64>().unwrap_or_else(|_| {
        panic!(
            "{}: column {:?} is not an integer ({:?})",
            label, name, text
        )
    })
}

/// A cell that must hold a number, such as a Tm or a `target_tm`.
fn decimal_of(label: &str, name: &str, text: String) -> f64 {
    text.parse::<f64>()
        .unwrap_or_else(|_| panic!("{}: column {:?} is not a number ({:?})", label, name, text))
}

/// Two numbers that must be equal, compared without a float `==`.
fn same_number(label: &str, name: &str, found: f64, wanted: f64) {
    assert!(
        (found - wanted).abs() < 1e-9,
        "{}: {}: {} vs {}",
        label,
        name,
        found,
        wanted
    );
}

/// A figure printed rounded to the nearest integer, against the unrounded
/// value the record holds.
fn rounded_number(label: &str, name: &str, printed: f64, recorded: f64) {
    assert!(
        (recorded.round() - printed).abs() < 1e-9,
        "{}: {}: prints {}, {} rounds to {}",
        label,
        name,
        printed,
        recorded,
        recorded.round()
    );
}

/// The row's `edits` cell as edits: `;` separates the edits of a `replace-set`.
fn case_edits(row: &CaseRow) -> Vec<Edit> {
    row.edits
        .split(';')
        .map(|cell| {
            let parts: Vec<String> = cell.split(':').map(|part| part.to_string()).collect();
            assert!(
                parts.len() == 4,
                "{}: edit cell {:?} is not start:end:original:replacement",
                row.case_id,
                cell
            );
            Edit {
                start: integer_of(&row.case_id, "edits start", parts[0].clone()),
                end: integer_of(&row.case_id, "edits end", parts[1].clone()),
                original: parts[2].clone(),
                replacement: parts[3].clone(),
            }
        })
        .collect()
}

/// The `designs.json` records a case names, in the order the file stores them:
/// one for q5p-01..03 and q5p-04, and q5p-07's two variants as `variant-a`
/// then `variant-b`.
fn records_for(case_id: String) -> Vec<Json> {
    reference_designs()
        .into_iter()
        .filter(|design| design.case_id == case_id)
        .map(|design| design.record.clone())
        .collect()
}

#[test]
fn every_case_names_the_digest_of_its_sequence_file() {
    let rows: Vec<CaseRow> = case_rows();
    assert!(
        rows.len() == CASE_COUNT,
        "cases.tsv has {} rows, expected {}",
        rows.len(),
        CASE_COUNT
    );
    for row in rows.iter() {
        let digest: String = template_of(row).content_hash().hex_digest()[..16].to_string();
        assert!(
            digest == row.sequence_digest_16,
            "{}: template digest {} vs cases.tsv {}",
            row.case_id,
            digest,
            row.sequence_digest_16
        );
    }
}

#[test]
fn every_case_edit_asserts_the_bases_its_template_holds() {
    for row in case_rows() {
        let text: String = template_of(&row).canonical_text();
        let edits: Vec<Edit> = case_edits(&row);
        assert!(
            !edits.is_empty(),
            "{}: the edits column names no edit",
            row.case_id
        );
        for edit in edits.iter() {
            assert!(
                edit.start >= 0 && edit.end >= edit.start,
                "{}: edit {}..{} is not an ordered interval",
                row.case_id,
                edit.start,
                edit.end
            );
            assert!(
                edit.end <= text.len() as i64,
                "{}: edit {}..{} lies past the {}-base template",
                row.case_id,
                edit.start,
                edit.end,
                text.len()
            );
            assert!(
                edit.end - edit.start == edit.original.len() as i64,
                "{}: edit {}..{} is {} wide, its original text is {} bases",
                row.case_id,
                edit.start,
                edit.end,
                edit.end - edit.start,
                edit.original.len()
            );
            let window: &str = &text[edit.start as usize..edit.end as usize];
            assert!(
                window == edit.original,
                "{}: the template at {}..{} differs from the edit's original text",
                row.case_id,
                edit.start,
                edit.end
            );
        }
    }
}

#[test]
fn every_case_kind_agrees_with_its_edits() {
    for row in case_rows() {
        let edits: Vec<Edit> = case_edits(&row);
        assert!(
            !edits.is_empty(),
            "{}: kind {:?} states an edit, the edits column names none",
            row.case_id,
            row.kind
        );
        let first: &Edit = edits.first().unwrap();
        match row.kind.as_str() {
            "replace" => {
                assert!(
                    edits.len() == 1,
                    "{}: kind replace names {} edits",
                    row.case_id,
                    edits.len()
                );
                assert!(
                    !first.original.is_empty() && !first.replacement.is_empty(),
                    "{}: a replace edit has an empty side",
                    row.case_id
                );
            }
            "insert" => {
                assert!(
                    edits.len() == 1,
                    "{}: kind insert names {} edits",
                    row.case_id,
                    edits.len()
                );
                assert!(
                    first.original.is_empty(),
                    "{}: an insert edit names original text",
                    row.case_id
                );
                assert!(
                    first.start == first.end,
                    "{}: an insert edit spans {}..{}, not a point",
                    row.case_id,
                    first.start,
                    first.end
                );
                assert!(
                    !first.replacement.is_empty(),
                    "{}: an insert edit writes nothing",
                    row.case_id
                );
            }
            "delete" => {
                assert!(
                    edits.len() == 1,
                    "{}: kind delete names {} edits",
                    row.case_id,
                    edits.len()
                );
                assert!(
                    first.replacement.is_empty(),
                    "{}: a delete edit writes text",
                    row.case_id
                );
                assert!(
                    !first.original.is_empty(),
                    "{}: a delete edit removes nothing",
                    row.case_id
                );
            }
            "replace-set" => assert!(
                edits.len() >= 2,
                "{}: a replace-set names {} edit(s)",
                row.case_id,
                edits.len()
            ),
            other => panic!("{}: unknown kind {:?}", row.case_id, other),
        }
    }
}

#[test]
fn every_reference_case_agrees_with_its_design_record() {
    for row in case_rows()
        .into_iter()
        .filter(|row| row.reference_design == "yes")
    {
        let edits: Vec<Edit> = case_edits(&row);
        let records: Vec<Json> = records_for(short_id(&row.case_id));
        assert!(
            !records.is_empty(),
            "{}: flagged reference_design, so designs.json must hold a record",
            row.case_id
        );
        assert!(
            edits.len() == records.len(),
            "{}: cases.tsv names {} edits, designs.json holds {} records",
            row.case_id,
            edits.len(),
            records.len()
        );
        // q5p-07 stores its two edits as one per variant, `variant-a` then
        // `variant-b` in the order `designs.json` writes them; the row lists
        // the same two edits in that order, so the comparison is exact.
        for (edit, record) in edits.iter().zip(&records) {
            let design: &Json = record.get("edit");
            assert!(
                edit.start == design.get("start").integer()
                    && edit.end == design.get("end").integer(),
                "{}: edit {}..{} vs the record's {}..{}",
                row.case_id,
                edit.start,
                edit.end,
                design.get("start").integer(),
                design.get("end").integer()
            );
            assert!(
                edit.original == design.get("original").text().to_uppercase(),
                "{}: edit original differs from the record's",
                row.case_id
            );
            assert!(
                edit.replacement == design.get("replacement").text().to_uppercase(),
                "{}: edit replacement differs from the record's",
                row.case_id
            );
            let request: &Json = record.get("request");
            let supplied: i64 = integer_of(&row.case_id, "min_overlap", row.min_overlap.clone());
            let stated: i64 = request.get("minOverlapSupplied").integer();
            assert!(
                supplied == stated,
                "{}: min_overlap {} vs request.minOverlapSupplied {}",
                row.case_id,
                supplied,
                stated
            );
            same_number(
                &row.case_id,
                "target_tm vs request.targetTm",
                decimal_of(&row.case_id, "target_tm", row.target_tm.clone()),
                request.get("targetTm").decimal(),
            );
        }
    }
}

#[test]
fn every_case_names_only_defined_invariants() {
    // §7 of `docs/q5-primer-design.md` numbers the hard requirements H1-H8;
    // §14.1 numbers the invariants I-1 to I-13.
    let hard: Vec<String> = (1..=8).map(|n| format!("H{}", n)).collect();
    let soft: Vec<String> = (1..=13).map(|n| format!("I-{}", n)).collect();
    for row in case_rows() {
        let tokens: Vec<String> = row
            .invariants
            .split(',')
            .map(|token| token.to_string())
            .collect();
        assert!(
            !tokens.is_empty(),
            "{}: the invariants column names nothing",
            row.case_id
        );
        for token in tokens.iter() {
            assert!(
                hard.contains(token) || soft.contains(token),
                "{}: invariant {:?} is not one of H1-H8 or I-1-I-13",
                row.case_id,
                token
            );
        }
    }
}

#[test]
fn every_reference_check_matches_its_record() {
    for row in case_rows() {
        if row.reference_design == "yes" {
            let records: Vec<Json> = records_for(short_id(&row.case_id));
            assert!(
                !records.is_empty(),
                "{}: flagged reference_design, so designs.json must hold a record",
                row.case_id
            );
            // The same field `luria_reaches_the_same_h1_outcome_as_each_reference_design`
            // reads: each record's `h1.satisfied`.
            let satisfied: Vec<bool> = records
                .iter()
                .map(|record| record.get("h1").get("satisfied").flag())
                .collect();
            let all: bool = satisfied.iter().all(|outcome| *outcome);
            assert!(
                all == satisfied.iter().any(|outcome| *outcome),
                "{}: its records disagree on H1: {:?}",
                row.case_id,
                satisfied
            );
            let claimed: &str = if all { "H1=pass" } else { "H1=VIOLATED" };
            assert!(
                row.reference_check.starts_with(claimed),
                "{}: designs.json H1 {} vs reference_check {:?}",
                row.case_id,
                all,
                row.reference_check
            );
        } else {
            assert!(
                row.reference_check.starts_with("unverified("),
                "{}: reference_check {:?} for a case with no design record",
                row.case_id,
                row.reference_check
            );
        }
    }
}

/// A `cases.tsv` row's single edit, applied through the mutation layer as
/// `native_edit` does: a nucleotide substitution for one base, a residue
/// substitution on a one-codon CDS for three. The result is verified, which is
/// what a design consumes (§2.1).
fn case_edit(row: &CaseRow, template: &Sequence) -> MutationResult {
    let edits: Vec<Edit> = case_edits(row);
    assert!(
        edits.len() == 1,
        "{}: the row names {} edits, one is expected",
        row.case_id,
        edits.len()
    );
    let edit: &Edit = edits.first().unwrap();
    let edited: Identifier = Identifier::create("q5p-edited".to_string()).unwrap();
    let (made, cds): (MutationResult, Option<Cds>) = match edit.end - edit.start {
        1 => {
            let intent: NucleotideMutationIntent = NucleotideMutationIntent::create(
                template.reference(),
                create(&template.reference(), edit.start, edit.end).unwrap(),
                edit.original.clone(),
                edit.replacement.clone(),
            )
            .unwrap();
            let made: MutationResult = apply_nucleotide(template, None, edited, intent)
                .unwrap_or_else(|error| panic!("{}: edit rejected: {:?}", row.case_id, error));
            (made, None)
        }
        3 => {
            let cds_id: Identifier = Identifier::create("edited-codon".to_string()).unwrap();
            let cds: Cds = Cds::create(
                template,
                cds_id.clone(),
                vec![create(&template.reference(), edit.start, edit.end).unwrap()],
                Strand::Plus,
                Completeness::Complete,
                "standard".to_string(),
                Numbering::FromCdsStart,
            )
            .unwrap_or_else(|error| panic!("{}: one-codon CDS rejected: {:?}", row.case_id, error));
            let intent: ResidueMutationIntent = ResidueMutationIntent::create(
                ResiduePosition::create(cds_id, 1).unwrap(),
                residue_symbol(&row.case_id, "original", &edit.original),
                residue_symbol(&row.case_id, "replacement", &edit.replacement),
                CodonSelectionPolicy::Supplied(Codon::create(edit.replacement.clone()).unwrap()),
            )
            .unwrap();
            let made: MutationResult = apply_residue(template, cds.clone(), edited, intent)
                .unwrap_or_else(|error| panic!("{}: edit rejected: {:?}", row.case_id, error));
            (made, Some(cds))
        }
        other => panic!("{}: a {}-base edit has no row form", row.case_id, other),
    };
    verify(template, cds, made)
        .unwrap_or_else(|error| panic!("{}: edit does not verify: {:?}", row.case_id, error))
}

#[test]
fn the_no_design_case_gets_no_design_from_luria() {
    let row: CaseRow = case_row("q5p-09");
    assert!(
        row.expected == "no-design",
        "q5p-09: expected {:?}, the case is the no-design case",
        row.expected
    );
    let template: Sequence = template_of(&row);
    let request: PrimerDesignRequest = PrimerDesignRequest::create(
        DesignStyle::Q5Polymerase,
        DesignPolicy::LuriaNative,
        integer_of(&row.case_id, "min_overlap", row.min_overlap.clone()),
        PrimerPlacement::Inline,
        BufferModel::q5_reaction(),
    )
    .unwrap();
    match design_primers(&template, case_edit(&row, &template), request) {
        Err(DomainError::NoPrimerDesignPossible(problem)) => assert!(
            problem.template_length == 30,
            "q5p-09: I-9 names {} template bases, expected 30",
            problem.template_length
        ),
        other => panic!(
            "q5p-09: expected NoPrimerDesignPossible, got {:?}",
            other.map(|_| "a design")
        ),
    }
}

#[test]
fn the_no_tm_target_case_reports_the_tm_input_as_unresolved() {
    let row: CaseRow = case_row("q5p-10");
    assert!(
        row.target_tm == "absent",
        "q5p-10: target_tm {:?}, the case states it absent",
        row.target_tm
    );
    let template: Sequence = template_of(&row);
    let request: PrimerDesignRequest = PrimerDesignRequest::create(
        DesignStyle::Q5Polymerase,
        DesignPolicy::LuriaNative,
        integer_of(&row.case_id, "min_overlap", row.min_overlap.clone()),
        PrimerPlacement::Inline,
        BufferModel::q5_reaction(),
    )
    .unwrap();
    let design: PrimerDesign = design_primers(&template, case_edit(&row, &template), request)
        .unwrap_or_else(|error| panic!("q5p-10: expected a design, got {:?}", error));
    // I-10: an absent constraint appears as absent in the output, and
    // `unresolved` is the accessor that reports it.
    let inputs: Vec<String> = design
        .unresolved()
        .into_iter()
        .map(|input| input.input)
        .collect();
    assert!(
        inputs.iter().any(|input| input == "minTm"),
        "q5p-10: unresolved inputs {:?}, expected one naming minTm",
        inputs
    );
}

// ---------------------------------------------------------------------------
// The NEBaseChanger files of `tests/fixtures/example-construct/` (C6, C9).
// ---------------------------------------------------------------------------

/// `raw/sdmsummary.txt`: the header and its single data row paired by column.
/// The header repeats `Length` for the two primers, so a lookup returns every
/// column of that name.
fn summary_fields() -> Vec<(String, String)> {
    let text: String = read("../example-construct/raw/sdmsummary.txt");
    let lines: Vec<String> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string())
        .collect();
    assert!(
        lines.len() == 2,
        "sdmsummary.txt has {} lines, expected a header and one row",
        lines.len()
    );
    let header: Vec<String> = lines[0].split('\t').map(|part| part.to_string()).collect();
    let row: Vec<String> = lines[1].split('\t').map(|part| part.to_string()).collect();
    assert!(
        header.len() == row.len(),
        "sdmsummary.txt row has {} fields, the header has {}",
        row.len(),
        header.len()
    );
    header
        .iter()
        .zip(&row)
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

/// The one summary column of that name.
fn summary_value(name: &str) -> String {
    let found: Vec<String> = summary_fields()
        .into_iter()
        .filter(|(column, _)| column == name)
        .map(|(_, value)| value)
        .collect();
    assert!(
        found.len() == 1,
        "sdmsummary.txt has {} columns named {:?}",
        found.len(),
        name
    );
    found.into_iter().next().unwrap()
}

/// Every summary column of that name: `Length` appears twice, once per primer.
fn summary_values(name: &str) -> Vec<String> {
    summary_fields()
        .into_iter()
        .filter(|(column, _)| column == name)
        .map(|(_, value)| value)
        .collect()
}

/// `raw/sdmprimer.txt`: one line per primer, tab-separated `id`, `sequence`,
/// and `Tm=<n> Ta=<n>`.
fn primer_lines() -> Vec<(String, String, f64, f64)> {
    let text: String = read("../example-construct/raw/sdmprimer.txt");
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let fields: Vec<String> = line.split('\t').map(|part| part.to_string()).collect();
            assert!(
                fields.len() == 3,
                "sdmprimer.txt line {:?} has {} fields, expected id, sequence, temperatures",
                line,
                fields.len()
            );
            let temperatures: Vec<String> =
                fields[2].split(' ').map(|part| part.to_string()).collect();
            assert!(
                temperatures.len() == 2,
                "sdmprimer.txt line {:?} has {} temperature fields, expected Tm and Ta",
                line,
                temperatures.len()
            );
            let value = |name: &str, cell: String| -> f64 {
                let digits: String = if let Some(stripped) = cell.strip_prefix(name) {
                    stripped.to_string()
                } else {
                    panic!("sdmprimer.txt: {:?} does not state {:?}", cell, name)
                };
                decimal_of("sdmprimer.txt", name, digits)
            };
            (
                fields[0].clone(),
                fields[1].clone(),
                value("Tm=", temperatures[0].clone()),
                value("Ta=", temperatures[1].clone()),
            )
        })
        .collect()
}

#[test]
fn the_reference_script_reproduces_the_nebasechanger_pair() {
    let designs: Json = designs_json();
    let record: &Json = designs.get("q5p-01");
    assert!(
        summary_value("Fwd Primer Seq").to_uppercase()
            == record.get("forward").get("sequence").text(),
        "q5p-01: sdmsummary.txt Fwd Primer Seq differs from forward.sequence"
    );
    assert!(
        summary_value("Rev Primer Seq").to_uppercase()
            == record.get("reverse").get("sequence").text(),
        "q5p-01: sdmsummary.txt Rev Primer Seq differs from reverse.sequence"
    );
    let printed_lengths: Vec<String> = summary_values("Length");
    assert!(
        printed_lengths.len() == 2,
        "sdmsummary.txt has {} Length columns, expected one per primer",
        printed_lengths.len()
    );
    let lengths: Vec<(&str, f64, f64)> = vec![
        (
            "Length (forward)",
            decimal_of("q5p-01", "Length", printed_lengths[0].clone()),
            record.get("forward").get("length").decimal(),
        ),
        (
            "Length (reverse)",
            decimal_of("q5p-01", "Length", printed_lengths[1].clone()),
            record.get("reverse").get("length").decimal(),
        ),
    ];
    for (name, printed, recorded) in lengths {
        same_number("q5p-01", name, printed, recorded);
    }
    // The summary prints a Tm rounded to the nearest integer; the record
    // carries the unrounded value.
    let temperatures: Vec<(&str, f64, f64)> = vec![
        (
            "Tm Fwd",
            decimal_of("q5p-01", "Tm Fwd", summary_value("Tm Fwd").clone()),
            record.get("forward").get("tm").decimal(),
        ),
        (
            "Tm Rev",
            decimal_of("q5p-01", "Tm Rev", summary_value("Tm Rev").clone()),
            record.get("reverse").get("tm").decimal(),
        ),
    ];
    for (name, printed, recorded) in temperatures {
        rounded_number("q5p-01", name, printed, recorded);
    }
    // The record's `ta` is optional: q5p-01 records one, so the summary's
    // `Ta` is compared against it rounded. A record without a `ta` states no
    // recommendation, and this comparison is skipped for it.
    if let Some(recommended) = record.opt("ta").map(Json::decimal) {
        rounded_number(
            "q5p-01",
            "Ta",
            decimal_of("q5p-01", "Ta", summary_value("Ta").clone()),
            recommended,
        );
    }
}

#[test]
fn the_primer_list_agrees_with_the_summary() {
    let lines: Vec<(String, String, f64, f64)> = primer_lines();
    assert!(
        lines.len() == 2,
        "sdmprimer.txt has {} primer lines, expected two",
        lines.len()
    );
    let summary: Vec<(String, f64, f64)> = vec![
        (
            summary_value("Fwd Primer Seq"),
            decimal_of("sdmsummary.txt", "Tm Fwd", summary_value("Tm Fwd").clone()),
            decimal_of("sdmsummary.txt", "Ta", summary_value("Ta").clone()),
        ),
        (
            summary_value("Rev Primer Seq"),
            decimal_of("sdmsummary.txt", "Tm Rev", summary_value("Tm Rev").clone()),
            decimal_of("sdmsummary.txt", "Ta", summary_value("Ta").clone()),
        ),
    ];
    for ((id, sequence, printed_tm, printed_ta), (want, want_tm, want_ta)) in
        lines.into_iter().zip(summary)
    {
        assert!(
            sequence.to_uppercase() == want.to_uppercase(),
            "sdmprimer.txt primer {:?} differs from the summary's primer",
            id
        );
        same_number("sdmprimer.txt", &format!("{} Tm", id), printed_tm, want_tm);
        same_number("sdmprimer.txt", &format!("{} Ta", id), printed_ta, want_ta);
    }
}

#[test]
fn the_normalized_record_transcribes_the_summary() {
    let expected: Json = Reader::parse(&read(
        "../example-construct/expected/expected_mutation.json",
    ));
    let forward: &Json = expected.get("forward_primer");
    let reverse: &Json = expected.get("reverse_primer");
    assert!(
        expected.get("id").text() == summary_value("ID"),
        "expected_mutation.json id differs from sdmsummary.txt ID"
    );
    assert!(
        forward.get("id").text() == summary_value("Fwd Primer ID"),
        "expected_mutation.json forward_primer.id differs from Fwd Primer ID"
    );
    assert!(
        reverse.get("id").text() == summary_value("Rev Primer ID"),
        "expected_mutation.json reverse_primer.id differs from Rev Primer ID"
    );
    // The sequences are transcribed exactly, case preserved: the lower-case
    // run marks the replacement, and a transcription must keep it.
    assert!(
        forward.get("sequence").text() == summary_value("Fwd Primer Seq"),
        "expected_mutation.json forward_primer.sequence differs from Fwd Primer Seq"
    );
    assert!(
        reverse.get("sequence").text() == summary_value("Rev Primer Seq"),
        "expected_mutation.json reverse_primer.sequence differs from Rev Primer Seq"
    );
    let printed_lengths: Vec<String> = summary_values("Length");
    assert!(
        printed_lengths.len() == 2,
        "sdmsummary.txt has {} Length columns, expected one per primer",
        printed_lengths.len()
    );
    let numbers: Vec<(&str, f64, f64)> = vec![
        (
            "forward_primer.length",
            forward.get("length").decimal(),
            decimal_of("sdmsummary.txt", "Length", printed_lengths[0].clone()),
        ),
        (
            "reverse_primer.length",
            reverse.get("length").decimal(),
            decimal_of("sdmsummary.txt", "Length", printed_lengths[1].clone()),
        ),
        (
            "forward_primer.tm",
            forward.get("tm").decimal(),
            decimal_of("sdmsummary.txt", "Tm Fwd", summary_value("Tm Fwd").clone()),
        ),
        (
            "reverse_primer.tm",
            reverse.get("tm").decimal(),
            decimal_of("sdmsummary.txt", "Tm Rev", summary_value("Tm Rev").clone()),
        ),
        (
            "ta_recommended",
            expected.get("ta_recommended").decimal(),
            decimal_of("sdmsummary.txt", "Ta", summary_value("Ta").clone()),
        ),
    ];
    for (name, found, want) in numbers {
        same_number("expected_mutation.json", name, found, want);
    }
    assert!(
        expected.get("replacement_bases").text() == "AAC",
        "expected_mutation.json replacement_bases differs from AAC"
    );
    let range: Vec<u64> = integers(expected.get("wild_type_range"));
    assert!(
        range == vec![1210, 1212],
        "expected_mutation.json wild_type_range is {:?}, expected [1210, 1212]",
        range
    );
}

#[test]
fn the_mutation_spec_is_case_q5p_01() {
    let spec: Json = Reader::parse(&read("../example-construct/input/mutation_spec.json"));
    let row: CaseRow = case_row("q5p-01");
    let edits: Vec<Edit> = case_edits(&row);
    assert!(
        edits.len() == 1,
        "q5p-01: {} edits, expected one",
        edits.len()
    );
    let edit: &Edit = edits.first().unwrap();
    // The spec states a one-based position; the case states a zero-based
    // half-open interval (D004, D015).
    assert!(
        spec.get("target_position").integer() - 1 == edit.start,
        "mutation_spec.json target_position {} is not the case's start {} plus one",
        spec.get("target_position").integer(),
        edit.start
    );
    assert!(
        spec.get("original_sequence").text() == edit.original,
        "mutation_spec.json original_sequence differs from q5p-01's edit original"
    );
    assert!(
        spec.get("desired_sequence").text() == edit.replacement,
        "mutation_spec.json desired_sequence differs from q5p-01's edit replacement"
    );
    let wanted: i64 = integer_of(&row.case_id, "min_overlap", row.min_overlap.clone());
    let stated: i64 = spec.get("minimum_overlap_bp").integer();
    assert!(
        stated == wanted,
        "mutation_spec.json minimum_overlap_bp {} vs q5p-01 min_overlap {}",
        stated,
        wanted
    );
}

#[test]
fn luria_reproduces_the_nebasechanger_final_construct() {
    let row: CaseRow = case_row("q5p-01");
    let template: Sequence = template_of(&row);
    let design: Design = reference_designs()
        .into_iter()
        .find(|design| design.case_id == "q5p-01")
        .unwrap_or_else(|| panic!("designs.json holds no q5p-01 record"));
    let result: MutationResult = edit_of(&design, &template);
    let raw: String = read("../example-construct/raw/sdmfinalseq.fa");
    let lines: Vec<String> = raw.lines().map(|line| line.to_string()).collect();
    assert!(
        lines.len() >= 2,
        "sdmfinalseq.fa has {} lines, expected a header and sequence text",
        lines.len()
    );
    let header: String = lines[0].clone();
    assert!(
        header.starts_with(">"),
        "sdmfinalseq.fa first line is not a FASTA header: {:?}",
        header
    );
    let written: String = lines[1..].join("").to_uppercase();
    assert!(
        written == result.edited().canonical_text(),
        "sdmfinalseq.fa differs from the edited artifact's canonical text"
    );
    // The header states the edit one-based inclusive; the fixture README
    // records that `replaced 1210 to 1212` is the interval and that the
    // description's `between bases 1209-1213` is not. One-based inclusive
    // [1210, 1212] is the zero-based half-open [1209, 1212) the case states.
    assert!(
        header.contains("replaced 1210 to 1212"),
        "sdmfinalseq.fa header {:?} does not state the replaced interval",
        header
    );
    let edits: Vec<Edit> = case_edits(&row);
    assert!(
        edits.len() == 1,
        "q5p-01: {} edits, expected one",
        edits.len()
    );
    let edit: &Edit = edits.first().unwrap();
    assert!(
        edit.start == 1209 && edit.end == 1212,
        "q5p-01: the edit is [{}, {}), expected [1209, 1212)",
        edit.start,
        edit.end
    );
}
