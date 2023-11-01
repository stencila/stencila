import * as Benchmark from "benchmark";
import { fromString, toString } from "./convert.js";
import { Article } from "@stencila/types";

const suite = new Benchmark.Suite();

suite
  .add("convert.fromString", {
    fn: async () => {
      await fromString('{"type": "Article", "content": []}');
    },
    minSamples: 100
  })
  .add("convert.toString", {
    fn: async () => {
      await toString(new Article([]));
    },
    minSamples: 100
  })
  .on("cycle", (event: Event) => {
    console.log(String(event.target));
  })
  .run({ async: true });
