//! The IDT order table (milestone 7g2, plan §5.4, M7-4, M7-7 to M7-13): the
//! rows `luria` puts into IDT's Bulk Input, built from a result document.
//!
//! Pure: no I/O, no terminal, no file name (7g1 writes the file, 7g3 shows the
//! rows on screen), so every rule of §5.4 is testable as a string. Every value
//! is the document's own: the sequence is `PrimerView.sequence` exactly, tail
//! included, and the scale is a fixed rule by `PrimerView.length` (M7-6), never
//! a choice on screen, never taken from the profile. Every row is `STD`
//! (M7-10). A primer the rule does not cover, under 15 or over 200 nt, and a
//! sequence holding a base other than `A`, `C`, `G` or `T` are refused with the
//! row's name, never given a guessed value (D005). A member with no design gets
//! no row and is named by the caller as left out. luria writes a file and never
//! submits an order (product brief, non-goals).

use crate::result::{PrimerView, ResultDocument, TemplateView};

/// The IDT scale code for a primer of `length` bases (plan §5.4, M7-10,
/// M7-13), or `None` outside 15 to 200.
pub fn scale(length: u64) -> Option<&'static str> {
    if length < 15 {
        return None;
    }
    if length <= 60 {
        return Some("25nm");
    }
    if length <= 90 {
        return Some("100nm");
    }
    if length <= 100 {
        return Some("250nm");
    }
    if length <= 200 {
        return Some("4nmU");
    }
    None
}

/// Why no table can be built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderRefusal {
    NotVerified { state: String },
    NothingDesigned,
    NoTemplate,
    LengthOutsideScales { name: String, length: u64 },
    NotPlainDna { name: String },
}

impl OrderRefusal {
    /// One line for the screen.
    pub fn message(&self) -> String {
        match self {
            OrderRefusal::NotVerified { state } => {
                format!("no order table: the result is {}, not verified", state)
            }
            OrderRefusal::NothingDesigned => {
                "no order table: no member has a primer pair".to_string()
            }
            OrderRefusal::NoTemplate => "no order table: the result names no template".to_string(),
            OrderRefusal::LengthOutsideScales { name, length } => format!(
                "no order table: {} is {} nt; IDT scales here cover 15 to 200 nt",
                name, length
            ),
            OrderRefusal::NotPlainDna { name } => format!(
                "no order table: {} holds a base other than A, C, G or T",
                name
            ),
        }
    }
}

/// The table, as the file's bytes, and the names of the members left out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderTable {
    pub text: String,
    pub left_out: Vec<String>,
}

/// The header row, tab-separated (M7-11, M7-7).
const HEADER: &str = "Name\tSequence\tScale\tPurification";

/// One primer's row: the name `<template id>_<member name>_F` or `_R` (M7-8,
/// M7-12), the sequence exactly as the document holds it, the scale by its full
/// length, and `STD` on every row (M7-10). The length check first, then the
/// base check: neither is given a guessed value (D005).
fn row_of(name: &str, primer: &PrimerView) -> Result<String, OrderRefusal> {
    let code: &'static str = match scale(primer.length) {
        Some(code) => code,
        None => {
            return Err(OrderRefusal::LengthOutsideScales {
                name: name.to_string(),
                length: primer.length,
            })
        }
    };
    if !primer
        .sequence
        .chars()
        .all(|base| matches!(base, 'A' | 'C' | 'G' | 'T'))
    {
        return Err(OrderRefusal::NotPlainDna {
            name: name.to_string(),
        });
    }
    Ok(format!("{}\t{}\t{}\tSTD", name, primer.sequence, code))
}

/// The order table for a run's result document (plan §5.4): the header, then
/// one row per primer of every designed member, forward then reverse, members in
/// `results` order, tab-separated, `\n` after every row (M7-7, M7-11). The
/// checks of §5.4 run in that list's order, and the first that fails is
/// returned; the length and base checks run over every primer in row order,
/// before any text. `left_out` holds the name of every member without a design,
/// in order.
pub fn order_table(document: &ResultDocument) -> Result<OrderTable, OrderRefusal> {
    // 1. A result that is not `verified` orders nothing (§5.4).
    if document.state != "verified" {
        return Err(OrderRefusal::NotVerified {
            state: document.state.to_string(),
        });
    }
    // 2. A result with no designed member orders nothing.
    if document
        .results
        .iter()
        .all(|member| member.design.is_none())
    {
        return Err(OrderRefusal::NothingDesigned);
    }
    // 3. The row names need the template's identifier (M7-8, M7-12).
    let template: &TemplateView = match document.provenance.template.as_ref() {
        Some(named) => named,
        None => return Err(OrderRefusal::NoTemplate),
    };

    // 4. and 5. The length and base checks, over every primer in row order.
    let mut rows: Vec<String> = Vec::new();
    let mut left_out: Vec<String> = Vec::new();
    for member in document.results.iter() {
        match member.design.as_ref() {
            Some(design) => {
                for (suffix, primer) in [("_F", &design.forward), ("_R", &design.reverse)] {
                    let name: String = format!("{}_{}{}", template.id, member.name, suffix);
                    rows.push(row_of(&name, primer)?);
                }
            }
            // A member with a design failure, or with neither, gets no row and
            // is listed as left out (§5.4, §3 of the contract).
            None => left_out.push(member.name.clone()),
        }
    }

    let mut lines: Vec<String> = Vec::new();
    lines.push(HEADER.to_string());
    lines.extend(rows);
    Ok(OrderTable {
        text: lines.iter().map(|line| format!("{}\n", line)).collect(),
        left_out,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::{
        AlgorithmRef, AnnealingView, DesignView, EditedView, ErrorView, MemberView, MutationView,
        PackageView, PairView, PartView, SourceView, TmView,
    };

    /// A primer's row name, as the table writes it (M7-8, M7-12).
    fn row_name(suffix: &str) -> String {
        format!("sid-1_m_{}", suffix)
    }

    /// A Tm, written by hand: no basis substitution, so a test sets the one
    /// field it is about. As `text.rs`'s `tm` is.
    fn tm() -> TmView {
        TmView {
            celsius: 60.0,
            basis: "matched_complement".to_string(),
            substituted_because: None,
            enthalpy_kcal: 0.0,
            entropy_cal: 0.0,
        }
    }

    /// An annealing portion written by hand, covering `span`. As `text.rs`'s
    /// `annealing` is.
    fn annealing(text: &str, span: (u64, u64)) -> AnnealingView {
        AnnealingView {
            sequence: text.to_string(),
            length: text.len() as u64,
            overlap: text.len() as u64,
            spans: vec![span],
        }
    }

    /// One part of a primer, written by hand: the `text` at the primer indices
    /// from `primer_start`, annealing, sitting at the template `span`.
    fn part(text: &str, span: (u64, u64)) -> PartView {
        PartView {
            role: "context".to_string(),
            annealing: true,
            sequence: text.to_string(),
            span: Some(span),
            primer_start: 0,
            primer_end: text.len() as u64,
        }
    }

    /// A primer, written by hand: no tail, no whole-primer Tm and no warning, so
    /// a test sets the one field it is about. Nothing here is a scientific
    /// value, as the primers a test builds are not core's. As `text.rs`'s
    /// `primer` is, and its `length` is its text's length, as core states it.
    fn primer(role: &str, bases: &str) -> PrimerView {
        let span: (u64, u64) = (0, bases.len() as u64);
        PrimerView {
            role: role.to_string(),
            sequence: bases.to_string(),
            length: bases.len() as u64,
            span,
            edited_span: None,
            tail: None,
            parts: vec![part(bases, span)],
            annealing: annealing(bases, span),
            tm: tm(),
            whole_primer_tm: None,
            warnings: Vec::new(),
        }
    }

    /// A primer of `n` `A` bases, so only its length is the number under test.
    fn primer_of(role: &str, n: usize) -> PrimerView {
        primer(role, &"A".repeat(n))
    }

    /// A design carrying these two primers, with the geometry of a hand-built
    /// pair. As `text.rs`'s `design` is.
    fn design(forward: PrimerView, reverse: PrimerView) -> DesignView {
        DesignView {
            policy: "luriaNative".to_string(),
            style: "q5_polymerase".to_string(),
            algorithm: AlgorithmRef {
                name: "luria-core-primer-design-native",
                version: "0.7.0",
            },
            min_annealing_overlap: 15,
            placement: "inline".to_string(),
            edit_span: (0, 3),
            forward,
            reverse,
            annealing_temperature: 60.0,
            tm_balance_met: true,
            pair: PairView {
                deleted_width: 0,
                separation: 0,
                boundary: None,
                abuts: false,
                relationship: "hand-built pair".to_string(),
            },
            unresolved: Vec::new(),
        }
    }

    /// The design failure a member carries when no pair fits, as `run.rs` writes
    /// one for `A20G` (G7, D029). Only its `code` is read here.
    fn failure() -> ErrorView {
        ErrorView {
            code: "no_primer_design_fits",
            rule: None,
            category: "unsupported",
            message: "no primer pair fits".to_string(),
            path: None,
            details: crate::result::Details(Vec::new()),
        }
    }

    /// A member with `name` and the mutation of `edit-a42g.json`, as `text.rs`'s
    /// `member` is: no design and no failure, so a test gives it one.
    fn member(name: &str) -> MemberView {
        MemberView {
            name: name.to_string(),
            mutation: MutationView::Substitution {
                state: "verified".to_string(),
                interval: (165, 168),
                original: "GCA".to_string(),
                written: "GGC".to_string(),
                coordinates: vec![166],
                edited: EditedView {
                    id: "synlin-A42G".to_string(),
                    sha256: "0".to_string(),
                    length: 393,
                },
            },
            warnings: Vec::new(),
            design: None,
            design_failure: None,
        }
    }

    /// The member whose name is `m`, designed with `forward` and `reverse`.
    fn designed_member(forward: PrimerView, reverse: PrimerView) -> MemberView {
        let mut member: MemberView = member("m");
        member.design = Some(design(forward, reverse));
        member
    }

    /// A template named by `id`, so the row names are the ones under test. The
    /// digest is a placeholder: `order` reads the `id` only.
    fn template(id: &str) -> TemplateView {
        TemplateView {
            id: id.to_string(),
            sha256: "0".to_string(),
            length: 393,
            topology: "linear".to_string(),
            source: SourceView::Fasta {
                path: "synthetic-linear.fa".to_string(),
                header: "a template".to_string(),
            },
            genbank: None,
        }
    }

    /// A document, for the table to be built from: verified, naming the `sid-1`
    /// template, with `members` as its results and nothing else.
    fn document(members: Vec<MemberView>) -> ResultDocument {
        ResultDocument {
            schema: crate::RESULT_SCHEMA,
            state: "verified",
            experimental_validation: "not_performed",
            provenance: crate::result::Provenance {
                package: PackageView {
                    name: env!("CARGO_PKG_NAME"),
                    version: env!("CARGO_PKG_VERSION"),
                },
                algorithms: Vec::new(),
                request_sha256: None,
                operation: None,
                template: Some(template("sid-1")),
                buffer: None,
            },
            request: None,
            target: None,
            clarifications: Vec::new(),
            candidates: Vec::new(),
            warnings: Vec::new(),
            results: members,
            error: None,
        }
    }

    /// The table's rows, split so a test states one row exactly. The `\n` after
    /// the last row gives no extra row.
    fn rows(text: &str) -> Vec<String> {
        text.lines().map(|line| line.to_string()).collect()
    }

    #[test]
    fn the_scale_follows_the_length() {
        let cases: Vec<(u64, Option<&'static str>)> = vec![
            (14, None),
            (15, Some("25nm")),
            (60, Some("25nm")),
            (61, Some("100nm")),
            (90, Some("100nm")),
            (91, Some("250nm")),
            (100, Some("250nm")),
            (101, Some("4nmU")),
            (200, Some("4nmU")),
            (201, None),
        ];
        for (length, expected) in cases {
            assert_eq!(scale(length), expected, "a primer of {} nt", length);
        }
    }

    #[test]
    fn a_primer_outside_the_scales_refuses_the_table() {
        // A forward primer of 201 nt, past the Ultramer limit (M7-13), with a
        // reverse primer the rule does cover: the row name is the forward's.
        assert_eq!(
            order_table(&document(vec![designed_member(
                primer_of("forward", 201),
                primer_of("reverse", 15)
            )])),
            Err(OrderRefusal::LengthOutsideScales {
                name: row_name("F"),
                length: 201
            }),
            "a primer over 200 nt is refused with its name and length"
        );

        // A reverse primer of 14 nt, under the smallest scale: the row name is
        // the reverse's, so the refusal names the row, not the primer's role.
        assert_eq!(
            order_table(&document(vec![designed_member(
                primer_of("forward", 15),
                primer_of("reverse", 14)
            )])),
            Err(OrderRefusal::LengthOutsideScales {
                name: row_name("R"),
                length: 14
            }),
            "a primer under 15 nt is refused with its name and length"
        );
    }

    #[test]
    fn a_long_tail_primer_is_an_ultramer() {
        // A forward primer of 120 nt, as a long insertion tail gives: inside the
        // Ultramer range (M7-13), so the row is `4nmU` and `STD`.
        let table: OrderTable = order_table(&document(vec![designed_member(
            primer_of("forward", 120),
            primer_of("reverse", 15),
        )]))
        .expect("120 nt is inside the Ultramer scale");
        let lines: Vec<String> = rows(&table.text);
        assert_eq!(
            lines[1],
            format!("sid-1_m_F\t{}\t4nmU\tSTD", "A".repeat(120)),
            "the long primer's row is its name, sequence, 4nmU and STD"
        );
        assert_eq!(
            lines[2],
            format!("sid-1_m_R\t{}\t25nm\tSTD", "A".repeat(15)),
            "the reverse primer of 15 nt is 25nm"
        );
        assert_eq!(table.left_out, Vec::<String>::new());
    }

    #[test]
    fn an_iupac_base_refuses_the_table() {
        // A forward primer holding `N`, the IUPAC base for any base.
        assert_eq!(
            order_table(&document(vec![designed_member(
                primer("forward", &format!("A{}", "N".repeat(14))),
                primer_of("reverse", 15)
            )])),
            Err(OrderRefusal::NotPlainDna {
                name: row_name("F")
            }),
            "a sequence with N is refused, not exported"
        );

        // A lower-case `a`: the table writes upper case, so this is not the
        // text a row may hold.
        assert_eq!(
            order_table(&document(vec![designed_member(
                primer("forward", &format!("a{}", "C".repeat(14))),
                primer_of("reverse", 15)
            )])),
            Err(OrderRefusal::NotPlainDna {
                name: row_name("F")
            }),
            "a sequence with a lower-case base is refused, not exported"
        );
    }

    #[test]
    fn a_member_without_a_design_is_left_out() {
        let mut first: MemberView = member("first");
        first.design_failure = Some(failure());
        let table: OrderTable = order_table(&document(vec![
            first,
            designed_member(primer_of("forward", 15), primer_of("reverse", 15)),
        ]))
        .expect("the second member is designed");
        let lines: Vec<String> = rows(&table.text);
        assert_eq!(
            lines.len(),
            3,
            "the header and the two rows of the designed member: {:?}",
            lines
        );
        assert_eq!(lines[0], "Name\tSequence\tScale\tPurification");
        assert_eq!(
            lines[1],
            format!("sid-1_m_F\t{}\t25nm\tSTD", "A".repeat(15)),
            "the designed member's forward row"
        );
        assert_eq!(
            lines[2],
            format!("sid-1_m_R\t{}\t25nm\tSTD", "A".repeat(15))
        );
        assert_eq!(
            table.left_out,
            vec!["first".to_string()],
            "the member with a design failure is named as left out"
        );
    }

    #[test]
    fn a_clarification_is_not_ordered() {
        let mut doc: ResultDocument = document(vec![designed_member(
            primer_of("forward", 15),
            primer_of("reverse", 15),
        )]);
        doc.state = "needs_clarification";
        assert_eq!(
            order_table(&doc),
            Err(OrderRefusal::NotVerified {
                state: "needs_clarification".to_string()
            }),
            "a result that is not verified orders nothing"
        );
    }

    #[test]
    fn no_template_is_refused() {
        let mut doc: ResultDocument = document(vec![designed_member(
            primer_of("forward", 15),
            primer_of("reverse", 15),
        )]);
        doc.provenance.template = None;
        assert_eq!(
            order_table(&doc),
            Err(OrderRefusal::NoTemplate),
            "a result naming no template gives no row name"
        );
    }

    #[test]
    fn every_message_is_one_line() {
        let refusals: Vec<OrderRefusal> = vec![
            OrderRefusal::NotVerified {
                state: "failed".to_string(),
            },
            OrderRefusal::NothingDesigned,
            OrderRefusal::NoTemplate,
            OrderRefusal::LengthOutsideScales {
                name: row_name("F"),
                length: 201,
            },
            OrderRefusal::NotPlainDna {
                name: row_name("R"),
            },
        ];
        let expected: Vec<String> = vec![
            "no order table: the result is failed, not verified".to_string(),
            "no order table: no member has a primer pair".to_string(),
            "no order table: the result names no template".to_string(),
            format!(
                "no order table: {} is 201 nt; IDT scales here cover 15 to 200 nt",
                row_name("F")
            ),
            format!(
                "no order table: {} holds a base other than A, C, G or T",
                row_name("R")
            ),
        ];
        for (refusal, line) in refusals.iter().zip(expected.iter()) {
            assert_eq!(&refusal.message(), line, "{:?}", refusal);
            assert!(
                !refusal.message().contains('\n'),
                "one line: {:?}",
                refusal.message()
            );
        }
    }
}
