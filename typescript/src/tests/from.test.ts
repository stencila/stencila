import {
  CreativeWork,
  Article,
  Organization,
  Paragraph,
  Person,
  Text,
  Thing,
  node
} from "../index.js";

test("Article from object", () => {
  const n = Article.from({
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
  });

  expect(n).toBeInstanceOf(Article);
  expect(n).toBeInstanceOf(CreativeWork);
  expect(n).toBeInstanceOf(Thing);

  expect(n.content[0]).not.toBeInstanceOf(Paragraph);

  // @ts-expect-error type of block unknown
  expect(n.content[0].content[0]).not.toBeInstanceOf(Text);
});

test("Node from object", () => {
  const n = node({
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

  expect(n).toBeInstanceOf(Article);
  expect(n).toBeInstanceOf(CreativeWork);
  expect(n).toBeInstanceOf(Thing);

  expect(n.content[0]).not.toBeInstanceOf(Paragraph);

  // @ts-expect-error type of block unknown
  expect(n.content[0].content[0]).not.toBeInstanceOf(Text);
});
