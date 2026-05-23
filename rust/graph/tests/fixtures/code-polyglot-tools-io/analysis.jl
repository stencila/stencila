using CSV

table = CSV.File("data/raw/samples.tsv")
CSV.write("results/julia-summary.tsv", table)
