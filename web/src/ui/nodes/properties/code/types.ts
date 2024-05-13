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
  count: number
  provenance: string
}

export type { ProvenanceMarker, CodeAuthorElement }
