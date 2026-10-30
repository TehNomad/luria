//! The three pure parts of a save (contract 9h §4.1, §4.4, §4.9; plan M9-7,
//! M9-11, M9-15, M9-16): the default name the `s` prompt starts with, the date
//! the saved file stamps, and the text the save writes.
//!
//! Nothing here touches a filesystem: `super::state` asks for a write through
//! `save_request`, `super::terminal` performs it with `crate::write::write_text`,
//! and the opened path and today's day count come from there (D003). What this
//! module answers is the name and the text, so the state can fill in a prompt
//! without ever reading a directory.
//!
//! The date is computed from a day count rather than read from a clock: §4.9
//! asks for a pure `DD-MMM-YYYY` conversion, and a test that fixes the day gets
//! the same line the LOCUS line will carry (M9-15: the date is the only date in
//! a save, so a whole file can be compared).

use luria_core::edit_buffer::EditBuffer;
use luria_core::fasta::render_fasta;
use luria_core::genbank::GenBankRecord;
use luria_core::genbank_write::write_genbank;
use luria_core::genbank_write::SaveStamp;
use luria_core::view_record::RecordFormat;
use luria_core::view_record::ViewRecord;
use std::path::Path;
use std::path::PathBuf;

/// The name a save writes to by default (§4.1): `<stem>.edited.gb` for a
/// GenBank record and `<stem>.edited.fa` for a FASTA one, in the opened file's
/// own directory. The stem is the opened file's name cut at its last
/// extension, which is `Path::file_stem`'s answer: `a.b.gb` stems to `a.b`.
///
/// The answer keeps the opened path's own directory and spelling, so what the
/// prompt shows is a name relative to the opened path as it was typed.
pub fn default_name(opened: &Path, format: RecordFormat) -> PathBuf {
    // A path the reader hands this function always has a file stem: `None` is
    // only `..` and the empty path, and `open_record` refuses both. The
    // fallback makes the function total rather than leaving the caller to
    // invent a name of its own.
    let stem: String = opened.file_stem().map_or_else(
        || String::from("record"),
        |raw| raw.to_string_lossy().into_owned(),
    );
    let suffix: &str = match format {
        RecordFormat::GenBank => ".edited.gb",
        RecordFormat::Fasta => ".edited.fa",
    };
    opened
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(format!("{stem}{suffix}"))
}

/// The months by the three-letter uppercase name the LOCUS line writes them
/// with (M9-15), index 0 being January, so a month number is the index minus
/// one.
const MONTHS: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

/// The date a save stamps in its LOCUS line, from days since 1970-01-01
/// (§4.9, M9-15): `DD-MMM-YYYY`, the shape GenBank itself writes.
///
/// The conversion is written out rather than pulled in with a dependency: no
/// clock is read here, and a test that fixes the day gets the line the LOCUS
/// line will carry.
pub fn date_from_days(days: u64) -> String {
    let (year, month, day): (i64, usize, u64) = civil_from_days(days);
    format!("{day:02}-{}-{year}", MONTHS[month - 1])
}

/// The days in a 400-year cycle, and the days from 0000-03-01 to 1970-01-01.
const CYCLE_DAYS: u64 = 146_097;
const MARCH_SHIFT: u64 = 719_468;

/// The year, the one-based month and the day of a day count since 1970-01-01,
/// with the Gregorian calendar taken proleptonically back through the year 0.
///
/// The shift of `MARCH_SHIFT` days puts a cycle's day 0 at 0000-03-01, which
/// ends a counted year in February and starts it in March: the leap day then
/// sits at the year's end, five 31-day months run from March to July, and the
/// month and day of a day-of-year count fall out of one division by 153 (the
/// 365-day pattern `5*month + 2` over 153 repeats over every two-month group).
fn civil_from_days(days: u64) -> (i64, usize, u64) {
    // The shifted count is cut into whole eras of `CYCLE_DAYS` days:
    // `div_euclid` and `rem_euclid` keep the day of an era in [0, CYCLE_DAYS-1]
    // whatever clock day the terminal happens to give.
    let shifted: i64 = days as i64 + MARCH_SHIFT as i64;
    let era: i64 = shifted.div_euclid(CYCLE_DAYS as i64);
    let day_of_era: u64 = shifted.rem_euclid(CYCLE_DAYS as i64) as u64; // [0, 146096]
                                                                        // The era's March-year the day belongs to: `day_of_era * 400 / CYCLE_DAYS`
                                                                        // is the 365.2425-days-a-year estimate, and the two corrections below put
                                                                        // it back where the whole-cycle count says it belongs. A March-year `y`
                                                                        // begins `365y + y/4 - y/100 + y/400` days into the era, the leaps minus
                                                                        // the skipped century days plus the 400th year's one.
    let mut year_of_era: u64 = day_of_era * 400 / CYCLE_DAYS; // [0, 399]
    while year_of_era < 399
        && 365 * (year_of_era + 1) + (year_of_era + 1) / 4 - (year_of_era + 1) / 100
            + (year_of_era + 1) / 400
            <= day_of_era
    {
        year_of_era += 1;
    }
    while 365 * year_of_era + year_of_era / 4 - year_of_era / 100 + year_of_era / 400 > day_of_era {
        year_of_era -= 1;
    }
    let day_of_year: u64 =
        day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100 + year_of_era / 400); // [0, 365]
                                                                                                    // The month of a March-year: ten months of 31 days and two of 30 in the
                                                                                                    // order March, April, ..., February.
    let month_of_year: u64 = (day_of_year * 5 + 2) / 153; // [0, 11]
    let day: u64 = day_of_year - (153 * month_of_year + 2) / 5 + 1; // [1, 31]
    let month: usize = if month_of_year < 10 {
        (month_of_year + 3) as usize // March (3) to December (12)
    } else {
        (month_of_year - 9) as usize // January (1) and February (2)
    };
    // The counted year opens in March, so a January or February date belongs
    // to the year before the one the count started in.
    let year: i64 = era * 400 + year_of_era as i64 + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

/// The whole text a save writes (§4.4): GenBank out for a GenBank file,
/// through 9c3's writer with the opened file's own header, and FASTA out for a
/// FASTA file, through `render_fasta` of the buffer's name and bases.
///
/// The two failures each arm could in principle meet are answered with an
/// `expect` naming its own invariant, because the view can only open a save
/// prompt on a record it already read: a GenBank record keeps the header it was
/// read with (the format is the reader's sniff), and `to_sequence` re-normalizes
/// bases the editor only ever took from the DNA alphabet.
pub fn save_text(buffer: &EditBuffer, opened: &ViewRecord, stamp: &SaveStamp) -> String {
    match opened.format {
        RecordFormat::GenBank => {
            let header: &GenBankRecord = opened
                .genbank
                .as_ref()
                .expect("a GenBank record keeps the header it was read with");
            write_genbank(buffer, header, stamp)
        }
        RecordFormat::Fasta => {
            let sequence = buffer
                .to_sequence()
                .expect("the buffer's bases are the alphabet they were read and edited from");
            render_fasta(&buffer.name(), &sequence)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::date_from_days;
    use super::default_name;
    use luria_core::view_record::RecordFormat;
    use std::path::Path;

    /// §5 test 1: the four days the conversion is pinned to, with the leap day
    /// in a century that keeps it, the day before a year boundary, and the
    /// epoch itself.
    #[test]
    fn date_from_days_names_the_day_the_locus_line_carries() {
        assert_eq!(date_from_days(20755), "29-OCT-2026");
        assert_eq!(
            date_from_days(11016),
            "29-FEB-2000",
            "2000 keeps its leap day, 1900 did not"
        );
        assert_eq!(date_from_days(10956), "31-DEC-1999", "the last day of 1999");
        assert_eq!(date_from_days(0), "01-JAN-1970", "day zero is the epoch");
    }

    /// §5 test 2: the stem is the opened file's name cut at its last extension,
    /// and the directory is the opened path's own.
    #[test]
    fn default_names_cut_at_the_last_extension_and_keep_the_directory() {
        assert_eq!(
            default_name(Path::new("dir/pbr322.gb"), RecordFormat::GenBank),
            Path::new("dir/pbr322.edited.gb"),
            "GenBank in, GenBank out beside it"
        );
        // A FASTA file takes the shorter suffix, and a bare name stays bare: no
        // directory is invented for a file that was opened with none.
        assert_eq!(
            default_name(Path::new("x.fasta"), RecordFormat::Fasta),
            Path::new("x.edited.fa")
        );
        // Only the last extension comes off, so a name with two dots keeps the
        // first of them.
        assert_eq!(
            default_name(Path::new("a.b.gb"), RecordFormat::GenBank),
            Path::new("a.b.edited.gb")
        );
    }
}
