//! The edit buffer: one record's bases and features, edited one edit at a
//! time (contract 9b).
//!
//! `EditBuffer` holds a record while the user edits it. `apply` takes one
//! `Edit` — insert, delete, replace or substitute — moves the bases, then
//! moves every feature by the rules of `tests/fixtures/editor/README.md`
//! (M9-6, M9-13, M9-14), which the table `tests/fixtures/editor/feature-edits.tsv`
//! pins row by row. `apply_undoable` is the same edit with the `Undo` that
//! puts it back, which `undo` applies (contract 9g).
//!
//! Nothing here reads or writes GenBank text. A location is the editor's own
//! `EditLocation` (M9-17): its ranges in the order they were written, an
//! origin-crossing feature's two ranges in D036 order, coordinates zero-based
//! and half-open (D006). `EditFeature::location_text` is kept while no edit
//! has moved the location and cleared by the first edit that does, so the
//! writer (9c3) can keep unmoved text exactly (M9-7).
//!
//! A partial mark (`<`, `>`) belongs to the feature's end, not to one
//! interval, so it outlives the interval that carried it (README "Partial
//! marks", Reading 1). Speed follows D056: one edit moves the letters once
//! and visits each feature and each range once; no `Sequence`, hash or copy
//! of the whole letters is made per edit.

use crate::edit_location::read_edit_location;
use crate::error::CoordinateOutOfBoundsProblem;
use crate::error::DomainError;
use crate::error::EmptyIntervalProblem;
use crate::error::EmptySequenceProblem;
use crate::error::FeatureWithoutIntervalsProblem;
use crate::error::InvalidSequenceCharactersProblem;
use crate::identity::Identifier;
use crate::sequence::is_dna_letter;
use crate::sequence::normalize;
use crate::sequence::Sequence;
use crate::sequence::SequenceTopology;
use crate::strand::Strand;
use crate::view_record::ViewRecord;

/// The one feature key whose qualifiers an edit may invalidate: only `CDS`
/// loses `/translation` or changes `/codon_start` (fixture README, columns).
const CDS_KEY: &str = "CDS";

/// The qualifier that is dropped when a CDS's bases changed (M9-6 h). The
/// status line never names it.
const TRANSLATION_KEY: &str = "translation";

/// The qualifier names carry a nucleotide `pos:` (M9-13 a).
const POSITION_KEYS: [&str; 2] = ["transl_except", "anticodon"];

/// The qualifier name that holds the reading frame (M9-13 b).
const CODON_START_KEY: &str = "codon_start";

/// What an absent `/codon_start` counts as (README Reading 2).
const DEFAULT_CODON_START: u64 = 1;

/// A feature's location as the editor holds it (M9-17): its ranges in the
/// file's order, zero-based half-open (D006), an origin-crossing feature's
/// two ranges in D036 order. A minus-strand join lists its ranges as
/// `complement(join(…))` writes them, low to high.
///
/// `PartialEq` rather than `Eq` because `Strand` (crates/core/src/strand.rs)
/// derives only `PartialEq`.
#[derive(Clone, Debug, PartialEq)]
pub struct EditLocation {
    pub strand: Strand,
    /// The ranges, in the order they were written: D036 order for an
    /// origin-crossing feature, unchecked here.
    pub ranges: Vec<(u64, u64)>,
    /// `<`, on the feature's 5' end. The mark belongs to the end, not to
    /// one interval, so it outlives the interval that carried it; a
    /// location's order, and whether its ranges touch, are not checked.
    pub open_start: bool,
    /// `>`, on the feature's 3' end, same rules.
    pub open_end: bool,
}

/// A feature as the editor holds it: its key, its location, its qualifiers
/// and, while nothing has moved it, the location text it was read from.
#[derive(Clone, Debug, PartialEq)]
pub struct EditFeature {
    pub key: String,
    pub location: EditLocation,
    /// Name and value in file order; a flag qualifier (`/pseudo`) has `None`.
    pub qualifiers: Vec<(String, Option<String>)>,
    /// The location text as read. Kept while no edit has changed
    /// `location`, and set to `None` by the first edit that does (a shift
    /// counts), so the writer can keep unmoved text exactly (plan M9-7).
    pub location_text: Option<String>,
    /// The line of the feature key in the file the record was opened from,
    /// counted from 1 (`GenBankFeature::line`). The status line names a
    /// feature with it (M9-12).
    pub line: u64,
}

/// One change of a record's bases.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edit {
    /// `bases` go before the zero-based base `at`; `at == length` appends.
    Insert { at: u64, bases: Vec<char> },
    /// Removes the bases `[start, end)`.
    Delete { start: u64, end: u64 },
    /// `[start, end)` replaced by `bases`, of any length (M9-6 e).
    Replace {
        start: u64,
        end: u64,
        bases: Vec<char>,
    },
    /// The bases from `start` overwritten one for one by `bases` (`r`, `R`).
    Substitute { start: u64, bases: Vec<char> },
}

/// What an edit did that the status line names.
#[derive(Clone, Debug, PartialEq)]
pub struct EditOutcome {
    /// Features left with no range, removed from the buffer, in the order
    /// they had in it, as they were before the edit (M9-6 c).
    pub removed: Vec<EditFeature>,
    /// Qualifiers M9-13 (a) removed: the feature's index in `features()`
    /// after the edit, and the qualifier's name, in feature then qualifier
    /// order. A removed feature is not indexed, so the indexes count only
    /// the features that stay; and only a `CDS` can lose qualifiers through
    /// an edit, which is why one name needs no second copy.
    pub dropped: Vec<(usize, String)>,
}

/// What it takes to put the buffer back as it was before one edit (contract
/// 9g). `wrote` is the span the edit wrote in the buffer as it stands after
/// the edit, so `undo` knows what to take back out; `removed` holds the
/// letters that stood in the span before the edit, and `features` every
/// feature exactly as it was: ranges, partial marks, qualifiers and location
/// text included.
#[derive(Clone, Debug, PartialEq)]
pub struct Undo {
    wrote: (u64, u64),
    removed: Vec<char>,
    features: Vec<EditFeature>,
}

/// A record being edited: its name, topology, bases and features.
#[derive(Clone, Debug, PartialEq)]
pub struct EditBuffer {
    name: Identifier,
    topology: SequenceTopology,
    letters: Vec<char>,
    features: Vec<EditFeature>,
}

/// One step of an edit in the buffer's own coordinates. A `Replace` is a
/// `Delete` followed by an `Insert` at the same place (M9-6 e), which is how
/// a range and a `pos:` each follow it (README "Replacement").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Atom {
    /// Bases went before the zero-based base `at`; `count` of them.
    Insert { at: u64, count: u64 },
    /// The bases `[start, end)` were removed.
    Delete { start: u64, end: u64 },
    /// The bases `[start, end)` were written over one for one: nothing moves.
    Substitute { start: u64, end: u64 },
}

/// An edit after it has been checked against the buffer: the same four forms
/// in the buffer's coordinates, with the number of bases each writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Plan {
    Insert { at: u64, count: u64 },
    Delete { start: u64, end: u64 },
    Replace { start: u64, end: u64, count: u64 },
    Substitute { start: u64, end: u64 },
}

impl Plan {
    /// The steps of the edit, in the order a range and a `pos:` must follow
    /// them.
    fn atoms(&self) -> Vec<Atom> {
        match *self {
            Plan::Insert { at, count } => vec![Atom::Insert { at, count }],
            Plan::Delete { start, end } => vec![Atom::Delete { start, end }],
            Plan::Replace { start, end, count } => vec![
                Atom::Delete { start, end },
                // M9-6 e: the new bases go where the removed ones ended, on
                // the record the deletion left.
                Atom::Insert { at: start, count },
            ],
            Plan::Substitute { start, end } => vec![Atom::Substitute { start, end }],
        }
    }

    /// The range of bases the edit replaces, empty for an insertion.
    fn span(&self) -> (u64, u64) {
        match *self {
            Plan::Insert { at, count: _ } => (at, at),
            Plan::Delete { start, end } | Plan::Replace { start, end, .. } => (start, end),
            Plan::Substitute { start, end } => (start, end),
        }
    }
}

fn empty_sequence(name: &str) -> DomainError {
    DomainError::EmptySequenceContent(EmptySequenceProblem {
        sequence: name.to_string(),
    })
}

fn invalid_letters(name: &str, position: u64, character: char) -> DomainError {
    DomainError::InvalidSequenceCharacters(InvalidSequenceCharactersProblem {
        sequence: name.to_string(),
        position,
        characters: character.to_string(),
    })
}

fn empty_interval(name: &str, position: u64) -> DomainError {
    DomainError::EmptyInterval(EmptyIntervalProblem {
        sequence: name.to_string(),
        position,
    })
}

fn out_of_bounds(name: &str, start: u64, end: u64, length: u64) -> DomainError {
    DomainError::CoordinateOutOfBounds(CoordinateOutOfBoundsProblem {
        sequence: name.to_string(),
        start,
        end,
        length,
    })
}

fn without_intervals(name: &str, key: &str) -> DomainError {
    DomainError::FeatureWithoutIntervals(FeatureWithoutIntervalsProblem {
        feature: key.to_string(),
        sequence: name.to_string(),
    })
}

/// Refuses written bases that are empty or hold a letter outside the DNA
/// alphabet, naming the first such letter and its index in `bases`.
fn check_letters(name: &str, bases: &[char]) -> Result<(), DomainError> {
    if bases.is_empty() {
        return Err(empty_sequence(name));
    }
    match bases.iter().position(|ch| !is_dna_letter(*ch)) {
        Some(index) => Err(invalid_letters(name, index as u64, bases[index])),
        None => Ok(()),
    }
}

/// Every range of every feature inside the record, refusing a feature with
/// no range first, then an empty range, then a range past the end (rule 1 of
/// contract 9b).
fn check_features(name: &str, features: &[EditFeature], length: u64) -> Result<(), DomainError> {
    for feature in features {
        if feature.location.ranges.is_empty() {
            return Err(without_intervals(name, &feature.key));
        }
    }
    for feature in features {
        for range in feature.location.ranges.iter() {
            if range.0 >= range.1 {
                return Err(empty_interval(name, range.0));
            }
        }
    }
    for feature in features {
        for range in feature.location.ranges.iter() {
            if range.1 > length {
                return Err(out_of_bounds(name, range.0, range.1, length));
            }
        }
    }
    Ok(())
}

/// True when the range `[start, end)` holds any base a feature selects.
fn overlaps(ranges: &[(u64, u64)], start: u64, end: u64) -> bool {
    for range in ranges {
        if start < range.1 && end > range.0 {
            return true;
        }
    }
    false
}

/// The ranges after one insertion (M9-6 a, b, and M9-14), and whether any
/// range grew. `length` is the record's length before the insertion.
///
/// On a circular record the junction between the range ending at the last
/// base and the next one, which starts at base 1, is inside the feature: an
/// insertion at the record's end grows the first range and one at base 1
/// grows the second, while every other range shifts. A linear record treats
/// those two points as edges, and so does any junction that is not D036's
/// origin split.
fn ranges_after_insert(
    ranges: &[(u64, u64)],
    at: u64,
    count: u64,
    length: u64,
    circular: bool,
) -> (Vec<(u64, u64)>, bool) {
    let mut out: Vec<(u64, u64)> = Vec::with_capacity(ranges.len());
    let mut grew: bool = false;
    for (index, range) in ranges.iter().enumerate() {
        let (start, end) = (range.0, range.1);
        let ends_at_origin =
            circular && end == length && index + 1 < ranges.len() && ranges[index + 1].0 == 0;
        let begins_at_origin = circular && start == 0 && index > 0 && ranges[index - 1].1 == length;
        if (ends_at_origin && at == length) || (begins_at_origin && at == 0) {
            out.push((start, end + count));
            grew = true;
        } else if at <= start {
            out.push((start + count, end + count));
        } else if at < end {
            out.push((start, end + count));
            grew = true;
        } else {
            out.push((start, end));
        }
    }
    (out, grew)
}

/// The ranges left by one deletion of `[start, end)`, and whether any of the
/// feature's bases went (M9-6 a, c, d). A range the deletion covers is
/// dropped; one it reaches into is trimmed at the end it reaches.
fn ranges_after_delete(ranges: &[(u64, u64)], start: u64, end: u64) -> (Vec<(u64, u64)>, bool) {
    let removed: u64 = end - start;
    let mut out: Vec<(u64, u64)> = Vec::with_capacity(ranges.len());
    let mut touched: bool = false;
    for range in ranges {
        let (from, to) = (range.0, range.1);
        if end <= from {
            out.push((from - removed, to - removed));
        } else if start >= to {
            out.push((from, to));
        } else if start <= from && end >= to {
            touched = true;
        } else if start <= from {
            touched = true;
            out.push((start, to - removed));
        } else if end >= to {
            touched = true;
            out.push((from, start));
        } else {
            touched = true;
            out.push((from, to - removed));
        }
    }
    (out, touched)
}

/// The number of the feature's own bases a deletion removes before the first
/// one that stays, read 5′ to 3′ (M9-13 b): for `+` from the low end of the
/// first range, for `-` from the high end of the last. The bases between
/// ranges do not count. `ranges` are the feature's ranges before the
/// deletion.
fn five_prime_trim(strand: &Strand, ranges: &[(u64, u64)], start: u64, end: u64) -> u64 {
    let mut trimmed: u64 = 0;
    let reads_from_the_low_end: bool = match strand {
        Strand::Plus => true,
        Strand::Minus => false,
    };
    if reads_from_the_low_end {
        for range in ranges {
            if range.0 < start || range.0 >= end {
                return trimmed;
            }
            trimmed += range.1.min(end) - range.0;
            if range.1 > end {
                return trimmed;
            }
        }
    } else {
        let mut index: usize = ranges.len();
        while index > 0 {
            index -= 1;
            let (start_of_range, end_of_range) = (ranges[index].0, ranges[index].1);
            if end_of_range <= start || end_of_range > end {
                return trimmed;
            }
            trimmed += end_of_range - start_of_range.max(start);
            if start_of_range < start {
                return trimmed;
            }
        }
    }
    trimmed
}

/// The ranges, and whether the edit changed any of the feature's bases,
/// after every step of the edit in order (M9-6 h). A change means a deletion
/// or substitution over one of the feature's ranges, or an insertion that
/// grew one; a shift or an insertion at a range's edge does not change them.
fn move_ranges(
    ranges: &[(u64, u64)],
    atoms: &[Atom],
    length: u64,
    circular: bool,
) -> (Vec<(u64, u64)>, bool) {
    let mut current: Vec<(u64, u64)> = ranges.to_vec();
    let mut changed: bool = false;
    let mut length_now: u64 = length;
    for atom in atoms {
        match *atom {
            Atom::Insert { at, count } => {
                let (next, grew) =
                    ranges_after_insert(current.as_slice(), at, count, length_now, circular);
                current = next;
                changed |= grew;
                length_now += count;
            }
            Atom::Delete { start, end } => {
                let (next, touched) = ranges_after_delete(current.as_slice(), start, end);
                current = next;
                changed |= touched;
                length_now -= end - start;
            }
            Atom::Substitute { start, end } => {
                changed |= overlaps(current.as_slice(), start, end);
            }
        }
    }
    (current, changed)
}

/// A `pos:` read out of a qualifier value: the zero-based half-open range it
/// names, and where in the value it stands and how it was written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Position {
    /// Index in the value where `pos:` begins.
    at: usize,
    /// Index in the value where the text it reads ends.
    until: usize,
    complement: bool,
    range: (u64, u64),
    /// One base rather than `low..high`.
    single: bool,
}

/// The first `pos:` in `value`, as the README's form reads it: `low..high` or
/// one base, inside `complement(…)` or not. `None` for a value that does not
/// carry one (rule 5 of contract 9b).
fn read_position(value: &str) -> Option<Position> {
    let at: usize = value.find("pos:")?;
    let bytes: &[u8] = value.as_bytes();
    let mut cursor: usize = at + "pos:".len();
    let complement: bool = value[cursor..].starts_with("complement(");
    if complement {
        cursor += "complement(".len();
    }
    let low_at: usize = cursor;
    while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
        cursor += 1;
    }
    if cursor == low_at {
        return None;
    }
    let low: u64 = value[low_at..cursor].parse().ok()?;
    let mut high: u64 = low;
    let mut single: bool = true;
    if value[cursor..].starts_with("..") {
        single = false;
        cursor += 2;
        let high_at: usize = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if cursor == high_at {
            return None;
        }
        high = value[high_at..cursor].parse().ok()?;
    }
    // A `)` is part of the position only when `complement(` opened it: a
    // plain `pos:` stops after its last digit, and whatever follows, `)`
    // included, belongs to the rest of the value and is copied back.
    if complement {
        if !value[cursor..].starts_with(")") {
            return None;
        }
        cursor += 1;
    }
    if low == 0 || high < low {
        return None;
    }
    Some(Position {
        at,
        until: cursor,
        complement,
        range: (low - 1, high),
        single,
    })
}

/// The value after every step of the edit (M9-13 a), or `None` when the edit
/// touches the position: a deletion or substitution over one of its bases, or
/// an insertion strictly inside it. A `pos:` follows an insertion or deletion
/// as a range would, but never grows, and the origin rule does not apply to
/// it.
fn move_position(value: &str, position: Position, atoms: &[Atom]) -> Option<String> {
    let mut range: (u64, u64) = position.range;
    for atom in atoms {
        match *atom {
            Atom::Insert { at, count } => {
                if range.0 < at && at < range.1 {
                    return None;
                }
                if at <= range.0 {
                    range.0 += count;
                    range.1 += count;
                }
            }
            Atom::Delete { start, end } => {
                if start < range.1 && end > range.0 {
                    return None;
                }
                if end <= range.0 {
                    let removed: u64 = end - start;
                    range.0 -= removed;
                    range.1 -= removed;
                }
            }
            Atom::Substitute { start, end } => {
                if start < range.1 && end > range.0 {
                    return None;
                }
            }
        }
    }
    let low: u64 = range.0 + 1;
    let high: u64 = range.1;
    let mut out: String = value[..position.at].to_string();
    out.push_str("pos:");
    if position.complement {
        out.push_str("complement(");
    }
    out.push_str(low.to_string().as_str());
    if !position.single {
        out.push('.');
        out.push('.');
        out.push_str(high.to_string().as_str());
    }
    if position.complement {
        out.push(')');
    }
    out.push_str(value[position.until..].to_string().as_str());
    Some(out)
}

/// The frame a CDS carries after its 5′ end lost `trimmed` bases (M9-13 b).
fn frame_after_trim(old: u64, trimmed: u64) -> u64 {
    let shifted: i64 = old as i64 - 1 - (trimmed as i64);
    (shifted.rem_euclid(3) + 1) as u64
}

/// The `/codon_start` a CDS carries after its 5′ end lost `trimmed` bases,
/// and the README's Reading 2: an absent qualifier counts as 1, and a new
/// value that is not 1 is added after the last qualifier. A value that is not
/// exactly 1, 2 or 3, and a `/codon_start` written as a flag, are left
/// exactly as written (rule 5 of contract 9b).
fn adjust_codon_start(qualifiers: &mut Vec<(String, Option<String>)>, trimmed: u64) {
    let mut slot: Option<usize> = None;
    let mut old: u64 = DEFAULT_CODON_START;
    for (index, qualifier) in qualifiers.iter().enumerate() {
        if qualifier.0 != CODON_START_KEY {
            continue;
        }
        let written: &str = match qualifier.1.as_deref() {
            Some(text) => text,
            None => return,
        };
        let value: u64 = match written {
            "1" => 1,
            "2" => 2,
            "3" => 3,
            _ => return,
        };
        slot = Some(index);
        old = value;
    }
    let new: u64 = frame_after_trim(old, trimmed);
    match slot {
        Some(index) => {
            qualifiers[index].1 = Some(new.to_string());
        }
        None => {
            if new != 1 {
                qualifiers.push((CODON_START_KEY.to_string(), Some(new.to_string())));
            }
        }
    }
}

impl EditBuffer {
    /// The record to edit: its name, topology, bases and features.
    ///
    /// Refuses, in this order: no bases at all; a letter outside the DNA
    /// alphabet, lower case included (`InvalidSequenceCharacters.position` is
    /// the first such letter's zero-based index); a feature with no range; a
    /// range whose start is past its end; a range reaching past the last
    /// base.
    pub fn new(
        name: Identifier,
        topology: SequenceTopology,
        letters: Vec<char>,
        features: Vec<EditFeature>,
    ) -> Result<EditBuffer, DomainError> {
        let name_text: String = name.value().to_string();
        check_letters(&name_text, letters.as_slice())?;
        let length: u64 = letters.len() as u64;
        check_features(&name_text, features.as_slice(), length)?;
        Ok(EditBuffer {
            name,
            topology,
            letters,
            features,
        })
    }

    pub fn name(&self) -> Identifier {
        self.name.clone()
    }

    pub fn topology(&self) -> SequenceTopology {
        self.topology.clone()
    }

    pub fn length(&self) -> u64 {
        self.letters.len() as u64
    }

    pub fn letters(&self) -> &[char] {
        self.letters.as_slice()
    }

    pub fn features(&self) -> &[EditFeature] {
        self.features.as_slice()
    }

    /// Applies one edit to the bases and to every feature. On `Err` the
    /// buffer is unchanged. The `Undo` of the edit is thrown away; the view
    /// keeps its own, and calls `apply_undoable` (contract 9g).
    pub fn apply(&mut self, edit: &Edit) -> Result<EditOutcome, DomainError> {
        self.apply_undoable(edit).map(|(outcome, _undo)| outcome)
    }

    /// `apply`, and the step that undoes it (contract 9g).
    pub fn apply_undoable(&mut self, edit: &Edit) -> Result<(EditOutcome, Undo), DomainError> {
        let plan: Plan = self.check(edit)?;
        let written: &[char] = match edit {
            Edit::Insert { bases, .. }
            | Edit::Replace { bases, .. }
            | Edit::Substitute { bases, .. } => bases,
            Edit::Delete { .. } => &[],
        };
        let atoms: Vec<Atom> = plan.atoms();
        let circular: bool = self.topology == SequenceTopology::Circular;
        let length_before: u64 = self.length();

        // The undo is made before anything moves: `span` is in the buffer's
        // coordinates, and its letters and its features are about to change.
        let span: (u64, u64) = plan.span();
        let undo: Undo = Undo {
            wrote: (span.0, span.0 + written.len() as u64),
            removed: self.letters[span.0 as usize..span.1 as usize].to_vec(),
            features: self.features.clone(),
        };

        // The bases: one splice, which moves the tail of the record once.
        let range: std::ops::Range<usize> = (span.0 as usize)..(span.1 as usize);
        self.letters.splice(range, written.iter().copied());

        let mut outcome: EditOutcome = EditOutcome {
            removed: Vec::new(),
            dropped: Vec::new(),
        };
        let mut kept: Vec<EditFeature> = Vec::with_capacity(self.features.len());
        for feature in self.features.iter() {
            // M9-13 b counts the bases the deletion took from the 5′ end, on
            // the ranges the feature had before the edit.
            let trimmed: u64 = match plan {
                Plan::Delete { start, end } | Plan::Replace { start, end, .. } => five_prime_trim(
                    &feature.location.strand,
                    feature.location.ranges.as_slice(),
                    start,
                    end,
                ),
                Plan::Insert { .. } | Plan::Substitute { .. } => 0,
            };
            let (ranges, changed) = move_ranges(
                feature.location.ranges.as_slice(),
                atoms.as_slice(),
                length_before,
                circular,
            );
            if ranges.is_empty() {
                outcome.removed.push(feature.clone());
                continue;
            }
            let location: EditLocation = EditLocation {
                strand: feature.location.strand.clone(),
                ranges,
                // A mark names the feature's end, so it stays with the
                // feature whatever happened to the range that carried it.
                open_start: feature.location.open_start,
                open_end: feature.location.open_end,
            };
            let moved: bool = location != feature.location;
            let index_in_buffer: usize = kept.len();
            let mut qualifiers: Vec<(String, Option<String>)> = Vec::new();
            for qualifier in feature.qualifiers.iter() {
                let key: &str = qualifier.0.as_str();
                if key == TRANSLATION_KEY && feature.key == CDS_KEY && changed {
                    continue;
                }
                if POSITION_KEYS.contains(&key) {
                    let value: &str = qualifier.1.as_deref().unwrap_or("");
                    match read_position(value) {
                        Some(position) => {
                            let moved_text: Option<String> =
                                move_position(value, position, atoms.as_slice());
                            if let Some(text) = moved_text {
                                qualifiers.push((qualifier.0.clone(), Some(text)));
                            } else {
                                outcome.dropped.push((index_in_buffer, key.to_string()));
                            }
                        }
                        // Rule 5: a `pos:` the form does not read is gone
                        // once the edit changes this feature.
                        None => {
                            if changed || moved {
                                outcome.dropped.push((index_in_buffer, key.to_string()));
                            } else {
                                qualifiers.push(qualifier.clone());
                            }
                        }
                    }
                    continue;
                }
                qualifiers.push(qualifier.clone());
            }
            if feature.key == CDS_KEY && trimmed > 0 {
                adjust_codon_start(&mut qualifiers, trimmed);
            }
            let mut edited: EditFeature = feature.clone();
            edited.location = location;
            edited.qualifiers = qualifiers;
            if moved {
                edited.location_text = None;
            }
            kept.push(edited);
        }
        self.features = kept;
        Ok((outcome, undo))
    }

    /// Puts back the letters and features `undo` holds (contract 9g). It
    /// must be the undo of the last edit applied and not yet undone: the
    /// view's `u` and `ctrl-r` keep that order. Anything else is a
    /// programming error, and the debug build says so.
    pub fn undo(&mut self, undo: Undo) {
        debug_assert!(
            undo.wrote.1 as usize <= self.letters.len(),
            "an undo whose span ends past the end of the bases is a programming error"
        );
        let range: std::ops::Range<usize> = (undo.wrote.0 as usize)..(undo.wrote.1 as usize);
        self.letters.splice(range, undo.removed.iter().copied());
        self.features = undo.features;
    }

    /// The bases as a `Sequence` with this name and topology, through
    /// `normalize`. Not called by `apply`: hashing is on demand.
    pub fn to_sequence(&self) -> Result<Sequence, DomainError> {
        let text: String = self.letters.iter().collect();
        normalize(self.name.clone(), text, self.topology.clone())
    }

    /// The edit as it may be applied to this buffer, in the buffer's
    /// coordinates, or the refusal naming what is wrong (rule 2 of contract
    /// 9b).
    fn check(&self, edit: &Edit) -> Result<Plan, DomainError> {
        let name: String = self.name.value().to_string();
        let length: u64 = self.length();
        match edit {
            Edit::Insert { at, bases } => {
                if *at > length {
                    return Err(out_of_bounds(&name, *at, *at, length));
                }
                check_letters(&name, bases.as_slice())?;
                Ok(Plan::Insert {
                    at: *at,
                    count: bases.len() as u64,
                })
            }
            Edit::Delete { start, end } => {
                if start >= end {
                    return Err(empty_interval(&name, *start));
                }
                if *end > length {
                    return Err(out_of_bounds(&name, *start, *end, length));
                }
                if end - start == length {
                    return Err(empty_sequence(&name));
                }
                Ok(Plan::Delete {
                    start: *start,
                    end: *end,
                })
            }
            Edit::Replace { start, end, bases } => {
                if start >= end {
                    return Err(empty_interval(&name, *start));
                }
                if *end > length {
                    return Err(out_of_bounds(&name, *start, *end, length));
                }
                check_letters(&name, bases.as_slice())?;
                Ok(Plan::Replace {
                    start: *start,
                    end: *end,
                    count: bases.len() as u64,
                })
            }
            Edit::Substitute { start, bases } => {
                let count: u64 = bases.len() as u64;
                let end: u64 = start.saturating_add(count);
                if end > length {
                    return Err(out_of_bounds(&name, *start, end, length));
                }
                check_letters(&name, bases.as_slice())?;
                Ok(Plan::Substitute { start: *start, end })
            }
        }
    }
}

/// Why a record cannot be edited (M9-12): the first feature, in file order,
/// whose location `read_edit_location` refuses.
#[derive(Clone, Debug, PartialEq)]
pub struct EditBlocked {
    /// Its index in `GenBankRecord::features()`.
    pub feature: usize,
    pub key: String,
    /// The name the feature list shows (`ViewFeature::name`).
    pub name: String,
    pub line: u64,
    pub error: DomainError,
}

/// The buffer for an opened record: its sequence's name, topology and
/// letters, and for a GenBank record every feature in file order, with
/// `location_text` set to `GenBankFeature::location()` and `qualifiers`
/// from `GenBankQualifier::name()`/`value()`. A FASTA record has none.
///
/// The features are read in file order, and the first location
/// `read_edit_location` refuses is the block (M9-12): nothing past it is
/// read, because the status line names one feature and one refusal only.
///
/// `EditBlocked` carries a `DomainError`, as every refusal in the workspace
/// does; boxing it would make every caller of `buffer_for` unwrap.
#[allow(clippy::result_large_err)]
pub fn buffer_for(record: &ViewRecord) -> Result<EditBuffer, EditBlocked> {
    let length: u64 = record.sequence.length();
    let topology: SequenceTopology = record.sequence.topology();
    let mut features: Vec<EditFeature> = Vec::new();

    // A FASTA record has no feature table at all (D049), so nothing to read
    // and nothing that can block it.
    if let Some(genbank) = record.genbank.as_ref() {
        for (index, feature) in genbank.features().iter().enumerate() {
            match read_edit_location(feature.location().as_str(), length, topology.clone()) {
                Ok(location) => {
                    let qualifiers: Vec<(String, Option<String>)> = feature
                        .qualifiers()
                        .iter()
                        .map(|qualifier| (qualifier.name(), qualifier.value()))
                        .collect();
                    features.push(EditFeature {
                        key: feature.key(),
                        location,
                        qualifiers,
                        location_text: Some(feature.location()),
                        line: feature.line(),
                    });
                }
                Err(error) => {
                    // The name is the feature list's own, taken from the
                    // feature at the same index.
                    return Err(EditBlocked {
                        feature: index,
                        key: feature.key(),
                        name: record.features[index].name.clone(),
                        line: feature.line(),
                        error,
                    });
                }
            }
        }
    }

    match EditBuffer::new(
        record.sequence.name(),
        topology,
        record.sequence.letters(),
        features,
    ) {
        // Every range was read against this sequence's own length, and a
        // sequence `open_record` accepted is made of DNA letters and has bases
        // at all: `EditBuffer::new` has nothing left to refuse.
        Ok(buffer) => Ok(buffer),
        Err(error) => unreachable!("a record open_record accepted: {}", error.code()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::GenBankLocationRule;
    use crate::error::UnusableGenBankLocationProblem;
    use crate::view_record::open_record;

    fn bases(count: usize) -> Vec<char> {
        vec!['A'; count]
    }

    fn letters(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    fn location(strand: Strand, ranges: &[(u64, u64)]) -> EditLocation {
        EditLocation {
            strand,
            ranges: ranges.to_vec(),
            open_start: false,
            open_end: false,
        }
    }

    fn feature(
        key: &str,
        location: EditLocation,
        qualifiers: &[(&str, Option<&str>)],
    ) -> EditFeature {
        EditFeature {
            key: key.to_string(),
            location,
            qualifiers: qualifiers
                .iter()
                .map(|(name, value)| (name.to_string(), value.map(|text| text.to_string())))
                .collect(),
            location_text: Some("join(1..2)".to_string()),
            line: 1,
        }
    }

    fn record(
        topology: SequenceTopology,
        letters: &[char],
        features: &[EditFeature],
    ) -> EditBuffer {
        EditBuffer::new(
            Identifier::create("record".to_string())
                .unwrap_or_else(|error| panic!("a name is created: {}", error.message())),
            topology,
            letters.to_vec(),
            features.to_vec(),
        )
        .unwrap_or_else(|error| panic!("the record is held: {}", error.message()))
    }

    fn name() -> Identifier {
        Identifier::create("record".to_string())
            .unwrap_or_else(|error| panic!("a name is created: {}", error.message()))
    }

    fn text(buffer: &EditBuffer) -> String {
        buffer.letters().iter().collect()
    }

    fn reason(found: &Result<EditBuffer, DomainError>) -> String {
        match found {
            Ok(_) => "accepted".to_string(),
            Err(error) => error.code().to_string(),
        }
    }

    #[test]
    fn new_refuses_in_order() {
        // The bases come before the features, and each rule in the order the
        // contract names, so a record wrong in several ways is refused for
        // the earliest one.
        let empty: Result<EditBuffer, DomainError> =
            EditBuffer::new(name(), SequenceTopology::Linear, Vec::new(), Vec::new());
        match empty {
            Err(DomainError::EmptySequenceContent(problem)) => {
                assert_eq!(problem.sequence, "record");
            }
            _ => panic!("{} for no bases at all", reason(&empty)),
        }

        let lowercase: Result<EditBuffer, DomainError> = EditBuffer::new(
            name(),
            SequenceTopology::Linear,
            letters("acgtacgt"),
            Vec::new(),
        );
        match lowercase {
            Err(DomainError::InvalidSequenceCharacters(problem)) => {
                // The first letter outside the alphabet, and the letter.
                assert_eq!(problem.sequence, "record");
                assert_eq!(problem.position, 0);
                assert_eq!(problem.characters, "a");
            }
            _ => panic!("{} for a letter outside the alphabet", reason(&lowercase)),
        }

        let no_ranges: Result<EditBuffer, DomainError> = EditBuffer::new(
            name(),
            SequenceTopology::Linear,
            letters("ACGTACGT"),
            vec![
                feature("CDS", location(Strand::Plus, &[]), &[]),
                feature("misc_feature", location(Strand::Plus, &[(3, 5)]), &[]),
            ],
        );
        match no_ranges {
            Err(DomainError::FeatureWithoutIntervals(problem)) => {
                assert_eq!(problem.feature, "CDS");
                assert_eq!(problem.sequence, "record");
            }
            _ => panic!(
                "{} for a feature with no range of bases",
                reason(&no_ranges)
            ),
        }

        let reversed: Result<EditBuffer, DomainError> = EditBuffer::new(
            name(),
            SequenceTopology::Linear,
            letters("ACGTACGT"),
            vec![
                feature("gene", location(Strand::Plus, &[(0, 8)]), &[]),
                feature("misc_feature", location(Strand::Plus, &[(6, 2)]), &[]),
                feature("other", location(Strand::Plus, &[(2, 20)]), &[]),
            ],
        );
        match reversed {
            Err(DomainError::EmptyInterval(problem)) => {
                assert_eq!(problem.sequence, "record");
                assert_eq!(problem.position, 6);
            }
            _ => panic!("{} for a range that starts past its end", reason(&reversed)),
        }

        let past_end: Result<EditBuffer, DomainError> = EditBuffer::new(
            name(),
            SequenceTopology::Linear,
            letters("ACGTACGT"),
            vec![
                feature("gene", location(Strand::Plus, &[(0, 8)]), &[]),
                feature("misc_feature", location(Strand::Plus, &[(2, 9)]), &[]),
            ],
        );
        match past_end {
            Err(DomainError::CoordinateOutOfBounds(problem)) => {
                assert_eq!(problem.sequence, "record");
                assert_eq!(problem.start, 2);
                assert_eq!(problem.end, 9);
                assert_eq!(problem.length, 8);
            }
            _ => panic!("{} for a range past the last base", reason(&past_end)),
        }
    }

    #[test]
    fn apply_refuses_and_changes_nothing() {
        let cds: EditFeature = feature(
            "CDS",
            location(Strand::Minus, &[(3, 8)]),
            &[
                ("product", Some("protein X")),
                ("transl_except", Some("pos:6..8")),
            ],
        );
        let mut buffer: EditBuffer = record(SequenceTopology::Linear, &letters("ACGTACGT"), &[cds]);
        let refused: Vec<Edit> = vec![
            Edit::Insert {
                at: 9,
                bases: letters("G"),
            },
            Edit::Insert {
                at: 2,
                bases: Vec::new(),
            },
            Edit::Insert {
                at: 2,
                bases: letters("GNP"),
            },
            Edit::Delete { start: 4, end: 4 },
            Edit::Delete { start: 4, end: 9 },
            Edit::Delete { start: 0, end: 8 },
            Edit::Replace {
                start: 3,
                end: 2,
                bases: letters("G"),
            },
            Edit::Replace {
                start: 6,
                end: 9,
                bases: letters("G"),
            },
            Edit::Replace {
                start: 1,
                end: 3,
                bases: Vec::new(),
            },
            Edit::Substitute {
                start: 7,
                bases: letters("AC"),
            },
            Edit::Substitute {
                start: 1,
                bases: Vec::new(),
            },
            Edit::Substitute {
                start: 1,
                bases: letters("CP"),
            },
        ];
        let letters_before: Vec<char> = buffer.letters().to_vec();
        let features_before: Vec<EditFeature> = buffer.features().to_vec();
        for edit in refused.iter() {
            let outcome: Result<EditOutcome, DomainError> = buffer.apply(edit);
            assert!(outcome.is_err(), "{:?} is refused", edit);
            assert_eq!(buffer.letters(), letters_before.as_slice(), "{:?}", edit);
            assert_eq!(buffer.features(), features_before.as_slice(), "{:?}", edit);
        }
    }

    #[test]
    fn letters_follow_each_edit() {
        let mut buffer: EditBuffer = record(
            SequenceTopology::Linear,
            &letters("ACGTACGT"),
            &[feature("CDS", location(Strand::Plus, &[(0, 8)]), &[])],
        );
        buffer
            .apply(&Edit::Insert {
                at: 2,
                bases: letters("GG"),
            })
            .unwrap_or_else(|error| panic!("an insertion is held: {}", error.message()));
        assert_eq!(text(&buffer).as_str(), "ACGGGTACGT");
        buffer
            .apply(&Edit::Delete { start: 0, end: 2 })
            .unwrap_or_else(|error| panic!("a deletion is held: {}", error.message()));
        assert_eq!(text(&buffer).as_str(), "GGGTACGT");
        buffer
            .apply(&Edit::Replace {
                start: 1,
                end: 3,
                bases: letters("T"),
            })
            .unwrap_or_else(|error| panic!("a replacement is held: {}", error.message()));
        assert_eq!(text(&buffer).as_str(), "GTTACGT");
        buffer
            .apply(&Edit::Substitute {
                start: 0,
                bases: letters("CC"),
            })
            .unwrap_or_else(|error| panic!("a substitution is held: {}", error.message()));
        assert_eq!(text(&buffer).as_str(), "CCTACGT");
    }

    #[test]
    fn several_features_and_removal_order() {
        let first: EditFeature = feature(
            "CDS",
            location(Strand::Plus, &[(0, 5)]),
            &[("translation", Some("MELE"))],
        );
        let second: EditFeature = feature(
            "CDS",
            location(Strand::Plus, &[(10, 20)]),
            &[
                ("product", Some("protein X")),
                ("transl_except", Some("pos:11..13")),
            ],
        );
        let third: EditFeature = feature("gene", location(Strand::Plus, &[(2, 4)]), &[]);
        let mut buffer: EditBuffer = record(
            SequenceTopology::Linear,
            &bases(30),
            &[first.clone(), second.clone(), third.clone()],
        );
        let outcome: EditOutcome = buffer
            .apply(&Edit::Delete { start: 0, end: 12 })
            .unwrap_or_else(|error| panic!("a deletion is held: {}", error.message()));
        // In the order they had in the buffer, and as they were in it.
        assert_eq!(outcome.removed, vec![first, third]);
        assert_eq!(
            outcome.dropped,
            vec![(0, "transl_except".to_string())],
            "the surviving feature is the only one in the buffer, so one name is indexed"
        );
        let expected: EditFeature = EditFeature {
            key: "CDS".to_string(),
            location: location(Strand::Plus, &[(0, 8)]),
            // Reading 2 of the README: the two bases the deletion took from
            // the 5' end shift the frame, and the new value is added after
            // the last qualifier.
            qualifiers: vec![
                ("product".to_string(), Some("protein X".to_string())),
                ("codon_start".to_string(), Some("2".to_string())),
            ],
            location_text: None,
            line: 1,
        };
        assert_eq!(buffer.features(), [expected]);
    }

    #[test]
    fn to_sequence_matches_normalize() {
        let mut buffer: EditBuffer = record(
            SequenceTopology::Circular,
            &letters("ACGTACGT"),
            &[feature("CDS", location(Strand::Plus, &[(1, 6)]), &[])],
        );
        buffer
            .apply(&Edit::Insert {
                at: 4,
                bases: letters("GG"),
            })
            .unwrap_or_else(|error| panic!("an insertion is held: {}", error.message()));
        buffer
            .apply(&Edit::Delete { start: 0, end: 2 })
            .unwrap_or_else(|error| panic!("a deletion is held: {}", error.message()));
        // ACGTGGACGT with its first two bases gone.
        assert_eq!(text(&buffer).as_str(), "GTGGACGT");
        let expected: Sequence = normalize(name(), text(&buffer), SequenceTopology::Circular)
            .unwrap_or_else(|error| panic!("the same bases normalize: {}", error.message()));
        assert_eq!(
            buffer.to_sequence().unwrap_or_else(|error| panic!(
                "the buffer reads as a sequence: {}",
                error.message()
            )),
            expected
        );
    }
    #[test]
    fn a_plain_position_keeps_what_follows_it() {
        // A `pos:` that `complement(` did not open stops after its last digit,
        // so the `)` written after it belongs to the value around it and stays
        // when the position moves (M9-13 a).
        let range: EditFeature = feature(
            "CDS",
            location(Strand::Plus, &[(29, 60)]),
            &[("transl_except", Some("(pos:45..47)"))],
        );
        let single: EditFeature = feature(
            "CDS",
            location(Strand::Plus, &[(29, 60)]),
            &[("transl_except", Some("(pos:60)"))],
        );
        let mut buffer: EditBuffer =
            record(SequenceTopology::Linear, &bases(100), &[range, single]);
        buffer
            .apply(&Edit::Insert {
                at: 10,
                bases: letters("GAG"),
            })
            .unwrap_or_else(|error| panic!("an insertion is held: {}", error.message()));
        assert_eq!(
            buffer.features()[0].qualifiers,
            vec![(
                "transl_except".to_string(),
                Some("(pos:48..50)".to_string())
            )],
            "the range form keeps the parenthesis written after it"
        );
        assert_eq!(
            buffer.features()[1].qualifiers,
            vec![("transl_except".to_string(), Some("(pos:63)".to_string()))],
            "the single-base form keeps it too"
        );
    }

    #[test]
    fn edits_stay_fast_at_the_ceiling() {
        // D056: the ceiling is 10 000 000 bases, and a record at it is legal.
        // One edit costs at most one move of the letters, plus one visit per
        // feature and per range, so 50 one-base inserts at the middle of a
        // record at the ceiling, and 50 one-base deletes there, with 2 000
        // one-range features along the record, finish far inside five seconds
        // in a debug build.
        let count: usize = 10_000_000;
        let mut features: Vec<EditFeature> = Vec::new();
        for index in 0..2_000_u64 {
            let low: u64 = index * 5_000 + 100;
            features.push(feature(
                "CDS",
                location(Strand::Plus, &[(low, low + 100)]),
                &[("product", Some("protein X"))],
            ));
        }
        let mut buffer: EditBuffer = record(SequenceTopology::Circular, &bases(count), &features);
        let middle: u64 = (count / 2) as u64;
        let started: std::time::Instant = std::time::Instant::now();
        for _ in 0..50 {
            buffer
                .apply(&Edit::Insert {
                    at: middle,
                    bases: letters("A"),
                })
                .unwrap_or_else(|error| panic!("an insertion is held: {}", error.message()));
        }
        for _ in 0..50 {
            buffer
                .apply(&Edit::Delete {
                    start: middle,
                    end: middle + 1,
                })
                .unwrap_or_else(|error| panic!("a deletion is held: {}", error.message()));
        }
        let seconds: f64 = started.elapsed().as_secs_f64();
        // The inserts and the deletes cancel out, and no feature's range runs
        // across the middle, so the record and every feature stand where they
        // were.
        assert_eq!(buffer.length(), count as u64);
        assert_eq!(buffer.features().len(), features.len());
        for (index, found) in buffer.features().iter().enumerate() {
            assert_eq!(found.location, features[index].location);
            assert_eq!(found.key, features[index].key);
            assert_eq!(found.qualifiers, features[index].qualifiers);
        }
        assert!(
            seconds < 5.0,
            "a hundred edits at the ceiling took {} seconds",
            seconds
        );
        // M9-7: a feature whose location never moved keeps its text, and one
        // that moved once does not come back to it, even when the ranges the
        // edit puts back.
        let half: usize = features.len() / 2;
        assert_eq!(
            buffer.features()[half - 1].location_text,
            features[half - 1].location_text
        );
        assert_eq!(buffer.features()[half + 1].location_text, None);
    }

    /// M9-12 (contract 9b2 §5 test 5): a record whose feature this reader
    /// cannot read is not editable at all, and the name given is the first
    /// such feature in file order. `synthetic.gb:61` holds
    /// `order(1..6,7..12)`, which is feature 18 of the table; every feature
    /// before it is a range, an origin join on this 60 bp circular record,
    /// or `<1..12`.
    #[test]
    fn a_blocked_record_names_its_first_feature() {
        let path: &str = "../../tests/fixtures/genbank/synthetic.gb";
        let text: String = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
        let name: Identifier = Identifier::create("synthetic".to_string())
            .unwrap_or_else(|error| panic!("a name is created: {}", error.message()));
        let record: ViewRecord = open_record(name, text)
            .unwrap_or_else(|error| panic!("open_record reads synthetic.gb: {}", error.message()));
        let blocked: EditBlocked = match buffer_for(&record) {
            Ok(buffer) => panic!(
                "synthetic.gb is blocked, not a buffer of {} features",
                buffer.features().len()
            ),
            Err(blocked) => blocked,
        };
        assert_eq!(blocked.feature, 18, "the index in the feature table");
        assert_eq!(blocked.key, "CDS", "the key on the feature line");
        assert_eq!(blocked.name, "ord", "the name the feature list shows");
        assert_eq!(blocked.line, 61, "the file line of the feature key");
        assert_eq!(
            blocked.error,
            DomainError::UnusableGenBankLocation(UnusableGenBankLocationProblem {
                location: "order(1..6,7..12)".to_string(),
                rule: GenBankLocationRule::Order,
                sequence_length: 60,
                cds: None,
                line: None,
            }),
            "the refusal that blocks the record"
        );
    }

    /// §5 test 6: a FASTA record gives a buffer with the sequence's letters
    /// and no features: it states no feature table at all (D049).
    #[test]
    fn fasta_opens_with_no_features() {
        let text: String = ">w16 no feature table\nACGTACGTACGTACGT\n".to_string();
        let name: Identifier = Identifier::create("w16".to_string())
            .unwrap_or_else(|error| panic!("a name is created: {}", error.message()));
        let record: ViewRecord = open_record(name, text).unwrap_or_else(|error| {
            panic!("open_record reads the FASTA text: {}", error.message())
        });
        assert!(
            record.genbank.is_none(),
            "a FASTA record has no GenBank record to read features from"
        );
        let buffer: EditBuffer = buffer_for(&record)
            .unwrap_or_else(|blocked| panic!("a FASTA record opens for editing: {:?}", blocked));
        assert_eq!(buffer.name(), record.sequence.name());
        assert_eq!(buffer.length(), 16);
        assert_eq!(buffer.letters(), letters("ACGTACGTACGTACGT").as_slice());
        assert!(buffer.features().is_empty(), "no feature to read");
    }

    /// §5 test 1 (full tier): `apply_undoable` then `undo` gives back the
    /// buffer exactly as it was: its length, its letters, and its features
    /// with their qualifiers, partial marks and location text. The rows are
    /// 15 of 9a's table rows, a list covering every rule group (M9-6 a
    /// through h, M9-13 a and b, and M9-14), built the way
    /// `crates/core/tests/feature_edit_fixtures.rs` builds them: the table's
    /// 1-based inclusive coordinates become the buffer's zero-based
    /// half-open ones (D006). Each row moves or drops its feature, so the
    /// undo has features as well as letters to put back.
    #[test]
    fn undo_restores_exactly() {
        // The row's one feature: its key, its shape, its location text and
        // its qualifiers. The text is `Some` because the feature is as the
        // row opens it, and `apply` clears it when the feature moves.
        fn one(
            key: &str,
            shape: EditLocation,
            text: &str,
            qualifiers: &[(&str, Option<&str>)],
        ) -> EditFeature {
            EditFeature {
                key: key.to_string(),
                location: shape,
                qualifiers: qualifiers
                    .iter()
                    .map(|(name, value)| (name.to_string(), value.map(|text| text.to_string())))
                    .collect(),
                location_text: Some(text.to_string()),
                line: 1,
            }
        }

        // The row: its id, topology, length, feature and edit.
        let rows: Vec<(&str, SequenceTopology, usize, EditFeature, Edit)> = vec![
            // M9-6 a: a deletion just before the feature shifts it left.
            (
                "a3: M9-6a, del 20-29",
                SequenceTopology::Linear,
                100,
                one(
                    "misc_feature",
                    location(Strand::Plus, &[(29, 40)]),
                    "30..40",
                    &[],
                ),
                Edit::Delete { start: 19, end: 29 },
            ),
            // M9-6 b: an insertion inside the feature, at either end.
            (
                "b1: M9-6b, ins 5 after 30",
                SequenceTopology::Linear,
                100,
                one(
                    "misc_feature",
                    location(Strand::Plus, &[(29, 40)]),
                    "30..40",
                    &[],
                ),
                Edit::Insert {
                    at: 30,
                    bases: vec!['A'; 5],
                },
            ),
            (
                "b2: M9-6b, ins 5 after 39",
                SequenceTopology::Linear,
                100,
                one(
                    "misc_feature",
                    location(Strand::Plus, &[(29, 40)]),
                    "30..40",
                    &[],
                ),
                Edit::Insert {
                    at: 39,
                    bases: vec!['A'; 5],
                },
            ),
            // M9-6 c: one deletion that leaves the feature with no range,
            // and one that takes the second range of a two-range feature.
            (
                "c3: M9-6c, del 25-45",
                SequenceTopology::Linear,
                100,
                one(
                    "misc_feature",
                    location(Strand::Plus, &[(29, 40)]),
                    "30..40",
                    &[],
                ),
                Edit::Delete { start: 24, end: 45 },
            ),
            (
                "c4: M9-6c, del 28-45",
                SequenceTopology::Linear,
                100,
                one(
                    "misc_feature",
                    location(Strand::Plus, &[(9, 20), (29, 40)]),
                    "join(10..20,30..40)",
                    &[],
                ),
                Edit::Delete { start: 27, end: 45 },
            ),
            // M9-6 d: the partial marks, each on the end the edit touches.
            (
                "d1: M9-6d, del 25-32",
                SequenceTopology::Linear,
                100,
                one(
                    "misc_feature",
                    EditLocation {
                        open_start: true,
                        ..location(Strand::Plus, &[(29, 40)])
                    },
                    "<30..40",
                    &[],
                ),
                Edit::Delete { start: 24, end: 32 },
            ),
            (
                "d2: M9-6d, del 38-45",
                SequenceTopology::Linear,
                100,
                one(
                    "misc_feature",
                    EditLocation {
                        open_end: true,
                        ..location(Strand::Plus, &[(29, 40)])
                    },
                    "30..>40",
                    &[],
                ),
                Edit::Delete { start: 37, end: 45 },
            ),
            // M9-6 e: a replacement of a different length, and a
            // substitution that leaves the feature in place.
            (
                "e1: M9-6e, rep 33-35 5",
                SequenceTopology::Linear,
                100,
                one(
                    "misc_feature",
                    location(Strand::Plus, &[(29, 40)]),
                    "30..40",
                    &[],
                ),
                Edit::Replace {
                    start: 32,
                    end: 35,
                    bases: vec!['A'; 5],
                },
            ),
            (
                "e5: M9-6e, sub 33-35",
                SequenceTopology::Linear,
                100,
                one(
                    "misc_feature",
                    location(Strand::Plus, &[(29, 40)]),
                    "30..40",
                    &[],
                ),
                Edit::Substitute {
                    start: 32,
                    bases: vec!['C'; 3],
                },
            ),
            // M9-6 f: an insertion between the two ranges of one feature.
            (
                "f1: M9-6f, ins 4 after 25",
                SequenceTopology::Linear,
                100,
                one(
                    "misc_feature",
                    location(Strand::Plus, &[(9, 20), (29, 40)]),
                    "join(10..20,30..40)",
                    &[],
                ),
                Edit::Insert {
                    at: 25,
                    bases: vec!['A'; 4],
                },
            ),
            // M9-6 g: an insertion into a join that crosses the origin of a
            // circular record (D036).
            (
                "g1: M9-6g, ins 3 after 5",
                SequenceTopology::Circular,
                62,
                one(
                    "CDS",
                    location(Strand::Plus, &[(55, 62), (0, 11)]),
                    "join(56..62,1..11)",
                    &[],
                ),
                Edit::Insert {
                    at: 5,
                    bases: vec!['A'; 3],
                },
            ),
            // M9-6 h: a `CDS` that carries `codon_start`, `translation` and
            // `product`, with an insertion before its first base.
            (
                "h1: M9-6h, ins 3 after 10",
                SequenceTopology::Linear,
                100,
                one(
                    "CDS",
                    location(Strand::Plus, &[(29, 40)]),
                    "30..40",
                    &[
                        ("codon_start", Some("1")),
                        ("translation", Some("MAAA")),
                        ("product", Some("x")),
                    ],
                ),
                Edit::Insert {
                    at: 10,
                    bases: vec!['A'; 3],
                },
            ),
            // M9-13 a: a `pos:` inside the range an insertion extends.
            (
                "t5: M9-13a, ins 2 after 45",
                SequenceTopology::Linear,
                100,
                one(
                    "CDS",
                    location(Strand::Plus, &[(29, 60)]),
                    "30..60",
                    &[("transl_except", Some("(pos:45..47,aa:Sec)"))],
                ),
                Edit::Insert {
                    at: 45,
                    bases: vec!['A'; 2],
                },
            ),
            // M9-13 b: a deletion at the 5′ end of a partial `CDS` renumbers
            // `codon_start`.
            (
                "s2: M9-13b, del 1-2",
                SequenceTopology::Linear,
                100,
                one(
                    "CDS",
                    EditLocation {
                        open_start: true,
                        ..location(Strand::Plus, &[(0, 60)])
                    },
                    "<1..60",
                    &[("codon_start", Some("1"))],
                ),
                Edit::Delete { start: 0, end: 2 },
            ),
            // M9-14: an insertion after the last base of a record moves the
            // origin that a join wraps around.
            (
                "o4: M9-14, ins 5 after 4361",
                SequenceTopology::Circular,
                4_361,
                one(
                    "misc_feature",
                    location(Strand::Plus, &[(3_999, 4_361), (0, 10)]),
                    "join(4000..4361,1..10)",
                    &[],
                ),
                Edit::Insert {
                    at: 4_361,
                    bases: vec!['A'; 5],
                },
            ),
        ];

        for (id, topology, length, feature, edit) in rows.into_iter() {
            let mut buffer: EditBuffer = record(topology, &bases(length), &[feature]);
            let before: EditBuffer = buffer.clone();
            let (_outcome, undo): (EditOutcome, Undo) =
                buffer.apply_undoable(&edit).unwrap_or_else(|error| {
                    panic!("{id}: the row's edit is held: {}", error.message())
                });
            // A substitution overwrites bases one for one and moves
            // nothing, so a feature keeps its ranges and its text; every
            // other row moves or removes the feature, so `undo` has
            // features as well as letters to put back.
            if id.contains("sub") {
                assert_eq!(
                    buffer.features(),
                    before.features(),
                    "{id}: a substitution moves nothing"
                );
            } else {
                assert_ne!(
                    buffer.features(),
                    before.features(),
                    "{id}: the edit moves or removes the feature"
                );
            }
            assert_ne!(buffer, before, "{id}: the edit changes the record");

            buffer.undo(undo);
            assert_eq!(buffer.length(), before.length(), "{id}: the length");
            assert_eq!(buffer.letters(), before.letters(), "{id}: the letters");
            assert_eq!(
                buffer.features(),
                before.features(),
                "{id}: the features with their qualifiers and location text"
            );
            assert_eq!(buffer, before, "{id}: the whole record");
        }
    }

    /// §5 test 2: three edits on pBR322's buffer, undone last first, give
    /// back the record exactly as it was opened.
    #[test]
    fn undo_in_reverse_order() {
        let path: &str = "../../tests/fixtures/genbank/raw/J01749.1.gb";
        let text: String = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
        let name: Identifier = Identifier::create("J01749.1".to_string())
            .unwrap_or_else(|error| panic!("a name is created: {}", error.message()));
        let view: ViewRecord = open_record(name, text)
            .unwrap_or_else(|error| panic!("open_record reads J01749.1: {}", error.message()));
        let mut buffer: EditBuffer = buffer_for(&view)
            .unwrap_or_else(|blocked| panic!("pBR322 opens for editing: {:?}", blocked));
        let opened: EditBuffer = buffer.clone();

        let edits: [Edit; 3] = [
            Edit::Insert {
                at: 40,
                bases: letters("GAT"),
            },
            Edit::Delete {
                start: 90,
                end: 150,
            },
            Edit::Substitute {
                start: 200,
                bases: letters("AA"),
            },
        ];
        let mut undos: Vec<Undo> = Vec::new();
        for edit in edits.iter() {
            let (_outcome, undo) = buffer
                .apply_undoable(edit)
                .unwrap_or_else(|error| panic!("an edit is held: {}", error.message()));
            undos.push(undo);
        }
        assert_ne!(buffer, opened, "three edits change the record");

        for undo in undos.into_iter().rev() {
            buffer.undo(undo);
        }
        assert_eq!(
            buffer, opened,
            "the record is back as opened, with its letters and every feature"
        );
    }
}
