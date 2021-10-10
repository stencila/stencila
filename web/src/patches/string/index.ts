import { DomOperation, Slot } from '@stencila/stencila'
import { assert, assertNumber, panic } from '../checks'
import GraphemeSplitter from 'grapheme-splitter'

/**
 * Apply a `DomOperation` to a string
 */
export function applyOp(text: string, op: DomOperation): string {
  const type = op.type
  switch (op.type) {
    case 'Add':
    case 'Remove':
    case 'Replace':
      {
        const address = op.address
        assert(address.length === 1, `Expected address to be of length`)

        const slot = address[0]
        assertNumber(slot)

        switch (type) {
          case 'Add':
            return applyAdd(text, slot, op.html)
          case 'Remove':
            return applyRemove(text, slot, op.items)
          case 'Replace':
            return applyReplace(text, slot, op.items, op.html)
        }
      }
      break
    default:
      throw panic(`Unexpected operation '${type}' for a string`)
  }
}

/**
 * Apply an `Add` operation to a `string`.
 */
export function applyAdd(text: string, slot: Slot, value: string): string {
  assertNumber(slot)

  const graphemes = toGraphemes(text)
  assert(
    slot >= 0 && slot <= graphemes.length,
    `Unexpected add slot '${slot}' for text node of length ${graphemes.length}`
  )

  return (
    graphemes.slice(0, slot).join('') + value + graphemes.slice(slot).join('')
  )
}

/**
 * Apply a `Remove` operation to a `string`.
 */
export function applyRemove(text: string, slot: Slot, items: number): string {
  assertNumber(slot)

  const graphemes = toGraphemes(text)
  assert(
    slot >= 0 && slot <= graphemes.length,
    `Unexpected remove slot '${slot}' for text node of length ${graphemes.length}`
  )
  assert(
    items > 0 && slot + items <= graphemes.length,
    `Unexpected remove items ${items} for text node of length ${graphemes.length}`
  )

  return (
    graphemes.slice(0, slot).join('') + graphemes.slice(slot + items).join('')
  )
}

/**
 * Apply a `Replace` operation to a string
 */
export function applyReplace(
  text: string,
  slot: Slot,
  items: number,
  value: string
): string {
  assertNumber(slot)

  const graphemes = toGraphemes(text)
  assert(
    slot >= 0 && slot <= graphemes.length,
    `Unexpected replace slot '${slot}' for text node of length ${graphemes.length}`
  )
  assert(
    items > 0 && slot + items <= graphemes.length,
    `Unexpected replace items ${items} for text node of length ${graphemes.length}`
  )

  return (
    graphemes.slice(0, slot).join('') +
    value +
    graphemes.slice(slot + items).join('')
  )
}

const GRAPHEME_SPLITTER = new GraphemeSplitter()

/**
 * Split a string into Unicode graphemes
 */
export function toGraphemes(text: string): string[] {
  return GRAPHEME_SPLITTER.splitGraphemes(text)
}
