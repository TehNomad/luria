//! The GenBank writer: an edit buffer written back as one GenBank record
//! (plan M9-7, M9-11, M9-15).
//!
//! The two pBR322 records in `tests/fixtures/editor/` pin this output byte
//! for byte; they are built from NCBI's file by
//! `tests/fixtures/editor/reference/saved.py`, which never runs luria.
//!
//! The output is written in the order the standard requires: the LOCUS
//! line, the header as read, the FEATURES table, and the ORIGIN block.
//! A feature that no edit changed is written as the file wrote it, line for
//! line (M9-7); the header comes back as read, but for the LOCUS line, the
//! dropped `VERSION` line and the save's own COMMENT line (M9-11, M9-15).

use crate::edit_buffer::EditBuffer;
use crate::edit_buffer::EditFeature;
use crate::edit_location::write_edit_location;
use crate::genbank::GenBankRecord;

/// The molecule type the LOCUS line falls back to when the opened record
/// carries none and has a bp count and topology (M9-15).
const DEFAULT_MOLECULE: &str = "DNA";

/// The division the LOCUS line falls back to when the opened record carries
/// none (M9-15).
const DEFAULT_DIVISION: &str = "UNK";

/// The FEATURES line, as the standard writes it.
const FEATURES_LINE: &str = "FEATURES             Location/Qualifiers";

/// The widest line the writer keeps, counted in characters.
const MAX_WIDTH: usize = 79;

/// The indentation of a qualifier line and of every continuation line: the
/// column the standard puts a qualifier's `/` at.
const INDENT: usize = 21;

/// The indentation of a feature's key line.
const KEY_INDENT: usize = 5;

/// The width the key of a feature's key line is left-aligned in.
const KEY_WIDTH: usize = 16;

/// The ORIGIN line, with no trailing space (§4.4).
const ORIGIN_LINE: &str = "ORIGIN";

/// The line that ends a GenBank record.
const END_LINE: &str = "//";

/// The bases on each ORIGIN line.
const BASES_PER_LINE: usize = 60;

/// The bases in each group of an ORIGIN line.
const GROUP: usize = 10;

/// Qualifiers whose value is written unquoted, so a long value is never
/// broken at the first space that fits: Biopython's `FTQUAL_NO_QUOTE`.
const NO_QUOTE_QUALIFIERS: [&str; 13] = [
    "anticodon",
    "citation",
    "codon_start",
    "compare",
    "direction",
    "estimated_length",
    "mod_base",
    "number",
    "rpt_type",
    "rpt_unit_range",
    "tag_peptide",
    "transl_except",
    "transl_table",
];

/// What the save stamps on the record (M9-11, M9-15): the LOCUS date as
/// `DD-MMM-YYYY`, the version of luria, and the edits counted as M9-16
/// counts them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveStamp {
    pub date: String,
    pub version: String,
    pub edits: usize,
}

/// The text of the save's own comment: `Edited with luria <version>: one
/// edit.` or `Edited with luria <version>: N edits.` (M9-11).
fn comment(stamp: &SaveStamp) -> String {
    let word: &str = if stamp.edits == 1 { "edit" } else { "edits" };
    format!(
        "Edited with luria {}: {} {}.",
        stamp.version, stamp.edits, word
    )
}

/// The record as a save writes it: `\n` line ends, ending `//\n`.
pub fn write_genbank(buffer: &EditBuffer, opened: &GenBankRecord, stamp: &SaveStamp) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push(locus_line(buffer, opened, stamp));
    lines.extend(header_lines(opened, stamp));
    lines.extend(feature_lines(buffer, opened));
    lines.extend(origin_lines(buffer));
    let mut text: String = lines.join("\n");
    text.push('\n');
    text
}

/// The LOCUS line: `LOCUS`, 7 spaces, the name left-aligned in 16 columns, a
/// space, the buffer's length right-aligned in 11 columns, ` bp `, the
/// molecule type split at its `-` into 3 and 6 columns, two spaces, the
/// topology left-aligned in 8 columns, a space, the division, a space, and
/// the stamp's date (M9-15).
fn locus_line(buffer: &EditBuffer, opened: &GenBankRecord, stamp: &SaveStamp) -> String {
    let name: String = opened.locus_name();
    let molecule: Option<String> = opened.molecule_type();
    let (dash, rest): (&str, &str) = split_molecule(molecule.as_deref());
    let topology: String = buffer.topology().name();
    let division: String = opened
        .division()
        .unwrap_or_else(|| DEFAULT_DIVISION.to_string());
    format!(
        "LOCUS       {:<16} {:>11} bp {:<3}{:<6}  {:<8} {} {}",
        name,
        buffer.length(),
        dash,
        rest,
        topology,
        division,
        stamp.date
    )
}

/// The molecule type split after its `-`: `ds-DNA` becomes `ds-` and `DNA`,
/// and a molecule type with no hyphen becomes three spaces plus itself,
/// which is how the standard pads that column when the record has none.
fn split_molecule(molecule: Option<&str>) -> (&str, &str) {
    let text: &str = molecule.unwrap_or(DEFAULT_MOLECULE);
    match text.find('-') {
        Some(at) => {
            let split: usize = at + 1;
            (&text[..split], &text[split..])
        }
        None => ("   ", text),
    }
}

/// The header lines as read, in order, except that the `VERSION` line and
/// its continuation lines are left out (M9-15), and the save's own comment
/// line is added after the last line of the last COMMENT block, or as a new
/// COMMENT section when the header has none (M9-11). One stamp per file
/// (M9-20): when that last line is already a luria stamp, the save writes
/// its own stamp over it, keeping the lead that line had, and adds nothing.
fn header_lines(opened: &GenBankRecord, stamp: &SaveStamp) -> Vec<String> {
    let read: Vec<String> = opened.header_lines();
    let mut out: Vec<String> = Vec::with_capacity(read.len() + 1);
    let mut skipping: bool = false;
    for line in read {
        if line.starts_with("VERSION") {
            skipping = true;
            continue;
        }
        if skipping && line.starts_with(' ') {
            continue;
        }
        skipping = false;
        out.push(line);
    }
    let edited: String = comment(stamp);
    match last_of_comment_block(&out) {
        Some(end) if is_stamp_line(&out[end]) => {
            let lead: &str = if out[end].starts_with("COMMENT") {
                "COMMENT     "
            } else {
                "            "
            };
            out[end] = format!("{lead}{edited}");
        }
        Some(end) => out.insert(end + 1, format!("{}{}", " ".repeat(12), edited)),
        None => out.push(format!("COMMENT     {edited}")),
    }
    out
}

/// Whether a header line is a luria save stamp, so the save recognises its
/// own earlier stamp and writes over it instead of adding a second one
/// (M9-20): `COMMENT` and 5 spaces, or 12 spaces, then the stamp text and
/// nothing after it. The stamp text is `Edited with luria `, a version of
/// one or more characters none of which is a space or `:`, `: `, one or
/// more ASCII digits, and ` edit.` or ` edits.`.
fn is_stamp_line(line: &str) -> bool {
    let text: &str = match line.strip_prefix("COMMENT     ") {
        Some(rest) => rest,
        None => match line.strip_prefix("            ") {
            Some(rest) => rest,
            None => return false,
        },
    };
    let after: &str = match text.strip_prefix("Edited with luria ") {
        Some(rest) => rest,
        None => return false,
    };
    // The version runs to the first `: `. It holds dots, as `0.1.0` does,
    // but no space and no colon: a second `: ` means the line is not a
    // stamp.
    let (version, count): (&str, &str) = match after.split_once(": ") {
        Some(parts) => parts,
        None => return false,
    };
    if version.is_empty() || version.contains([' ', ':']) {
        return false;
    }
    // The count is ASCII digits, then the word the save wrote, then the end
    // of the line: `and more` after the full stop is not a stamp.
    let digits: usize = count
        .bytes()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    match digits {
        0 => false,
        _ => matches!(&count[digits..], " edit." | " edits."),
    }
}

/// The index, in a record's header lines, of the last line of the last
/// COMMENT block: the line after which the save's own comment is added
/// (M9-11). `None` when the header has no COMMENT block.
fn last_of_comment_block(lines: &[String]) -> Option<usize> {
    let start: usize = lines.iter().rposition(|line| line.starts_with("COMMENT"))?;
    let mut end: usize = start;
    while end + 1 < lines.len() && lines[end + 1].starts_with(' ') {
        end += 1;
    }
    Some(end)
}

/// The FEATURES line, then one block per buffer feature, in buffer order.
/// `used` marks the opened features already taken by a buffer feature, so a
/// feature the buffer has already consumed is not matched again (§4.3).
fn feature_lines(buffer: &EditBuffer, opened: &GenBankRecord) -> Vec<String> {
    let mut out: Vec<String> = vec![FEATURES_LINE.to_string()];
    let mut used: Vec<bool> = vec![false; opened.feature_slice().len()];
    for feature in buffer.features() {
        match unchanged_feature(feature, opened, &mut used) {
            Some(lines) => out.extend(lines),
            None => out.extend(new_feature_lines(feature)),
        }
    }
    out
}

/// The first opened feature that is not yet used, that the buffer feature
/// was read from and that no edit has changed (M9-7): the same key, a
/// `location()` equal to the buffer's kept `location_text`, and the same
/// `(name, value)` qualifiers in the same order. The opened one taken is
/// marked in `used`, and its `lines()` are written verbatim; `None` says the
/// buffer feature is changed and has to be written fresh.
fn unchanged_feature(
    feature: &EditFeature,
    opened: &GenBankRecord,
    used: &mut [bool],
) -> Option<Vec<String>> {
    let text: &str = feature.location_text.as_deref()?;
    let at: usize = opened
        .feature_slice()
        .iter()
        .zip(used.iter())
        .position(|(found, used)| {
            !*used
                && found.key() == feature.key
                && found.location() == text
                && found
                    .qualifiers()
                    .iter()
                    .map(|qualifier| (qualifier.name(), qualifier.value()))
                    .eq(feature
                        .qualifiers
                        .iter()
                        .map(|(name, value)| (name.clone(), value.clone())))
        })?;
    used[at] = true;
    let lines: Vec<String> = opened.feature_slice()[at].lines();
    if lines.is_empty() {
        // An opened feature with no lines is not written verbatim; write
        // the buffer feature fresh instead.
        None
    } else {
        Some(lines)
    }
}

/// A changed feature written fresh: its key line, then each of the buffer's
/// qualifiers.
fn new_feature_lines(feature: &EditFeature) -> Vec<String> {
    let location: String = feature
        .location_text
        .clone()
        .unwrap_or_else(|| write_edit_location(&feature.location));
    let mut out: Vec<String> = key_line(&feature.key, location.as_str());
    for (name, value) in feature.qualifiers.iter() {
        out.extend(qualifier_lines(name, value.as_deref()));
    }
    out
}

/// A feature's key line and its location's continuation lines: 5 spaces,
/// the key left-aligned in 16 columns (a longer key keeps one space), then
/// its location (§4.3).
fn key_line(key: &str, location: &str) -> Vec<String> {
    let padded: String = if key.chars().count() > KEY_WIDTH {
        format!("{} ", key)
    } else {
        format!("{:<1$}", key, KEY_WIDTH)
    };
    let head: String = format!("{}{}", " ".repeat(KEY_INDENT), padded);
    location_lines(head.as_str(), location)
}

/// The pieces of a key line's location: the first closes the key line, and
/// every one past 79 characters comes as its own line of 21 spaces, broken
/// after the last `,` that fits (§4.3).
fn location_lines(head: &str, location: &str) -> Vec<String> {
    let indent: String = " ".repeat(INDENT);
    let mut lines: Vec<String> = Vec::new();
    let mut line_start: String = head.to_string();
    let mut rest: &str = location;
    loop {
        if line_start.chars().count() >= MAX_WIDTH {
            line_start = indent.clone();
        }
        let room: usize = MAX_WIDTH - line_start.chars().count();
        let fits: usize = take_prefix(rest, room);
        if fits >= rest.len() {
            lines.push(format!("{}{}", line_start, rest));
            break;
        }
        let cut: usize = match rest[..fits].rfind(',') {
            Some(at) => at + 1,
            None => fits,
        };
        lines.push(format!("{}{}", line_start, &rest[..cut]));
        rest = &rest[cut..];
        line_start = indent.clone();
    }
    lines
}

/// The byte index right after the first `max` codepoints of `text`, or
/// `text.len()` when it holds no more of them.
fn take_prefix(text: &str, max: usize) -> usize {
    match text.char_indices().nth(max) {
        Some((at, _)) => at,
        None => text.len(),
    }
}

/// The lines of one qualifier: `/name` for a flag; `/name=value` unquoted
/// for the `NO_QUOTE_QUALIFIERS`; otherwise a quoted value (§4.3).
fn qualifier_lines(name: &str, value: Option<&str>) -> Vec<String> {
    let indent: String = " ".repeat(INDENT);
    match value {
        None => vec![format!("{}/{}", indent, name)],
        Some(text) if NO_QUOTE_QUALIFIERS.contains(&name) => {
            vec![format!("{}/{}={}", indent, name, text)]
        }
        Some(text) => quoted_lines(name, text, indent.as_str()),
    }
}

/// The lines of a quoted qualifier: the value's `"` are doubled, one opening
/// `"` follows `/name=` on the first line, and one closing `"` ends the last
/// line; continuation lines are the indent then the text, with no quotes of
/// either kind. A line past 79 characters breaks at the last space that fits,
/// which is
/// dropped, as the reader joins the lines with one space. A run with no
/// space breaks at 79 characters (§4.3).
fn quoted_lines(name: &str, value: &str, indent: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let quoted: String = value.replace('"', "\"\"");
    let mut rest: &str = quoted.as_str();
    let head: String = format!("{indent}/{name}=\"", indent = indent, name = name);
    let cont: String = indent.to_string();
    let mut first: bool = true;
    loop {
        let start: &str = if first { head.as_str() } else { cont.as_str() };
        // Every line but the last keeps a column free for the closing
        // quote, so its last character can sit at column 79.
        let room: usize = MAX_WIDTH - start.chars().count();
        let fill: usize = take_prefix(rest, room);
        let end: usize = take_prefix(rest, room.saturating_sub(1));
        if end >= rest.len() {
            lines.push(format!("{start}{rest}\""));
            break;
        }
        // A space sitting right past the last character that fits is itself
        // usable as a break: it is dropped, so the line still ends at column
        // 79. Hence the search covers one character more than the fill.
        let up_to: usize = take_prefix(rest, room + 1);
        match rest[..up_to].rfind(' ') {
            // The space is dropped, as the reader joins the lines with one.
            Some(at) if at > 0 => {
                lines.push(format!("{start}{}", &rest[..at]));
                rest = &rest[at + 1..];
            }
            // A leading space would write an empty continuation line, which
            // is never what the reader sees: break at the fill instead.
            _ => {
                lines.push(format!("{start}{}", &rest[..fill]));
                rest = &rest[fill..];
            }
        }
        first = false;
    }
    lines
}

/// The ORIGIN block: the `ORIGIN` line, then the buffer's bases lower case,
/// 60 to a line, each numbered by the 1-based number of its first base
/// right-aligned in 9 columns, a space, and its groups of 10 joined by
/// single spaces. Then the `//` that ends the record (§4.4).
fn origin_lines(buffer: &EditBuffer) -> Vec<String> {
    let mut out: Vec<String> = vec![ORIGIN_LINE.to_string()];
    let letters: Vec<char> = buffer
        .letters()
        .iter()
        .map(|letter| letter.to_ascii_lowercase())
        .collect();
    for (index, line) in letters.chunks(BASES_PER_LINE).enumerate() {
        let groups: Vec<String> = line
            .chunks(GROUP)
            .map(|group| group.iter().collect::<String>())
            .collect();
        out.push(format!(
            "{:>9} {}",
            index * BASES_PER_LINE + 1,
            groups.join(" ")
        ));
    }
    out.push(END_LINE.to_string());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genbank::read_genbank;
    use crate::identity::Identifier;

    /// The 21 spaces a qualifier line and each of its continuation lines
    /// start with (§4 rule 3); test 5 pins the count so a miscounted literal
    /// is caught at once.
    const INDENT_TEXT: &str = "                     ";

    /// No written line may be wider than 79 characters (§4 rule 3).
    fn within_width(lines: &[String]) {
        for line in lines {
            assert!(
                line.chars().count() <= MAX_WIDTH,
                "a written line is wider than {MAX_WIDTH} columns: {line}"
            );
        }
    }

    /// §5 test 5: the lines of a changed feature's qualifiers, wrapped
    /// exactly as §4 rule 3 prescribes.
    #[test]
    fn qualifiers_wrap_at_79() {
        assert_eq!(
            INDENT_TEXT.chars().count(),
            INDENT,
            "the indent of a written qualifier is 21 spaces"
        );

        // A 200-character note of words: 39 four-letter words plus one
        // five-letter word, so 160 letters and 39 spaces. Each line breaks at
        // the last space that fits: 10 words on the opening line, then 11 and
        // 11, and the last 8 words carry the closing quote.
        let mut words: Vec<&str> = vec!["word"; 39];
        words.push("words");
        let note: String = words.join(" ");
        assert_eq!(note.chars().count(), 200);
        let wrapped: Vec<String> = qualifier_lines("note", Some(&note));
        assert_eq!(
            wrapped,
            [
                "                     /note=\"word word word word word word word word word word",
                "                     word word word word word word word word word word word",
                "                     word word word word word word word word word word word",
                "                     word word word word word word word words\"",
            ]
        );
        // A wrapped value has one opening quote on the first line, one
        // closing quote on the last, and neither on the lines in between.
        assert_eq!(wrapped[0].matches('"').count(), 1);
        assert_eq!(wrapped[0].chars().next_back(), Some('d'));
        assert!(wrapped.last().unwrap().ends_with("words\""));
        for line in &wrapped[1..wrapped.len() - 1] {
            assert!(!line.contains('"'), "no quote mid-value: {line}");
        }

        // A 100-character word has no space, so it breaks at 79 characters:
        // 51 letters on the opening line, the other 49 and the closing quote
        // on the next.
        let one_word: String = "abcdefghij".repeat(10);
        assert_eq!(one_word.chars().count(), 100);
        let hard: Vec<String> = qualifier_lines("note", Some(&one_word));
        assert_eq!(
            hard,
            [
                "                     /note=\"abcdefghijabcdefghijabcdefghijabcdefghijabcdefghija",
                "                     bcdefghijabcdefghijabcdefghijabcdefghijabcdefghij\"",
            ]
        );

        // A 51-character run of `a`, a space, then `bbb`: the space sits right
        // past the last character that fits, and because it is dropped the
        // opening line still ends at column 79, with `bbb` and the closing
        // quote on the next line.
        let edge: Vec<String> = qualifier_lines("note", Some(&("a".repeat(51) + " bbb")));
        assert_eq!(
            edge,
            [
                "                     /note=\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "                     bbb\"",
            ]
        );

        // A flag qualifier is just its name; an entry in NO_QUOTE_QUALIFIERS
        // is written without quotes.
        let flag: Vec<String> = qualifier_lines("pseudo", None);
        let unquoted: Vec<String> = qualifier_lines("codon_start", Some("2"));
        assert_eq!(flag, ["                     /pseudo"]);
        assert_eq!(unquoted, ["                     /codon_start=2"]);

        // A value holding a quote has it doubled, so the reader sees the
        // value's own quotes as part of the value.
        let quoted: Vec<String> = qualifier_lines("note", Some("say \"hi\" to this"));
        assert_eq!(
            quoted,
            ["                     /note=\"say \"\"hi\"\" to this\""]
        );

        within_width(&wrapped);
        within_width(&hard);
        within_width(&edge);
        within_width(&flag);
        within_width(&unquoted);
        within_width(&quoted);
    }

    /// A hand-written record for the header rules: `extra` holds the lines
    /// between the LOCUS line and the ORIGIN line, in column layout.
    fn record(extra: &[&str]) -> GenBankRecord {
        let text: String = format!(
            "LOCUS       test1                     10 bp    DNA     linear UNK \
             17-OCT-2026\n{extra}\nORIGIN\n        1 atgcatgcat\n//\n",
            extra = extra.join("\n")
        );
        read_genbank(Identifier::create("head".to_string()).unwrap(), text)
            .expect("the hand-written record is read")
    }

    /// §5 test 6: where the save's own COMMENT line goes, and that the
    /// opened record's `VERSION` line and its continuation lines are left out
    /// (M9-11, M9-15).
    #[test]
    fn comment_placement() {
        let stamp: SaveStamp = SaveStamp {
            date: "29-OCT-2026".to_string(),
            version: "9.9.9".to_string(),
            edits: 0,
        };
        let text: String = comment(&stamp);
        // The save's own comment as a new section, and as a continuation line.
        let new_section: String = format!("COMMENT     {text}");
        let indented: String = format!("            {text}");

        // With no COMMENT, the save's own COMMENT section becomes the last
        // header line, and the opened record's VERSION line is gone.
        let no_comment: Vec<String> = header_lines(
            &record(&[
                "DEFINITION  no COMMENT block in this header.",
                "ACCESSION   NOCOMMENT",
                "VERSION     NOCOMMENT.1",
                "KEYWORDS    .",
            ]),
            &stamp,
        );
        assert_eq!(
            no_comment,
            [
                "DEFINITION  no COMMENT block in this header.",
                "ACCESSION   NOCOMMENT",
                "KEYWORDS    .",
                new_section.as_str(),
            ]
        );

        // One COMMENT block with a continuation line, followed by another
        // section: the line goes after the continuation, before that section.
        let one_block: Vec<String> = header_lines(
            &record(&[
                "DEFINITION  one COMMENT block with a continuation.",
                "COMMENT     first block",
                "            continued",
                "VERSION     ONEBLOCK.1",
                "            the VERSION continuation is dropped too",
                "KEYWORDS    .",
            ]),
            &stamp,
        );
        assert_eq!(
            one_block,
            [
                "DEFINITION  one COMMENT block with a continuation.",
                "COMMENT     first block",
                "            continued",
                indented.as_str(),
                "KEYWORDS    .",
            ]
        );
        // The header holds its four lines, plus the one added line: the two
        // `VERSION` lines (the line and its continuation) are gone.
        assert_eq!(one_block.len(), 5, "the VERSION line and its continuation                                         are the only lines dropped");

        // Two COMMENT blocks: the line goes after the second one, because
        // that is the last COMMENT block in the header.
        let two_blocks: Vec<String> = header_lines(
            &record(&[
                "DEFINITION  two COMMENT blocks.",
                "COMMENT     first block",
                "            continued-a",
                "COMMENT     second block",
                "            continued-b",
                "SOURCE      synthetic",
                "  ORGANISM  synthetic",
            ]),
            &stamp,
        );
        assert_eq!(
            two_blocks,
            [
                "DEFINITION  two COMMENT blocks.",
                "COMMENT     first block",
                "            continued-a",
                "COMMENT     second block",
                "            continued-b",
                indented.as_str(),
                "SOURCE      synthetic",
                "  ORGANISM  synthetic",
            ]
        );
    }

    /// The stamp a unit-test save writes: version `9.9.9`, one edit.
    fn one_edit_stamp() -> SaveStamp {
        SaveStamp {
            date: "29-OCT-2026".to_string(),
            version: "9.9.9".to_string(),
            edits: 1,
        }
    }

    /// §5 test 1: when the last line of the last COMMENT block is already a
    /// luria stamp, the save writes its own stamp over that line and adds no
    /// line of its own, so the file keeps one stamp (M9-20).
    #[test]
    fn an_earlier_stamp_is_replaced() {
        let stamp: SaveStamp = one_edit_stamp();
        let read: [&str; 5] = [
            "DEFINITION  a COMMENT block that ends in a luria stamp.",
            "COMMENT     first",
            "            Edited with luria 0.1.0: 3 edits.",
            "VERSION     ENDED.1",
            "KEYWORDS    .",
        ];
        let header: Vec<String> = header_lines(&record(&read), &stamp);
        assert_eq!(
            header,
            [
                "DEFINITION  a COMMENT block that ends in a luria stamp.",
                "COMMENT     first",
                "            Edited with luria 9.9.9: 1 edit.",
                "KEYWORDS    .",
            ]
        );
        // The stamp line is replaced in place and nothing is inserted: only
        // the VERSION line is gone.
        assert_eq!(header.len(), read.len() - 1, "no stamp line is added");
    }

    /// §5 test 2: when the header's only COMMENT section is a luria stamp,
    /// the save writes its own stamp over that section's line, keeping the
    /// `COMMENT     ` lead, and adds no second COMMENT (M9-20).
    #[test]
    fn a_stamp_section_is_replaced() {
        let stamp: SaveStamp = one_edit_stamp();
        let header: Vec<String> = header_lines(
            &record(&[
                "DEFINITION  the header's only COMMENT is a luria stamp.",
                "COMMENT     Edited with luria 0.1.0: 0 edits.",
                "VERSION     STAMPED.1",
                "KEYWORDS    .",
            ]),
            &stamp,
        );
        assert_eq!(
            header,
            [
                "DEFINITION  the header's only COMMENT is a luria stamp.",
                "COMMENT     Edited with luria 9.9.9: 1 edit.",
                "KEYWORDS    .",
            ]
        );
        // One COMMENT section, as the file had: the new stamp does not start
        // a second one.
        let comments: usize = header
            .iter()
            .filter(|line| line.starts_with("COMMENT"))
            .count();
        assert_eq!(comments, 1, "the save adds no COMMENT section");
    }

    /// §5 test 3: only the last line of the last COMMENT block is ever
    /// replaced. A stamp higher up the block, and a line that looks like a
    /// stamp but is not one, stay as the file wrote them and the new stamp is
    /// added after the block, as today (M9-20).
    #[test]
    fn near_misses_are_kept() {
        let stamp: SaveStamp = one_edit_stamp();
        let indented: String = format!("            {}", comment(&stamp));

        // (a) The stamp is the block's first line and `more text` follows it:
        // it is not the block's last line, so it is left alone.
        let first_line: Vec<String> = header_lines(
            &record(&[
                "DEFINITION  a COMMENT block that starts with a luria stamp.",
                "COMMENT     Edited with luria 0.1.0: 3 edits.",
                "            more text",
                "VERSION     FIRSTLINE.1",
                "KEYWORDS    .",
            ]),
            &stamp,
        );
        assert_eq!(
            first_line,
            [
                "DEFINITION  a COMMENT block that starts with a luria stamp.",
                "COMMENT     Edited with luria 0.1.0: 3 edits.",
                "            more text",
                indented.as_str(),
                "KEYWORDS    .",
            ]
        );

        // (b) `Edited with luria:` carries no version, so it is not a stamp.
        let no_version: Vec<String> = header_lines(
            &record(&[
                "DEFINITION  a COMMENT block ending in a line like a stamp.",
                "COMMENT     first",
                "            Edited with luria: 3 edits.",
                "VERSION     NOVERSION.1",
                "KEYWORDS    .",
            ]),
            &stamp,
        );
        assert_eq!(
            no_version,
            [
                "DEFINITION  a COMMENT block ending in a line like a stamp.",
                "COMMENT     first",
                "            Edited with luria: 3 edits.",
                indented.as_str(),
                "KEYWORDS    .",
            ]
        );

        // (c) The line runs on past the stamp's full stop, so the stamp does
        // not end the line and it is not a stamp.
        let runs_on: Vec<String> = header_lines(
            &record(&[
                "DEFINITION  a COMMENT block ending in a line like a stamp.",
                "COMMENT     first",
                "            Edited with luria 0.1.0: 3 edits. and more",
                "VERSION     RUNSON.1",
                "KEYWORDS    .",
            ]),
            &stamp,
        );
        assert_eq!(
            runs_on,
            [
                "DEFINITION  a COMMENT block ending in a line like a stamp.",
                "COMMENT     first",
                "            Edited with luria 0.1.0: 3 edits. and more",
                indented.as_str(),
                "KEYWORDS    .",
            ]
        );
    }
}
