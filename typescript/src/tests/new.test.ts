import {
  CreativeWork,
  Article,
  Organization,
  Paragraph,
  Person,
  Text,
  Thing,
} from "../index.js";

test("new Article", () => {
  const n = new Article([new Paragraph([new Text("Hello world!")])]);

  expect(n).toBeInstanceOf(Article);
  expect(n).toBeInstanceOf(CreativeWork);
  expect(n).toBeInstanceOf(Thing);

  expect(n.content[0]).toBeInstanceOf(Paragraph);

  // @ts-expect-error type of block unknown
  expect(n.content[0].content[0]).toBeInstanceOf(Text);
});

test("new Article with options", () => {
  const n = new Article([new Paragraph([new Text("Hello world!")])], {
    authors: [
      new Person({
        givenNames: ["Alice"],
        familyNames: ["Alvarez"],
        affiliations: [
          new Organization({
            name: "Aardvark University",
          }),
        ],
      }),
    ],
  });

  expect(n.authors?.[0]).toBeInstanceOf(Person);

  // @ts-expect-error type of author unknown
  expect(n.authors?.[0].affiliations[0]).toBeInstanceOf(Organization);
});
