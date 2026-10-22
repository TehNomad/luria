//! new: a filled `luria new` form to one request document, or a precise
//! refusal (milestone 7k2, contract 7k2).
//!
//! The form's template, mutation lines, codon choice and overlap become a
//! `RequestDocument` through one pure function with no terminal and no file
//! write: 7k3 builds the form around it, and 7k5 writes the request and the
//! paste. Every rule is `tests/fixtures/new/reference/requests.py`'s, applied
//! in its order, and the request is built from `request.rs`'s types, so
//! `parse_request` and `request_echo` see exactly what a request file carries.
//!
//! # The rules this module applies (§2 of the plan, M7-32 to M7-37)
//!
//! - the template's bases, its digest and its name come from core:
//!   `read_fasta` for FASTA text, where a paste of raw bases is read as one
//!   record named by the form's `name` (M7-32), `read_genbank` for a GenBank
//!   file, and `Sequence::content_hash` for `template.sha256` (5e2's rule, as
//!   `prepare.rs` compares it);
//! - a text, a paste's letters or a name core refuses gives
//!   `template_unreadable` with no line: `build` never panics on what a user
//!   typed (decided 2026-10-21 on w7k2's question);
//! - for FASTA and paste the CDS is the form's: "whole sequence" is
//!   `[0, length]`, typed 1-based inclusive bases are `[first - 1, last]`
//!   (M7-24, D006), and a CDS outside the template is `outside_template`; for
//!   GenBank the CDS is the id picked from 6c's catalogue, and the template's
//!   `id` is the record's LOCUS name;
//! - `scope` is that CDS's id, and a residue edit names it as its `cds`;
//! - every line is read by `shorthand::parse_line` (7k1), whose refusal is
//!   `line_refused` with that line's 1-based number; blank lines are skipped;
//! - a substitution's or replacement's typed `from` must be the template's
//!   bases there, else `from_mismatch`; a deletion's `from`, and an
//!   insertion's `before` and `after`, are read from the template;
//! - a residue's codon is the line's own when it gives one, otherwise the
//!   form's codon choice must be the default policy, else `missing_codon`;
//! - core refuses an insertion at either end of the template, so a point at
//!   the first base or past the last is `insertion_at_end` (D032);
//! - one line gives `edit`, two or more `variants` with `separate_variants`
//!   in line order (M7-34), and a name two lines share is `duplicate_name`;
//! - `design` is `luriaNative`, `q5_polymerase`, the form's overlap, `inline`
//!   placement (M7-39: `q5 design` refuses `confine_to_tail`, D029) and the
//!   Q5 reaction buffer;
//! - a paste carries the file name it will be saved as and the FASTA text
//!   `render_fasta` gives, for 7k5 to write beside the request (M7-37); a
//!   file template names itself with its file name only, as the request is
//!   written beside it (M7-37). Nothing is written here.

use crate::request::{
    BufferSpec, CdsSpec, CodonSpec, CompositionSpec, DesignSpec, EditSpec, MemberSpec,
    OperationSpec, PlacementSpec, PolicySpec, RequestDocument, StrandSpec, StyleSpec, TemplateSpec,
    TopologySpec,
};
use crate::shorthand::{parse_line, Reading};

use luria_core::fasta::{read_fasta, render_fasta, FastaRecord};
use luria_core::genbank::{read_genbank, GenBankRecord};
use luria_core::identity::Identifier;
use luria_core::sequence::{Sequence, SequenceTopology};

pub mod confirm;
pub mod form;
pub mod terminal;
pub mod text;
pub mod view;

/// Where the template comes from.
#[derive(Clone, Debug, PartialEq)]
pub enum TemplateInput {
    /// A FASTA file already read: its file name (no directory) and text.
    Fasta {
        file_name: String,
        text: String,
        name: String,
        topology: TopologySpec,
        cds: CdsChoice,
    },
    /// A pasted sequence: raw bases or FASTA text, the name to give it, and
    /// the file name it will be saved as.
    Paste {
        text: String,
        name: String,
        save_as: String,
        topology: TopologySpec,
        cds: CdsChoice,
    },
    /// A GenBank file already read, and the CDS id the user picked.
    GenBank {
        file_name: String,
        text: String,
        cds: String,
    },
}

/// The CDS a FASTA or pasted template is read through (M7-32); a GenBank
/// record states its own.
#[derive(Clone, Debug, PartialEq)]
pub enum CdsChoice {
    /// The whole sequence, on the form's strand.
    Whole(StrandSpec),
    /// 1-based inclusive bases, as the user typed them.
    Bases {
        first: i64,
        last: i64,
        strand: StrandSpec,
    },
}

/// The form's codon choice: the default policy for every residue, or a codon
/// typed on each line that needs one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodonChoice {
    DefaultPolicy,
    TypedPerMutation,
}

/// One filled form, as 7k3 collects it.
#[derive(Clone, Debug, PartialEq)]
pub struct Form {
    pub template: TemplateInput,
    /// The mutation lines as typed; blank lines are skipped.
    pub lines: Vec<String>,
    pub codon: CodonChoice,
    /// `None` until the user or a `--design` profile gives it.
    pub overlap: Option<u64>,
}

/// What a valid form gives: the request, and the paste's file when the
/// template was pasted.
#[derive(Clone, Debug, PartialEq)]
pub struct Built {
    pub request: RequestDocument,
    /// For a paste: the file name and the FASTA text to write.
    pub paste: Option<(String, String)>,
}

/// Why a form does not build; `code()` gives `refusals.tsv`'s code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildRefusal {
    NoMutations,
    LineRefused,
    FromMismatch,
    OutsideTemplate,
    InsertionAtEnd,
    MissingCodon,
    MissingOverlap,
    DuplicateName,
    /// Core refused the template's text (`read_fasta`, `read_genbank`,
    /// `normalize` or the name's `Identifier`). Added 2026-10-21 on w7k2's
    /// question: `build` never panics on what a user typed.
    TemplateUnreadable,
}

impl BuildRefusal {
    /// The code `refusals.tsv` writes for this refusal.
    pub fn code(self) -> &'static str {
        match self {
            BuildRefusal::NoMutations => "no_mutations",
            BuildRefusal::LineRefused => "line_refused",
            BuildRefusal::FromMismatch => "from_mismatch",
            BuildRefusal::OutsideTemplate => "outside_template",
            BuildRefusal::InsertionAtEnd => "insertion_at_end",
            BuildRefusal::MissingCodon => "missing_codon",
            BuildRefusal::MissingOverlap => "missing_overlap",
            BuildRefusal::DuplicateName => "duplicate_name",
            BuildRefusal::TemplateUnreadable => "template_unreadable",
        }
    }

    /// One English sentence a user can act on, naming what to change on the
    /// form; ASCII or the screens' Unicode, no trailing full stop.
    pub fn message(self) -> &'static str {
        match self {
            BuildRefusal::NoMutations => {
                "the mutations box holds nothing to build from; write at least one line, such as `D404N`"
            }
            BuildRefusal::LineRefused => {
                "a mutation line is not one luria reads; write `D404N`, `1210 G>A`, `1210-1212 GAT>TT`, `del 1210-1212` or `ins GAT after 1209`"
            }
            BuildRefusal::FromMismatch => {
                "the bases written for the change are not the bases the template holds there; read the template and write its bases"
            }
            BuildRefusal::OutsideTemplate => {
                "a position, or the CDS, lies outside the template; write positions inside it"
            }
            BuildRefusal::InsertionAtEnd => {
                "an insertion needs a base on each side; write a point between two bases, such as `ins GAT after 1209`"
            }
            BuildRefusal::MissingCodon => {
                "the codon choice is `type per mutation` and a residue line gives no codon; write one after `:`, such as `D404N:AAC`"
            }
            BuildRefusal::MissingOverlap => {
                "the minimum annealing overlap is empty; write a length in nucleotides"
            }
            BuildRefusal::DuplicateName => {
                "two mutation lines give the same member name; write mutations whose names differ"
            }
            BuildRefusal::TemplateUnreadable => {
                "the template's text is not a sequence luria can read; write only A, C, G or T, and check the name and the file"
            }
        }
    }
}

/// The request the form builds, and the paste to write with it; the refusal,
/// and the 1-based number of the mutation line it is about, when it is about
/// one.
pub fn build(form: &Form) -> Result<Built, (BuildRefusal, Option<usize>)> {
    if form.lines.iter().all(|line| line.trim().is_empty()) {
        return Err((BuildRefusal::NoMutations, None));
    }
    let overlap: i64 = match form.overlap {
        None => return Err((BuildRefusal::MissingOverlap, None)),
        Some(nt) => nt as i64,
    };

    // The template first: its bases are what every line is checked against.
    let read: Read = read_template(&form.template).map_err(|refusal| (refusal, None))?;

    let mut members: Vec<MemberSpec> = Vec::new();
    for (index, line) in form.lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let number: usize = index + 1;
        let reading: Reading = match parse_line(line) {
            Ok(found) => found,
            Err(_) => return Err((BuildRefusal::LineRefused, Some(number))),
        };
        let member: MemberSpec = match member_of(reading, &read, form.codon) {
            Ok(found) => found,
            Err(refusal) => return Err((refusal, Some(number))),
        };
        if members.iter().any(|known| known.name == member.name) {
            return Err((BuildRefusal::DuplicateName, Some(number)));
        }
        members.push(member);
    }

    let operation: OperationSpec = if members.len() == 1 {
        OperationSpec::Edit(members[0].clone())
    } else {
        OperationSpec::Variants {
            composition: CompositionSpec::SeparateVariants,
            members,
        }
    };

    Ok(Built {
        request: RequestDocument {
            template: read.template,
            scope: Some(read.scope),
            operation,
            design: DesignSpec {
                policy: PolicySpec::LuriaNative,
                style: StyleSpec::Q5Polymerase,
                min_annealing_overlap: overlap,
                placement: PlacementSpec::Inline,
                buffer: BufferSpec::Q5Reaction,
            },
        },
        paste: read.paste,
    })
}

/// The template as `build` reads it: the request's `template`, the bases
/// every line is checked against, the CDS id that is the request's `scope`,
/// and the FASTA text a paste is saved as.
struct Read {
    template: TemplateSpec,
    bases: String,
    scope: String,
    paste: Option<(String, String)>,
}

/// The form's template, read by core, which gives the bases, the digest and
/// the paste's text; nothing here reads the text a second time.
fn read_template(form: &TemplateInput) -> Result<Read, BuildRefusal> {
    match form {
        TemplateInput::Fasta {
            file_name,
            text,
            name,
            topology,
            cds,
        } => {
            let id: Identifier = named(name)?;
            let record: FastaRecord =
                read_fasta(id.clone(), text.clone(), topology_of(topology.clone()))
                    .map_err(|_| BuildRefusal::TemplateUnreadable)?;
            let bases: String = canonical(record.sequence());
            Ok(Read {
                template: TemplateSpec::Fasta {
                    id: name.clone(),
                    path: file_name.clone(),
                    topology: topology.clone(),
                    sha256: Some(digest_of(record.sequence())),
                    cds: declared(name, &bases, cds)?,
                },
                bases,
                scope: name.clone(),
                paste: None,
            })
        }
        TemplateInput::Paste {
            text,
            name,
            save_as,
            topology,
            cds,
        } => {
            let id: Identifier = named(name)?;
            let record: FastaRecord = read_fasta(
                id.clone(),
                fasta_of(name, text),
                topology_of(topology.clone()),
            )
            .map_err(|_| BuildRefusal::TemplateUnreadable)?;
            let bases: String = canonical(record.sequence());
            let saved: String = render_fasta(&id, &record.sequence());
            Ok(Read {
                template: TemplateSpec::Fasta {
                    id: name.clone(),
                    path: save_as.clone(),
                    topology: topology.clone(),
                    sha256: Some(digest_of(record.sequence())),
                    cds: declared(name, &bases, cds)?,
                },
                bases,
                scope: name.clone(),
                paste: Some((save_as.clone(), saved)),
            })
        }
        TemplateInput::GenBank {
            file_name,
            text,
            cds,
        } => {
            // A request's `template.id` is the LOCUS name, so the record is
            // read once to learn it and once under it: `read_genbank` takes
            // the identifier from its caller, and D007's rule that the
            // identifier is the request's is a request's rule, not a form's.
            let placeholder: Identifier = named("luria-new")?;
            let first: GenBankRecord = read_genbank(placeholder, text.clone())
                .map_err(|_| BuildRefusal::TemplateUnreadable)?;
            let locus: String = first.locus_name();
            let id: Identifier = named(&locus)?;
            let record: GenBankRecord =
                read_genbank(id, text.clone()).map_err(|_| BuildRefusal::TemplateUnreadable)?;
            let bases: String = canonical(record.sequence());
            Ok(Read {
                template: TemplateSpec::GenBank {
                    id: locus.clone(),
                    path: file_name.clone(),
                    sha256: Some(digest_of(record.sequence())),
                    cds: vec![cds.clone()],
                },
                bases,
                scope: cds.clone(),
                paste: None,
            })
        }
    }
}

/// The form's name, as core's identifier: a single printable ASCII token.
fn named(name: &str) -> Result<Identifier, BuildRefusal> {
    Identifier::create(name.to_string()).map_err(|_| BuildRefusal::TemplateUnreadable)
}

/// The template's canonical bases, as core read them: upper-case letters, no
/// whitespace.
fn canonical(sequence: Sequence) -> String {
    sequence.letters().iter().collect()
}

/// `template.sha256`: the SHA-256 of the bases, the one
/// `Sequence::content_hash` gives (5e2's rule).
fn digest_of(sequence: Sequence) -> String {
    sequence.content_hash().hex_digest().to_string()
}

/// A paste as FASTA text: text whose first written line is a header line is
/// the text as typed; raw bases become one record named by the form's name,
/// which is what `read_fasta` reads (D008).
fn fasta_of(name: &str, text: &str) -> String {
    match text.lines().find(|line| !line.trim().is_empty()) {
        Some(line) if line.trim_start().starts_with('>') => text.to_string(),
        _ => format!(">{}\n{}\n", name, text),
    }
}

/// The CDS the form states for a FASTA or pasted template (M7-32): the whole
/// sequence, or the typed 1-based inclusive bases as the request's zero-based
/// half-open interval (M7-24, D006). A CDS outside the template is refused.
fn declared(name: &str, bases: &str, cds: &CdsChoice) -> Result<Vec<CdsSpec>, BuildRefusal> {
    let length: i64 = bases.len() as i64;
    let interval: (i64, i64) = match cds {
        CdsChoice::Whole(_) => (0, length),
        CdsChoice::Bases { first, last, .. } => {
            if *first < 1 || *last > length || first > last {
                return Err(BuildRefusal::OutsideTemplate);
            }
            (first - 1, *last)
        }
    };
    let strand: StrandSpec = match cds {
        CdsChoice::Whole(strand) | CdsChoice::Bases { strand, .. } => strand.clone(),
    };
    Ok(vec![CdsSpec {
        id: name.to_string(),
        intervals: vec![interval],
        strand,
        completeness: "complete".to_string(),
        genetic_code: "standard".to_string(),
        numbering: "from-cds-start".to_string(),
    }])
}

/// The form's topology, as core states it. `prepare.rs`'s `topology_of` is
/// private, so this is `new.rs`'s own two-arm match (approved 2026-10-21); it
/// holds no domain logic.
fn topology_of(spec: TopologySpec) -> SequenceTopology {
    match spec {
        TopologySpec::Linear => SequenceTopology::Linear,
        TopologySpec::Circular => SequenceTopology::Circular,
    }
}

/// One mutation line as a named member: the name is built from the positions
/// the user typed, 1-based, and the bases a deletion, an insertion or a typed
/// `from` needs come from the template (§2 of the plan).
fn member_of(
    reading: Reading,
    read: &Read,
    codon: CodonChoice,
) -> Result<MemberSpec, BuildRefusal> {
    let bases: &str = &read.bases;
    match reading {
        Reading::Residue {
            number,
            from,
            to,
            codon: given,
        } => {
            let choice: CodonSpec = match given {
                Some(text) => CodonSpec::Supplied(text),
                None => match codon {
                    CodonChoice::DefaultPolicy => CodonSpec::Default,
                    CodonChoice::TypedPerMutation => return Err(BuildRefusal::MissingCodon),
                },
            };
            Ok(MemberSpec {
                name: format!("{}{}{}", from, number, to),
                edit: EditSpec::Residue {
                    cds: Some(read.scope.clone()),
                    number,
                    from: from.to_string(),
                    to: to.to_string(),
                    codon: choice,
                },
            })
        }
        Reading::Substitution {
            start,
            end,
            from,
            to,
        } => {
            inside(bases, start, end)?;
            if bases[start as usize..end as usize] != from {
                return Err(BuildRefusal::FromMismatch);
            }
            Ok(MemberSpec {
                name: base_name(start, end, &from, &to),
                edit: EditSpec::Substitution {
                    interval: (start, end),
                    from,
                    to,
                },
            })
        }
        Reading::Replacement {
            start,
            end,
            from,
            to,
        } => {
            inside(bases, start, end)?;
            if bases[start as usize..end as usize] != from {
                return Err(BuildRefusal::FromMismatch);
            }
            Ok(MemberSpec {
                name: base_name(start, end, &from, &to),
                edit: EditSpec::Replacement {
                    interval: (start, end),
                    from,
                    to,
                },
            })
        }
        Reading::Deletion { start, end } => {
            inside(bases, start, end)?;
            Ok(MemberSpec {
                name: if end - start == 1 {
                    format!("del{}", end)
                } else {
                    format!("del{}-{}", start + 1, end)
                },
                edit: EditSpec::Deletion {
                    interval: (start, end),
                    from: bases[start as usize..end as usize].to_string(),
                },
            })
        }
        Reading::Insertion { point, insert } => {
            let length: i64 = bases.len() as i64;
            if point > length {
                return Err(BuildRefusal::OutsideTemplate);
            }
            // An insertion needs a base on each side (D032).
            if point == 0 || point == length {
                return Err(BuildRefusal::InsertionAtEnd);
            }
            let at: usize = point as usize;
            Ok(MemberSpec {
                name: if short(&insert) {
                    format!("ins{}{}", point, insert)
                } else {
                    format!("ins{}+{}", point, insert.chars().count())
                },
                edit: EditSpec::Insertion {
                    point,
                    before: bases[at - 1..at].to_string(),
                    after: bases[at..at + 1].to_string(),
                    insert,
                },
            })
        }
    }
}

/// A typed range against the template: a position past its end is refused.
fn inside(bases: &str, start: i64, end: i64) -> Result<(), BuildRefusal> {
    let length: i64 = bases.len() as i64;
    if end > length || start > length {
        return Err(BuildRefusal::OutsideTemplate);
    }
    Ok(())
}

/// A base change's name: `G1210A` for one base to one base, `GAT1210-1212AAC`
/// when both texts are short, else `rep1210-1212` (§2 of the plan), with the
/// 1-based positions the user typed.
fn base_name(start: i64, end: i64, from: &str, to: &str) -> String {
    if end - start == 1 && to.chars().count() == 1 {
        format!("{}{}{}", from, end, to)
    } else if short(from) && short(to) {
        format!("{}{}-{}{}", from, start + 1, end, to)
    } else {
        format!("rep{}-{}", start + 1, end)
    }
}

/// A name's text is short when it is at most 10 bases (§2 of the plan).
fn short(text: &str) -> bool {
    text.chars().count() <= 10
}
