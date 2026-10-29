//! The view's search: the prompt `/` opens, which mode its text is in, and the
//! hits `enter` puts on the screen (contract 8l1 §4; plan §5.4, proposal §4;
//! D048 M8-12, D049 M8-3, D052, D054).
//!
//! This module holds the prompt's text and mode, the hits as the view keeps them
//! and what a hit is named by. `super::state` owns the record, the cursor and the
//! selection, routes the keys while the prompt is open, and decides what a search
//! does to the screen (`super::screen` paints the hits on the bases); matching
//! itself stays in core, `search::find_dna` and `protein_search::find_protein`,
//! which is what `luria find` calls too, so the view and the report cannot
//! disagree about what a pattern matches (D048's shared entry point).
//!
//! The cap is here and nowhere else (M8-12, D048): core returns every hit, at
//! every position, overlapping included (D050's M8-16), and the view keeps the
//! first `MAX_HITS` of them and says how many there really are. There is no
//! second cap in `state` or `screen`, and no search history.

use crate::find::nucleotide_letter;
use luria_core::cds::Cds;
use luria_core::protein_search::find_protein;
use luria_core::protein_search::ProteinHit;
use luria_core::protein_search::ProteinTarget;
use luria_core::search::find_dna;
use luria_core::search::HitStrand;
use luria_core::sequence::normalize;
use luria_core::sequence::Sequence;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;
use luria_core::view_record::TopologySource;
use luria_core::view_record::ViewRecord;

/// The most hits the view marks, counts and cycles through (M8-12, D048). It is
/// presentation, not a rule of core, and `luria find` caps its printed rows at the
/// same number (`find::MAX_ROWS`).
const MAX_HITS: usize = 200;

/// Which of core's two matchers the prompt's text is searched with (plan §5.4,
/// proposal §4): IUPAC nucleotide codes over both strands, or one-letter residues
/// over the selected CDS or the six frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchMode {
    Dna,
    Aa,
}

impl SearchMode {
    /// The badge the prompt shows (proposal §4): `DNA` or `AA`.
    pub(crate) fn badge(self) -> &'static str {
        match self {
            SearchMode::Dna => "DNA",
            SearchMode::Aa => "AA",
        }
    }

    /// The other mode, which is what `tab` switches to (§4 rule 2).
    fn flipped(self) -> SearchMode {
        match self {
            SearchMode::Dna => SearchMode::Aa,
            SearchMode::Aa => SearchMode::Dna,
        }
    }
}

/// The mode `text` is in on its own (§4 rule 2, plan §5.4): `Aa` as soon as one
/// character is not a nucleotide letter in either case — `E`, `F`, `I`, `L`, `P`,
/// `Q`, `X`, `*`, a digit — and `Dna` otherwise, the empty text included. The
/// letters read are the ones `luria find` reads (find.rs §4.5), so the prompt and
/// the report agree that `MKSNNA`, whose every letter is an IUPAC code, is DNA.
pub fn auto_mode(text: &str) -> SearchMode {
    if text.chars().all(nucleotide_letter) {
        SearchMode::Dna
    } else {
        SearchMode::Aa
    }
}

/// The open prompt: what has been typed, the mode it is in, and whether `tab` has
/// been pressed since it opened. While it is open the text and the mode are here
/// and nothing else changes (§4 rule 2); `enter` takes them out and searches.
pub struct SearchPrompt {
    text: String,
    mode: SearchMode,
    /// True once `tab` has been pressed: from then on the mode no longer follows
    /// the text, which is what lets a peptide of IUPAC letters — `DAVNG`, `GSG` —
    /// be searched as protein at all (plan §5.4, proposal §4).
    tabbed: bool,
}

impl SearchPrompt {
    /// The prompt `/` opens with: nothing typed, `Dna`, `tab` not pressed (§4 rule
    /// 1).
    pub(crate) fn open() -> SearchPrompt {
        SearchPrompt {
            text: String::new(),
            mode: SearchMode::Dna,
            tabbed: false,
        }
    }

    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    pub fn mode(&self) -> SearchMode {
        self.mode
    }

    /// A character typed: it goes on the end of the text, and until `tab` has been
    /// pressed the mode follows what the text now says (§4 rule 2).
    pub(crate) fn type_char(&mut self, ch: char) {
        self.text.push(ch);
        self.follow_text();
    }

    /// `backspace`: the last character typed goes, and with it — while `tab` has
    /// not been pressed — the switch it caused, so deleting the letter that forced
    /// `AA` returns the badge to `DNA` (plan §5.4).
    pub(crate) fn backspace(&mut self) {
        // `pop` removes the last character whole, so a multi-byte character is
        // never cut in half.
        self.text.pop();
        self.follow_text();
    }

    /// `tab`: the mode flips, and stops following the text from then on for the
    /// rest of this prompt (§4 rule 2). Pressing it again flips back.
    pub(crate) fn press_tab(&mut self) {
        self.mode = self.mode.flipped();
        self.tabbed = true;
    }

    /// The mode follows the text only while `tab` has not been pressed (§4 rule 2,
    /// rule 4 of plan §5.4).
    fn follow_text(&mut self) {
        if !self.tabbed {
            self.mode = auto_mode(&self.text);
        }
    }
}

/// One hit as the view keeps it: where its run of bases starts, how many bases it
/// covers, what the status line names it by — the strand symbol for DNA, the
/// frame in six frames, the residue numbers in a CDS (§4 rule 4), and which
/// strand the hit was found on, which says which row's bases go yellow (M8-24).
/// A DNA hit keeps core's `HitStrand` as it is; a protein hit has no `Both`,
/// because a pattern is read on the one strand its CDS or frame sits on.
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub start: u64,
    pub length: u64,
    pub label: String,
    pub strand: HitStrand,
}

impl Hit {
    /// True when `base` is one of the bases this hit covers. Its run is
    /// `[start, start + length)` in reference order; where that run passes the
    /// origin of a circular record the bases it wraps on to, `[0, end - size)`, are
    /// covered too, and both runs are marked (§4 rule 8, D052). The arithmetic
    /// stays in `u64` without overflow: a hit's length is at most the record's
    /// length and `start` is inside the record, so `end` is under twice the record
    /// length, which no record reaches (D048 M8-14).
    pub(crate) fn covers(&self, base: u64, size: u64) -> bool {
        let end: u64 = self.start + self.length;
        if base >= self.start && base < end {
            return true;
        }
        end > size && base < end - size
    }

    /// The hit's 1-based span as the status line shows it (8l1 §4 rule 7), in
    /// reference coordinates: `59..62` for the hit at 58 covering 4 bases, and
    /// `4359..3` for one whose run runs past the record's last base, because the
    /// last base is numbered again from the first. That is the span `luria find`
    /// prints, base for base (`find::last_base`, 8i2): the last base is
    /// `(start + length - 1) % size + 1`. `size` is the record's length, which is
    /// never zero because a reader refuses a record with no bases (D008).
    pub(crate) fn span(&self, size: u64) -> String {
        format!(
            "{}..{}",
            self.start + 1,
            self.start.saturating_add(self.length.saturating_sub(1)) % size + 1
        )
    }
}

/// The hits of one search, as the screen holds them and `n` and `N` walk through
/// them: the hits shown — at most `MAX_HITS`, in core's order (D054) — how many
/// core returned, and which of the shown ones is current (§4 rules 4 and 5).
#[derive(Clone, Debug, PartialEq)]
pub struct Hits {
    shown: Vec<Hit>,
    total: usize,
    current: usize,
}

impl Hits {
    pub fn shown(&self) -> &[Hit] {
        self.shown.as_slice()
    }

    pub fn total(&self) -> usize {
        self.total
    }

    /// The index into `shown` that is current (§4 rule 4).
    pub fn current(&self) -> usize {
        self.current
    }

    /// The current hit, which `state` parks the cursor on. `Hits` is never built
    /// without at least one hit — core finding nothing is `Outcome::Empty` — so
    /// `shown` is never empty here.
    pub(crate) fn current_hit(&self) -> &Hit {
        &self.shown[self.current]
    }

    /// `n` and `N`: the next or previous shown hit, wrapping inside what is shown
    /// and never past it (§4 rule 5, M8-12). Returns the hit now current.
    pub(crate) fn step(&mut self, forward: bool) -> &Hit {
        debug_assert!(
            !self.shown.is_empty(),
            "hits are never held without at least one"
        );
        let last: usize = self.shown.len() - 1;
        self.current = match (forward, self.current) {
            (true, at) if at == last => 0,
            (true, at) => at + 1,
            (false, 0) => last,
            (false, at) => at - 1,
        };
        &self.shown[self.current]
    }
}

/// What `enter` at the prompt answers (§4 rule 3): core's message when it refused,
/// that it found nothing, or the hits it found.
pub(crate) enum Outcome {
    Refused(String),
    Empty,
    Found(Hits),
}

/// Search `record` for `text` in `mode`: `find_dna`, or `find_protein` over the
/// selected CDS (`cds`, already screened for `CdsStatus::Usable` by `state`) or
/// over the six frames when none is selected (M8-11). `cursor` picks which hit is
/// current (§4 rule 4).
///
/// The sequence searched is the record's own, except when the view has switched an
/// assumed topology to circular (D049 M8-3): then, exactly as `luria find
/// --circular` does (find.rs §4.4), the same bases are put through
/// `sequence::normalize` as a circular sequence, so a match across the origin is
/// found. A record whose file states its topology needs no copy — core reads that
/// topology and wraps on its own (D049).
pub(crate) fn search_record(
    record: &ViewRecord,
    cds: Option<&Cds>,
    circular: bool,
    mode: SearchMode,
    text: &str,
    cursor: u64,
) -> Outcome {
    let searched: Sequence = if circular && record.topology == TopologySource::Assumed {
        // The letters came from a `Sequence` already, so they are canonical and
        // `normalize` cannot refuse them; if it ever did, its message is what the
        // user would see (find.rs' same comment).
        match normalize(
            record.sequence.name(),
            record.sequence.canonical_text(),
            SequenceTopology::Circular,
        ) {
            Ok(copy) => copy,
            Err(error) => return Outcome::Refused(error.message()),
        }
    } else {
        record.sequence.clone()
    };

    let found: Vec<Hit> = match mode {
        SearchMode::Dna => match find_dna(&searched, text) {
            Ok(rows) => rows
                .iter()
                .map(|row| Hit {
                    start: row.start,
                    length: row.length,
                    label: row.strand.symbol(),
                    strand: row.strand,
                })
                .collect(),
            Err(error) => return Outcome::Refused(error.message()),
        },
        SearchMode::Aa => {
            let target: ProteinTarget = match cds {
                Some(cds) => ProteinTarget::Cds(cds),
                None => ProteinTarget::SixFrames,
            };
            match find_protein(&searched, target, text) {
                Ok(rows) => rows
                    .iter()
                    .map(|row| Hit {
                        start: row.start,
                        length: row.length,
                        label: protein_label(row),
                        strand: protein_strand(&row.strand),
                    })
                    .collect(),
                Err(error) => return Outcome::Refused(error.message()),
            }
        }
    };

    if found.is_empty() {
        return Outcome::Empty;
    }
    let total: usize = found.len();
    // The first `MAX_HITS` of core's hits, in core's order (D054): the cap says
    // nothing about which hits are kept beyond that they are the leftmost (M8-12).
    let shown: Vec<Hit> = found.into_iter().take(MAX_HITS).collect();
    // The current hit is the first at or after the cursor the search started on,
    // and the first shown one when none lies that far right (§4 rule 4).
    let current: usize = shown
        .iter()
        .position(|hit| hit.start >= cursor)
        .unwrap_or(0);
    Outcome::Found(Hits {
        shown,
        total,
        current,
    })
}

/// The strand a protein hit's bases are marked on (M8-24): the strand its target
/// — the selected CDS, or the frame it was read in — lies on. `Strand` has no
/// third state, so a protein hit is never `Both`.
fn protein_strand(strand: &Strand) -> HitStrand {
    match strand {
        Strand::Plus => HitStrand::Plus,
        Strand::Minus => HitStrand::Minus,
    }
}

/// What a protein hit is named by (§4 rule 4): the frame it was read in — `+2` —
/// in six-frame mode, and in a CDS the numbers of its first and last residue. The
/// terminal stop codon has no residue number (D010, D054), so a CDS hit that ends
/// on one is named by its first residue and the words `to the stop codon` —
/// `284 to the stop codon` (M8-17, D050) — and a hit whose codons have none at
/// all, the pattern `*` matching a CDS's stop alone, falls back to the strand its
/// codons are read on.
fn protein_label(hit: &ProteinHit) -> String {
    match hit.frame {
        Some(frame) => format!("{}{}", hit.strand.clone().symbol(), frame.index()),
        None => match (hit.first, hit.last) {
            (Some(first), Some(last)) if first != last => format!("{first}..{last}"),
            // D050's M8-17: it is the stop codon that ends the run, and the stop
            // has no residue number, so the label names where the run ends.
            (Some(first), None) => format!("{first} to the stop codon"),
            (Some(first), Some(_)) => first.to_string(),
            (None, Some(last)) => last.to_string(),
            (None, None) => hit.strand.clone().symbol(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::auto_mode;
    use super::protein_label;
    use super::Hit;
    use super::HitStrand;
    use super::ProteinHit;
    use super::SearchMode;
    use super::Strand;

    /// §4 rule 2 and plan §5.4: `Aa` only once a character that cannot be a
    /// nucleotide letter appears, either case (contract 8l1 §5).
    #[test]
    fn the_mode_follows_the_letters() {
        assert_eq!(auto_mode(""), SearchMode::Dna);
        assert_eq!(auto_mode("gaattc"), SearchMode::Dna);
        // Every letter of these is an IUPAC code, so they read as DNA.
        assert_eq!(auto_mode("RATY"), SearchMode::Dna);
        assert_eq!(auto_mode("MKSNNA"), SearchMode::Dna);
        assert_eq!(auto_mode("E"), SearchMode::Aa);
        assert_eq!(auto_mode("GAE"), SearchMode::Aa);
        assert_eq!(auto_mode("A1"), SearchMode::Aa);
        assert_eq!(auto_mode("*"), SearchMode::Aa);
    }

    /// §4 rule 8 and D052: a hit whose run passes the origin covers both runs, and
    /// a hit that does not cover nothing outside its own span. pBR322's EcoRI site
    /// sits at 4358 with 6 bases on a 4361 bp record, so it covers 4358..4361 and
    /// 0..3.
    #[test]
    fn a_hit_across_the_origin_covers_both_runs() {
        let wraps: Hit = Hit {
            start: 4358,
            length: 6,
            label: "both".to_string(),
            strand: HitStrand::Both,
        };
        let inside: Hit = Hit {
            start: 58,
            length: 4,
            label: "both".to_string(),
            strand: HitStrand::Both,
        };
        for base in [4358, 4359, 4360, 0, 1, 2] {
            assert!(wraps.covers(base, 4361), "{base} is covered");
        }
        for base in [3, 57, 58, 61, 4357] {
            assert!(!wraps.covers(base, 4361), "{base} is not covered");
        }
        for base in [58, 59, 60, 61] {
            assert!(inside.covers(base, 4361), "{base} is covered");
        }
        for base in [57, 62, 0, 4360] {
            assert!(!inside.covers(base, 4361), "{base} is not covered");
        }
    }

    /// D050's M8-17 and contract 8r1 item 3: a CDS hit that ends on the stop
    /// codon, which has no residue number, says so in its label.
    #[test]
    fn a_cds_hit_ending_on_the_stop_says_so() {
        let hit: ProteinHit = ProteinHit {
            // Ten codons in CDS mode, so no frame: residues 284 to 293, and the
            // codon after them is the stop, which `last` cannot number.
            start: 849,
            length: 30,
            strand: Strand::Plus,
            frame: None,
            first: Some(284),
            last: None,
            wraps_origin: false,
        };
        assert_eq!(protein_label(&hit), "284 to the stop codon");
    }
}
