import { readdirSync, readFileSync } from "fs";
import path from "path";

import { expect, it } from "vitest";

import { encode } from "./index.js";

// Test all `.json` files in the fixtures directory against the snapshots
const fixtures = path.join(__dirname, "..", "fixtures");
const snapshots = path.join(__dirname, "..", "snapshots");

const files = readdirSync(fixtures).filter(
  (file) => path.extname(file) === ".json"
);

it.each(files)("encodes %s as expected", (file) => {
  const json = readFileSync(path.join(fixtures, file), "utf8");
  const node = JSON.parse(json);
  const [myst, info] = encode(node);

  const snapshotFilename = path.join(snapshots, file.replace(".json", ".md"));
  expect(myst).toMatchFileSnapshot(snapshotFilename);

  const mappingFilename = path.join(snapshots, file.replace(".json", ".map"));
  const mapping = info.mapping
    ?.map(
      (entry) => `${entry.start} ${entry.end} ${entry.nodeType} ${entry.nodeId}`
    )
    .join("\n");
  expect(mapping).toMatchFileSnapshot(mappingFilename);
});
