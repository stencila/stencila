import { NodeType } from "@stencila/types";
import { NodeId, Mapping } from "@stencila/plugin";

/**
 * A context for encoding MyST
 *
 * Analogous to the Rust `MarkdownEncodeContext` at https://github.com/stencila/stencila/blob/53887fb9f12e6e8e2ea83be1650afa205d07362c/rust/codec-markdown-trait/src/lib.rs#L25
 * Maintains the encoding state primarily to be able to map character positions in the encoded
 * MyST to node types and ids.
 */
export class MySTEncodeContext {
  /**
   * The encoded MyST content
   */
  content: string = "";

  /**
   * The UTF8 character position at the end of the content.
   */
  private charIndex: number = 0;

  /**
   * The stack of node types, ids and start position
   */
  nodeStack: [NodeType, NodeId, number][] = [];

  /**
   * The mapping between UTF8 character positions and node type and ids
   */
  mapping: Mapping = [];

  /**
   * Push content onto the content
   *
   * Keeps track of the UTF8 character index at the end of the content
   */
  pushString(value: string): MySTEncodeContext {
    this.content += value;

    for (let i = 0; i < value.length; i++) {
      const codePoint = value.charCodeAt(i);
      if (codePoint <= 0x7f) {
        this.charIndex += 1;
      } else if (codePoint <= 0x7ff) {
        this.charIndex += 2;
      } else if (codePoint >= 0xd800 && codePoint <= 0xdbff) {
        // Surrogate pair means this and the next codeUnit are part of one Unicode character
        this.charIndex += 4; // Surrogate pairs are always 4 bytes in UTF-8
        i++; // Skip the next code unit since it's part of this surrogate pair
      } else {
        this.charIndex += 3;
      }
    }

    return this;
  }

  /**
   * Enter into a node and record its type and id
   */
  enterNode(nodeType: NodeType, nodeId: NodeId): MySTEncodeContext {
    this.nodeStack.push([nodeType, nodeId, this.charIndex]);

    return this;
  }

  /**
   * Exit a node and add its character range to the mapping
   */
  exitNode(): MySTEncodeContext {
    const last = this.nodeStack.pop();
    if (last) {
      const [nodeType, nodeId, start] = last;
      const end = this.charIndex + 1;
      this.mapping.push({ start, end, nodeType, nodeId });
    }

    return this;
  }
}
