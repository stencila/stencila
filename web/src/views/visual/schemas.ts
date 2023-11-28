import { Schema } from "prosemirror-model";

import { blocks } from "./blocks";
import { inlines } from "./inlines";
import { marks } from "./marks";
import { Article } from "./works";


export const article = new Schema({
  nodes: {
    doc: {
      content: "Article*",
    },
    Article,
    ...blocks,
    ...inlines,
  },
  marks,
});
