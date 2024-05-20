/**
 * An item in the `code-authorship` property describing a run
 * of code having the same authorship history
 */
export type AuthorshipRun = [
  // Start position of run
  number,
  // End position of run
  number,
  // The number of authors
  number,
  // An array of the authors
  number[],
  // A text description of the provenance e.g. MwHe
  string,
  // The machine influence rank of the provenance
  number,
]

/**
 * A marker for an authorship run
 */
export type AuthorshipMarker = {
  from: number
  to: number
  count: number
  provenance: string
  mi: number
}
