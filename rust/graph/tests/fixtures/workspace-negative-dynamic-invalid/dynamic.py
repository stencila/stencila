import pandas as pd

sample = "S1"
table = pd.read_csv(f"data/{sample}.csv")
table.to_csv(f"results/{sample}.csv")
