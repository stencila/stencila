import { Article, Paragraph, Strong, Text } from "@stencila/types";
import * as tmp from "tmp";

import { toPath, toString, fromPath, fromString, fromTo } from "../convert.mjs";

test("fromString", async () => {
  const node = await fromString(
    '{type: "Article", content: [{type: "Paragraph", content: "Hello world"}]}',
    {
      format: "json5",
    }
  );

  expect(node instanceof Article);
  expect(node.content[0] instanceof Paragraph);
  expect(node).toMatchSnapshot();
});

test("fromPath", async () => {
  const node = await fromPath("../examples/paragraph/paragraph.json");

  expect(node instanceof Article);
  expect(node.content[0] instanceof Paragraph);
  expect(node).toMatchSnapshot();
});

test("toString", async () => {
  const node = new Article([
    new Paragraph(["Hello ", new Strong(["again"]), "!"]),
  ]);
  const jats = await toString(node, { format: "jats" });

  expect(jats).toMatchSnapshot();
});

test("toPath", async () => {
  const original = new Article([
    new Paragraph([new Text("Hello file system!")]),
  ]);

  let temp = tmp.fileSync({ postfix: ".jats" }).name;
  await toPath(original, temp);
  const roundTrip = await fromPath(temp);

  expect(roundTrip).toEqual(original);
});

test("fromTo", async () => {
  const md = await fromTo("../examples/paragraph/paragraph.json", null, null, {
    format: "md",
  });
  expect(md).toMatchSnapshot();

  const html = await fromTo(
    "../examples/paragraph/paragraph.json",
    null,
    null,
    {
      format: "html",
      compact: true,
    }
  );
  expect(html).toMatchSnapshot();
});
