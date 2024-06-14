import { Node, NodeType } from "@stencila/types";

export type NodeId = string;

/**
 * An entry in a mapping
 */
export interface MappingEntry {
  /**
   * The index of the UTF-8 character at the start of the node
   */
  start: number;

  /**
   * The index of the UTF-8 character immediately after the end of the node
   */
  end: number;

  /**
   * The type of the node
   */
  nodeType: NodeType;

  /**
   * The id of the node
   */
  nodeId: NodeId;
}

/**
 * A mapping between node type and id and UTF-8 character positions in
 * decoded or encoded content
 */
export type Mapping = MappingEntry[];

/**
 * Information collected during decoding a format to Stencila Schema nodes
 *
 * This partially mirrors the Rust struct in `rust/codec-info/src/lib.rs`.
 */
export interface DecodeInfo {
  // TODO: Add `losses`s
  mapping?: Mapping;
}

/**
 * Information collected during encoding a Stencila Schema node to a format
 *
 * This partially mirrors the Rust struct in `rust/codec-info/src/lib.rs`.
 */
export interface EncodeInfo {
  // TODO: Add `losses`s
  mapping?: Mapping;
}

/**
 * The type for the name of a codec
 */
export type CodecName = string;

/**
 * An encoder-decoder to convert content between a format and Stencila Schema nodes
 *
 * This partially mirrors the Rust trait in `rust/codec/src/lib.rs`.
 */
export abstract class Codec {
  /**
   * Decode a Stencila Schema node from a string
   *
   * This default implementation does nothing and returns the
   * string with no decoding information.
   */
  fromString(content: string): [Node, DecodeInfo] {
    return [content, {}];
  }

  /**
   * Decode a Stencila Schema node from a string
   *
   * This default implementation does nothing and returns the
   * node as JSON.
   */
  toString(node: Node): [string, EncodeInfo] {
    return [JSON.stringify(node, null, "  "), {}];
  }
}
