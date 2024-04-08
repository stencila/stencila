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
  | 'interactive'
  | 'visual'
  | 'dynamic'
  | 'source'
  | 'split'
  | 'directory'

/**
 * Types of events for updating the main context provider.
 */
export type MainContextEvent =
  | 'stencila-directory-toggle'
  | 'stencila-open-document'
  | 'stencila-close-document'
  | 'stencila-view-change'
  | 'stencila-settings-toggle'
  | 'stencila-config-toggle'

/**
 * Languages that are allowed to execute within Stencila and can have their
 * content displayed in node-cards.
 */

export type AvailableLanguages =
  | 'asciimath'
  | 'bash'
  | 'jinja'
  | 'js'
  | 'javascript'
  | 'latex'
  | 'mathml'
  | 'node'
  | 'py'
  | 'python'
  | 'r'
  | 'rhai'
  | 'shell'
  | 'sql'
  | 'tex'

export type ProgrammingLanguage = {
  displayName: string
  icon: [string, string]
}

export type ProgrammingLanguages = {
  [Name in AvailableLanguages | 'default']: ProgrammingLanguage
}
