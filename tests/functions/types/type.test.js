import test from 'tape'

import type from '../../../src/functions/types/type'
import {type as type_} from '../../../src/value'

test('type', t => {
  t.equal(type(null), type_(null))
  t.equal(type(3.14), type_(3.14))
  t.equal(type([]), type_([]))
  t.equal(type([{}]), type_([{}]))

  t.end()
})
