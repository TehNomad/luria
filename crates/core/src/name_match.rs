//! Matching a value a user typed against the names of caller-numbered items,
//! and the CDS names that question is asked about (D049's M8-15, D053, D056;
//! contract 8i1).
//!
//! There is one matcher, shared by `luria find --cds` (8i2), the view's go-to
//! (8k2) and its feature list (8k3), so that a command line and a screen
//! cannot disagree about which CDS a value names (D056). `match_name` knows
//! nothing about CDSs: it takes `NamedItem`s — names attached to items the
//! caller numbers — and answers which items the value names.
//!
//! It returns candidates, never a choice and never a refusal: `One`, `Several`
//! or `Nothing`. There is no `DomainError` variant and no sentence of message
//! text in this module, because the cli words the answers and owns the exit
//! codes (D056, and 5a1 made those codes the CLI's contract): core says which
//! items matched and with which name, and `find` turns no match or several
//! into exit 2 listing the candidates with the source and span that matched.
//!
//! The rules, in the order that decides an answer (D049's M8-15; the bracketed
//! numbers are the contract's §5 tests):
//!
//! 1. **The fold** [2, 7]. Comparison is ASCII case-folded and every
//!    whitespace character is dropped from both sides, so ` rop  PROTEIN ` and
//!    `ROPprotein` are `ROP protein`. A value that folds to nothing matches
//!    `Nothing`: no name is named by a blank, and matching every item is never
//!    what an empty argument means.
//! 2. **Exact first** [1, 6]. A name matches exactly when the two folded
//!    strings are equal, and once any name matches exactly, only exact matches
//!    count. `LACZ` is `lacZ` and not the start of `lacZalpha`, which is what
//!    makes a complete id answer one CDS rather than several.
//! 3. **Then a prefix, at a word start** [3, 6]. A word starts at an
//!    alphanumeric character that is the first character of the name or
//!    follows a non-alphanumeric one, in the name as written, and a name
//!    matches when the fold of everything from some word start begins with the
//!    folded value. So `lact` and `BETA-LAC` reach `beta-lactamase` but
//!    `actam` does not, and `prot` reaches `ROP protein` where `rotein` does
//!    not. The list form `name (start..end)` is whole-only and takes no part
//!    in this tier: D049 reads a span only as the whole string, so `tet (86`
//!    is a half-written span, not a name.
//! 4. **Items, not names** [4, 5]. Matches are grouped by item: a CDS named
//!    twice — by its display name and by its `/gene` — is one match, reported
//!    with the first of its matching names in the order the names were given.
//!    One matching item is `One`, two or more are `Several` in ascending item
//!    order, and the caller decides what several means.
//!
//! A CDS's names are the ids D049 lists: the display name of its feature
//! (D053), each non-empty `/gene`, `/locus_tag` and `/protein_id` value, and
//! the list form `name (first..last)` built from its span in D036 order. The
//! display name is the one `view_record` computes (D053) and `ViewCds::names`
//! carries the rest, so this module writes no second display-name rule.
//! `/product` is not an id: it reaches a CDS only when it *is* the display
//! name, which is why pBR322's ROP CDS answers `rop` but its `beta-lactamase`
//! product answers nothing for `bla`. A span is read in D036 order, origin
//! joins included, so `join(55..60,1..12)` is `pj (55..12)` and
//! `complement(3293..4153)` is `bla (3293..4153)`.

use crate::display_location::DisplayLocation;
use crate::view_record::FeatureShape;
use crate::view_record::ViewCds;
use crate::view_record::ViewRecord;

/// Which id of an item matched (D049). The list form is a display form of the
/// display name plus a span, not a qualifier of the record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameSource {
    DisplayName,
    Gene,
    LocusTag,
    ProteinId,
    ListForm,
}

/// One name of one item: `item` is the caller's index, and for a CDS it is the
/// index into `ViewRecord::cdss`.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedItem {
    pub item: usize,
    pub source: NameSource,
    /// The name as written in the record, not as folded.
    pub name: String,
    /// True for the list form: matched only as the whole string, never as a
    /// prefix (D049).
    pub whole_only: bool,
}

/// One item's answer: which item, and which of its names matched it.
#[derive(Clone, Debug, PartialEq)]
pub struct NameMatch {
    pub item: usize,
    pub source: NameSource,
    pub name: String,
}

/// What a value named: one item, several candidates in ascending item order,
/// or nothing. Choosing among several is the view's picker and refusing is the
/// cli's (D049, D056); this module does neither.
#[derive(Clone, Debug, PartialEq)]
pub enum Matched {
    One(NameMatch),
    Several(Vec<NameMatch>),
    Nothing,
}

/// The fold both sides of every comparison go through (D049): ASCII
/// lower-casing, with every whitespace character dropped. Nothing else is
/// dropped — a `-` or a `(` written in a value has to be written in the name
/// too, which is what makes `BETA-LAC` reach `beta-lactamase` while `BETALAC`
/// does not.
fn fold(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_whitespace())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

/// The folded rest of `name` from every word start, longest first (rule 3). A
/// word starts at an alphanumeric character that is the first of the name or
/// follows a non-alphanumeric one, in the name as written — the fold drops
/// characters, so the boundary is read before it does.
fn word_rests(name: &str) -> Vec<String> {
    let mut starts: Vec<usize> = Vec::new();
    let mut previous: Option<char> = None;
    for pair in name.char_indices() {
        let (index, ch): (usize, char) = pair;
        let at_start: bool =
            ch.is_alphanumeric() && !matches!(previous, Some(before) if before.is_alphanumeric());
        if at_start {
            starts.push(index);
        }
        previous = Some(ch);
    }
    starts.iter().map(|index| fold(&name[*index..])).collect()
}

/// Whether `value` names any of `items`, by the rules in the module header.
pub fn match_name(items: &[NamedItem], value: &str) -> Matched {
    let want: String = fold(value);
    if want.is_empty() {
        return Matched::Nothing;
    }

    // Rule 2: exact first, and an exact match rules the prefix tier out for
    // every item, not only for the item it matched.
    let mut hits: Vec<&NamedItem> = Vec::new();
    for named in items.iter() {
        if fold(&named.name) == want {
            hits.push(named);
        }
    }

    // Rule 3: only when nothing matched exactly, and never on a whole-only
    // name.
    if hits.is_empty() {
        for named in items.iter() {
            if named.whole_only {
                continue;
            }
            if word_rests(&named.name)
                .iter()
                .any(|rest| rest.starts_with(want.as_str()))
            {
                hits.push(named);
            }
        }
    }

    // Rule 4: one answer per item, carrying the first of its matching names in
    // the order the names were given, then items in ascending order.
    let mut matched: Vec<NameMatch> = Vec::new();
    for named in hits.iter() {
        if matched.iter().any(|found| found.item == named.item) {
            continue;
        }
        matched.push(NameMatch {
            item: named.item,
            source: named.source,
            name: named.name.clone(),
        });
    }
    matched.sort_by_key(|found| found.item);

    match matched.len() {
        0 => Matched::Nothing,
        1 => Matched::One(matched[0].clone()),
        _ => Matched::Several(matched),
    }
}

/// The 1-based first and last base of a CDS's feature, in D036 order: the
/// first interval's start plus one and the last interval's end, which is what
/// GenBank writes for `join(55..60,1..12)` and for a complemented range alike
/// (D036). `None` when `cds` is not a CDS of the record or its feature is not
/// drawn, which is the shape the refusal of the display parser leaves behind
/// (D053).
pub fn cds_span(record: &ViewRecord, cds: usize) -> Option<(u64, u64)> {
    let view: &ViewCds = record.cdss.get(cds)?;
    let shape: &FeatureShape = &record.features.get(view.feature)?.shape;
    let drawn: &DisplayLocation = match shape {
        FeatureShape::Drawn(drawn) => drawn,
        FeatureShape::NotDrawn(_) => return None,
    };
    let intervals: Vec<(u64, u64)> = drawn.location.intervals();
    let first: &(u64, u64) = intervals.first()?;
    let last: &(u64, u64) = intervals.last()?;
    Some((first.0 + 1, last.1))
}

/// Every name of every CDS of the record, in `record.cdss` order and, within a
/// CDS, in the order of its `names`; then the list form
/// `"{display name} ({first}..{last})"` of each drawn CDS, whole-only
/// (D049). `item` is the index into `record.cdss`, which is what `find` and
/// the view name a CDS by.
pub fn cds_names(record: &ViewRecord) -> Vec<NamedItem> {
    let mut named: Vec<NamedItem> = Vec::new();
    for pair in record.cdss.iter().enumerate() {
        let item: usize = pair.0;
        let view: &ViewCds = pair.1;
        for entry in view.names.iter() {
            named.push(NamedItem {
                item,
                source: entry.0,
                name: entry.1.clone(),
                whole_only: false,
            });
        }
        // The display name is the feature's, already computed once by
        // `view_record`; this module does not derive it a second time (D053).
        let display: &str = &record.features[view.feature].name;
        if let Some((first, last)) = cds_span(record, item) {
            named.push(NamedItem {
                item,
                source: NameSource::ListForm,
                name: format!("{} ({}..{})", display, first, last),
                whole_only: true,
            });
        }
    }
    named
}

/// The one entry point the cli and the view use: what `value` names among the
/// CDSs of `record` (D056).
pub fn match_cds(record: &ViewRecord, value: &str) -> Matched {
    match_name(&cds_names(record), value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DN: NameSource = NameSource::DisplayName;
    const GENE: NameSource = NameSource::Gene;
    const PID: NameSource = NameSource::ProteinId;
    const LIST: NameSource = NameSource::ListForm;

    /// One hand-built name of a hand-built item.
    fn named(item: usize, source: NameSource, name: &str, whole_only: bool) -> NamedItem {
        NamedItem {
            item,
            source,
            name: name.to_string(),
            whole_only,
        }
    }

    /// Test 4's items: three `/protein_id`s that share a prefix.
    fn protein_ids() -> Vec<NamedItem> {
        vec![
            named(0, PID, "AAB59735.1", false),
            named(1, PID, "AAB59736.1", false),
            named(2, PID, "AAB59737.1", false),
        ]
    }

    /// The single match, refusing anything else.
    fn one(items: &[NamedItem], value: &str) -> NameMatch {
        match match_name(items, value) {
            Matched::One(found) => found,
            Matched::Several(found) => {
                panic!(
                    "{:?}: expected One, got Several {:?} ({})",
                    value,
                    found,
                    found.len()
                )
            }
            Matched::Nothing => panic!("{:?}: expected One, got Nothing", value),
        }
    }

    /// The items of a `Several`, in the order they were reported.
    fn several(items: &[NamedItem], value: &str) -> Vec<usize> {
        match match_name(items, value) {
            Matched::Several(found) => found.iter().map(|found| found.item).collect(),
            Matched::One(found) => panic!("{:?}: expected Several, got One {:?}", value, found),
            Matched::Nothing => panic!("{:?}: expected Several, got Nothing", value),
        }
    }

    /// The value matches nothing at all.
    fn nothing(items: &[NamedItem], value: &str) {
        match match_name(items, value) {
            Matched::Nothing => {}
            other => panic!("{:?}: expected Nothing, got {:?}", value, other),
        }
    }

    // Rule 2: an exact name wins over a prefix of another name, in both
    // directions — the exact tier runs first, and it answers before the prefix
    // tier has anything to say about the other item.
    #[test]
    fn an_exact_name_beats_a_prefix() {
        let items: Vec<NamedItem> = vec![
            named(0, DN, "lacZ", false),
            named(1, DN, "lacZalpha", false),
        ];
        assert_eq!(
            one(&items, "LACZ"),
            NameMatch {
                item: 0,
                source: DN,
                name: "lacZ".to_string(),
            },
            "LACZ is lacZ, not the start of lacZalpha"
        );
        assert_eq!(
            one(&items, "lacza"),
            NameMatch {
                item: 1,
                source: DN,
                name: "lacZalpha".to_string(),
            },
            "lacza is a prefix of lacZalpha alone"
        );
    }

    // Rule 1: ASCII case and every whitespace character of a name or a value
    // are ignored, leading and trailing included.
    #[test]
    fn case_and_interior_whitespace_are_ignored() {
        let items: Vec<NamedItem> = vec![named(0, DN, "ROP protein", false)];
        for value in ["rop protein", "ROPprotein", " rop  PROTEIN "] {
            assert_eq!(
                one(&items, value),
                NameMatch {
                    item: 0,
                    source: DN,
                    name: "ROP protein".to_string(),
                },
                "{:?}: the name as written is reported, whatever the value looked like",
                value
            );
        }
    }

    // Rule 3: a prefix has to start a word — a word starts at an alphanumeric
    // character that is the first of the name or follows a non-alphanumeric
    // one — and the rest of the name is read from there, not from anywhere.
    #[test]
    fn a_prefix_must_start_a_word() {
        let beta: Vec<NamedItem> = vec![named(0, DN, "beta-lactamase", false)];
        for value in ["lact", "beta", "BETA-LAC"] {
            assert_eq!(
                one(&beta, value).item,
                0,
                "{:?}: matches beta-lactamase",
                value
            );
        }
        nothing(&beta, "actam");

        let rop: Vec<NamedItem> = vec![named(0, DN, "ROP protein", false)];
        assert_eq!(one(&rop, "prot").item, 0, "prot starts the second word");
        nothing(&rop, "rotein");
    }

    // Rule 4: several items matching a prefix are `Several`, in ascending item
    // order; the longer prefix that isolates one of them is `One`.
    #[test]
    fn several_items_are_several() {
        let items: Vec<NamedItem> = protein_ids();
        assert_eq!(
            several(&items, "AAB5973"),
            vec![0, 1, 2],
            "the shared prefix names all three"
        );
        assert_eq!(
            one(&items, "AAB59736"),
            NameMatch {
                item: 1,
                source: PID,
                name: "AAB59736.1".to_string(),
            },
            "the longer prefix names one"
        );
    }

    // Rule 4: two names of one item are one match, reported with the first of
    // them in the order the names were given.
    #[test]
    fn one_item_named_twice_is_one_match() {
        let items: Vec<NamedItem> = vec![named(0, DN, "tet", false), named(0, GENE, "tet", false)];
        assert_eq!(
            one(&items, "tet"),
            NameMatch {
                item: 0,
                source: DN,
                name: "tet".to_string(),
            },
            "one item, not two matches"
        );
    }

    // Rule 3 and D049: the list form is matched only as the whole string, so a
    // half-written span is nothing rather than a prefix.
    #[test]
    fn a_whole_only_name_is_never_a_prefix() {
        let items: Vec<NamedItem> = vec![
            named(0, DN, "tet", false),
            named(0, LIST, "tet (86..1276)", true),
        ];
        assert_eq!(
            one(&items, "TET (86..1276)"),
            NameMatch {
                item: 0,
                source: LIST,
                name: "tet (86..1276)".to_string(),
            },
            "the whole list form, spaces and case ignored"
        );
        nothing(&items, "tet (86");
    }

    // Rule 1: a value that folds to nothing matches nothing, and does not match
    // everything.
    #[test]
    fn an_empty_value_matches_nothing() {
        let items: Vec<NamedItem> = protein_ids();
        for value in ["", "   "] {
            nothing(&items, value);
        }
    }
}
