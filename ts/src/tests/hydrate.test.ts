import { hydrate } from "../hydrate.js";
import { CreativeWork, Article, Paragraph, Text, Thing } from "../index.js";

test("hydrate", () => {
  const n = hydrate({
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

  expect(n.content[0]).toBeInstanceOf(Paragraph);

  // @ts-expect-error type of block unknown
  expect(n.content[0].content[0]).toBeInstanceOf(Text);

  // @ts-expect-error type of block and type of inline unknown
  expect(n.content[0].content[0].value).toBe("Hello world!");
});
