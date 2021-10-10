import {
  Address,
  DomOperation,
  DomOperationAdd,
  DomOperationMove,
  DomOperationRemove,
  DomOperationReplace,
  DomOperationTransform,
  DomPatch,
  Slot,
} from '@stencila/stencila'
import {
  assert,
  assertArray,
  assertDefined,
  assertIndex,
  assertName,
  assertString,
  isArray,
  isObject,
  JsonValue,
  panic,
} from '../checks'
import {
  applyAdd as applyAddString,
  applyRemove as applyRemoveString,
  applyReplace as applyReplaceString,
} from '../string'

/**
 * Apply a `DomPatch` to a JSON value.
 */
export function applyPatch(value: JsonValue, patch: DomPatch): void {
  for (const op of patch.ops) {
    applyOp(value, op)
  }
}

/**
 * Apply a `DomOperation` to a JSON value.
 */
export function applyOp(value: JsonValue, op: DomOperation): void {
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
export function applyAdd(value: JsonValue, op: DomOperationAdd): void {
  const { address } = op
  const json = op.json as JsonValue
  const [parent, key, target, slot] = resolveAddress(value, address)

  if (isArray(target)) {
    assertIndex(slot)
    assertArray(json)
    target.splice(slot, 0, ...json)
  } else if (isObject(target)) {
    assertName(slot)
    target[slot] = json
  } else if (typeof target === 'string') {
    assertIndex(slot)
    assertString(json)
    replaceString(parent, key, applyAddString(target, slot, json))
  }
}

/**
 * Apply a `Remove` operation to a JSON value.
 */
export function applyRemove(value: JsonValue, op: DomOperationRemove): void {
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
export function applyReplace(value: JsonValue, op: DomOperationReplace): void {
  const { address, items } = op
  const json = op.json as JsonValue
  const [parent, key, target, slot] = resolveAddress(value, address)

  if (isArray(target)) {
    assertIndex(slot)
    assertArray(json)
    target.splice(slot, items, ...json)
  } else if (isObject(target)) {
    assertName(slot)
    assert(
      items === 1,
      `Replace 'items' for object is not equal to one: ${items}`
    )
    target[slot] = json
  } else if (typeof target === 'string') {
    assertIndex(slot)
    assertString(json)
    replaceString(parent, key, applyReplaceString(target, slot, items, json))
  }
}

/**
 * Apply a `Move` operation to a JSON value.
 */
export function applyMove(_value: JsonValue, _op: DomOperationMove): void {
  panic('Not yet implemented')
}

/**
 * Apply a `Transform` operation to a JSON value.
 */
export function applyTransform(
  _value: JsonValue,
  _op: DomOperationTransform
): void {
  panic('Not yet implemented')
}
