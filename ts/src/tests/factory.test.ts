import { test, expect } from "vitest";

import {
  Article,
  Organization,
  Paragraph,
  Person,
  article,
  block,
  cord,
  organization,
  paragraph,
  person,
  subscript,
  text,
} from "../index.js";

test("new Article with options", () => {
  const n = article([paragraph([text(cord("Hello world!"))])], {
    authors: [
      person({
        givenNames: ["Alice"],
        familyNames: ["Alvarez"],
        affiliations: [
          organization({
            name: "Aardvark University",
          }),
        ],
      }),
    ],
  });

  expect(n).toBeInstanceOf(Article);

  expect(n.content[0]).toBeInstanceOf(Paragraph);

  expect(n.authors?.[0]).toBeInstanceOf(Person);

  // @ts-expect-error type of author unknown
  expect(n.authors?.[0].affiliations[0]).toBeInstanceOf(Organization);
});

test("new block", () => {
  const p1 = block({
    type: "Paragraph",
    content: [],
  });
  expect(p1).toBeInstanceOf(Paragraph);

  const p2 = block(paragraph([]));
  expect(p2).toBeInstanceOf(Paragraph);

  // @ts-expect-error because it is intentional - it is what we're testing
  expect(() => block(subscript([]))).toThrowErrorMatchingInlineSnapshot(
    `[Error: Unexpected type for Block: Subscript]`,
  );
});
