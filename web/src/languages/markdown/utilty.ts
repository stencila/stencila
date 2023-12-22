import { InlineContext, LeafBlock } from '@lezer/markdown'

/**
 * Find the end point of a leaf block's content.
 * @param leaf `LeafBlock`
 * @returns number
 */
const getLeafEnd = (leaf: LeafBlock): number =>
  leaf.start + leaf.content.trim().length

/**
 * Checks the inline text before the position,
 * return true if the specified delimiter is present without a closing delimiter,
 * otherwise returns false.
 * @param cx `InlineContext`
 * @param pos position within inline
 * @param delimiter delimiter to search for
 */
const hasOpeningDelimitir = (
  cx: InlineContext,
  pos: number,
  openDelim: string,
  closeDelim: string
): boolean => {
  const text = cx.slice(0, pos)
  const indices = []
  let index = text.indexOf(openDelim)
  while (index !== -1) {
    indices.push(index)
    index = text.indexOf(openDelim, index + 1)
  }

  // no open delim exists -> return false
  if (indices.length === 0) {
    return false
  }
  const lastOccurence = indices[indices.length - 1]

  /* 
    if close delim exists between lastOccurence and pos 
    return false
    else true
  */
  return text.indexOf(closeDelim, lastOccurence) === -1
}

export { getLeafEnd, hasOpeningDelimitir }
