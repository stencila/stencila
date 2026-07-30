# R I/O path shapes that static analysis should and should not resolve.

# Module constant.
INPUT <- "data/input.csv"

# Output written at the end of the script.
OUTPUT <- "results/r-summary.csv"

# Literal collection iterated below.
SOURCES <- c("data/first.csv", "data/second.csv")

# Base segment used to build a template path.
BASE <- "data"

# One level of helper indirection.
load_table <- function(path) {
  read.csv(path)
}

# Module constant.
input <- read.csv(INPUT)

# Iteration over a literal collection.
for (source_path in SOURCES) {
  extra <- read.csv(source_path)
}

# Single-assignment local.
local_path <- "data/local.csv"
local_table <- read.csv(local_path)

# Fully resolvable template.
templated <- read.csv(paste0(BASE, "/template.csv"))

# One level of helper function.
helper <- load_table("data/helper.csv")

# Path-preserving wrapper constructor.
wrapped <- readLines(file("data/wrapped.csv"))

# Negative: assigned conditionally.
if (nrow(input) > 0) {
  conditional <- "data/if-branch.csv"
} else {
  conditional <- "data/else-branch.csv"
}
conditional_table <- read.csv(conditional)

# Negative: assigned more than once in the same scope.
reassigned <- "data/first-value.csv"
reassigned <- "data/second-value.csv"
reassigned_table <- read.csv(reassigned)

# Negative: bound to an expression rather than a literal.
computed <- paste0(nrow(input), ".csv")
computed_table <- read.csv(computed)

write.csv(input, OUTPUT)
