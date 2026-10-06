//! Strand: the explicit direction of a feature or CDS.
//!
//! Strand is never inferred. A feature without a strand is a missing
//! attribute, not a default.

#[derive(Clone, Debug, PartialEq)]
pub enum Strand {
    Plus,
    Minus,
}

impl Strand {
    /// The conventional one-character symbol.
    pub fn symbol(self) -> String {
        match self {
            Strand::Plus => "+".to_string(),
            Strand::Minus => "-".to_string(),
        }
    }

    /// True when the strand runs in the direction of increasing coordinates.
    pub fn is_plus(self) -> bool {
        match self {
            Strand::Plus => true,
            Strand::Minus => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strand_has_exactly_two_values() {
        let symbols: Vec<String> = vec![Strand::Plus.symbol(), Strand::Minus.symbol()];
        assert!(symbols == vec!["+".to_string(), "-".to_string()]);
    }

    #[test]
    fn there_is_no_unknown_strand() {
        assert!(Strand::Plus.is_plus());
        assert!(!Strand::Minus.is_plus());
    }
}
