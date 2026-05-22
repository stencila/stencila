import { readFileSync, writeFileSync } from "fs-extra";

function summarize() {
  const input = readFileSync("data/raw/samples.tsv", "utf8");
  writeFileSync("results/js-summary.tsv", input.trim() + "\n");
}

summarize();
