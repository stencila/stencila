const {pack, unpack} = require('../src/packing')

const test = require('tape')

function check (t, object, type, format, value) {
  let p = pack(object)
  t.equal(p.type, type)
  t.equal(p.format, format)
  t.equal(p.value, value)
}

test('pack works for primitive types', function (t) {
  check(t, null, 'null', 'text', 'null')

  check(t, true, 'bool', 'text', 'true')
  check(t, false, 'bool', 'text', 'false')

  check(t, 42, 'int', 'text', '42')
  check(t, 1000000000, 'int', 'text', '1000000000')

  check(t, 3.14, 'flt', 'text', '3.14')
  check(t, Math.PI, 'flt', 'text', '3.141592653589793')

  check(t, 1.1e20, 'int', 'text', '110000000000000000000')
  check(t, 1.1e-20, 'flt', 'text', '1.1e-20')

  check(t, '', 'str', 'text', '')
  check(t, 'Yo!', 'str', 'text', 'Yo!')

  t.end()
})

test('pack errors for unhandled types', function (t) {
  t.throws(() => pack(function () {}))
  t.end()
})

test('pack works for Objects', function (t) {
  check(t, {}, 'obj', 'json', '{}')
  check(t, {a: 1, b: 3.14, c: 'foo', d: {e: 1, f: 2}}, 'obj', 'json', '{"a":1,"b":3.14,"c":"foo","d":{"e":1,"f":2}}')
  t.end()
})

test('pack works for Arrays', function (t) {
  check(t, [], 'arr', 'json', '[]')
  check(t, [1, 2, 3, 4], 'arr', 'json', '[1,2,3,4]')
  check(t, [1.1, 2.1], 'arr', 'json', '[1.1,2.1]')
  t.end()
})

test('pack works for an array of objects', function (t) {
  check(t, [{a: 1}, {a: 2}, {a: 3}], 'tab', 'csv', 'a\n1\n2\n3')
  check(t, [{a: 1, b: 'x'}, {a: 2, b: 'y'}, {a: 3, b: 'z'}], 'tab', 'csv', 'a,b\n1,x\n2,y\n3,z')
  t.end()
})

test('unpack can take a list or a JSON string', function (t) {
  t.equal(unpack('{"type":"null","format":"text","value":"null"}'), null)
  t.equal(unpack({type: 'null', format: 'text', value: 'null'}), null)
  t.end()
})

test('unpack errors if package is malformed', function (t) {
  t.throws(() => unpack(1), 'should be a list')

  t.throws(() => unpack({}), 'should have fields `type`, `format`, `value`')
  t.throws(() => unpack('{}'))
  t.throws(() => unpack({type: 'null'}))
  t.throws(() => unpack({type: 'null', format: 'text'}))

  t.throws(() => unpack({type: 'foo', format: 'foo', value: 'bar'}))

  t.end()
})

test('unpack works for primitive types', function (t) {
  t.equal(unpack({type: 'null', format: 'text', value: 'null'}), null)

  t.equal(unpack({type: 'bool', format: 'text', value: 'true'}), true)
  t.equal(unpack({type: 'bool', format: 'text', value: 'false'}), false)

  t.equal(unpack({type: 'int', format: 'text', value: '42'}), 42)
  t.equal(unpack({type: 'int', format: 'text', value: '1000000000'}), 1000000000)

  t.equal(unpack({type: 'flt', format: 'text', value: '3.12'}), 3.12)
  t.equal(unpack({type: 'flt', format: 'text', value: '1e20'}), 1e20)

  t.equal(unpack({type: 'str', format: 'text', value: 'Yo!'}), 'Yo!')

  t.end()
})

test('unpack works for objects', function (t) {
  t.deepEqual(unpack({type: 'obj', format: 'json', value: '{}'}), {})
  t.deepEqual(unpack({type: 'obj', format: 'json', value: '{"a":1,"b":"foo","c":[1,2,3]}'}), {a: 1, b: 'foo', c: [1, 2, 3]})
  t.end()
})

test('unpack works for arrays', function (t) {
  t.deepEqual(unpack({type: 'arr', format: 'json', value: '[]'}), [])
  t.deepEqual(unpack({type: 'arr', format: 'json', value: '[1,2,3,4,5]'}), [1, 2, 3, 4, 5])
  t.end()
})

test('unpack works for tabular data', function (t) {
  let result = JSON.stringify([ { a: '1', b: 'x' }, { a: '2', b: 'y' }, { a: '3', b: 'z' } ])
  t.equal(JSON.stringify(unpack({type: 'tab', format: 'csv', value: 'a,b\n1,x\n2,y\n3,z'})), result)
  t.equal(JSON.stringify(unpack({type: 'tab', format: 'tsv', value: 'a\tb\n1\tx\n2\ty\n3\tz'})), result)
  t.throws(() => unpack({type: 'tab', format: 'foo', value: 'bar'}))
  t.end()
})
