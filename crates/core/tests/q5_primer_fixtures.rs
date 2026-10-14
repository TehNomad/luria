//! Integration tests driven by the fixtures in `tests/fixtures/q5-primer`.
//!
//! The reference designs in `reference/designs.json` were produced by
//! `reference/neb-q5-sdm-reference.py`, an independent transcription of the
//! reference tool's primer construction. These tests feed each reference design
//! through `primer.rs` and check that luria derives the same quantities the
//! generator recorded: the decomposition, the annealing overlap, the
//! whole-primer match count, the pair relationship, and the H1 outcome.
//!
//! Nothing here asserts a luria design. luria has no design algorithm yet; the
//! reference designs are inputs to the validation layer, not expected outputs.
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
use luria_core::error::DomainError;
use luria_core::genetic_code::lookup;
use luria_core::genetic_code::CodonOutcome;
use luria_core::identity::Identifier;
use luria_core::interval::create;
use luria_core::interval::NucleotideInterval;
use luria_core::mutation::apply_residue;
use luria_core::mutation::Codon;
use luria_core::mutation::CodonSelectionPolicy;
use luria_core::mutation::MutationResult;
use luria_core::mutation::ResidueMutationIntent;
use luria_core::primer::assemble_pair;
use luria_core::primer::construct_primer;
use luria_core::primer::require_design_constraints;
use luria_core::primer::Primer;
use luria_core::primer::PrimerPair;
use luria_core::primer::PrimerPlacement;
use luria_core::primer::PrimerRole;
use luria_core::sequence::normalize;
use luria_core::sequence::Sequence;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;

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

struct CaseRow {
    case_id: String,
    sequence_file: String,
    sequence_digest_16: String,
    reference_design: String,
}

fn case_rows() -> Vec<CaseRow> {
    let text: String = read("cases.tsv");
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header: Vec<&str> = lines
        .next()
        .expect("cases.tsv has a header")
        .split('\t')
        .collect();
    let column = |name: &str| -> usize {
        header
            .iter()
            .position(|column| *column == name)
            .unwrap_or_else(|| panic!("cases.tsv has no column {:?}", name))
    };
    let (id, file, digest, design) = (
        column("case_id"),
        column("sequence_file"),
        column("sequence_digest_16"),
        column("reference_design"),
    );
    lines
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert!(
                fields.len() == header.len(),
                "cases.tsv row has {} fields, header has {}",
                fields.len(),
                header.len()
            );
            CaseRow {
                case_id: fields[id].to_string(),
                sequence_file: fields[file].to_string(),
                sequence_digest_16: fields[digest].to_string(),
                reference_design: fields[design].to_string(),
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
    let raw: String = read(&row.sequence_file);
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
    .unwrap_or_else(|error| panic!("{}: template does not normalize: {:?}", row.case_id, error))
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
