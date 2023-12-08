import { Schema } from "prosemirror-model";

import * as blocks from "./blocks";
import * as inlines from "./inlines";
import { marks } from "./marks";
import { Article } from "./works";

export const article = {
  schema: new Schema({
    nodes: {
      doc: {
        content: "Article*",
      },
      Article,
      ...blocks.specs,
      ...inlines.specs,
    },
    marks,
  }),

  views: {
    ...blocks.views,
    ...inlines.views,
  },
};
