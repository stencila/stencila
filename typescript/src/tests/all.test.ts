import { Article } from "../types/Article.js";
import {
  CreativeWork,
  Entity,
  Paragraph,
  Text,
  Thing,
  nodeFrom,
} from "../index.js";

test("constructors", () => {
  const node = new Article([new Paragraph([new Text("Hello world!")])]);

  expect(node).toBeInstanceOf(Article);
  expect(node).toBeInstanceOf(CreativeWork);
  expect(node).toBeInstanceOf(Thing);

  expect(node.content[0]).toBeInstanceOf(Paragraph);
  expect(node.content[0]).toBeInstanceOf(Entity);

  // @ts-expect-error type of block unknown
  expect(node.content[0].content[0]).toBeInstanceOf(Text);
});

test("Article from object", () => {
  const node = Article.from({
    type: "Article",
    content: [
      {
        type: "Paragraph",
        content: [
          {
            type: "Text",
            value: "Hello world!",
          },
        ],
      },
    ],
  } as Article);

  expect(node).toBeInstanceOf(Article);
  expect(node).toBeInstanceOf(CreativeWork);
  expect(node).toBeInstanceOf(Thing);

  expect(node.content[0]).not.toBeInstanceOf(Paragraph);

  // @ts-expect-error type of block unknown
  expect(node.content[0].content[0]).not.toBeInstanceOf(Text);
});

test("Node from object", () => {
  const node = nodeFrom({
    type: "Article",
    content: [
      {
        type: "Paragraph",
        content: [
          {
            type: "Text",
            value: "Hello world!",
          },
        ],
      },
    ],
  }) as Article;

  expect(node).toBeInstanceOf(Article);
  expect(node).toBeInstanceOf(CreativeWork);
  expect(node).toBeInstanceOf(Thing);

  expect(node.content[0]).not.toBeInstanceOf(Paragraph);

  // @ts-expect-error type of block unknown
  expect(node.content[0].content[0]).not.toBeInstanceOf(Text);
});
