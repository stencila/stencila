import test from 'tape'

import { childrenTypes, descendantTypes } from '../src/types'

test('childrenTypes', t => {
  t.deepEqual(childrenTypes['number'], ['integer'])
  t.deepEqual(childrenTypes['table'], [])

  t.end()
})

test('descendantTypes', t => {
  t.deepEqual(descendantTypes['table'], [])

  t.end()
})
