//! Codon usage for *Escherichia coli* K-12, used by the `Default` codon
//! selection policy (D011).
//!
//! Source: Codon Usage Database (Kazusa DNA Research Institute), entry
//! "Escherichia coli W3110 [gbbct]", species 316407: 4332 CDSs, 1372057
//! codons, genetic code 1 (standard).
//! <https://www.kazusa.or.jp/codon/cgi-bin/showcodon.cgi?species=316407&aa=1&style=N>
//! Retrieved 2026-10-14. W3110 is a K-12 substrain.
//!
//! The database's entry named "Escherichia coli K12" (species 83333) is built
//! from only 14 CDSs (5122 codons), so the W3110 entry is used instead. Both
//! entries give the same highest-count codon for each of the twenty residues,
//! and so does NEBaseChanger's built-in `Ecoli_K12` table; see
//! `docs/tasks/contracts/milestone-2-mutation.md`, "Comparison with NEBaseChanger codon
//! selection".
//!
//! The table below is the raw codon count column of that entry, in standard
//! table order (base order `T, C, A, G`), with `U` written as `T`. The counts
//! sum to 1372057. Stop codons are included for completeness and are never
//! selected: only a codon that `genetic_code::lookup` maps to a residue can
//! become a `Codon`.

/// Human-readable source of `ECOLI_K12_CODON_COUNTS`, recorded in mutation
/// provenance (D021).
pub const ECOLI_K12_CODON_USAGE_SOURCE: &str =
    "Kazusa Codon Usage Database, Escherichia coli W3110 (K-12), species 316407, retrieved 2026-10-14";

/// Codon counts for *E. coli* K-12 (W3110), in standard table order.
#[rustfmt::skip]
pub const ECOLI_K12_CODON_COUNTS: [(&str, u64); 64] = [
    ("TTT", 30462), ("TTC", 22705), ("TTA", 18894), ("TTG", 18664),
    ("TCT", 11512), ("TCC", 11802), ("TCA", 9620), ("TCG", 12210),
    ("TAT", 22037), ("TAC", 16795), ("TAA", 2765), ("TAG", 321),
    ("TGT", 7016), ("TGC", 8797), ("TGA", 1249), ("TGG", 20889),
    ("CTT", 15082), ("CTC", 15272), ("CTA", 5266), ("CTG", 72898),
    ("CCT", 9540), ("CCC", 7490), ("CCA", 11569), ("CCG", 32080),
    ("CAT", 17791), ("CAC", 13399), ("CAA", 21121), ("CAG", 39835),
    ("CGT", 28866), ("CGC", 30530), ("CGA", 4810), ("CGG", 7401),
    ("ATT", 41644), ("ATC", 34568), ("ATA", 5733), ("ATG", 38167),
    ("ACT", 12119), ("ACC", 32265), ("ACA", 9452), ("ACG", 19820),
    ("AAT", 24106), ("AAC", 29581), ("AAA", 46116), ("AAG", 14174),
    ("AGT", 11924), ("AGC", 22067), ("AGA", 2771), ("AGG", 1496),
    ("GTT", 24991), ("GTC", 21050), ("GTA", 14901), ("GTG", 36108),
    ("GCT", 20813), ("GCC", 35252), ("GCA", 27567), ("GCG", 46524),
    ("GAT", 44217), ("GAC", 26270), ("GAA", 54431), ("GAG", 24629),
    ("GGT", 33875), ("GGC", 40849), ("GGA", 10774), ("GGG", 15115),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genetic_code::TRIPLET_BASES;

    #[test]
    fn the_counts_sum_to_the_published_total() {
        let mut total: u64 = 0;
        for (_codon, count) in ECOLI_K12_CODON_COUNTS.iter() {
            total += count;
        }
        assert!(total == 1372057);
    }

    #[test]
    fn the_table_is_in_standard_table_order() {
        let bases: Vec<char> = TRIPLET_BASES.chars().collect();
        for (index, (codon, _count)) in ECOLI_K12_CODON_COUNTS.iter().enumerate() {
            let expected: String = [bases[index / 16], bases[index / 4 % 4], bases[index % 4]]
                .iter()
                .collect();
            assert!(*codon == expected);
        }
    }
}
