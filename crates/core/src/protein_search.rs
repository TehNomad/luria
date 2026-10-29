//! Protein search: an amino-acid pattern in one CDS's translation or in the six
//! frames of a record (plan §4.5; D054, D050 M8-16 and M8-17, D052 M8-2, D010,
//! D036, D006).
//!
//! The rules, as §4.5 and the decisions give them:
//!
//! * **The pattern is ASCII-folded, then validated.** `A` and `a` are the same
//!   search; the legal letters are the 20 residue letters, `X` and `*`. An
//!   empty pattern is `DomainError::EmptySearchPattern` and a letter outside
//!   that alphabet — `B`, `J`, `O`, `U`, `Z`, a digit, a space — is
//!   `DomainError::InvalidSearchPatternCharacters`, naming the pattern as
//!   written, the letter as written and its zero-based position. Those are the
//!   only refusals this module adds, and they are 8e's two search refusals:
//!   one rule, one code (`find_dna` builds them the same way).
//! * **`X` matches every symbol of the target, `*` and `?` included** (D054).
//!   `*` matches only `*`; every other pattern letter matches only itself. That
//!   is why `?`, the unreadable codon, is matched by `X` and by nothing else,
//!   and why `G` finds nothing where the codon is `GGN` even though every
//!   reading of `GGN` is glycine (D010 refuses the guess, D052 makes `N` always
//!   `?`).
//! * **Two targets, one matcher.** `ProteinTarget::Cds` reads the CDS's own
//!   codons — `translate`'s residues, with `*` added when `terminal_stop()` —
//!   and `ProteinTarget::SixFrames` reads `frames::frame_residues` over the
//!   whole record, three frames on each strand. The matching, the coordinates
//!   and the order below are the same for both; only where the symbols and the
//!   numbers come from differs. The six frame reads are built one at a time and
//!   dropped as soon as their hits are kept, because a frame's codons are as long
//!   as the record — a rule of size, not of speed (see `frame_read`).
//! * **Coordinates are zero-based and half-open** (D006): `start` is the first
//!   base, in reference order, of the run of bases the hit's codons cover, and
//!   `length` is three times the pattern's length. A hit's run is read forward
//!   from `start`, so on the minus strand it begins at the **last codon the
//!   reading reaches**: the reading runs to decreasing coordinates, and its run
//!   still starts at the low end (`m1`'s `QQ` is residues 2–3 over bases
//!   `[15,21)`). `wraps_origin` is computed as `start + length > length of the
//!   record`, never inferred.
//! * **Only a CDS that crosses the origin can wrap** (D036). `pj` is
//!   `join(55..60,1..12)`, so codon 1 sits at zero-based 54 and a hit that
//!   starts there and runs for 18 bases crosses the origin; a frame never
//!   crosses it, because a frame's codons are complete codons of the linear
//!   coordinate line and a codon that would wrap is not translated at all
//!   (D052 M8-2). Frames are read with `[0, length())` as their window and
//!   `sequence.topology()` is never consulted here, so a circular record is
//!   searched exactly as a linear one is.
//! * **Whose numbers a hit carries** (D050 M8-17, D010). In CDS mode they are
//!   the CDS's residue numbers, and the terminal stop codon carries none — a
//!   hit running onto the stop has `last == None`, which is why `first` and
//!   `last` are optional at all. In frame mode they count codons within that
//!   frame, from the frame's own first complete codon, and are never `None`;
//!   `strand` and `frame` together say which scale they are on.
//! * **Every match, no cap** (D050 M8-16). Every offset at which the whole
//!   pattern fits is tried and every overlapping hit is reported: `KK` on twelve
//!   `A`s is seven hits, not two. A pattern longer than the target finds
//!   nothing, which is not an error. There is no cap in core; the 200-hit limit
//!   is presentation (D048).
//! * **Order** (D054): by `start` ascending in both modes, plus strand before
//!   minus on a tie. In CDS mode that is *not* reading order — on `pj`, `M` at
//!   residue 3 (base 0) comes before `M` at residue 1 (base 54), and a hit that
//!   crosses the origin sorts by its start, near the end of the record.
//!
//! A symbol a target could hold that rule 2 does not answer would be one that
//! is neither a residue letter, nor `?`, nor `*` — and of a pattern letter that
//! is not a residue, `X` or `*`. Within §4.5's two targets there is none: a
//! CDS's residues come from `genetic_code::lookup`, which returns only the 20
//! letters (an unreadable codon is refused there, D010), and a frame's symbols
//! are those 20 plus `*` and `?` (`frame::codon_symbol`). No target holds a
//! literal `X`, so `X` is only ever a pattern letter here.
//!
//! `tests/fixtures/view/search-protein-patterns.tsv` and
//! `search-protein-hits.tsv` are the answers over `synthetic`, pBR322 and
//! `framesamb`, produced by `tests/fixtures/view/reference/search_protein.py`,
//! which shares no code with this module (D039);
//! `crates/core/tests/protein_search_fixtures.rs` compares them. Biopython is
//! that script's oracle for the 64-codon table and nothing else — the README
//! beside the tables records the limits (D052, D010).

use crate::cds::Cds;
use crate::error::DomainError;
use crate::error::InvalidSearchPatternCharactersProblem;
use crate::frames::frame_residues;
use crate::frames::ReadingFrame;
use crate::mutation::codon_positions;
use crate::sequence::Sequence;
use crate::strand::Strand;
use crate::translation::translate;

/// The alphabet a protein pattern is built from: the 20 one-letter residues of
/// the standard code (`genetic_code::STANDARD_GENETIC_CODE` holds exactly
/// these, with `*` for its three stops), `X` for "any symbol of the target"
/// (D054) and `*` for a stop codon. `B`, `J`, `O`, `U` and `Z` are deliberately
/// absent: they stand for nothing in the code this milestone translates (D010),
/// and `U` is a base, not a residue — the protein alphabet has no `U` → `T`
/// fold either, since a protein pattern is letters of residues, not of bases.
const PROTEIN_ALPHABET: &str = "ACDEFGHIKLMNPQRSTVWYX*";

/// What a protein search reads (§4.5).
#[derive(Clone, Copy, Debug)]
pub enum ProteinTarget<'a> {
    /// One CDS's translation, with its terminal stop as `*`.
    Cds(&'a Cds),
    /// The six frames of the whole record, as `frame_residues` reads them.
    SixFrames,
}

/// One place a pattern matched: zero-based, half-open (D006). The hit knows both
/// scales — the bases its codons cover, and the residue or codon numbers of the
/// target it was read from (D050 M8-17) — so the view can highlight the codons
/// and name the residues without re-deriving either.
#[derive(Clone, Debug, PartialEq)]
pub struct ProteinHit {
    /// Zero-based first base, in reference order, of the run of bases the hit's
    /// codons cover (D006). On a CDS that crosses the origin the run may wrap.
    pub start: u64,
    /// 3 × the pattern's length.
    pub length: u64,
    pub strand: Strand,
    /// `None` in CDS mode.
    pub frame: Option<ReadingFrame>,
    /// The first and last codons' numbers. CDS mode: the residue number, `None`
    /// for the stop codon (D010, D050). Frame mode: the codon's number within
    /// its frame, never `None`.
    pub first: Option<u64>,
    pub last: Option<u64>,
    /// `start + length > sequence.length()`.
    pub wraps_origin: bool,
}

/// One codon of a target, as the matcher sees it: its symbol, the number the hit
/// reports for it, and the two ends of its three bases **in reading order** —
/// ascending along the strand on `+`, descending on `−`, which is D040's fact
/// that reading order and reference order are two orders over the same bases.
struct TargetCodon {
    symbol: char,
    /// `None` only for a CDS's terminal stop codon: a codon, not a residue
    /// (D010). A frame codon always has a number (D050 M8-17).
    number: Option<u64>,
    /// The codon's first base as the target reads it.
    reading_first: u64,
    /// The codon's last base as the target reads it: the lowest of its three
    /// bases on `−`, where the reading runs to decreasing coordinates.
    reading_last: u64,
}

/// One searchable stretch of symbols — one CDS, or one frame of one strand — and
/// how to report a hit read from it.
struct TargetRead {
    strand: Strand,
    /// `None` in CDS mode.
    frame: Option<ReadingFrame>,
    /// The codons in reading order: the CDS's codon 1 first, or the frame's
    /// first complete codon first.
    codons: Vec<TargetCodon>,
}

/// Rule 1: ASCII case is dropped, then the pattern is validated, in one pass
/// that records the position in the pattern **as written** — the fold is one
/// character to one character, so an index into the folded pattern is still an
/// index into the pattern as given. An empty pattern is refused before anything
/// is read, so `""` is refused in both modes and over a CDS that could not be
/// translated at all: a missing pattern is the caller's problem, not the CDS's.
fn fold_pattern(pattern: &str) -> Result<Vec<char>, DomainError> {
    let written: Vec<char> = pattern.chars().collect();
    if written.is_empty() {
        return Err(DomainError::EmptySearchPattern);
    }
    for (position, as_written) in written.iter().enumerate() {
        let letter: char = as_written.to_ascii_uppercase();
        if !PROTEIN_ALPHABET.contains(letter) {
            return Err(DomainError::InvalidSearchPatternCharacters(
                InvalidSearchPatternCharactersProblem {
                    pattern: pattern.to_string(),
                    position: position as u64,
                    characters: as_written.to_string(),
                },
            ));
        }
    }
    Ok(written
        .iter()
        .map(|letter| letter.to_ascii_uppercase())
        .collect())
}

/// Rule 2 (D054): `X` matches every symbol of the target — a residue, the stop
/// `*`, and the unreadable codon `?` — and every other pattern letter matches
/// only itself. So `*` is matched by `*` and `X` alone, and `?` by `X` alone.
fn symbol_matches(pattern: char, symbol: char) -> bool {
    pattern == 'X' || pattern == symbol
}

/// The one CDS target: `translate`'s residues, plus `*` when the CDS ends on a
/// stop codon, with every codon's bases taken from `mutation::codon_positions`
/// (which already knows about joined and origin-crossing CDSs — no codon
/// arithmetic is repeated here, D036). `translate`'s own refusals are returned
/// as they are: an incomplete CDS, an unusable numbering, a length that is not a
/// whole number of codons, an internal stop and an unreadable codon are refused
/// before any pattern is matched, and §4.5 gives protein search no opinion on
/// them.
fn cds_read(sequence: &Sequence, cds: &Cds) -> Result<TargetRead, DomainError> {
    let translation = translate(sequence, cds)?;
    let mut symbols: Vec<char> = translation.residues_text().chars().collect();
    // The terminal stop is a codon of the target and matches `*` and `X` (D054);
    // it carries no residue number (D010), which is what `number` below records.
    let stops: u64 = if translation.terminal_stop() {
        symbols.push('*');
        1
    } else {
        0
    };
    let codons: u64 = symbols.len() as u64;
    // `translate` refuses a CDS whose length is not a multiple of three, so its
    // codons are exactly its residues plus the terminal stop, if any. This is
    // not a new rule: it is the same arithmetic D010 and `cds_map` use, stated
    // so a mismatch is a bug in this module rather than a silent short read.
    debug_assert_eq!(
        codons,
        cds.total_length() / 3,
        "residues + stop vs total_length / 3 for {}",
        cds.id().value()
    );
    debug_assert_eq!(stops + translation.residue_count() as u64, codons);

    let strand: Strand = cds.strand();
    let intervals = cds.intervals();
    let total_length: u64 = cds.total_length();
    let mut read: TargetRead = TargetRead {
        strand: strand.clone(),
        frame: None,
        codons: Vec::new(),
    };
    for (index, symbol) in symbols.iter().enumerate() {
        // Codon `index + 1` in reading order, the stop codon included:
        // `codon_positions` maps it through the CDS's intervals, so the
        // origin-crossing arc of `pj` needs no special case here.
        let positions: Vec<u64> = codon_positions(
            intervals.clone(),
            strand.clone(),
            index as u64 + 1,
            total_length,
        )?;
        // `codon_positions` builds exactly three positions or refuses, so a list
        // of any other length is a bug in that walk rather than an answer. The
        // order it returns is the reading order, which `cds_map` documents as
        // descending on a minus strand.
        debug_assert_eq!(
            positions.len(),
            3,
            "codon {} of {}",
            index + 1,
            cds.id().value()
        );
        // Every residue is numbered by its own codon, and the terminal stop is
        // the last codon when it exists and carries no number (D010, D050 M8-17).
        let is_stop: bool = stops == 1 && index as u64 == codons - 1;
        let number: Option<u64> = if is_stop {
            None
        } else {
            Some(index as u64 + 1)
        };
        read.codons.push(TargetCodon {
            symbol: *symbol,
            number,
            reading_first: positions[0],
            reading_last: positions[2],
        });
    }
    Ok(read)
}

/// One frame target: one strand and one frame, read over the whole record with
/// the window `[0, length())`. `frame_residues` does not stop at `*` or `?`, does
/// not draw a leftover base, and never runs across the origin (D052 M8-2,
/// D050 M8-18); `topology()` is never read, which is why a circular `tiny` and a
/// linear one give the same hits.
///
/// A residue's `middle` is the position of its codon's middle base, so the other
/// two bases are `middle - 1` and `middle + 1` and both exist: `frame_residues`
/// emits a residue only when three whole bases stand, and says so of the
/// conversion it makes there.
///
/// One frame, not six: a `TargetCodon` is 40 bytes and a frame holds a third of
/// the record's bases, so the six reads of a 10 Mb record — D048's ceiling —
/// would stand together at some 800 MB, and `find_protein` searches each frame on
/// its own and needs only its hits afterwards. That is why this function is given
/// one strand and one frame, and why the caller drops the read it returns before
/// asking for the next. What is alive at once is this frame's residue list and
/// the codons built from it, which is the floor `frame_residues` sets; the codons
/// are reserved in one go so that growing them cannot double the peak.
fn frame_read(
    sequence: &Sequence,
    strand: Strand,
    frame: ReadingFrame,
) -> Result<TargetRead, DomainError> {
    let residues = frame_residues(sequence, strand.clone(), frame, 0, sequence.length())?;
    let mut read: TargetRead = TargetRead {
        strand: strand.clone(),
        frame: Some(frame),
        codons: Vec::with_capacity(residues.len()),
    };
    for residue in residues.iter() {
        // Reading order on this strand: a minus frame walks to decreasing
        // coordinates, so the codon's first base is the higher one (D040).
        let (reading_first, reading_last) = if strand.clone().is_plus() {
            (residue.middle - 1, residue.middle + 1)
        } else {
            (residue.middle + 1, residue.middle - 1)
        };
        read.codons.push(TargetCodon {
            symbol: residue.symbol,
            number: Some(residue.number),
            reading_first,
            reading_last,
        });
    }
    Ok(read)
}

/// Rule 2 over one read: every codon of the pattern matches the symbol under it.
fn matches_at(read: &TargetRead, pattern: &[char], offset: usize) -> bool {
    for (index, letter) in pattern.iter().enumerate() {
        if !symbol_matches(*letter, read.codons[offset + index].symbol) {
            return false;
        }
    }
    true
}

/// The hit a run of matched codons gives. On `+` the run's bases begin at its
/// first codon's first base; on `−` they begin at its **last** codon's last base
/// in reading order, which is the lowest reference base the run covers, because
/// the reading runs downhill (rules 3 and 4). A frame's `reading_last` is its
/// residue's `middle - 1`, and a CDS's is `codon_positions(..)[2]`.
fn hit_of(read: &TargetRead, sequence: &Sequence, run: &[TargetCodon], length: u64) -> ProteinHit {
    let first: &TargetCodon = &run[0];
    let last: &TargetCodon = &run[run.len() - 1];
    let start: u64 = if read.strand.clone().is_plus() {
        first.reading_first
    } else {
        last.reading_last
    };
    ProteinHit {
        start,
        length,
        strand: read.strand.clone(),
        frame: read.frame,
        first: first.number,
        last: last.number,
        // Computed from the coordinates, never inferred (D006). A frame's codons
        // are all inside the record, so this is false there without being
        // declared so (D052 M8-2); only an origin-crossing CDS makes it true.
        wraps_origin: start.saturating_add(length) > sequence.length(),
    }
}

/// Every hit of `pattern` in one target read, in the order the read reaches
/// them. The scan steps **one codon**, never the width of the pattern, so every
/// overlapping match is found (D050 M8-16); a pattern longer than the read has
/// no offset at which the whole pattern fits, and finds nothing (§4.5).
fn hits_in(read: &TargetRead, sequence: &Sequence, pattern: &[char]) -> Vec<ProteinHit> {
    let width: u64 = pattern.len() as u64;
    let mut hits: Vec<ProteinHit> = Vec::new();
    if (read.codons.len() as u64) < width {
        return hits;
    }
    let mut offset: usize = 0;
    while offset + pattern.len() <= read.codons.len() {
        if matches_at(read, pattern, offset) {
            hits.push(hit_of(
                read,
                sequence,
                &read.codons[offset..offset + pattern.len()],
                width * 3,
            ));
        }
        offset += 1;
    }
    hits
}

/// Every place `pattern` matches `target` on `sequence`, in the order D054 gives.
///
/// * An empty pattern is `DomainError::EmptySearchPattern`; a letter outside the
///   protein alphabet is `DomainError::InvalidSearchPatternCharacters`, naming
///   the pattern and the letter as written and the zero-based position. Both are
///   8e's refusals; this module adds no error variant of its own.
/// * `ProteinTarget::Cds` refuses what `translate` refuses, with `translate`'s
///   own errors, and `ProteinTarget::SixFrames` what `frame_residues` refuses.
/// * A pattern longer than the target, and a pattern that matches nowhere, give
///   an empty vector rather than an error. There is no cap, and every
///   overlapping hit is kept.
/// * Hits are ordered by `start` ascending, plus strand before minus on a tie —
///   in both modes, which in CDS mode is not reading order (D054).
pub fn find_protein(
    sequence: &Sequence,
    target: ProteinTarget<'_>,
    pattern: &str,
) -> Result<Vec<ProteinHit>, DomainError> {
    // The pattern is folded and validated before any target is read: a bad
    // pattern is the caller's error even when the target could not be read at
    // all, and the fold happens here rather than in the matcher so the refusal
    // can name the letter as the user wrote it.
    let folded: Vec<char> = fold_pattern(pattern)?;
    let width: u64 = folded.len() as u64 * 3;
    let mut hits: Vec<ProteinHit> = Vec::new();
    // One target read at a time, never all six frames at once: each frame is
    // searched on its own, and only the hits it yields have to outlive it (see
    // `frame_read` for the size that makes this a rule and not a refinement).
    // The visit order is `Plus` frame 1, 2, 3 then `Minus` frame 1, 2, 3, as
    // before; the sort below, not the order the reads were built in, is what
    // gives D054's order, so building them one at a time changes no result.
    match target {
        // One CDS is one read, and it was always built alone: its codons are its
        // own length, which `translate` has already refused when it is unusable.
        ProteinTarget::Cds(cds) => {
            let read: TargetRead = cds_read(sequence, cds)?;
            hits.extend(hits_in(&read, sequence, &folded));
        }
        ProteinTarget::SixFrames => {
            for strand in [Strand::Plus, Strand::Minus] {
                for frame in ReadingFrame::ALL {
                    let read: TargetRead = frame_read(sequence, strand.clone(), frame)?;
                    hits.extend(hits_in(&read, sequence, &folded));
                }
            }
        }
    }
    // D054: left to right by base, in both modes, and a six-frame tie is decided
    // plus strand first. `start` first, then `−` after `+`. This is not reading
    // order for a CDS: a minus-strand CDS lists its later residues first, and a
    // hit whose run crosses the origin sorts by its start, near the end of the
    // record (`pj`'s `M`: residue 3 at base 0 before residue 1 at base 54).
    // No two hits share a start *and* a strand — a CDS's codons are disjoint,
    // and one strand's three frames begin on bases that differ mod 3 — so this
    // tie-break decides only between strands, and `sort_by_key`'s stability
    // leaves the frames of one strand in the order they were read.
    hits.sort_by_key(|hit| (hit.start, hit.strand.clone() == Strand::Minus));
    debug_assert!(hits.iter().all(|hit| hit.length == width));
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genbank::read_genbank;
    use crate::genbank::GenBankRecord;
    use crate::genbank_cds::import_cds;
    use crate::genbank_cds::GenBankImport;
    use crate::identity::Identifier;
    use crate::sequence::normalize;
    use crate::sequence::SequenceTopology;

    /// A record made in the test: text plus the topology its caller states
    /// (D049). `tiny` and `k` below are FASTA bodies in the contract, and a
    /// FASTA record states no topology, so the topology is exactly what the
    /// caller gives it here — which is what test 11 changes.
    fn dna(text: &str, topology: SequenceTopology) -> Sequence {
        let name: Identifier = Identifier::create("unit".to_string())
            .unwrap_or_else(|error| panic!("cannot name a test record: {:?}", error));
        normalize(name, text.to_string(), topology)
            .unwrap_or_else(|error| panic!("{:?} cannot be stored as DNA: {:?}", text, error))
    }

    /// The record `tests/fixtures/genbank/synthetic.gb`, read the way the view
    /// reads it. Its usable CDSs `p1`, `m1` and `pj` are the ones
    /// `cds-map-counts.tsv` lists (D041), and the protein tables name the same
    /// three. The path is relative to the package root, where cargo runs unit
    /// tests from.
    fn synthetic() -> GenBankRecord {
        let path: &str = "../../tests/fixtures/genbank/synthetic.gb";
        let text: String = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
        let name: Identifier = Identifier::create("synthetic.gb".to_string())
            .unwrap_or_else(|error| panic!("cannot name the synthetic record: {:?}", error));
        let record: GenBankRecord = read_genbank(name, text)
            .unwrap_or_else(|error| panic!("read_genbank refused {}: {:?}", path, error.message()));
        record
    }

    /// The CDS a fixture id names, imported the way the view imports it
    /// (`import_cds`), so its intervals and strand are core's own reading of
    /// the file rather than coordinates written into this test.
    fn cds_of(record: &GenBankRecord, id: &str) -> Cds {
        let wanted: Identifier = Identifier::create(id.to_string())
            .unwrap_or_else(|error| panic!("cannot ask for {:?}: {:?}", id, error));
        let import: GenBankImport = import_cds(record, &[wanted])
            .unwrap_or_else(|error| panic!("import_cds refused {:?}: {:?}", id, error.message()));
        assert!(
            import.cds().len() == 1,
            "importing {:?} gives {} CDSs, expected 1",
            id,
            import.cds().len()
        );
        import.cds()[0].cds()
    }

    /// `pj` and the record it belongs to: `join(55..60,1..12)`, plus strand,
    /// 18 bases, `MKMAW*`, codons 1..6 beginning at zero-based 54, 57, 0, 3, 6,
    /// 9 — the CDS D036 lets cross the origin, and 54 is the only start in
    /// these fixtures whose run can wrap.
    fn pj(record: &GenBankRecord) -> Cds {
        cds_of(record, "pj")
    }

    /// `m1` and its record: `complement(13..24)`, minus strand, `MQQ*`, codon 1
    /// at 23, 22, 21 and codon 4 (the stop) at 14, 13, 12.
    fn m1(record: &GenBankRecord) -> Cds {
        cds_of(record, "m1")
    }

    /// A frame number as `search-protein-hits.tsv` writes it, `0` for the `-`
    /// a CDS hit carries: no frame.
    const NO_FRAME: u64 = 0;

    /// One expected hit, written out by hand in the order §5 writes one:
    /// (start, length, strand, frame, first, last, wraps). `first` and `last`
    /// are `None` where the codon is a CDS's stop codon, which carries no
    /// residue number (D010, D050 M8-17).
    fn want(
        start: u64,
        length: u64,
        strand: Strand,
        frame: u64,
        first: Option<u64>,
        last: Option<u64>,
        wraps: bool,
    ) -> ProteinHit {
        let read: Option<ReadingFrame> = ReadingFrame::from_index(frame);
        assert!(
            (frame == NO_FRAME) == read.is_none(),
            "{} is neither 0 (no frame) nor 1, 2 or 3: a test wrote a frame this test does not mean",
            frame
        );
        ProteinHit {
            start,
            length,
            strand,
            frame: read,
            first,
            last,
            wraps_origin: wraps,
        }
    }

    /// A number column as the tables write it: `stop` for `None`.
    fn number(value: Option<u64>) -> String {
        match value {
            Some(seen) => seen.to_string(),
            None => "stop".to_string(),
        }
    }

    /// One hit as one short line, so a failure says which field of which hit
    /// differs, and a count is readable. The shape is the tables': strand, then
    /// frame (`-` for a CDS hit), then the base run, then the two numbers.
    fn row(hit: &ProteinHit) -> String {
        format!(
            "{}{}@{}+{}#{}-{}{}",
            hit.strand.clone().symbol(),
            match hit.frame {
                Some(frame) => frame.index().to_string(),
                None => "-".to_string(),
            },
            hit.start,
            hit.length,
            number(hit.first),
            number(hit.last),
            if hit.wraps_origin { " wraps" } else { "" }
        )
    }

    fn rows(hits: &[ProteinHit]) -> Vec<String> {
        hits.iter().map(row).collect()
    }

    /// The hits of a pattern core is expected to search, not refuse.
    fn hits_of(sequence: &Sequence, target: ProteinTarget<'_>, pattern: &str) -> Vec<ProteinHit> {
        match find_protein(sequence, target, pattern) {
            Ok(hits) => hits,
            Err(error) => panic!(
                "find_protein refused {:?} ({}: {}), and the test expects it to be searched",
                pattern,
                error.code(),
                error.message()
            ),
        }
    }

    /// The hits of a pattern that must match nothing — while still being
    /// searched. "Finds nothing" and "is refused" are different answers, and
    /// collapsing them is how a missing rule hides.
    fn finds_nothing(sequence: &Sequence, target: ProteinTarget<'_>, pattern: &str) {
        match find_protein(sequence, target, pattern) {
            Ok(hits) => assert!(
                hits.is_empty(),
                "{:?} gives {} hits ({:?}), expected none",
                pattern,
                hits.len(),
                rows(&hits)
            ),
            Err(error) => panic!(
                "{:?} is refused ({}: {}) where §4.5 answers it with no hits",
                pattern,
                error.code(),
                error.message()
            ),
        }
    }

    /// The refusal of a pattern that must be refused.
    fn refused(sequence: &Sequence, target: ProteinTarget<'_>, pattern: &str) -> DomainError {
        let outcome: Result<Vec<ProteinHit>, DomainError> = find_protein(sequence, target, pattern);
        match outcome {
            Ok(hits) => panic!(
                "{:?} gives {} hits ({:?}); §4.5 refuses it",
                pattern,
                hits.len(),
                rows(&hits)
            ),
            Err(error) => error,
        }
    }

    /// The `+1` hits of a pattern over a record's six frames, as short rows. A
    /// frame-mode hit always carries strand and frame (D050 M8-17), so this
    /// selects one frame's answers and nothing else.
    fn plus_first_hits(sequence: &Sequence, pattern: &str) -> Vec<String> {
        hits_of(sequence, ProteinTarget::SixFrames, pattern)
            .iter()
            .filter(|hit| {
                hit.strand.clone() == Strand::Plus && hit.frame == Some(ReadingFrame::First)
            })
            .map(row)
            .collect()
    }

    /// Rule 1: an empty pattern is refused, in both modes — and refused before
    /// the target is read, so the refusal is about the pattern even over frames
    /// and over a real CDS.
    #[test]
    fn empty_pattern_is_refused() {
        let tiny: Sequence = dna("ATGAAATAG", SequenceTopology::Linear);
        let error: DomainError = refused(&tiny, ProteinTarget::SixFrames, "");
        assert!(
            error == DomainError::EmptySearchPattern,
            "an empty pattern over six frames is refused {:?}, expected EmptySearchPattern",
            error
        );
        let record: GenBankRecord = synthetic();
        let sequence: Sequence = record.sequence();
        let joined: Cds = pj(&record);
        let other: DomainError = refused(&sequence, ProteinTarget::Cds(&joined), "");
        assert!(
            other == DomainError::EmptySearchPattern,
            "an empty pattern over the CDS `pj` is refused {:?}, expected EmptySearchPattern",
            other
        );
    }

    /// Rule 1: a letter outside the protein alphabet is named where it sits,
    /// as written. `B`, `J`, `O`, `U` and `Z` stand for nothing in the code this
    /// milestone translates, and a protein pattern has no `U` → `T` fold: `U` is
    /// refused even though `find_dna` would read it as `T`.
    #[test]
    fn a_letter_outside_the_protein_alphabet_names_itself() {
        let record: GenBankRecord = synthetic();
        let sequence: Sequence = record.sequence();
        let joined: Cds = pj(&record);
        let target: ProteinTarget<'_> = ProteinTarget::Cds(&joined);

        for (pattern, position, letter) in [
            ("MbW", 1u64, "b"),
            ("MAU", 2, "U"),
            ("M W", 1, " "),
            ("MZ", 1, "Z"),
            ("M2", 1, "2"),
        ] {
            let error: DomainError = refused(&sequence, target, pattern);
            match &error {
                DomainError::InvalidSearchPatternCharacters(problem) => {
                    assert!(
                        problem.pattern == pattern,
                        "the refusal names pattern {:?}, expected {:?} as written",
                        problem.pattern,
                        pattern
                    );
                    assert!(
                        problem.position == position,
                        "the refusal points at zero-based {}, expected {} where {:?} sits: {:?}",
                        problem.position,
                        position,
                        pattern,
                        problem
                    );
                    assert!(
                        problem.characters == letter,
                        "the refusal names {:?}, expected {:?} as written",
                        problem.characters,
                        letter
                    );
                }
                other => panic!(
                    "the refusal for {:?} is {:?} ({}), expected InvalidSearchPatternCharacters",
                    pattern,
                    other,
                    other.code()
                ),
            }
            assert!(
                error.code() == "invalid_search_pattern_characters",
                "the code is {}, expected invalid_search_pattern_characters",
                error.code()
            );
            let message: String = error.message();
            assert!(
                message.contains(letter),
                "the message {:?} does not name the offending letter {:?}, which §4.5 requires",
                message,
                letter
            );
            assert!(
                message.contains(pattern),
                "the message {:?} does not name the pattern it refused",
                message
            );
        }

        // The fold: `mkm` is the pattern `MKM`. Without case folding this is a
        // refusal, which is what the fold test is about.
        let upper: Vec<String> = rows(&hits_of(&sequence, target, "MKM"));
        let lower: Vec<String> = rows(&hits_of(&sequence, target, "mkm"));
        assert!(
            upper == vec!["+-@54+9#1-3 wraps".to_string()],
            "MKM on `pj` gives {:?}, expected the one wrapping run at 54",
            upper
        );
        assert!(
            lower == upper,
            "mkm gives {:?}, the same hits MKM gives {:?}",
            lower,
            upper
        );
    }

    /// Rules 2 and 6 (D054): `X` matches every symbol, the stop included, and
    /// the hits come left to right by base, plus strand first on a tie. `tiny`
    /// is 9 bp: `+1` `MK*`, `+2` `*N`, `+3` `EI`, `−1` `LFH`, `−2` `YF`, `−3`
    /// `IS`, so 3+2+2+3+2+2 = 14 codons and 14 hits, two sharing each of the
    /// starts 0..=6.
    #[test]
    fn x_matches_every_symbol_the_stop_included() {
        let tiny: Sequence = dna("ATGAAATAG", SequenceTopology::Linear);
        let found: Vec<ProteinHit> = hits_of(&tiny, ProteinTarget::SixFrames, "X");
        let expected: Vec<ProteinHit> = vec![
            want(0, 3, Strand::Plus, 1, Some(1), Some(1), false),
            want(0, 3, Strand::Minus, 1, Some(3), Some(3), false),
            want(1, 3, Strand::Plus, 2, Some(1), Some(1), false),
            want(1, 3, Strand::Minus, 3, Some(2), Some(2), false),
            want(2, 3, Strand::Plus, 3, Some(1), Some(1), false),
            want(2, 3, Strand::Minus, 2, Some(2), Some(2), false),
            want(3, 3, Strand::Plus, 1, Some(2), Some(2), false),
            want(3, 3, Strand::Minus, 1, Some(2), Some(2), false),
            want(4, 3, Strand::Plus, 2, Some(2), Some(2), false),
            want(4, 3, Strand::Minus, 3, Some(1), Some(1), false),
            want(5, 3, Strand::Plus, 3, Some(2), Some(2), false),
            want(5, 3, Strand::Minus, 2, Some(1), Some(1), false),
            want(6, 3, Strand::Plus, 1, Some(3), Some(3), false),
            want(6, 3, Strand::Minus, 1, Some(1), Some(1), false),
        ];
        assert!(
            found == expected,
            "X on tiny's six frames gives {} hits ({}), expected the 14 codons of the frame ({})",
            found.len(),
            rows(&found).join(", "),
            rows(&expected).join(", ")
        );
        // `X` is a matching rule only: the stop codon still carries no residue
        // number, and `X` reaches it (D054, D010). `pj`'s hit at base 9 is that
        // codon, and it is `X` that matched it.
        assert!(
            symbol_matches('X', '*') && symbol_matches('X', '?') && symbol_matches('X', 'M'),
            "`X` must match a stop, an unreadable codon and a residue (D054)"
        );
        assert!(
            !symbol_matches('*', '?') && !symbol_matches('M', '?') && !symbol_matches('G', '?'),
            "only `X` matches `?`: §4.5, and `framesamb`'s `**X` row pins the same"
        );
    }

    /// Rule 2, the other half: `?` is a complete codon that cannot be read, and
    /// it matches nothing but `X`. `ATGNNNTAA` reads `+1` as `M ? *`, so `MX*`
    /// and `MXX` cross it and no residue letter can.
    #[test]
    fn an_unreadable_codon_matches_only_x() {
        let ambiguous: Sequence = dna("ATGNNNTAA", SequenceTopology::Linear);
        let cross: Vec<String> = rows(&hits_of(&ambiguous, ProteinTarget::SixFrames, "MX*"));
        assert!(
            cross == vec!["+1@0+9#1-3".to_string()],
            "MX* on ATGNNNTAA gives {:?}, expected the one +1 hit at 0 over 9 bases",
            cross
        );
        let wide: Vec<String> = rows(&hits_of(&ambiguous, ProteinTarget::SixFrames, "MXX"));
        assert!(
            wide == vec!["+1@0+9#1-3".to_string()],
            "MXX on the same record gives {:?}, expected the same hit: `X` matches `?`",
            wide
        );

        // The 20 residue letters, each between `M` and `*`: none of them matches
        // the `?` codon, so no `M{L}*` has a `+1` hit. `M**` is refused nothing
        // and finds no `+1` hit either: `?` is not a stop, even though every
        // reading of `NNN` would be one.
        let letters: Vec<char> = "ACDEFGHIKLMNPQRSTVWY".chars().collect();
        assert!(
            letters.len() == 20,
            "the residue letters are {}, expected the 20 of the standard code",
            letters.iter().collect::<String>()
        );
        for letter in letters.iter() {
            let pattern: String = format!("M{}*", letter);
            let plus_first: Vec<String> = plus_first_hits(&ambiguous, &pattern);
            assert!(
                plus_first.is_empty(),
                "{:?} on ATGNNNTAA gives {:?}, expected no +1 hit: the middle codon is `?`, which only `X` matches",
                pattern,
                plus_first
            );
        }
        let stops: Vec<String> = plus_first_hits(&ambiguous, "M**");
        assert!(
            stops.is_empty(),
            "\"M**\" on ATGNNNTAA gives {:?}, expected no +1 hit: an unreadable codon is not a stop",
            stops
        );
    }

    /// Rule 3 and rule 5: every codon of a CDS that crosses the origin is
    /// searchable, the run is read forward from its start and wraps when it
    /// runs past the last base (D036, D006), and a pattern longer than the
    /// target finds nothing rather than erroring.
    #[test]
    fn every_codon_of_a_wrapping_cds_is_searchable() {
        let record: GenBankRecord = synthetic();
        let sequence: Sequence = record.sequence();
        let joined: Cds = pj(&record);
        let target: ProteinTarget<'_> = ProteinTarget::Cds(&joined);
        assert!(
            sequence.length() == 60 && joined.total_length() == 18,
            "`pj` is {} bases of a {}-base record, expected 18 of 60",
            joined.total_length(),
            sequence.length()
        );
        // Codons 3, 4, 5 sit after the origin, at 0, 3, 6: no wrap.
        assert!(
            rows(&hits_of(&sequence, target, "MAW")) == vec!["+-@0+9#3-5".to_string()],
            "MAW on `pj` must be the run at 0 over 9 bases, not a wrapping one"
        );
        // Codons 2, 3 begin at 57 and run to base 3: 57 + 6 > 60.
        assert!(
            rows(&hits_of(&sequence, target, "KM")) == vec!["+-@57+6#2-3 wraps".to_string()],
            "KM on `pj` must begin at 57 and wrap"
        );
        // The whole reading of codons 1..3, from the case-folded pattern.
        assert!(
            rows(&hits_of(&sequence, target, "mkm")) == vec!["+-@54+9#1-3 wraps".to_string()],
            "mkm on `pj` must begin at 54 and wrap"
        );
        // Every codon, in base order: the stop codon at 9 has no numbers, and
        // the two codons before the origin sort after it.
        let every: Vec<ProteinHit> = hits_of(&sequence, target, "X");
        let expected: Vec<ProteinHit> = vec![
            want(0, 3, Strand::Plus, NO_FRAME, Some(3), Some(3), false),
            want(3, 3, Strand::Plus, NO_FRAME, Some(4), Some(4), false),
            want(6, 3, Strand::Plus, NO_FRAME, Some(5), Some(5), false),
            want(9, 3, Strand::Plus, NO_FRAME, None, None, false),
            want(54, 3, Strand::Plus, NO_FRAME, Some(1), Some(1), false),
            want(57, 3, Strand::Plus, NO_FRAME, Some(2), Some(2), false),
        ];
        assert!(
            every == expected,
            "X on `pj` gives {} hits ({}), expected the six codons in base order ({})",
            every.len(),
            rows(&every).join(", "),
            rows(&expected).join(", ")
        );
        // `X` matches the terminal stop (D054), and `*` matches it too; the run
        // of all six codons starts at 54 and wraps.
        for pattern in ["MKMAWX", "MKMAW*"] {
            assert!(
                rows(&hits_of(&sequence, target, pattern))
                    == vec!["+-@54+18#1-stop wraps".to_string()],
                "{:?} on `pj` must be the whole CDS at 54, wrapping, last `stop`",
                pattern
            );
        }
        // As long as the target and one letter off: no hit, no refusal.
        finds_nothing(&sequence, target, "MKMAWW");
        // Longer than the target's six symbols.
        finds_nothing(&sequence, target, "MKMAW*M");
    }

    /// Rule 6 (D054): CDS hits come in base order, which is **not** reading
    /// order. `pj`'s `M` is residue 3 at base 0 first and residue 1 at base 54
    /// last; `m1` reads `M Q Q *` downhill, so its residue 3 (base 15) precedes
    /// its residue 2 (base 18).
    #[test]
    fn cds_hits_come_in_base_order() {
        let record: GenBankRecord = synthetic();
        let sequence: Sequence = record.sequence();
        let joined: Cds = pj(&record);
        let reverse: Cds = m1(&record);
        let plus: Vec<String> = rows(&hits_of(&sequence, ProteinTarget::Cds(&joined), "M"));
        assert!(
            plus == vec!["+-@0+3#3-3".to_string(), "+-@54+3#1-1".to_string()],
            "M on `pj` gives {:?}, expected residue 3 at base 0 before residue 1 at base 54",
            plus
        );
        let minus: Vec<String> = rows(&hits_of(&sequence, ProteinTarget::Cds(&reverse), "Q"));
        assert!(
            minus == vec!["--@15+3#3-3".to_string(), "--@18+3#2-2".to_string()],
            "Q on `m1` gives {:?}, expected residue 3 at base 15 before residue 2 at base 18",
            minus
        );
    }

    /// Rule 3 with D010 and D050 M8-17: the terminal stop codon is in the
    /// target and matches `*` and `X`, and it carries no residue number, so a
    /// hit that ends on it has no last number.
    #[test]
    fn the_stop_codon_has_no_residue_number() {
        let record: GenBankRecord = synthetic();
        let sequence: Sequence = record.sequence();
        let joined: Cds = pj(&record);
        let target: ProteinTarget<'_> = ProteinTarget::Cds(&joined);
        let expected: Vec<ProteinHit> =
            vec![want(6, 6, Strand::Plus, NO_FRAME, Some(5), None, false)];
        for pattern in ["W*", "WX"] {
            let found: Vec<ProteinHit> = hits_of(&sequence, target, pattern);
            assert!(
                found == expected,
                "{:?} on `pj` gives ({}), expected one hit at 6 over 6 bases, first 5 and last `stop`",
                pattern,
                rows(&found).join(", ")
            );
        }
        let stop: Vec<ProteinHit> = hits_of(&sequence, target, "*");
        assert!(
            stop == vec![want(9, 3, Strand::Plus, NO_FRAME, None, None, false)],
            "* on `pj` gives ({}), expected the one stop codon at 9 with no numbers",
            rows(&stop).join(", ")
        );
    }

    /// Rule 3 on a minus-strand CDS: the run of bases a hit covers starts at
    /// the **last** codon the reading reaches, because the reading runs to
    /// decreasing coordinates and the span is still reported forward (`m1`,
    /// `complement(13..24)`).
    #[test]
    fn a_minus_strand_cds_starts_its_run_at_the_last_codon() {
        let record: GenBankRecord = synthetic();
        let sequence: Sequence = record.sequence();
        let reverse: Cds = m1(&record);
        let target: ProteinTarget<'_> = ProteinTarget::Cds(&reverse);
        assert!(
            rows(&hits_of(&sequence, target, "QQ")) == vec!["--@15+6#2-3".to_string()],
            "QQ on `m1` must cover [15,21) with numbers 2 and 3: the reading reaches codon 3 last"
        );
        assert!(
            rows(&hits_of(&sequence, target, "Q*")) == vec!["--@12+6#3-stop".to_string()],
            "Q* on `m1` must cover [12,18), numbered 3 and `stop`"
        );
        assert!(
            rows(&hits_of(&sequence, target, "MQQ*")) == vec!["--@12+12#1-stop".to_string()],
            "the whole of `m1` must cover [12,24), numbered 1 to `stop`"
        );
    }

    /// Rule 4 and rule 6 in frame mode: hits are ordered by `start` with `+`
    /// before `−` on a tie, a hit's start is its first residue's `middle - 1`
    /// on `+` and its last residue's `middle - 1` on `−`, and a minus frame's
    /// numbers count codons from the far end of the record.
    #[test]
    fn frame_hits_come_in_base_order_plus_first() {
        let tiny: Sequence = dna("ATGAAATAG", SequenceTopology::Linear);
        let target: ProteinTarget<'_> = ProteinTarget::SixFrames;
        // `F` is −2 codon 2 (middles 6,3,1) and −1 codon 2 (middles 7,4,1): the
        // run starts at 2 and at 3, both minus, so ordering is by base alone.
        let phenylalanine: Vec<ProteinHit> = hits_of(&tiny, target, "F");
        assert!(
            phenylalanine
                == vec![
                    want(2, 3, Strand::Minus, 2, Some(2), Some(2), false),
                    want(3, 3, Strand::Minus, 1, Some(2), Some(2), false),
                ],
            "F on tiny gives ({}), expected −2 residue 2 at 2 then −1 residue 2 at 3",
            rows(&phenylalanine).join(", ")
        );
        // `*` is +2 codon 1 (`TGA`, middles 2) and +1 codon 3 (`TAG`, middles 7).
        let stops: Vec<ProteinHit> = hits_of(&tiny, target, "*");
        assert!(
            stops
                == vec![
                    want(1, 3, Strand::Plus, 2, Some(1), Some(1), false),
                    want(6, 3, Strand::Plus, 1, Some(3), Some(3), false),
                ],
            "* on tiny gives ({}), expected +2 codon 1 at 1 then +1 codon 3 at 6",
            rows(&stops).join(", ")
        );
        // `I` is −3 codon 1 (`ATT` read on −, middles 5) and +3 codon 2 (`ATA`,
        // middles 6): 4 then 5.
        let isoleucine: Vec<ProteinHit> = hits_of(&tiny, target, "I");
        assert!(
            isoleucine
                == vec![
                    want(4, 3, Strand::Minus, 3, Some(1), Some(1), false),
                    want(5, 3, Strand::Plus, 3, Some(2), Some(2), false),
                ],
            "I on tiny gives ({}), expected −3 codon 1 at 4 then +3 codon 2 at 5",
            rows(&isoleucine).join(", ")
        );
    }

    /// Rule 5 (D050 M8-16): every offset is tried, so every overlapping hit is
    /// reported, and there is no cap. Twelve `A`s: every plus codon is `AAA` =
    /// `K`, every minus codon `TTT` = `F`, so `KK` has hits in the three plus
    /// frames only — 3 + 2 + 2 = 7 of them, at starts 0..=6.
    #[test]
    fn overlapping_frame_hits_are_all_reported() {
        let poly: Sequence = dna("AAAAAAAAAAAA", SequenceTopology::Linear);
        let found: Vec<ProteinHit> = hits_of(&poly, ProteinTarget::SixFrames, "KK");
        let expected: Vec<ProteinHit> = vec![
            want(0, 6, Strand::Plus, 1, Some(1), Some(2), false),
            want(1, 6, Strand::Plus, 2, Some(1), Some(2), false),
            want(2, 6, Strand::Plus, 3, Some(1), Some(2), false),
            want(3, 6, Strand::Plus, 1, Some(2), Some(3), false),
            want(4, 6, Strand::Plus, 2, Some(2), Some(3), false),
            want(5, 6, Strand::Plus, 3, Some(2), Some(3), false),
            want(6, 6, Strand::Plus, 1, Some(3), Some(4), false),
        ];
        assert!(
            found == expected,
            "KK on twelve A's gives {} hits ({}), expected the 7 overlapping plus-strand pairs ({})",
            found.len(),
            rows(&found).join(", "),
            rows(&expected).join(", ")
        );
        let starts: Vec<u64> = found.iter().map(|hit| hit.start).collect();
        assert!(
            starts == vec![0u64, 1, 2, 3, 4, 5, 6],
            "KK starts come out {:?}, expected 0..=6: stepping one base per try, never the pattern's width",
            starts
        );
    }

    /// Rule 4 with D052 M8-2: a frame is counted on the linear coordinates and
    /// `topology()` is never read in frame mode, so the same text read as
    /// circular gives exactly the hits the linear reading gives — while a
    /// wrapping codon is not translated at all, which is why nothing is added.
    #[test]
    fn frames_ignore_topology() {
        let linear: Sequence = dna("ATGAAATAG", SequenceTopology::Linear);
        let circular: Sequence = dna("ATGAAATAG", SequenceTopology::Circular);
        assert!(
            linear.length() == circular.length() && circular.topology() != linear.topology(),
            "the two records must differ in topology alone"
        );
        for pattern in ["X", "*"] {
            let straight: Vec<ProteinHit> = hits_of(&linear, ProteinTarget::SixFrames, pattern);
            let round: Vec<ProteinHit> = hits_of(&circular, ProteinTarget::SixFrames, pattern);
            assert!(
                round == straight,
                "{:?} over a circular tiny gives {} hits ({}), expected the {} of the linear reading ({})",
                pattern,
                round.len(),
                rows(&round).join(", "),
                straight.len(),
                rows(&straight).join(", ")
            );
            assert!(
                round.iter().all(|hit| !hit.wraps_origin),
                "{:?} over a circular tiny gives a hit that wraps: a frame never runs across the origin (D052)",
                pattern
            );
        }
    }
}
