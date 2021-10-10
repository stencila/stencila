/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck because the `DomOperationAdd` has incorrect type for `json`

import { applyAdd, applyRemove, applyReplace, diff } from '.'

test('diff:array', () => {
  const arraySimple = [1, 2, 3]
  expect(diff(arraySimple, [1, 2, 3]).ops).toEqual([])
  expect(diff(arraySimple, [1, 3]).ops).toEqual([
    {
      type: 'Remove',
      address: [1],
      items: 1,
    },
  ])
  expect(diff(arraySimple, [1, 2, 4, 5, 3]).ops).toEqual([
    {
      type: 'Add',
      address: [2],
      value: [4, 5],
      length: 2,
    },
  ])

  const arrayNested = [[1], [2], [3]]
  expect(diff(arrayNested, [[1], [2], [3]]).ops).toEqual([])
  expect(diff(arrayNested, [[1], [2, 4], [3]]).ops).toEqual([
    {
      type: 'Add',
      address: [1, 1],
      value: [4],
      length: 1,
    },
  ])
})

test('diff:object', () => {
  const obj = { a: 'foo', b: [1, 2, 3] }
  expect(diff(obj, { a: 'foo', b: [1, 2, 3] }).ops).toEqual([])
  expect(diff(obj, { a: 'foo', b: [1, 3] }).ops).toEqual([
    { type: 'Remove', address: ['b', 1], items: 1 },
  ])
  expect(diff(obj, { a: 'foo' }).ops).toEqual([
    { type: 'Remove', address: ['b'], items: 1 },
  ])
  expect(diff(obj, { a: true, c: 42 }).ops).toEqual([
    { type: 'Replace', address: ['a'], items: 1, value: true, length: 1 },
    { type: 'Remove', address: ['b'], items: 1 },
    { type: 'Add', address: ['c'], value: 42, length: 1 },
  ])
})

test('applyAdd', () => {
  const value = { string: '', array: [], object: {} }

  applyAdd(value, { type: 'Add', address: ['string', 0], json: '' })
  expect(value.string).toEqual('')

  applyAdd(value, { type: 'Add', address: ['string', 0], json: '12' })
  expect(value.string).toEqual('12')

  applyAdd(value, { type: 'Add', address: ['string', 1], json: '34' })
  expect(value.string).toEqual('1342')

  applyAdd(value, { type: 'Add', address: ['array', 0], json: [] })
  expect(value.array).toEqual([])

  applyAdd(value, { type: 'Add', address: ['array', 0], json: [1, 2] })
  expect(value.array).toEqual([1, 2])

  applyAdd(value, { type: 'Add', address: ['array', 1], json: [3, 4] })
  expect(value.array).toEqual([1, 3, 4, 2])

  applyAdd(value, { type: 'Add', address: ['object', 'a'], json: true })
  expect(value.object).toEqual({ a: true })

  applyAdd(value, { type: 'Add', address: ['object', 'b'], json: 'foo' })
  expect(value.object).toEqual({ a: true, b: 'foo' })

  applyAdd(value, { type: 'Add', address: ['object', 'c'], json: 42 })
  expect(value.object).toEqual({ a: true, b: 'foo', c: 42 })

  applyAdd(value, { type: 'Add', address: ['object', 'b', 3], json: 'd' })
  expect(value.object).toEqual({ a: true, b: 'food', c: 42 })
})

test('applyRemove', () => {
  const value = { string: 'abcd', array: [1, 2, 3, 4], object: { a: 1, b: 2 } }

  applyRemove(value, { type: 'Remove', address: ['string', 0], items: 1 })
  expect(value.string).toEqual('bcd')

  applyRemove(value, { type: 'Remove', address: ['string', 1], items: 2 })
  expect(value.string).toEqual('b')

  applyRemove(value, { type: 'Remove', address: ['array', 0], items: 1 })
  expect(value.array).toEqual([2, 3, 4])

  applyRemove(value, { type: 'Remove', address: ['array', 2], items: 1 })
  expect(value.array).toEqual([2, 3])

  applyRemove(value, { type: 'Remove', address: ['object', 'a'], items: 1 })
  expect(value.object).toEqual({ b: 2 })

  applyRemove(value, { type: 'Remove', address: ['object', 'b'], items: 1 })
  expect(value.object).toEqual({})
})

test('applyReplace', () => {
  const value = { string: 'abcd', array: [1, 2, 3, 4], object: { a: 1, b: 2 } }

  applyReplace(value, {
    type: 'Replace',
    address: ['string', 1],
    items: 2,
    json: 'ef',
  })
  expect(value.string).toEqual('aefd')

  applyReplace(value, {
    type: 'Replace',
    address: ['array', 1],
    items: 2,
    json: [5, 6, 7],
  })
  expect(value.array).toEqual([1, 5, 6, 7, 4])

  applyReplace(value, {
    type: 'Replace',
    address: ['object', 'a'],
    items: 1,
    json: false,
  })
  expect(value.object).toEqual({ a: false, b: 2 })
})
