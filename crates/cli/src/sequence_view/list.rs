//! The feature list and the CDS picker: what `f` and `c` open, which rows they
//! hold, what a typed filter does to them, what one row reads as and what it is
//! coloured as (contract 8k3 §4; 8m16 §4; plan §5.2, proposal §3, D052, D053,
//! D056).
//!
//! A list is rows and a highlight, and nothing else. The record, the cursor, the
//! selection and the message stay with `super::state`, which routes the keys
//! while a list is open and decides what `enter` does (§4 rules 3 to 5); the
//! window of rows to draw is `ViewState::list_rows`, and `super::screen` puts
//! them where the blocks would be (rule 7).
//!
//! The filter is core's shared matcher, `match_name` (D056), so the screen,
//! `luria find --cds` and go-to cannot disagree about what a typed value names.
//! The feature list asks it of a feature's key as well as its names (8m16 §4
//! rule 3), while `feature_names` alone — what go-to answers — keeps names only.
//! A feature row also carries the colour `layout` gives its feature (8m16 §4
//! rule 1), which `super::screen` draws when the view's colours are on.
//! An item that `enter` refuses stays in the list: unusable CDSs are shown with
//! the reason they cannot be selected and are never selected (D052), and a
//! feature that is not drawn says which refusal keeps it off the screen (D053).
//! The reason shown is a code core already returns — this module renders codes,
//! it decides no biology.

use crate::result::rule_code;
use crate::sequence_view::layout::feature_colour;
use crate::sequence_view::layout::key_colour;
use luria_core::display_location::DisplayLocation;
use luria_core::error::DomainError;
use luria_core::name_match::cds_names;
use luria_core::name_match::match_name;
use luria_core::name_match::Matched;
use luria_core::name_match::NameSource;
use luria_core::name_match::NamedItem;
use luria_core::strand::Strand;
use luria_core::view_record::CdsStatus;
use luria_core::view_record::FeatureShape;
use luria_core::view_record::ViewCds;
use luria_core::view_record::ViewFeature;
use luria_core::view_record::ViewRecord;
use ratatui::style::Color;

/// Which of the view's two lists is open (§4): `f` lists every entry of
/// `record.features`, `c` every entry of `record.cdss`. The rows of both are
/// indices into the list they come from, in file order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKind {
    Features,
    Cdss,
}

/// The names a feature is filtered by (§4): its display name, and for a CDS
/// feature every entry of its `ViewCds::names` on top of that — `/gene`,
/// `/locus_tag` and `/protein_id` (D049, D053). `item` is the index into
/// `record.features`, which is what a feature row is.
///
/// The display name comes twice for a CDS feature, because it is the first entry
/// of `ViewCds::names` too. That adds no row: the matcher answers items, not
/// names (its rule 4). D049's list form `name (first..last)` is not added here
/// either — it is a whole-string id for a typed command-line argument, and a
/// span is not a name of the feature beside the CDS in the feature list.
pub fn feature_names(record: &ViewRecord) -> Vec<NamedItem> {
    let mut named: Vec<NamedItem> = Vec::new();
    for (item, feature) in record.features.iter().enumerate() {
        named.push(NamedItem {
            item,
            source: NameSource::DisplayName,
            name: feature.name.clone(),
            whole_only: false,
        });
        // A CDS names its own feature by index, so the ids go to the item that
        // feature is, and to no other feature that happens to share the name.
        for view in record.cdss.iter().filter(|cds| cds.feature == item) {
            for entry in view.names.iter() {
                named.push(NamedItem {
                    item,
                    source: entry.0,
                    name: entry.1.clone(),
                    whole_only: false,
                });
            }
        }
    }
    named
}

/// The key as a name of its own feature (contract 8m16 §4 rule 3): one entry per
/// feature, its `name` the feature's key. `rows_for` adds these to
/// `feature_names` for the feature list alone, so that `f` answers `CDS` and
/// `bind` as well as the names, while `feature_names` — and with it go-to —
/// stays names only (8m16 §1).
///
/// The source is `DisplayName`, which `rows_for` reads nothing of: the matcher
/// answers items, so a feature named by both its name and its key is still one
/// row, in file order (D056's rule 4).
fn key_names(record: &ViewRecord) -> Vec<NamedItem> {
    record
        .features
        .iter()
        .enumerate()
        .map(|(item, feature)| NamedItem {
            item,
            source: NameSource::DisplayName,
            name: feature.key.clone(),
            whole_only: false,
        })
        .collect()
}

/// The rows a filter answers (rule 2): every item for an empty filter, and
/// otherwise the items `match_name` answers — `One` or `Several`, each item once
/// and in file order, `Nothing` for none. A filter that folds to nothing (a
/// blank typed) answers no rows, which is the matcher's rule and not a second
/// one here (D056).
fn rows_for(record: &ViewRecord, kind: ListKind, filter: &str) -> Vec<usize> {
    if filter.is_empty() {
        let items: usize = match kind {
            ListKind::Features => record.features.len(),
            ListKind::Cdss => record.cdss.len(),
        };
        return (0..items).collect();
    }
    let named: Vec<NamedItem> = match kind {
        ListKind::Features => {
            // 8m16 §4 rule 3: the feature list is filtered by the names and by
            // the key; the picker is filtered by the CDS ids alone.
            let mut names: Vec<NamedItem> = feature_names(record);
            names.extend(key_names(record));
            names
        }
        ListKind::Cdss => cds_names(record),
    };
    match match_name(&named, filter) {
        Matched::One(found) => vec![found.item],
        // The matcher already gives one entry per item in ascending item order,
        // which for these lists is file order.
        Matched::Several(found) => found.iter().map(|found| found.item).collect(),
        Matched::Nothing => Vec::new(),
    }
}

/// The code a refusal is shown by (D053, contract §2): the rule's code when the
/// error carries one — `no_id`, `pseudo`, `order`, `fuzzy_end` — and the
/// variant's own code when it does not.
pub(super) fn refusal_code(error: &DomainError) -> String {
    match rule_code(error) {
        Some(code) => code.to_string(),
        None => error.code().to_string(),
    }
}

/// The span of a drawn feature the way GenBank writes it: 1-based, in D036
/// order, with `<` before the first number when the feature may start earlier
/// and `>` before the last when it may run on (D053, §4 rule 6). Only outer ends
/// carry a marker, which is what `DisplayLocation` records. `None` asks for the
/// `not drawn` wording instead: a drawn shape with no interval to name.
fn span_words(drawn: &DisplayLocation) -> Option<String> {
    let intervals: Vec<(u64, u64)> = drawn.location.intervals();
    let first: &(u64, u64) = intervals.first()?;
    let last: &(u64, u64) = intervals.last()?;
    let before: &str = if drawn.open_before.is_some() { "<" } else { "" };
    let after: &str = if drawn.open_after.is_some() { ">" } else { "" };
    Some(format!("{before}{}..{after}{}", first.0 + 1, last.1))
}

/// One row of the feature list (§4 rule 6): the name, the feature key, then the
/// span with its strand, or `not drawn: {code}` for a feature the display
/// parser refuses — which stays in the list with its reason (plan §4.1, D053).
fn feature_words(record: &ViewRecord, index: usize) -> String {
    let feature: &ViewFeature = match record.features.get(index) {
        Some(found) => found,
        None => return String::new(),
    };
    match &feature.shape {
        FeatureShape::NotDrawn(error) => format!(
            "{} {} not drawn: {}",
            feature.name,
            feature.key,
            refusal_code(error)
        ),
        FeatureShape::Drawn(drawn) => match span_words(drawn) {
            Some(span) => {
                let strand: &str = match drawn.location.strand() {
                    Strand::Plus => "(+)",
                    Strand::Minus => "(-)",
                };
                format!("{} {} {} {}", feature.name, feature.key, span, strand)
            }
            None => format!("{} {} not drawn", feature.name, feature.key),
        },
    }
}

/// One row of the CDS picker (§4 rule 6): the name and the span — or `not
/// drawn`, when the feature behind it is not — then `unusable: {code}` for a
/// CDS `import_cds` refused. pBR322's ROP is there, greyed, reading `unusable:
/// no_id` (D052, D053). No strand here: the picker names CDSs, and the span it
/// shows is in D036 order whatever the strand.
fn cds_words(record: &ViewRecord, index: usize) -> String {
    let view: &ViewCds = match record.cdss.get(index) {
        Some(found) => found,
        None => return String::new(),
    };
    let feature: &ViewFeature = match record.features.get(view.feature) {
        Some(found) => found,
        None => return String::new(),
    };
    let span: String = match &feature.shape {
        FeatureShape::NotDrawn(error) => format!("not drawn: {}", refusal_code(error)),
        FeatureShape::Drawn(drawn) => match span_words(drawn) {
            Some(span) => span,
            None => "not drawn".to_string(),
        },
    };
    match &view.status {
        CdsStatus::Usable(_cds) => format!("{} {}", feature.name, span),
        CdsStatus::Unusable(error) => format!(
            "{} {} unusable: {}",
            feature.name,
            span,
            refusal_code(error)
        ),
    }
}

/// Whether `enter` refuses this row (§4 rule 7) — which is both what the screen
/// draws DIM and what keeps the list open with a reason: a feature that is not
/// drawn, one whose start is not stated, and an unusable CDS in the picker. A
/// CDS feature whose CDS is unusable is not refused in the feature list, where
/// `enter` still jumps to the feature and says separately why it was not
/// selected (§4 rule 4).
fn refused(record: &ViewRecord, kind: ListKind, index: usize) -> bool {
    match kind {
        ListKind::Features => match record.features.get(index) {
            Some(feature) => match &feature.shape {
                FeatureShape::NotDrawn(_error) => true,
                FeatureShape::Drawn(drawn) => drawn.open_before.is_some(),
            },
            None => true,
        },
        ListKind::Cdss => match record.cdss.get(index) {
            Some(view) => matches!(&view.status, CdsStatus::Unusable(_refusal)),
            None => true,
        },
    }
}

/// The colour one feature row is drawn in (contract 8m16 §4 rule 1): the lane
/// colour 8m7 gives, taken on the strand the feature's drawn location was
/// written on (D009), and its key's colour for a feature the display parser
/// refuses — which has no location to be written on. `None` for an index naming
/// no feature, as every other row helper returns nothing for one.
fn feature_row_colour(record: &ViewRecord, index: usize) -> Option<Color> {
    let feature: &ViewFeature = record.features.get(index)?;
    match &feature.shape {
        FeatureShape::Drawn(drawn) => feature_colour(feature, &drawn.location.strand()),
        FeatureShape::NotDrawn(_error) => key_colour(&feature.key),
    }
}

/// One list row as the screen draws it (§4 rule 7): the text rule 6 gives,
/// whether it is DIM because `enter` refuses it, whether it carries the
/// highlight, and the colour contract 8m16 §4 rule 1 gives it.
pub(super) struct Drawn {
    pub(super) text: String,
    pub(super) dim: bool,
    pub(super) highlighted: bool,
    /// The row's foreground colour, or `None` when it has none — which is what a
    /// feature with no colour of its own has, and what every CDS picker row has
    /// (8m16 §4 rule 1). `screen` applies it only with the colours on.
    pub(super) colour: Option<Color>,
}

/// A row as drawn: its text, its two marks and its colour (§4 rules 6 and 7,
/// 8m16 §4 rule 1).
fn drawn(record: &ViewRecord, kind: ListKind, index: usize, highlighted: bool) -> Drawn {
    Drawn {
        text: match kind {
            ListKind::Features => feature_words(record, index),
            ListKind::Cdss => cds_words(record, index),
        },
        dim: refused(record, kind, index),
        highlighted,
        colour: match kind {
            ListKind::Features => feature_row_colour(record, index),
            ListKind::Cdss => None,
        },
    }
}

/// The open list: which record it lists, what is typed into it, its rows and
/// where the highlight is. The rows are recomputed whenever the filter changes
/// (rule 2); nothing here is stored about the scroll, because the rows to draw
/// follow from the highlight and the body's height, and a resize takes the list
/// as it is.
pub struct List {
    kind: ListKind,
    filter: String,
    rows: Vec<usize>,
    /// The highlighted entry of `rows`, `None` when the list has no rows.
    at: Option<usize>,
}

impl List {
    /// A list opened on `record` with `filter` (§4): rows as rule 2 gives them,
    /// the first one highlighted.
    pub(super) fn open(record: &ViewRecord, kind: ListKind, filter: &str) -> List {
        let rows: Vec<usize> = rows_for(record, kind, filter);
        let at: Option<usize> = if rows.is_empty() { None } else { Some(0) };
        List {
            kind,
            filter: filter.to_string(),
            rows,
            at,
        }
    }

    pub fn kind(&self) -> ListKind {
        self.kind
    }

    pub fn filter(&self) -> &str {
        self.filter.as_str()
    }

    /// The items the list shows, as indices into `record.features` or
    /// `record.cdss`, in file order (§4).
    pub fn rows(&self) -> &[usize] {
        self.rows.as_slice()
    }

    /// The index into `rows` that the highlight is on, `None` when the list has
    /// no rows to highlight (§4).
    pub fn at(&self) -> Option<usize> {
        self.at
    }

    /// The highlighted item, `None` when there is no highlight and so no row for
    /// `enter` to act on (§4 rule 3).
    pub(super) fn item(&self) -> Option<usize> {
        Some(*self.rows.get(self.at?)?)
    }

    /// A character typed into the filter: it goes on the end, and the rows and
    /// the highlight follow (§4 rules 2 and 3).
    pub(super) fn type_char(&mut self, record: &ViewRecord, ch: char) {
        let mut text: String = self.filter.clone();
        text.push(ch);
        self.set_filter(record, text);
    }

    /// `backspace`: the last character of the filter goes (rule 3), and with it
    /// whatever it was filtering out (§4 rule 2).
    pub(super) fn backspace(&mut self, record: &ViewRecord) {
        let mut text: String = self.filter.clone();
        text.pop();
        self.set_filter(record, text);
    }

    /// The rows and the highlight after the filter changed: the highlight goes
    /// to the first row whenever the rows change (rule 2), and a list that ends
    /// up with no rows has no highlight at all.
    fn set_filter(&mut self, record: &ViewRecord, filter: String) {
        self.filter = filter;
        self.rows = rows_for(record, self.kind, &self.filter);
        self.at = if self.rows.is_empty() { None } else { Some(0) };
    }

    /// `up` and `down`: one row, and the ends stay put (rule 3). A list with no
    /// rows has nothing to move.
    pub(super) fn step_row(&mut self, steps: i64) {
        let current: i64 = match self.at {
            Some(at) => at as i64,
            None => return,
        };
        let last: usize = match self.rows.len().checked_sub(1) {
            Some(last) => last,
            None => return,
        };
        let moved: i64 = (current + steps).clamp(0, last as i64);
        self.at = Some(moved as usize);
    }

    /// `pageup` and `pagedown`: by `page` rows, which is the number of body rows
    /// the list draws in, clamped at the ends like `step_row` (rule 3).
    pub(super) fn page_row(&mut self, page: usize, direction: i64) {
        let steps: i64 = direction * i64::try_from(page).unwrap_or(i64::MAX);
        self.step_row(steps);
    }

    /// `home` and `end`: the first and last row (rule 3), and no highlight at
    /// all for a list with no rows.
    pub(super) fn go_first(&mut self) {
        self.at = if self.rows.is_empty() { None } else { Some(0) };
    }

    pub(super) fn go_last(&mut self) {
        self.at = self.rows.len().checked_sub(1);
    }

    /// The rows the screen draws: a window of `body` rows — the frame's body rows
    /// — that holds the highlighted row, moved down only as far as it has to be
    /// (§4 rule 7). It starts at the top while the highlight is inside the first
    /// window, and never starts where the window would run past the last row.
    /// Empty for a list with no rows, and for a body with no rows to draw.
    pub(super) fn window(&self, body: usize) -> std::ops::Range<usize> {
        let total: usize = self.rows.len();
        let window: usize = body.min(total);
        if window == 0 {
            return 0..0;
        }
        let first: usize = match self.at {
            Some(at) if at + 1 > window => at + 1 - window,
            _ => 0,
        };
        first..(first + window)
    }

    /// The window as the screen draws it, text and marks included (§4 rules 6
    /// and 7).
    pub(super) fn drawn(&self, record: &ViewRecord, body: usize) -> Vec<Drawn> {
        self.window(body)
            .filter_map(|index| Some((index, *self.rows.get(index)?)))
            .map(|(index, item)| drawn(record, self.kind, item, Some(index) == self.at))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequence_view::list::ListKind::Cdss;
    use crate::sequence_view::list::ListKind::Features;
    use luria_core::identity::Identifier;
    use luria_core::view_record::open_record;
    use std::fs;
    use std::path::Path;

    /// One fixture record, as `open_record` reads it — the same helper as
    /// `state.rs`'s tests and `tests/sequence_view_list.rs`.
    fn record(path: &str, name: &str) -> ViewRecord {
        let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
        let text: String =
            fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
        open_record(Identifier::create(name.to_string()).unwrap(), text)
            .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
    }

    /// pBR322: 50 features, and three CDSs — `tet`, the ROP CDS with no id, and
    /// `bla` (`tests/fixtures/genbank/pbr322-features.tsv`).
    fn pbr() -> ViewRecord {
        record("../../tests/fixtures/genbank/raw/J01749.1.gb", "SYNPBR322")
    }

    /// synthetic: 27 features, 24 of them CDSs, and the refusals of
    /// `tests/fixtures/genbank/synthetic-unused.tsv`.
    fn syn() -> ViewRecord {
        record("../../tests/fixtures/genbank/synthetic.gb", "synthetic")
    }

    /// §4: a CDS feature is filtered by its display name and by its qualifiers,
    /// a feature that is not a CDS only by its display name, and the names are
    /// attached to the feature's own index.
    #[test]
    fn a_cds_feature_carries_its_qualifier_names() {
        let named: Vec<NamedItem> = feature_names(&syn());
        // Feature 2 is the `m1` CDS: its display name is `ignored-gene`, its
        // `/locus_tag` is `m1`, and the names are both there (D053).
        let item: Vec<&NamedItem> = named.iter().filter(|found| found.item == 2).collect();
        assert!(
            item.iter().any(
                |found| found.name == "ignored-gene" && found.source == NameSource::DisplayName
            ),
            "display name: {item:?}"
        );
        assert!(
            item.iter()
                .any(|found| found.name == "m1" && found.source == NameSource::LocusTag),
            "locus tag: {item:?}"
        );
        // Feature 25 is `filler`, a misc_feature: one name, and no CDS ids.
        let filler: Vec<&NamedItem> = named.iter().filter(|found| found.item == 25).collect();
        assert_eq!(filler.len(), 1, "one name for a non-CDS: {filler:?}");
        assert_eq!(filler[0].name, "filler");
        // No list form name: D049's `name (start..end)` is not in here.
        assert!(
            named
                .iter()
                .all(|found| found.source != NameSource::ListForm),
            "no list form in the feature list"
        );
    }

    /// §4 rule 6: a drawn feature reads with its span in D036 order and its
    /// strand; a refused one reads with the refusal that stands without its
    /// marker (D053).
    #[test]
    fn a_feature_row_names_its_span_or_its_refusal() {
        let pbr: ViewRecord = pbr();
        assert_eq!(
            feature_words(&pbr, 9),
            "tet gene 86..1276 (+)",
            "the tet gene's row"
        );
        assert_eq!(
            feature_words(&pbr, 38),
            "bla CDS 3293..4153 (-)",
            "a minus-strand CDS feature"
        );
        // Feature 18 of synthetic is `order(1..6,7..12)`, refused `order`.
        assert_eq!(
            feature_words(&syn(), 18),
            "ord CDS not drawn: order",
            "a not-drawn feature keeps its reason"
        );
    }

    /// §4 rule 6 and D052: an unusable CDS is in the picker with the code that
    /// refuses it, and a usable one says nothing extra.
    #[test]
    fn a_cds_row_shows_the_code_that_refuses_it() {
        let pbr: ViewRecord = pbr();
        assert_eq!(cds_words(&pbr, 0), "tet 86..1276", "the tet CDS");
        assert_eq!(
            cds_words(&pbr, 1),
            "ROP protein 1915..2106 unusable: no_id",
            "the ROP CDS, greyed with its reason"
        );
    }

    /// §4 rule 7: the window is `body` rows and holds the highlight, moving down
    /// only when the highlight falls below it, and it never runs past the last
    /// row.
    #[test]
    fn the_window_holds_the_highlight() {
        let pbr: ViewRecord = pbr();
        let mut list: List = List::open(&pbr, Features, "");
        assert_eq!(list.rows().len(), 50, "every feature");
        assert_eq!(list.window(20), 0..20, "the first window");

        list.at = Some(20);
        assert_eq!(list.window(20), 1..21, "one row short: one row down");
        list.at = Some(49);
        assert_eq!(list.window(20), 30..50, "the last window, not 31..51");

        list.at = Some(2);
        assert_eq!(
            list.window(60),
            0..50,
            "fewer rows than the body: all of them"
        );
        list.set_filter(&pbr, "nothingmatches".to_string());
        assert_eq!(list.window(20), 0..0, "no rows, no window");
        assert_eq!(list.window(0), 0..0, "no body, no window");
    }

    /// §4 rules 2 and 3: the rows follow the filter, and the highlight goes back
    /// to the first row when they change.
    #[test]
    fn the_rows_follow_the_filter() {
        let pbr: ViewRecord = pbr();
        let mut list: List = List::open(&pbr, Cdss, "");
        assert_eq!(list.rows(), [0, 1, 2], "every CDS");
        list.at = Some(2);
        list.type_char(&pbr, 'r');
        assert_eq!(list.filter(), "r");
        assert_eq!(list.rows(), [1], "ROP protein only");
        assert_eq!(list.at(), Some(0), "the highlight back to the first row");
        list.backspace(&pbr);
        assert_eq!(list.filter(), "", "and backspace undoes the character");
        assert_eq!(list.rows(), [0, 1, 2]);
        list.type_char(&pbr, 'q');
        list.type_char(&pbr, 'q');
        assert!(
            list.rows().is_empty(),
            "a filter naming nothing gives no rows"
        );
        assert_eq!(list.at(), None, "and nothing to highlight");
    }

    /// §4 rule 7: the rows DIM are the ones `enter` refuses — a feature that is
    /// not drawn or whose start is not stated, and an unusable CDS in the
    /// picker, but not a CDS feature the feature list still jumps to.
    #[test]
    fn only_a_refused_row_is_dim() {
        let syn: ViewRecord = syn();
        // Feature 17 is `<1..12`: drawn, with a start that is not stated.
        assert!(refused(&syn, Features, 17), "an open start is refused");
        // Feature 18 is not drawn at all.
        assert!(refused(&syn, Features, 18), "a refusal is refused");
        // Feature 25 `filler` is drawn with a stated start.
        assert!(!refused(&syn, Features, 25), "an ordinary feature is not");
        // CDS 10 of synthetic is `ps`, refused `pseudo`; CDS 0 is `p1`.
        assert!(refused(&syn, Cdss, 10), "an unusable CDS is refused");
        assert!(!refused(&syn, Cdss, 0), "a usable one is not");
        // The `ps` CDS's feature, in the feature list: `enter` jumps to it and
        // says it was not selected, so the row is not DIM.
        assert!(
            !refused(&syn, Features, 11),
            "a CDS feature with an unusable CDS still jumps"
        );
    }
}
