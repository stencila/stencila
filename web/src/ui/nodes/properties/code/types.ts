type CodeAuthorElement = [
  number,
  number,
  number,
  [number, number],
  string,
  number,
]

type ProvenanceMarker = {
  from: number
  to: number
  mi: number
  info?: string
}

export type { ProvenanceMarker, CodeAuthorElement }
