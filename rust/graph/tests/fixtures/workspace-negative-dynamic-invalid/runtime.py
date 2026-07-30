import io
import os
import sys

import pandas as pd

# The path does not exist until the program runs, so nothing may be asserted
# about it. Widening static resolution must not start guessing here.
sample = sys.argv[1]
table = pd.read_csv(f"data/{sample}.csv")
table.to_csv(os.environ["OUTPUT_DIR"] + "/summary.csv")

# A buffer is not a location. Its contents must never become a resource.
buffer = pd.read_csv(io.StringIO("site,count\na,1\n"))
buffer.to_csv(io.BytesIO(b"results/never-written.csv"))
