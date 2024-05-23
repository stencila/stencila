import { readdirSync, readFileSync } from "fs";
import path from "path";

import { expect, it } from "vitest";

import { decode } from "./index.js";

// Automatically test all.md files in the fixtures directory against the snapshots
const fixtures = path.join(__dirname, "..", "fixtures");
const snapshots = path.join(__dirname, "..", "snapshots");

const files = readdirSync(fixtures).filter(
  (file) => path.extname(file) === ".md"
);

it.each(files)("decodes %s as expected", (file) => {
  const myst = readFileSync(path.join(fixtures, file), "utf8");
  const [node, info] = decode(myst);

  const snapshotFilename = path.join(snapshots, file.replace(".md", ".json"));

  const json = JSON.stringify(node, null, "  ");
  expect(json).toMatchFileSnapshot(snapshotFilename);
});
