import { DecodeInfo } from "@stencila/plugin";
import { Article, Node } from "@stencila/types";
import { mystParse } from "myst-parser";
import { FlowContent } from "myst-spec";
import { basicTransformations } from "myst-transforms";
import { VFile } from "vfile";

import { mdsToBlocks } from "./blocks.js";

/**
 * Decode MyST content to a Stencila Schema `Article`
 */
export function decode(content: string): [Node, DecodeInfo] {
  const root = mystParse(content);
  // Unpack and convert various MyST containers etc to a simpler tree
  basicTransformations(root, new VFile());
  // We currently assume only one top-level Block exists after transform
  const blocks = root.children[0]
    ? mdsToBlocks(root.children[0].children as FlowContent[])
    : [];

  const article = new Article(blocks);
  const info: DecodeInfo = {}; // TODO

  return [article, {}];
}
