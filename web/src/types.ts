// The `Node` type allows for values which can be deserialized to the
// Rust `Node` type when the patch is received by the server.
// We could have used the `Node` type in `@stencila/types`
// but that would create another dependency and unnecessary bloat.
export type Node = null | boolean | number | string | Array | Object
type Array = Node[]
type Object = { [key: string]: Node }

export type NodeId = string

export type DocumentId = string

export type DocumentAccess =
  | "read"
  | "comment"
  | "suggest"
  | "input"
  | "code"
  | "edit"
  | "write"
  | "admin";
