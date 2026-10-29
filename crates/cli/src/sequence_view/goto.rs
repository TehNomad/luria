//! What one line typed at the go-to prompt means (contract 8k2 §4 rule 1;
//! plan §5.3, D057).
//!
//! This module only reads the text and says which of the three kinds of target
//! it is. Moving the cursor, refusing a residue whose letter does not match and
//! matching a name against the record's features all stay with `super::state`,
//! which owns the record, the cursor and the selection.
//!
//! D057 settles what a bare number is: digits alone are a 1-based base, the
//! numbering the status line shows, whatever CDS is selected, and a residue is
//! always a letter and a number (`D404`). One input has one meaning, so the same
//! text never goes somewhere else because the selection changed. Anything that
//! is neither shape is taken as a feature name and is handed to core's shared
//! matcher (D056), which is why `d404` and `m1` are names: only an ASCII
//! uppercase letter starts the residue form.

/// What one go-to line asks for, as `parse_target` reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// Nothing was typed: go-to changes nothing.
    Nothing,
    /// A 1-based base, as typed (D057). A number too large for `u64` is
    /// `u64::MAX`, which no record reaches and `super::state` refuses.
    Base(u64),
    /// A residue of the selected CDS: an uppercase letter and a 1-based residue
    /// number (D057).
    Residue { letter: char, number: u64 },
    /// Any other text, trimmed: a feature name for `luria_core::name_match`
    /// (D056).
    Name(String),
}

/// The digits of `text`, or `u64::MAX` when they do not fit. The largest number
/// a record can be is far below `u64::MAX` (D048), so a number that does not fit
/// is out of range whatever the record is, and saying so stays with the state.
fn number(text: &str) -> u64 {
    text.parse::<u64>().unwrap_or(u64::MAX)
}

/// One go-to line, read by its shape (rule 1): empty after the whitespace is
/// trimmed is nothing, ASCII digits alone are a base, one ASCII uppercase letter
/// followed by at least one ASCII digit is a residue, and everything else is the
/// trimmed text as a name. The text is otherwise kept as typed: only the
/// whitespace around it is dropped, and nothing is folded or case-changed here —
/// the matcher does its own folding (D056).
pub fn parse_target(text: &str) -> Target {
    let typed: &str = text.trim();
    if typed.is_empty() {
        return Target::Nothing;
    }
    if typed.bytes().all(|byte| byte.is_ascii_digit()) {
        return Target::Base(number(typed));
    }
    if let Some(head) = typed.chars().next() {
        if head.is_ascii_uppercase() {
            let rest: &str = &typed[head.len_utf8()..];
            // A letter alone names no residue: there has to be a number after it,
            // and every digit has to be ASCII, so `404D`, `D4x` and `d404` are
            // text rather than a refused residue.
            if !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit()) {
                return Target::Residue {
                    letter: head,
                    number: number(rest),
                };
            }
        }
    }
    Target::Name(typed.to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_target;
    use super::Target;

    /// Rule 1, read shape by shape (contract 8k2 §5 test 9).
    #[test]
    fn parse_target_by_shape() {
        assert_eq!(parse_target(""), Target::Nothing);
        assert_eq!(parse_target("  "), Target::Nothing);
        assert_eq!(parse_target("1210"), Target::Base(1210));
        assert_eq!(parse_target(" 42 "), Target::Base(42));
        assert_eq!(
            parse_target("D404"),
            Target::Residue {
                letter: 'D',
                number: 404
            }
        );
        // Only an uppercase letter starts a residue; a digit has to follow it.
        assert_eq!(parse_target("d404"), Target::Name("d404".to_string()));
        assert_eq!(parse_target("m1"), Target::Name("m1".to_string()));
        assert_eq!(parse_target("404D"), Target::Name("404D".to_string()));
        assert_eq!(parse_target("D4x"), Target::Name("D4x".to_string()));
        // Too large for u64 is u64::MAX, which no record reaches (D048).
        assert_eq!(
            parse_target("99999999999999999999999"),
            Target::Base(u64::MAX)
        );
    }
}
