import { Node, node } from "@stencila/types";

import { DecodeOptions, EncodeOptions } from "../bindings.js";
import * as index from "../bindings.js";

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
  return node(JSON.parse(await index.fromString(string, options))) as T;
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
  return node(JSON.parse(await index.fromPath(string, options))) as T;
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
  return index.toString(JSON.stringify(node), options);
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
  return index.toPath(JSON.stringify(node), path, options);
}

/**
 * Convert a document from one format to another
 *
 * @param node The node to encode
 * @param path The path to encode to
 * @param options Encoding options
 */
export function fromTo(
  input?: string,
  output?: string,
  decodeOptions?: DecodeOptions,
  encodeOptions?: EncodeOptions
): Promise<string> {
  return index.fromTo(input, output, decodeOptions, encodeOptions);
}
