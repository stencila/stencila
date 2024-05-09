import { readdirSync, readFileSync } from "fs";
import path from "path";

import { expect, it } from "vitest";

import { decode } from "./index.js";

it("decodes fixtures as expected", () => {
  const fixtures = path.join(__dirname, "..", "fixtures");
  const snapshots = path.join(__dirname, "..", "snapshots");

  for (const file of readdirSync(fixtures).filter(
    (file) => path.extname(file) === ".md"
  )) {
    const myst = readFileSync(path.join(fixtures, file), "utf8");
    const [node, info] = decode(myst);

    const json = JSON.stringify(node, null, "  ");
    const snapshot = path.join(snapshots, file.replace(".md", ".json"));
    expect(json).toMatchFileSnapshot(snapshot);
  }
});
