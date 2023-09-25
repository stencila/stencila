import "jest";
import { Article } from "../types/Article.js";
import { Paragraph, Text, blockFrom, nodeFrom } from "../index.js";

test("constructors", () => {
  const doc = new Article([new Paragraph([new Text("Hello world!")])]);

  expect(doc).toBeInstanceOf(Article);
  expect(doc.content[0]).toBeInstanceOf(Paragraph);
  // @ts-expect-error type of block unknown
  expect(doc.content[0].content[0]).toBeInstanceOf(Text);
});

test("Article from object", () => {
  const doc = Article.from({
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
  } as unknown as Article);

  expect(doc).toBeInstanceOf(Article);
  expect(doc.content[0]).not.toBeInstanceOf(Paragraph);
  // @ts-expect-error type of block unknown
  expect(doc.content[0].content[0]).not.toBeInstanceOf(Text);
});

test("Node from object", () => {
  const node = nodeFrom({
    type: "Article",
    content: [],
  }) as Article;

  expect(node).toBeInstanceOf(Article);
  expect(node.type).toBe("Article");
});

test("Block from object", () => {
  const para = blockFrom({
    type: "Paragraph",
    content: [],
  });

  expect(para).toBeInstanceOf(Paragraph);
  expect(para.type).toBe("Paragraph");
});
