import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv("../data/raw/observations.csv")
clean = df[["site", "count"]]
mean_count = df.loc[:, "count"].mean()
clean.to_csv("../outputs/python-clean.csv")
plt.savefig("../figures/python-counts.png")
