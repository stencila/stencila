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
