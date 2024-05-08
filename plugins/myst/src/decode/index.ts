import { DecodeInfo } from "@stencila/plugin";
import { Article, Node } from "@stencila/types";
import { mystParse } from "myst-parser";
import type { BlockContent } from "mdast";

import { mdsToBlocks } from "./blocks.js";

/**
 * Decode MyST content to a Stencila Schema `Article`
 */
export function decode(content: string): [Node, DecodeInfo] {
  const root = mystParse(content);
  const blocks = mdsToBlocks(root.children as BlockContent[]);

  const article = new Article(blocks);
  const info: DecodeInfo = {}; // TODO

  return [article, {}];
}

