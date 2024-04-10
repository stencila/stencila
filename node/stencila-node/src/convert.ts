import { type Node, node } from "@stencila/types";

// eslint-disable-next-line import/no-unresolved
import { type DecodeOptions, type EncodeOptions } from "./bindings.d.js";
// eslint-disable-next-line import/no-unresolved
import * as bindings from "./bindings.js";

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
  options?: DecodeOptions,
): Promise<T> {
  return node(JSON.parse(await bindings.fromString(string, options))) as T;
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
  options?: DecodeOptions,
): Promise<T> {
  return node(JSON.parse(await bindings.fromPath(string, options))) as T;
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
  options?: EncodeOptions,
): Promise<string> {
  return bindings.toString(JSON.stringify(node), options);
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
  options?: EncodeOptions,
): Promise<void> {
  return bindings.toPath(JSON.stringify(node), path, options);
}

/**
 * Convert a document from one format to another
 *
 * @param node The node to encode
 * @param path The path to encode to
 * @param options Encoding options
 */
export async function fromTo(
  input?: string,
  output?: string,
  decodeOptions?: DecodeOptions,
  encodeOptions?: EncodeOptions,
): Promise<string> {
  return bindings.fromTo(input, output, decodeOptions, encodeOptions);
}
