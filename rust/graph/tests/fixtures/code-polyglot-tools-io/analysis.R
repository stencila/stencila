library(readr)

table <- read_tsv("data/raw/samples.tsv", show_col_types = FALSE)
write_tsv(table, "results/r-summary.tsv")
