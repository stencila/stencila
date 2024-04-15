/**
 * Compiles a `tmGrammar.json` file for Stencila Markdown by converting
 * `tmGrammar.yaml` to JSON, expanding certain rules for languages, and
 * saving to JSON.
 */

const { readFileSync, writeFileSync } = require("fs");
const path = require("path");
const yaml = require("js-yaml");

// Read in the YAML grammar
const grammar = yaml.load(readFileSync(path.join(__dirname, "tmGrammar.yaml")));

// Definitions for each of the languages that rules will be expanded for
const langs = [
  {
    name: "javascript",
    aliases: "js|javascript",
    include: "source.js",
    chunks: true,
  },
  {
    name: "python",
    aliases: "py|python",
    include: "source.python",
    chunks: true,
  },
  { name: "r", aliases: "r|R", include: "source.r", chunks: true },
  { name: "sql", aliases: "sql", include: "source.sql", chunks: true },
  {
    name: "tex",
    aliases: "tex",
    include: "text.tex",
    chunks: false,
  },
  {
    name: "latex",
    aliases: "latex",
    include: "text.tex.latex",
    chunks: false,
  },
];

// Expand the `code-block.template` rule for each language
{
  const template = JSON.stringify(grammar.repository["code-block.template"]);
  for (const lang of langs) {
    // Interpolate lang vars into template and add new rule
    const name = "code-block.LANG_NAME".replace("LANG_NAME", lang.name);
    const interpolated = template
      .replace("LANG_NAME", lang.name)
      .replace("LANG_ALIASES", lang.aliases)
      .replace("LANG_INCLUDE", lang.include);
    grammar.repository[name] = JSON.parse(interpolated);

    // Add an reference to the new rule before the `code-block.no-lang` rule
    const patterns = grammar.repository["code-block"].patterns;
    patterns.splice(patterns.length - 1, 0, { include: "#" + name });
  }
  // Remove the template
  delete grammar.repository["code-block.template"];
}

// Expand the `code-chunk.template` rule for each language
{
  const template = JSON.stringify(grammar.repository["code-chunk.template"]);
  for (const lang of langs.filter((lang) => lang.chunks)) {
    // Interpolate lang vars into template and add new rule
    const name = "code-chunk.LANG_NAME".replace("LANG_NAME", lang.name);
    const interpolated = template
      .replace("LANG_NAME", lang.name)
      .replace("LANG_ALIASES", lang.aliases)
      .replace("LANG_INCLUDE", lang.include);
    grammar.repository[name] = JSON.parse(interpolated);

    // Add an reference to the new rule before the `code-chunk.unknown-lang` rule
    const patterns = grammar.repository["code-chunk"].patterns;
    patterns.splice(patterns.length - 2, 0, { include: "#" + name });
  }
  // Remove the template
  delete grammar.repository["code-chunk.template"];
}

// Write to JSON
writeFileSync(
  path.join(__dirname, "tmGrammar.json"),
  JSON.stringify(grammar, null, " ")
);
