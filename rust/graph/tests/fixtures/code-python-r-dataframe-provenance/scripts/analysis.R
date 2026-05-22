library(readr)
library(ggplot2)

df <- read.csv("../data/raw/observations.csv")
df$site
df[["count"]]
write.csv(df, "../outputs/r-summary.csv")
ggsave("../figures/r-counts.png")
