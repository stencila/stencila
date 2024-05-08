import { DecodeInfo } from "@stencila/plugin";
import { Article, Node } from "@stencila/types";
import { mystParse } from "myst-parser";
import type { BlockContent } from "mdast";

import { mdsToBlocks } from "./blocks.js";

export function decode(content: string): [Node, DecodeInfo] {
  const root = mystParse(content);
  const blocks = mdsToBlocks(root.children as BlockContent[]);

  const article = new Article(blocks);
  const info: DecodeInfo = {};

  return [article, {}];
}

