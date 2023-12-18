import { LeafBlock } from '@lezer/markdown'

/**
 * Find the end point of a leaf block's content.
 * @param leaf `LeafBlock`
 * @returns number
 */
const getLeafEnd = (leaf: LeafBlock): number =>
  leaf.start + leaf.content.trim().length

export { getLeafEnd }
