#!/usr/bin/env node

import { Codec, type DecodeInfo, type EncodeInfo, Plugin } from "@stencila/plugin";
import type { Node } from "@stencila/types";

import { decode } from "./decode/index.js";
import { encode } from "./encode/index.js";

/**
 * An codec for decoding from MyST to Stencila Schema nodes
 */
class MySTCodec extends Codec {
  fromString(content: string): [Node, DecodeInfo] {
    return decode(content);
  }

  toString(node: Node): [string, EncodeInfo] {
    return encode(node)
  }
}

/**
 * An example Stencila plugin
 */
class MySTPlugin extends Plugin {
  constructor() {
    super();

    // @ts-expect-error TODO: Add `codec` to `Plugin`
    this.codecs = {
      myst: MySTCodec,
    };
  }
}

if (require.main === module) {
  new MySTPlugin().run().catch(console.error);
}
