import { Article } from "@stencila/types";
import { mystParse } from "myst-parser";
import { mdsToBlocks } from "./blocks";
export function decode(content) {
    const root = mystParse(content);
    const blocks = mdsToBlocks(root.children);
    const article = new Article(blocks);
    const info = {};
    return [article, {}];
}
