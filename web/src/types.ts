export * from '@stencila/stencila'

export type SessionId = string

export interface Session {
  id: SessionId
}

export interface SessionEvent {
  type: 'Updated' | 'Heartbeat'
  session: Session
}

export type ProjectId = string

export type SnapshotId = string

export type DocumentPath = string

export type DocumentId = string | number

export type JSONValue =
  | string
  | number
  | boolean
  | null
  | JSONValue[]
  | { [key: string | number]: JSONValue }

export type NodeUpdate = {
  id: string
  node: JSONValue
}
