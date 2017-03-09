import test from 'tape'

import {type, pack, unpack} from '../src/value'

test('value.type', t => {
  t.equal(type(null), 'null')

  t.equal(type(true), 'bool')
  t.equal(type(false), 'bool')

  t.equal(type(42), 'int')
  t.equal(type(1000000000), 'int')
  t.equal(type(1.1e20), 'int')

  t.equal(type(3.14), 'flt')
  t.equal(type(Math.PI), 'flt')
  t.equal(type(1.1e-20), 'flt')

  t.equal(type(''), 'str')
  t.equal(type('Yo!'), 'str')

  t.equal(type([]), 'arr')
  t.equal(type([1, 2, 3]), 'arr')

  t.equal(type({}), 'obj')
  t.equal(type({a: 1, b: 2}), 'obj')

  t.equal(type([{}]), 'tab')
  t.equal(type([{a: 1, b: 2}]), 'tab')
  t.equal(type([{a: 1, b: 2}, 'non-an-object']), 'arr')

  t.equal(type({type: 'html', content: '<img>'}), 'html')
  t.equal(type({type: 'vegalite', data: null}), 'vegalite')

  t.equal(type(function () {}), 'unk')

  t.end()
})

function check (t, object, type, format, content) {
  let p = pack(object)
  t.equal(p.type, type)
  t.equal(p.format, format)
  t.equal(p.content, content)
}

test('value.pack works for primitive types', t => {
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

test('value.pack errors for unhandled types', t => {
  t.throws(() => pack(function () {}))
  t.end()
})

test('value.pack works for Objects', t => {
  check(t, {}, 'obj', 'json', '{}')
  check(t, {a: 1, b: 3.14, c: 'foo', d: {e: 1, f: 2}}, 'obj', 'json', '{"a":1,"b":3.14,"c":"foo","d":{"e":1,"f":2}}')
  t.end()
})

test('value.pack works for Arrays', t => {
  check(t, [], 'arr', 'json', '[]')
  check(t, [1, 2, 3, 4], 'arr', 'json', '[1,2,3,4]')
  check(t, [1.1, 2.1], 'arr', 'json', '[1.1,2.1]')
  t.end()
})

test('value.pack works for an array of objects', t => {
  check(t, [{a: 1}, {a: 2}, {a: 3}], 'tab', 'csv', 'a\n1\n2\n3\n')
  check(t, [{a: 1, b: 'x'}, {a: 2, b: 'y'}, {a: 3, b: 'z'}], 'tab', 'csv', 'a,b\n1,x\n2,y\n3,z\n')
  t.end()
})

test('value.pack works for custom types', t => {
  let p = pack({
    type: 'vegalite',
    data: {
      values: [{'x': 1, 'y': 1}]
    },
    mark: 'point',
    encoding: {
      x: {
        field: 'x',
        type: 'quantitative'
      },
      y: {
        field: 'y',
        type: 'quantitative'
      }
    }
  })
  t.equal(p.type, 'vegalite')
  t.equal(p.format, 'json')
  t.equal(p.content, '{"type":"vegalite","data":{"values":[{"x":1,"y":1}]},"mark":"point","encoding":{"x":{"field":"x","type":"quantitative"},"y":{"field":"y","type":"quantitative"}}}')
  t.end()
})

test('value.unpack can take a list or a JSON string', t => {
  t.equal(unpack('{"type":"null","format":"text","content":"null"}'), null)
  t.equal(unpack({type: 'null', format: 'text', content: 'null'}), null)
  t.end()
})

test('value.unpack errors if package is malformed', t => {
  t.throws(() => unpack(1), 'should be a list')

  t.throws(() => unpack({}), 'should have fields `type`, `format`, `content`')
  t.throws(() => unpack('{}'))
  t.throws(() => unpack({type: 'null'}))
  t.throws(() => unpack({type: 'null', format: 'text'}))

  t.throws(() => unpack({type: 'foo', format: 'foo', content: 'bar'}))

  t.end()
})

test('value.unpack works for primitive types', t => {
  t.equal(unpack({type: 'null', format: 'text', content: 'null'}), null)

  t.equal(unpack({type: 'bool', format: 'text', content: 'true'}), true)
  t.equal(unpack({type: 'bool', format: 'text', content: 'false'}), false)

  t.equal(unpack({type: 'int', format: 'text', content: '42'}), 42)
  t.equal(unpack({type: 'int', format: 'text', content: '1000000000'}), 1000000000)

  t.equal(unpack({type: 'flt', format: 'text', content: '3.12'}), 3.12)
  t.equal(unpack({type: 'flt', format: 'text', content: '1e20'}), 1e20)

  t.equal(unpack({type: 'str', format: 'text', content: 'Yo!'}), 'Yo!')

  t.end()
})

test('value.unpack works for objects', t => {
  t.deepEqual(unpack({type: 'obj', format: 'json', content: '{}'}), {})
  t.deepEqual(unpack({type: 'obj', format: 'json', content: '{"a":1,"b":"foo","c":[1,2,3]}'}), {a: 1, b: 'foo', c: [1, 2, 3]})
  t.end()
})

test('value.unpack works for arrays', t => {
  t.deepEqual(unpack({type: 'arr', format: 'json', content: '[]'}), [])
  t.deepEqual(unpack({type: 'arr', format: 'json', content: '[1,2,3,4,5]'}), [1, 2, 3, 4, 5])
  t.end()
})

test('value.unpack works for tabular data', t => {
  let result = JSON.stringify([ { a: '1', b: 'x' }, { a: '2', b: 'y' }, { a: '3', b: 'z' } ])
  t.equal(JSON.stringify(unpack({type: 'tab', format: 'csv', content: 'a,b\n1,x\n2,y\n3,z\n'})), result)
  t.equal(JSON.stringify(unpack({type: 'tab', format: 'tsv', content: 'a\tb\n1\tx\n2\ty\n3\tz\n'})), result)
  t.throws(() => unpack({type: 'tab', format: 'foo', content: 'bar'}))
  t.end()
})

test('value.unpack works for custom types', t => {
  t.deepEqual(unpack({type: 'html', format: 'json', content: '{"type":"html", "content":"<img>"}'}), {type: 'html', content: '<img>'})
  t.end()
})
