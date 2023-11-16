import { Article, Paragraph, Strong, Text } from "@stencila/types";
import * as tmp from "tmp";

import { toPath, toString, fromPath, fromString, fromTo } from "./convert.js";

test("fromString", async () => {
  const node = await fromString(
    '{type: "Article", content: [{type: "Paragraph", content: [{type: "Text", value: "Hello world"}]}]}',
    {
      format: "json5",
    }
  );

  expect(node instanceof Article);
  expect((node as Article).content[0] instanceof Paragraph);
  expect(JSON.stringify(node, null, " ")).toMatchSnapshot();
});

test("fromPath", async () => {
  const node = await fromPath("../examples/nodes/paragraph/paragraph.json");

  expect(node instanceof Article);
  expect((node as Article).content[0] instanceof Paragraph);
  expect(JSON.stringify(node, null, " ")).toMatchSnapshot();
});

test("toString", async () => {
  const node = new Article([
    new Paragraph([
      new Text("Hello "),
      new Strong([new Text("again")]),
      new Text("!"),
    ]),
  ]);
  const jats = await toString(node, { format: "jats" });

  expect(jats).toMatchSnapshot();
});

test("toPath", async () => {
  const original = new Article([
    new Paragraph([new Text("Hello file system!")]),
  ]);

  const temp = tmp.fileSync({ postfix: ".jats" }).name;
  await toPath(original, temp);
  const roundTrip = await fromPath(temp);

  expect(roundTrip).toEqual(original);
});

test("fromTo", async () => {
  const md = await fromTo(
    "../examples/nodes/paragraph/paragraph.json",
    undefined,
    undefined,
    {
      format: "md",
    }
  );
  expect(md).toMatchSnapshot();

  const html = await fromTo(
    "../examples/nodes/paragraph/paragraph.json",
    undefined,
    undefined,
    {
      format: "html",
      compact: true,
    }
  );
  expect(html).toMatchSnapshot();
});
