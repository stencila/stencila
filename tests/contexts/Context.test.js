import Context from '../../src/contexts/Context'

import test from 'tape'

test('Context._transpile', t => {
  let c = new Context()
  let symbols

  symbols = {}
  t.equal(c._transpile('A1', symbols), 'A1')
  t.deepEqual(symbols, {
    A1: { type: 'cell', name: 'A1', row: 0, col: 0 }
  })

  symbols = {}
  t.equal(c._transpile('A1:Z9', symbols), 'A1_Z9')
  t.deepEqual(symbols, {
    A1_Z9: { type: 'range', name: 'A1_Z9', startRow: 0, startCol: 0, endRow: 8, endCol: 25 } 
  })

  symbols = {}
  t.equal(c._transpile('A10:Z100', symbols), 'A10_Z100')
  t.deepEqual(symbols, {
    A10_Z100: { type: 'range', name: 'A10_Z100', startRow: 9, startCol: 0, endRow: 99, endCol: 25 }
  })

  symbols = {}
  t.equal(c._transpile('func(A1, A10:Z100, J99:P100)', symbols), 'func(A1, A10_Z100, J99_P100)')
  t.deepEqual(symbols, {
    A1: { type: 'cell', name: 'A1', row: 0, col: 0 }, 
    A10_Z100: { type: 'range', name: 'A10_Z100', startRow: 9, startCol: 0, endRow: 99, endCol: 25 }, 
    J99_P100: { type: 'range', name: 'J99_P100', startRow: 98, startCol: 9, endRow: 99, endCol: 15 }
  })

  t.end()
})
