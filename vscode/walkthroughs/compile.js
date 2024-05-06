/**
 * Compiles each sub-folder into a JSON object and inserts them into
 * the `contributes.walkthroughs` of the `package.json`
 *
 * See https://code.visualstudio.com/api/references/contribution-points#contributes.walkthroughs
 */

const { readFileSync, writeFileSync, readdirSync, statSync } = require("fs");
const path = require("path");
const yaml = require("js-yaml");

// Generate the walkthroughs
const walkthroughs = readdirSync(__dirname)
  .filter((entry) => statSync(path.join(__dirname, entry)).isDirectory())
  .map((folder) => folderToWalkthrough(path.join(__dirname, folder)));

// Insert them into package.json
const packagePath = path.join(__dirname, "..", "package.json");
const package = JSON.parse(readFileSync(packagePath, "utf8"));
package.contributes.walkthroughs = walkthroughs;
writeFileSync(packagePath, JSON.stringify(package, null, "  "), "utf8");

// Create a walkthrough from the content of a folder
function folderToWalkthrough(folder) {
  // Read in main
  const walkthrough = yaml.load(readFileSync(path.join(folder, "main.yaml")));
  walkthrough.id = path.basename(folder);

  // Parse step Markdown files and add to it
  walkthrough.steps = readdirSync(folder)
    .filter((file) => path.extname(file) === ".md")
    .map((file) => markdownToStep(path.join(folder, file)));

  return walkthrough;
}

/// Create a walkthrough step from a Markdown file
function markdownToStep(file) {
  const md = readFileSync(file, "utf8");
  const [ignore, header, description, ...sources] = md.split("---");

  const step = yaml.load(header);
  step.id = path.basename(file, ".md");

  if (step.media === undefined) {
    // Each step has to define `media` to show on right.
    step.media = { image: "walkthroughs/blank.svg", altText: "" };
  }

  step.description = description.replace(/\(type\:(\d+)\)/g, (match, index) => {
    const source = sources[index];
    if (source === undefined) {
      throw new Error(`Invalid source index '${index}' in ${file}`);
    }

    // Remove the first and last newlines
    const trimmed = source.replace(/^\n|\n$/g, "");

    const arg = encodeURIComponent(JSON.stringify(trimmed));
    return `(command:stencila.walkthroughType?${arg})`;
  });

  return step;
}
