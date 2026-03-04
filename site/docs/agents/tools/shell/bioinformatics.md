---
title: "Bioinformatics"
description: "Guards against destructive bioinformatics sequence analysis operations"
---

This page lists the safe and destructive patterns in the **Sequence Tools** shell guard pack. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## Sequence Tools

**Pack ID:** `bioinformatics.sequence_tools`

Guards against destructive bioinformatics sequence analysis operations

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `bioinformatics.sequence_tools.samtools_view` | `^samtools\s+view\b[^\|><]*$` |
| `bioinformatics.sequence_tools.samtools_flagstat` | `^samtools\s+flagstat\b[^\|><]*$` |
| `bioinformatics.sequence_tools.samtools_stats` | `^samtools\s+stats\b[^\|><]*$` |
| `bioinformatics.sequence_tools.samtools_idxstats` | `^samtools\s+idxstats\b[^\|><]*$` |
| `bioinformatics.sequence_tools.samtools_depth` | `^samtools\s+depth\b[^\|><]*$` |
| `bioinformatics.sequence_tools.bcftools_view` | `^bcftools\s+view\b[^\|><]*$` |
| `bioinformatics.sequence_tools.bcftools_query` | `^bcftools\s+query\b[^\|><]*$` |
| `bioinformatics.sequence_tools.bcftools_stats` | `^bcftools\s+stats\b[^\|><]*$` |
| `bioinformatics.sequence_tools.bedtools_intersect` | `^bedtools\s+intersect\b[^\|><]*$` |
| `bioinformatics.sequence_tools.bedtools_coverage` | `^bedtools\s+coverage\b[^\|><]*$` |
| `bioinformatics.sequence_tools.fastqc` | `^fastqc\b[^\|><]*$` |
| `bioinformatics.sequence_tools.multiqc` | `^multiqc\b[^\|><]*$` |
| `bioinformatics.sequence_tools.blastn` | `^blastn\b[^\|><]*$` |
| `bioinformatics.sequence_tools.blastp` | `^blastp\b[^\|><]*$` |
| `bioinformatics.sequence_tools.blastx` | `^blastx\b[^\|><]*$` |
| `bioinformatics.sequence_tools.feature_counts` | `^featureCounts\b[^\|><]*$` |
| `bioinformatics.sequence_tools.htseq_count` | `^htseq-count\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `bioinformatics.sequence_tools.samtools_reheader` | Replaces BAM/CRAM header in-place, potentially corrupting alignment metadata | Backup the file first; verify the new header with `samtools view -H` | Medium |
| `bioinformatics.sequence_tools.bcftools_annotate_remove` | Removes annotations from VCF/BCF files, which may discard important variant metadata | Backup the VCF file first; review annotations with `bcftools query` | Medium |
| `bioinformatics.sequence_tools.bgzip_force` | Force-compresses files, overwriting existing .gz outputs without confirmation | Remove the `-f` flag; rename or backup existing compressed files first | Medium |
| `bioinformatics.sequence_tools.tabix_force` | Force-rebuilds index files, overwriting existing indices | Backup existing index files first | Medium |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/bioinformatics.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/bioinformatics.rs).
