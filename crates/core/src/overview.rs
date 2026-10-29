//! The overview scale: the base range each column of the overview inset
//! stands for, and the column a base is drawn in (plan §4.6 of
//! `docs/tasks/milestone-8-view.md`; D054, D006, D048; contract 8g).
//!
//! D054 fixes the shape. Over `L` bases with a strip `N` columns wide there
//! are `w = min(N, L)` columns, and column `i` stands for the half-open range
//! `[⌊i·L/w⌋, ⌊(i+1)·L/w⌋)` (D006). The ranges are in order, tile the record
//! exactly — the first begins at 0, each one begins where the previous ended,
//! the last ends at `L` — and their sizes differ by at most one base, with the
//! larger columns spread through the strip by the floor formula rather than
//! bunched at either end. pBR322's 4361 bases on 80 columns give 41 columns of
//! 55 bases and 39 of 54, interleaved.
//!
//! When `N > L` the strip is drawn narrow (D054): one column per base, `L`
//! columns of one base each, so no column is empty and no base is stretched
//! over two columns. `width == 0` or `length == 0` gives no column at all, and
//! no base has a column; neither input panics.
//!
//! `overview_column` is the inverse, and the closed form
//! `⌊((p + 1)·w − 1)/L⌋` is the same index: writing `b(i) = ⌊i·L/w⌋`, the
//! inequalities `b(i) <= p` and `p < b(i + 1)` each hold exactly when `i` is
//! that index, so it is the only column whose range holds `p`.
//!
//! Everything here is integer arithmetic with `u128` intermediates: no
//! floating point, no dependency, and no input overflows. D048's ceiling of
//! 10 000 000 bases belongs to the reader, not to these functions, which take
//! any `u64`. The module knows nothing about features, hits, the cursor or
//! topology: a feature that crosses the origin is two intervals (D036) and the
//! caller maps each one, which is how it comes to be drawn in both end ranges.

/// The base range `[start, end)` (D006) each overview column stands for, in
/// column order. There are `w = min(width, length)` columns, and column `i`
/// is `[⌊i·length/w⌋, ⌊(i+1)·length/w⌋)` (D054).
pub fn overview_columns(length: u64, width: u64) -> Vec<(u64, u64)> {
    if length == 0 || width == 0 {
        return Vec::new();
    }

    // D054: never more columns than bases, so a short record is drawn narrow
    // and no column comes out empty.
    let columns: u64 = length.min(width);
    let mut ranges: Vec<(u64, u64)> = Vec::new();
    let mut index: u64 = 0;
    while index < columns {
        ranges.push((
            bound(length, columns, index),
            bound(length, columns, index + 1),
        ));
        index += 1;
    }
    ranges
}

/// The column base `position` is drawn in: the one `i` whose range holds it,
/// which is `⌊((position + 1)·w − 1) / length⌋` with `w = min(width, length)`.
/// `None` when `position >= length`, or when `length` or `width` is 0.
pub fn overview_column(length: u64, width: u64, position: u64) -> Option<u64> {
    if length == 0 || width == 0 || position >= length {
        return None;
    }

    let columns: u64 = length.min(width);
    // `position + 1 <= length` and `columns <= length`, so the product fits in
    // `u128` for every `u64` input and no cast truncates.
    let offset: u128 = (position + 1) as u128 * columns as u128 - 1;
    Some((offset / length as u128) as u64)
}

/// The boundary base `index` names in a `columns`-wide row over `length`
/// bases: `⌊index·length / columns⌋` (D054). `index` may equal `columns`,
/// which gives `length`, the end of the row.
fn bound(length: u64, columns: u64, index: u64) -> u64 {
    (index as u128 * length as u128 / columns as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    // Rule 1 and rule 2 on the two rows the plan's formula gives by hand:
    // 10 bases split 3 ways are 3, 3, 4 bases, and split 4 ways 2, 3, 2, 3,
    // the larger column in the middle rather than at the front.
    #[test]
    fn ten_bases_on_three_and_four_columns() {
        let three: Vec<(u64, u64)> = overview_columns(10, 3);
        let want_three: Vec<(u64, u64)> = vec![(0, 3), (3, 6), (6, 10)];
        assert_eq!(three, want_three);

        let four: Vec<(u64, u64)> = overview_columns(10, 4);
        let want_four: Vec<(u64, u64)> = vec![(0, 2), (2, 5), (5, 7), (7, 10)];
        assert_eq!(four, want_four);
    }

    // One column covers the whole record, whatever its length.
    #[test]
    fn one_column_holds_everything() {
        let pbr: Vec<(u64, u64)> = overview_columns(4361, 1);
        assert_eq!(pbr, vec![(0, 4361)]);

        let single: Vec<(u64, u64)> = overview_columns(1, 1);
        assert_eq!(single, vec![(0, 1)]);
    }

    // Rule 3 and D054: more columns than bases draws one column per base, so
    // a column is never empty and a base is never stretched over two columns.
    #[test]
    fn a_short_record_is_drawn_one_column_per_base() {
        let three: Vec<(u64, u64)> = overview_columns(3, 5);
        let want_three: Vec<(u64, u64)> = vec![(0, 1), (1, 2), (2, 3)];
        assert_eq!(three, want_three);
        assert_eq!(overview_column(3, 5, 0), Some(0));
        assert_eq!(overview_column(3, 5, 1), Some(1));
        assert_eq!(overview_column(3, 5, 2), Some(2));

        let one: Vec<(u64, u64)> = overview_columns(1, 80);
        assert_eq!(one, vec![(0, 1)]);

        let sixty: Vec<(u64, u64)> = overview_columns(60, 80);
        assert_eq!(sixty.len(), 60);
        let mut position: u64 = 0;
        while position < 60 {
            assert_eq!(sixty[position as usize], (position, position + 1));
            assert_eq!(overview_column(60, 80, position), Some(position));
            position += 1;
        }
    }

    // D054's worked example, hand-derived in the contract: boundaries
    // ⌊4361·i/80⌋ give 0, 54, 109 … 4306, 4361; 4361 = 80·54 + 41, so 41
    // columns hold 55 bases and 39 hold 54.
    #[test]
    fn pbr322_on_eighty_columns() {
        let columns: Vec<(u64, u64)> = overview_columns(4361, 80);
        assert_eq!(columns.len(), 80);
        assert_eq!(columns[0], (0, 54));
        assert_eq!(columns[1], (54, 109));
        assert_eq!(columns[79], (4306, 4361));

        let mut of_55: usize = 0;
        let mut of_54: usize = 0;
        let mut other: Vec<(u64, u64)> = Vec::new();
        for range in columns.iter() {
            match range.1 - range.0 {
                55 => of_55 += 1,
                54 => of_54 += 1,
                _ => other.push(*range),
            }
        }
        assert_eq!(of_55, 41);
        assert_eq!(of_54, 39);
        assert_eq!(other, Vec::new());

        assert_eq!(overview_column(4361, 80, 0), Some(0));
        assert_eq!(overview_column(4361, 80, 53), Some(0));
        assert_eq!(overview_column(4361, 80, 54), Some(1));
        assert_eq!(overview_column(4361, 80, 4360), Some(79));
    }

    // Rule 4: nothing to draw, and no base to draw.
    #[test]
    fn zero_length_or_width_gives_nothing() {
        assert_eq!(overview_columns(0, 80), Vec::new());
        assert_eq!(overview_columns(4361, 0), Vec::new());
        assert_eq!(overview_column(0, 80, 0), None);
        assert_eq!(overview_column(4361, 0, 0), None);
        assert_eq!(overview_column(4361, 80, 4361), None);
    }

    // Rules 1, 2 and 5 over every small shape, read off the ranges by walking
    // them: the row is checked to start at 0, abut at every join, and end at
    // `length`; the sizes are compared with each other rather than with
    // `length / width`; and each base's column is found by looking for the one
    // range that holds it. The formula is never re-evaluated here, so the
    // sweep checks the rules instead of restating the implementation.
    #[test]
    fn every_small_case_tiles_evenly_and_inverts() {
        let mut length: u64 = 1;
        while length <= 64 {
            let mut width: u64 = 1;
            while width <= 96 {
                let columns: Vec<(u64, u64)> = overview_columns(length, width);
                assert_eq!(
                    columns.len(),
                    length.min(width) as usize,
                    "{} bases on {} columns give {} ranges",
                    length,
                    width,
                    columns.len()
                );

                let mut cursor: u64 = 0;
                let mut smallest: u64 = u64::MAX;
                let mut largest: u64 = 0;
                for range in columns.iter() {
                    assert_eq!(
                        range.0, cursor,
                        "{} bases on {} columns: a range starts at {} where the row is at {}",
                        length, width, range.0, cursor
                    );
                    assert!(
                        range.1 > range.0,
                        "{} bases on {} columns give the empty range ({}, {})",
                        length,
                        width,
                        range.0,
                        range.1
                    );
                    let size: u64 = range.1 - range.0;
                    if size < smallest {
                        smallest = size;
                    }
                    if size > largest {
                        largest = size;
                    }
                    cursor = range.1;
                }
                assert_eq!(
                    cursor, length,
                    "{} bases on {} columns end at {}",
                    length, width, cursor
                );
                assert!(
                    largest - smallest <= 1,
                    "{} bases on {} columns range in size from {} to {}",
                    length,
                    width,
                    smallest,
                    largest
                );

                // Rule 5: exactly one range holds each base, and the inverse
                // names it.
                let mut position: u64 = 0;
                while position < length {
                    let mut holder: Option<usize> = None;
                    for (index, range) in columns.iter().enumerate() {
                        if range.0 <= position && position < range.1 {
                            assert!(
                                holder.is_none(),
                                "{} bases on {} columns: base {} is held twice",
                                length,
                                width,
                                position
                            );
                            holder = Some(index);
                        }
                    }
                    assert_eq!(
                        overview_column(length, width, position),
                        Some(holder.expect("a base must be held by some range") as u64),
                        "base {} of {} bases on {} columns",
                        position,
                        length,
                        width
                    );
                    position += 1;
                }

                width += 1;
            }
            length += 1;
        }
    }

    // Rule 6: the `u128` intermediates. `(u64::MAX · u64::MAX)` is below
    // `u128::MAX`, so both calls answer. `overview_columns(u64::MAX,
    // u64::MAX)` is not called: it would ask for `u64::MAX` ranges.
    #[test]
    fn no_input_overflows() {
        let columns: Vec<(u64, u64)> = overview_columns(u64::MAX, 3);
        assert_eq!(columns.len(), 3);
        assert_eq!(columns[0].0, 0);
        assert_eq!(columns[2].1, u64::MAX);
        let mut cursor: u64 = 0;
        for range in columns.iter() {
            assert_eq!(range.0, cursor);
            assert!(range.1 > range.0);
            cursor = range.1;
        }
        assert_eq!(cursor, u64::MAX);

        assert_eq!(
            overview_column(u64::MAX, u64::MAX, u64::MAX - 1),
            Some(u64::MAX - 1)
        );
    }
}
