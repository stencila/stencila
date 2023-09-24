import { Node, node } from "@stencila/types";

import {
  DecodeOptions,
  EncodeOptions,
  fromPath as fromPath_,
  fromString as fromString_,
  toString as toString_,
  toPath as toPath_,
} from "./index";

export { DecodeOptions, EncodeOptions };

/**
 * Decode a Stencila Schema node from a string
 * 
 * @param string The string to decode to a node
 * @param options Decoding options
 * @returns A Stencila Schema node
 */
export async function fromString(
  string: string,
  options?: DecodeOptions
): Promise<Node> {
  return node(JSON.parse(await fromString_(string, options)));
}

export async function fromPath(
  string: string,
  options?: DecodeOptions
): Promise<Node> {
  return node(JSON.parse(await fromPath_(string, options)));
}

export async function toString(
  node: Node,
  options?: EncodeOptions
): Promise<string> {
  return await toString_(JSON.stringify(node), options);
}

export async function toPath(
  node: Node,
  path: string,
  options?: EncodeOptions
): Promise<void> {
  return await toPath_(JSON.stringify(node), path, options);
}

export { convert } from "./index";
