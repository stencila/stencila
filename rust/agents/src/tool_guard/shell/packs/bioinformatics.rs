//! Bioinformatics pack: `bioinformatics.sequence_tools`.

use super::{Confidence, Pack, PatternRule, destructive_pattern};

// ---------------------------------------------------------------------------
// bioinformatics.sequence_tools
// ---------------------------------------------------------------------------

pub static SEQUENCE_TOOLS_PACK: Pack = Pack {
    id: "bioinformatics.sequence_tools",
    name: "Sequence Tools",
    description: "Guards against destructive bioinformatics sequence analysis operations",
    destructive_patterns: &[
        destructive_pattern!(
            "samtools_reheader",
            r"\bsamtools\s+reheader\b",
            "Replaces BAM/CRAM header in-place, potentially corrupting alignment metadata",
            "Backup the file first; verify the new header with `samtools view -H`",
            Confidence::Medium
        ),
        destructive_pattern!(
            "bcftools_annotate_remove",
            r"\bbcftools\s+annotate\b.*--remove\b",
            "Removes annotations from VCF/BCF files, which may discard important variant metadata",
            "Backup the VCF file first; review annotations with `bcftools query`",
            Confidence::Medium
        ),
        destructive_pattern!(
            "bgzip_force",
            r"\bbgzip\b.*(?:--force|-f)\b",
            "Force-compresses files, overwriting existing .gz outputs without confirmation",
            "Remove the `-f` flag; rename or backup existing compressed files first",
            Confidence::Medium
        ),
        destructive_pattern!(
            "tabix_force",
            r"\btabix\b.*(?:--force|-f)\b",
            "Force-rebuilds index files, overwriting existing indices",
            "Backup existing index files first",
            Confidence::Medium
        ),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::super::tests::rule_by_id;
    use super::*;

    // -- Sequence Tools --

    #[test]
    fn samtools_reheader_matches() {
        let re = Regex::new(rule_by_id(&SEQUENCE_TOOLS_PACK, "samtools_reheader").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("samtools reheader new_header.sam input.bam"));
        assert!(!re.is_match("samtools view -H input.bam"));
        assert!(!re.is_match("samtools sort input.bam"));
        assert!(!re.is_match("samtools index input.bam"));
    }

    #[test]
    fn bcftools_annotate_remove_matches() {
        let re =
            Regex::new(rule_by_id(&SEQUENCE_TOOLS_PACK, "bcftools_annotate_remove").pattern)
                .expect("pattern should compile");
        assert!(re.is_match("bcftools annotate --remove INFO/DP input.vcf"));
        assert!(re.is_match("bcftools annotate -x INFO/DP --remove FORMAT/GQ input.vcf"));
        assert!(!re.is_match("bcftools annotate -a annotations.vcf input.vcf"));
        assert!(!re.is_match("bcftools view input.vcf"));
        assert!(!re.is_match("bcftools query -f '%CHROM\t%POS\n' input.vcf"));
    }

    #[test]
    fn bgzip_force_matches() {
        let re = Regex::new(rule_by_id(&SEQUENCE_TOOLS_PACK, "bgzip_force").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("bgzip -f input.vcf"));
        assert!(re.is_match("bgzip --force input.vcf"));
        assert!(!re.is_match("bgzip input.vcf"));
        assert!(!re.is_match("bgzip -d input.vcf.gz"));
    }

    #[test]
    fn tabix_force_matches() {
        let re = Regex::new(rule_by_id(&SEQUENCE_TOOLS_PACK, "tabix_force").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("tabix -f input.vcf.gz"));
        assert!(re.is_match("tabix --force input.vcf.gz"));
        assert!(!re.is_match("tabix input.vcf.gz"));
        assert!(!re.is_match("tabix -p bed input.bed.gz"));
    }
}
