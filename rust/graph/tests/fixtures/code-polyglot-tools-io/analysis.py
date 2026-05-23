import pandas as pd


def summarize():
    table = pd.read_csv("data/raw/samples.tsv", sep="\t")
    table.to_csv("results/python-summary.tsv", sep="\t", index=False)


summarize()
