/**
 * Compiles a `tmGrammar.json` file for Stencila DocSQL by converting
 * `tmGrammar.yaml` to JSON
 */

const { readFileSync, writeFileSync } = require("fs");
const path = require("path");
const yaml = require("js-yaml");

// Read in the YAML grammar
const grammar = yaml.load(readFileSync(path.join(__dirname, "tmGrammar.yaml")));

// Write to JSON
writeFileSync(
  path.join(__dirname, "tmGrammar.json"),
  JSON.stringify(grammar, null, " ")
);
