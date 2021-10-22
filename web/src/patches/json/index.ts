import {
  Address,
  Operation,
  OperationAdd,
  OperationMove,
  OperationRemove,
  OperationReplace,
  OperationTransform,
  Patch,
  Slot,
} from '@stencila/stencila'
import { getPatch, PatchItem } from 'fast-array-diff'
import equal from 'fast-deep-equal'
import {
  assert,
  assertArray,
  assertDefined,
  assertIndex,
  assertName,
  assertString,
  isArray,
  isObject,
  isString,
  JsonValue,
  panic,
} from '../checks'
import {
  applyAdd as applyAddString,
  applyRemove as applyRemoveString,
  applyReplace as applyReplaceString,
  diff as diffString,
} from '../string'

/**
 * Generate a `Patch` describing the difference between two JSON values.
 *
 * @why To be able to generate a `Patch` from two JSON values (usually whole
 * documents, or nodes within documents) to send to the server to represent
 * a change to a document.
 *
 * @how Mimics the `diff` function in Rust including (a) for arrays attempting to
 * generate smaller diffs by calling itself when Remove/Add pairs are found,
 * (b) for objects, using `type` to decide whether to replace, or attempt to
 * generate smaller diffs.
 */
export function diff(a: JsonValue, b: JsonValue, address: Address = []): Patch {
  if (isString(a) && isString(b)) {
    return diffString(a, b, address)
  } else if (isArray(a) && isArray(b)) {
    const ops: Operation[] = []
    let prev: PatchItem<JsonValue> | undefined
    for (const curr of getPatch(a, b, equal)) {
      if (curr.type === 'add') {
        if (
          prev?.type === 'remove' &&
          prev?.newPos === curr.newPos &&
          prev.items.length === 1 &&
          curr.items.length === 1
        ) {
          // Replace the previous `Remove` with the inner diff
          ops.pop()
          const inner = diff(prev.items[0] ?? null, curr.items[0] ?? null, [
            ...address,
            prev.oldPos,
          ])
          ops.push(...inner.ops)
        } else {
          ops.push({
            type: 'Add',
            address: [...address, curr.newPos],
            value: curr.items,
            length: curr.items.length,
          })
        }
      } else {
        ops.push({
          type: 'Remove',
          address: [...address, Math.max(0, curr.newPos)],
          items: curr.items.length,
        })
      }
      prev = curr
    }
    return { ops }
  } else if (isObject(a) && isObject(b)) {
    if (a.type !== b.type) {
      return {
        ops: [{ type: 'Replace', address, items: 1, value: b, length: 1 }],
      }
    }

    const ops: Operation[] = []
    for (const key in a) {
      if (key === 'type') continue
      if (key in b) {
        ops.push(
          ...diff(a[key] as JsonValue, b[key] as JsonValue, [...address, key])
            .ops
        )
      } else {
        ops.push({ type: 'Remove', address: [...address, key], items: 1 })
      }
    }
    for (const key in b) {
      if (key === 'type') continue
      if (!(key in a)) {
        ops.push({
          type: 'Add',
          address: [...address, key],
          value: b[key],
          length: 1,
        })
      }
    }
    return { ops }
  } else if (!equal(a, b)) {
    return {
      ops: [{ type: 'Replace', address, items: 1, value: b, length: 1 }],
    }
  } else {
    return { ops: [] }
  }
}

/**
 * Apply a `Patch` to a JSON value.
 */
export function applyPatch(value: JsonValue, patch: Patch): void {
  for (const op of patch.ops) {
    applyOp(value, op)
  }
}

/**
 * Apply a `Operation` to a JSON value.
 */
export function applyOp(value: JsonValue, op: Operation): void {
  switch (op.type) {
    case 'Add':
      return applyAdd(value, op)
    case 'Remove':
      return applyRemove(value, op)
    case 'Replace':
      return applyReplace(value, op)
    case 'Move':
      return applyMove(value, op)
    case 'Transform':
      return applyTransform(value, op)
  }
}

/**
 * Resolve an address into a target value to be modified
 *
 * Also returns the parent value and the key that the target
 * has in the parent. This allows for string replacements.
 */
export function resolveAddress(
  value: JsonValue,
  address: Address
): [JsonValue, Slot | undefined, JsonValue, Slot | undefined] {
  assert(address.length > 0, `Address is unexpectedly empty`)

  let parent: JsonValue = value
  let target: JsonValue = value
  for (let index = 0; index < address.length - 1; index++) {
    parent = target
    const slot = address[index]
    if (isArray(parent)) {
      assertIndex(slot)
      const child = parent[slot]
      assertDefined(child)
      target = child
    } else if (isObject(parent)) {
      assertName(slot)
      const child = parent[slot]
      assertDefined(child)
      target = child
    } else {
      panic(`Unexpected type of value: ${typeof parent}`)
    }
  }
  const key = address[address.length - 2]
  const slot = address[address.length - 1]

  return [parent, key, target, slot]
}

/**
 * Replace a string in a JSON value.
 *
 * This is necessary because JSON strings are immutable so
 * we much replace the string with a new one when we apply operations.
 */
export function replaceString(
  parent: JsonValue,
  key: Slot | undefined,
  value: string
): void {
  if (isArray(parent)) {
    assertIndex(key)
    parent[key] = value
  } else if (isObject(parent)) {
    assertName(key)
    parent[key] = value
  } else {
    panic(`Unexpected type on which to replace a string: ${typeof parent}`)
  }
}

/**
 * Apply an `Add` operation to a JSON value.
 */
export function applyAdd(value: JsonValue, op: OperationAdd): void {
  const { address } = op
  const [parent, key, target, slot] = resolveAddress(value, address)
  const newValue = op.value as JsonValue

  if (isArray(target)) {
    // Adding item/s to an array
    assertIndex(slot)
    assertArray(newValue)
    target.splice(slot, 0, ...newValue)
  } else if (isObject(target)) {
    // Adding a property to an object
    assertName(slot)
    target[slot] = newValue
  } else if (isString(target)) {
    // Add a substring to a string
    assertIndex(slot)
    assertString(newValue)
    replaceString(parent, key, applyAddString(target, slot, newValue))
  } else {
    panic(`Add operation has unexpected target type: ${typeof target}`)
  }
}

/**
 * Apply a `Remove` operation to a JSON value.
 */
export function applyRemove(value: JsonValue, op: OperationRemove): void {
  const { address, items } = op
  const [parent, key, target, slot] = resolveAddress(value, address)

  if (isArray(target)) {
    assertIndex(slot)
    target.splice(slot, items)
  } else if (isObject(target)) {
    assertName(slot)
    assert(
      items === 1,
      `Remove 'items' for object is not equal to one: ${items}`
    )
    delete target[slot]
  } else if (typeof target === 'string') {
    assertIndex(slot)
    replaceString(parent, key, applyRemoveString(target, slot, items))
  }
}

/**
 * Apply a `Replace` operation to a JSON value.
 */
export function applyReplace(value: JsonValue, op: OperationReplace): void {
  const { address, items } = op
  const [parent, key, target, slot] = resolveAddress(value, address)
  const newValue = op.value as JsonValue

  if (isArray(target)) {
    assertIndex(slot)
    assertArray(newValue)
    target.splice(slot, items, ...newValue)
  } else if (isObject(target)) {
    assertName(slot)
    assert(
      items === 1,
      `Replace 'items' for object is not equal to one: ${items}`
    )
    target[slot] = newValue
  } else if (typeof target === 'string') {
    assertIndex(slot)
    assertString(newValue)
    replaceString(
      parent,
      key,
      applyReplaceString(target, slot, items, newValue)
    )
  }
}

/**
 * Apply a `Move` operation to a JSON value.
 */
export function applyMove(_value: JsonValue, _op: OperationMove): void {
  panic('Not yet implemented')
}

/**
 * Apply a `Transform` operation to a JSON value.
 */
export function applyTransform(
  _value: JsonValue,
  _op: OperationTransform
): void {
  panic('Not yet implemented')
}
