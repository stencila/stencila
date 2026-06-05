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
 * i.e `static` only requires `read` access whereas `edit`
 * requires write access to a document format.
 */
export type DocumentView = 'static' | 'live' | 'interactive' | 'dynamic' | 'edit'
