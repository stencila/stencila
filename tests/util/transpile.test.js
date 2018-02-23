import test from 'tape'

import transpile from '../../src/shared/transpile'

test('transpile: local variables should not be transpiled', t => {
  let source = 'x + y'
  let map = {}
  let transpiled = transpile(source, map)
  t.equal(Object.keys(map).length, 0, 'no symbols should have been transpiled')
  t.end()
})

test('transpile: local cells should not be transpiled', t => {
  let source = 'x + A10'
  let map = {}
  let transpiled = transpile(source, map)
  t.equal(Object.keys(map).length, 0, 'no symbols should have been transpiled')
  t.end()
})

test('transpile: local cell range', t => {
  let source = 'x + A1:B10'
  let map = {}
  let transpiled = transpile(source, map)
  t.equal(transpiled, 'x + A1_B10', 'source should have been transpiled correctly')
  t.equal(map['A1_B10'], 'A1:B10', 'symbol mapping should have been registered')
  t.equal(transpiled.length, source.length, 'transpiled source should have the same length')
  t.end()
})

test('transpile: remote variable', t => {
  let source = 'x + doc1!z'
  let map = {}
  let transpiled = transpile(source, map)
  t.equal(transpiled, 'x + doc1_z', 'source should have been transpiled correctly')
  t.equal(map['doc1_z'], 'doc1!z', 'symbol mapping should have been registered')
  t.equal(transpiled.length, source.length, 'transpiled source should have the same length')
  t.end()
})

test('transpile: remote variable (document name with spaces)', t => {
  let source = "x + 'My Document'!z"
  let map = {}
  let transpiled = transpile(source, map)
  t.equal(transpiled, 'x + _My_Document__z', 'source should have been transpiled correctly')
  t.equal(map['_My_Document__z'], "'My Document'!z", 'symbol mapping should have been registered')
  t.equal(transpiled.length, source.length, 'transpiled source should have the same length')
  t.end()
})

test('transpile: remote cell', t => {
  let source = 'x + Sheet1!A1'
  let map = {}
  let transpiled = transpile(source, map)
  t.equal(transpiled, 'x + Sheet1_A1', 'source should have been transpiled correctly')
  t.equal(map['Sheet1_A1'], 'Sheet1!A1', 'symbol mapping should have been registered')
  t.equal(transpiled.length, source.length, 'transpiled source should have the same length')
  t.end()
})

test('transpile: remote cell (document name with spaces)', t => {
  let source = "x + 'My Sheet'!A1"
  let map = {}
  let transpiled = transpile(source, map)
  t.equal(transpiled, 'x + _My_Sheet__A1', 'source should have been transpiled correctly')
  t.equal(map['_My_Sheet__A1'], "'My Sheet'!A1", 'symbol mapping should have been registered')
  t.equal(transpiled.length, source.length, 'transpiled source should have the same length')
  t.end()
})

test('transpile: remote cell range', t => {
  let source = 'x + Sheet1!A1:B10'
  let map = {}
  let transpiled = transpile(source, map)
  t.equal(transpiled, 'x + Sheet1_A1_B10', 'source should have been transpiled correctly')
  t.equal(map['Sheet1_A1_B10'], 'Sheet1!A1:B10', 'symbol mapping should have been registered')
  t.equal(transpiled.length, source.length, 'transpiled source should have the same length')
  t.end()
})

test('transpile: remote cell range (document name with spaces)', t => {
  let source = "x + 'My Sheet'!A1:B10"
  let map = {}
  let transpiled = transpile(source, map)
  t.equal(transpiled, 'x + _My_Sheet__A1_B10', 'source should have been transpiled correctly')
  t.equal(map['_My_Sheet__A1_B10'], "'My Sheet'!A1:B10", 'symbol mapping should have been registered')
  t.equal(transpiled.length, source.length, 'transpiled source should have the same length')
  t.end()
})

test('transpile: crazy document name', t => {
  let source = "x + 'My @heet i$ sup4r aw3s0m3!!!'!A1"
  let map = {}
  let transpiled = transpile(source, map)
  t.equal(transpiled, 'x + _My__heet_i__sup4r_aw3s0m3_____A1', 'source should have been transpiled correctly')
  t.equal(map['_My__heet_i__sup4r_aw3s0m3_____A1'], "'My @heet i$ sup4r aw3s0m3!!!'!A1", 'symbol mapping should have been registered')
  t.equal(transpiled.length, source.length, 'transpiled source should have the same length')
  t.end()
})
