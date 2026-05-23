import { readFileSync, writeFileSync } from "node:fs";
import parse from "csv-parse";

const table = readFileSync("data/raw/samples.tsv", "utf8");
writeFileSync("results/ts-summary.tsv", table);
parse(table);
