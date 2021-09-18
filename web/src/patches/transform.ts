import { DomOperationTransform } from '@stencila/stencila'
import { assertElement, panic, resolveNode } from './utils'

/**
 * Apply a transform operation
 *
 * Transform operations allow for a lightweight diff where only the type
 * of the node has changed. See the `diff_transform` function in `rust/src/patches/inlines.rs`
 * This function should be able to apply all the transforms potentially
 * generated on the server.
 */
export function applyTransform(op: DomOperationTransform): void {
  const { address } = op

  const node = resolveNode(address)
  assertElement(node)

  throw panic('TODO')
}
