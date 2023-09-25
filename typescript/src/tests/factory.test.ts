import {
  Article,
  Organization,
  Paragraph,
  Person,
  article,
  organization,
  paragraph,
  person,
  text,
} from "../index.js";

test("new Article with options", () => {
  const n = article([paragraph([text("Hello world!")])], {
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
