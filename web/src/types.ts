/**
 * The type for a node identifier
 */
export type NodeId = string

/**
 * The type for a document identifier
 */
export type DocumentId = string

/**
 * The access level that a client has for a document
 */
export type DocumentAccess =
  | "read"
  | "comment"
  | "suggest"
  | "input"
  | "code"
  | "edit"
  | "write"
  | "admin";
