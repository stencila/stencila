import { Node, nodeFrom } from "@stencila/types";

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
export async function fromString<T = Node>(
  string: string,
  options?: DecodeOptions
): Promise<T> {
  return nodeFrom(JSON.parse(await fromString_(string, options))) as T;
}

/**
 * Decode a Stencila Schema node from a filesystem path
 * 
 * @param string The path to decode to a node
 * @param options Decoding options
 * @returns A Stencila Schema node
 */
export async function fromPath<T = Node>(
  string: string,
  options?: DecodeOptions
): Promise<T> {
  return nodeFrom(JSON.parse(await fromPath_(string, options))) as T;
}

/**
 * Encode a Stencila Schema node to a string
 * 
 * Use the `format` property of options to specify the destination
 * format (defaults to JSON).
 * 
 * @param node The node to encode
 * @param options Encoding options
 * @returns The string in the format
 */
export async function toString(
  node: Node,
  options?: EncodeOptions
): Promise<string> {
  return await toString_(JSON.stringify(node), options);
}

/**
 * Encode a Stencila Schema node to a filesystem path
 * 
 * @param node The node to encode
 * @param path The path to encode to
 * @param options Encoding options
 */
export async function toPath(
  node: Node,
  path: string,
  options?: EncodeOptions
): Promise<void> {
  return await toPath_(JSON.stringify(node), path, options);
}

export { convert } from "./index";
