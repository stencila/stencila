/**
 * The type for the name of a node type
 */
export type NodeType = string

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
  | 'read'
  | 'comment'
  | 'suggest'
  | 'input'
  | 'code'
  | 'edit'
  | 'write'
  | 'admin'

/**
 * The type of document view
 *
 * These are ordered in rough order of increasing document access
 * i.e `static` and `print` only require `read` access whereas
 * `visual` requires at least `edit` access.
 */
export type DocumentView =
  | 'static'
  | 'live'
  | 'dynamic'
  | 'source'
  | 'split'
  | 'visual'
