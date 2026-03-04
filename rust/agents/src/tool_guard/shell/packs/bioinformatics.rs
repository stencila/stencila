//! Bioinformatics pack: `bioinformatics.sequence_tools`.

use super::{Confidence, Pack, PatternRule, destructive_pattern, safe_pattern, tokenize_or_bail};

// ---------------------------------------------------------------------------
// bioinformatics.sequence_tools
// ---------------------------------------------------------------------------

/// Validator for `samtools view`: returns `false` if `-o` (output file) is
/// present, since that writes to a file which could overwrite existing data.
fn samtools_view_safe_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, false);
    !tokens.iter().any(|t| t.value == "-o")
}

pub static SEQUENCE_TOOLS_PACK: Pack = Pack {
    id: "bioinformatics.sequence_tools",
    name: "Sequence Tools",
    description: "Guards against destructive bioinformatics sequence analysis operations",
    safe_patterns: &[
        safe_pattern!(
            "samtools_view",
            r"^samtools\s+view\b[^|><]*$",
            samtools_view_safe_validator
        ),
        safe_pattern!("samtools_flagstat", r"^samtools\s+flagstat\b[^|><]*$"),
        safe_pattern!("samtools_stats", r"^samtools\s+stats\b[^|><]*$"),
        safe_pattern!("samtools_idxstats", r"^samtools\s+idxstats\b[^|><]*$"),
        safe_pattern!("samtools_depth", r"^samtools\s+depth\b[^|><]*$"),
        safe_pattern!("bcftools_view", r"^bcftools\s+view\b[^|><]*$"),
        safe_pattern!("bcftools_query", r"^bcftools\s+query\b[^|><]*$"),
        safe_pattern!("bcftools_stats", r"^bcftools\s+stats\b[^|><]*$"),
        safe_pattern!("bedtools_intersect", r"^bedtools\s+intersect\b[^|><]*$"),
        safe_pattern!("bedtools_coverage", r"^bedtools\s+coverage\b[^|><]*$"),
        safe_pattern!("fastqc", r"^fastqc\b[^|><]*$"),
        safe_pattern!("multiqc", r"^multiqc\b[^|><]*$"),
        safe_pattern!("blastn", r"^blastn\b[^|><]*$"),
        safe_pattern!("blastp", r"^blastp\b[^|><]*$"),
        safe_pattern!("blastx", r"^blastx\b[^|><]*$"),
        safe_pattern!("feature_counts", r"^featureCounts\b[^|><]*$"),
        safe_pattern!("htseq_count", r"^htseq-count\b[^|><]*$"),
    ],
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
    fn samtools_view_safe_validator_flags() {
        assert!(samtools_view_safe_validator("samtools view input.bam"));
        assert!(samtools_view_safe_validator("samtools view -h input.bam"));
        assert!(samtools_view_safe_validator("samtools view -b input.sam"));
        assert!(!samtools_view_safe_validator(
            "samtools view -o output.bam input.bam"
        ));
    }

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
        let re = Regex::new(rule_by_id(&SEQUENCE_TOOLS_PACK, "bcftools_annotate_remove").pattern)
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
