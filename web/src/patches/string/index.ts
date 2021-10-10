import { Operation, DomOperation, Patch, Slot } from '@stencila/stencila'
import { getPatch } from 'fast-array-diff'
import GraphemeSplitter from 'grapheme-splitter'
import { assert, assertIndex, panic } from '../checks'

/**
 * Generate a `Patch` describing the difference between two strings
 */
export function diff(a: string, b: string): Patch {
  const patch = getPatch(toGraphemes(a), toGraphemes(b))
  const ops = patch.map((op): Operation => {
    if (op.type === 'add') {
      return {
        type: 'Add',
        address: [op.newPos],
        value: op.items.join(''),
        length: op.items.length,
      }
    } else {
      return {
        type: 'Remove',
        address: [op.newPos],
        items: op.items.length,
      }
    }
  })
  return { ops }
}

/**
 * Apply a `DomOperation` to a string.
 */
export function applyOp(text: string, op: DomOperation): string {
  const type = op.type
  if (type === 'Add' || type === 'Remove' || type === 'Replace') {
    const address = op.address
    assert(address.length === 1, `Expected address to be of length`)

    const slot = address[0]
    assertIndex(slot)

    switch (type) {
      case 'Add':
        return applyAdd(text, slot, op.html)
      case 'Remove':
        return applyRemove(text, slot, op.items)
      case 'Replace':
        return applyReplace(text, slot, op.items, op.html)
    }
  }

  throw panic(`Unexpected operation '${type}' for a string`)
}

/**
 * Apply an `Add` operation to a string.
 */
export function applyAdd(text: string, slot: Slot, value: string): string {
  assertIndex(slot)

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
 * Apply a `Remove` operation to a string.
 */
export function applyRemove(text: string, slot: Slot, items: number): string {
  assertIndex(slot)

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
  assertIndex(slot)

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
