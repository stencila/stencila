// JavaScript I/O path shapes that static analysis should and should not resolve.

import { readFileSync, writeFileSync } from "node:fs";

// Module constant.
const INPUT = "data/input.csv";

// Output written at the end of the script.
const OUTPUT = "results/js-summary.csv";

// Literal collection iterated below.
const SOURCES = ["data/first.csv", "data/second.csv"];

// Base segment used to build a template path.
const BASE = "data";

// One level of helper indirection.
function load(path) {
  return readFileSync(path, "utf8");
}

// Module constant.
let text = readFileSync(INPUT, "utf8");

// Iteration over a literal collection.
for (const source of SOURCES) {
  text += readFileSync(source, "utf8");
}

// Single-assignment local.
const local = "data/local.csv";
text += readFileSync(local, "utf8");

// Fully resolvable template.
text += readFileSync(`${BASE}/template.csv`, "utf8");

// One level of helper function.
text += load("data/helper.csv");

// Negative: assigned conditionally.
let conditional;
if (text.length > 0) {
  conditional = "data/if-branch.csv";
} else {
  conditional = "data/else-branch.csv";
}
text += readFileSync(conditional, "utf8");

// Negative: bound to an expression rather than a literal.
const computed = `${text.slice(0, 3)}.csv`;
text += readFileSync(computed, "utf8");

writeFileSync(OUTPUT, text);
