import { Node } from "@stencila/types";

/**
 * Information collected during decoding MyST to Stencila Schema nodes
 * 
 * This partially mirrors the Rust struct in `rust/codec-info/src/lib.rs`.
 */
export interface DecodeInfo {
  // TODO: Add `mapping` and `losses`s
}

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
    return [content, {}]
  }
}
