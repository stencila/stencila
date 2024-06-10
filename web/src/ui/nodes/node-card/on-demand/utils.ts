import { NodeType } from '@stencila/types'

const NON_CARD_NODES: NodeType[] = [
  'Article',
  'ListItem',
  'Table',
  'TableRow',
  'Text',
] as const

/**
 * Value in pixels, to increment the offset by.
 */
const Y_OFFSET_INCREMENT_VALUE: number = 5

/**
 * The max number of offset increments
 */
const MAX_INCREMENTS: number = 4

export const calculateChipOffset = (
  depth?: number,
  ancestors?: NodeType[]
): number => {
  let offset: number = 0
  if (depth && ancestors && depth > 1) {
    ancestors.forEach((node) => {
      if (
        NON_CARD_NODES.indexOf(node) === -1 &&
        offset < Y_OFFSET_INCREMENT_VALUE * MAX_INCREMENTS
      ) {
        offset += Y_OFFSET_INCREMENT_VALUE
      }
    })

    return offset
  }
  return offset
}
