from pathlib import Path

Path("data/raw/S1.fastq").write_text("@S1\nACGT\n+\n!!!!\n")
