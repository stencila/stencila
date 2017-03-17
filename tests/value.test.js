import test from 'tape'

import {type, pack, unpack} from '../src/value'

test('value.type', t => {
  t.equal(type(null), 'null')

  t.equal(type(true), 'boolean')
  t.equal(type(false), 'boolean')

  t.equal(type(42), 'integer')
  t.equal(type(1000000000), 'integer')
  t.equal(type(1.1e20), 'integer')

  t.equal(type(3.14), 'float')
  t.equal(type(Math.PI), 'float')
  t.equal(type(1.1e-20), 'float')

  t.equal(type(''), 'string')
  t.equal(type('Yo!'), 'string')

  t.equal(type([]), 'array')
  t.equal(type([1, 2, 3]), 'array')

  t.equal(type({}), 'object')
  t.equal(type({a: 1, b: 2}), 'object')

  t.equal(type({type: 'table'}), 'table')
  t.equal(type({type: 'table', data: {a: 1, b: 2}}), 'table')

  t.equal(type({type: 'html', content: '<img>'}), 'html')
  t.equal(type({type: 'vegalite', data: null}), 'vegalite')

  t.equal(type(function () {}), 'unknown')

  t.end()
})

function check (t, value, type, format, content) {
  let p = pack(value)
  t.equal(p.type, type)
  t.equal(p.format, format)
  t.equal(p.content, content)
}

test('value.pack works for primitive types', t => {
  check(t, null, 'null', 'text', 'null')

  check(t, true, 'boolean', 'text', 'true')
  check(t, false, 'boolean', 'text', 'false')

  check(t, 42, 'integer', 'text', '42')
  check(t, 1000000000, 'integer', 'text', '1000000000')

  check(t, 3.14, 'float', 'text', '3.14')
  check(t, Math.PI, 'float', 'text', '3.141592653589793')

  check(t, 1.1e20, 'integer', 'text', '110000000000000000000')
  check(t, 1.1e-20, 'float', 'text', '1.1e-20')

  check(t, '', 'string', 'text', '')
  check(t, 'Yo!', 'string', 'text', 'Yo!')

  check(t, function () {}, 'unknown', 'text', 'function () {}')

  t.end()
})

test('value.pack works for Objectects', t => {
  check(t, {}, 'object', 'json', '{}')
  check(t, {a: 1, b: 3.14, c: 'foo', d: {e: 1, f: 2}}, 'object', 'json', '{"a":1,"b":3.14,"c":"foo","d":{"e":1,"f":2}}')
  t.end()
})

test('value.pack works for Arrayays', t => {
  check(t, [], 'array', 'json', '[]')
  check(t, [1, 2, 3, 4], 'array', 'json', '[1,2,3,4]')
  check(t, [1.1, 2.1], 'array', 'json', '[1.1,2.1]')
  t.end()
})

test('value.pack works for custom types', t => {
  let p = pack({
    type: 'vegalite',
    data: {
      values: [{'x': 1, 'y': 1}]
    },
    mark: 'pointeger',
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
  t.equal(p.content, '{"type":"vegalite","data":{"values":[{"x":1,"y":1}]},"mark":"pointeger","encoding":{"x":{"field":"x","type":"quantitative"},"y":{"field":"y","type":"quantitative"}}}')
  t.end()
})

test('value.unpack can take a list or a JSON stringing', t => {
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

  t.equal(unpack({type: 'boolean', format: 'text', content: 'true'}), true)
  t.equal(unpack({type: 'boolean', format: 'text', content: 'false'}), false)

  t.equal(unpack({type: 'integer', format: 'text', content: '42'}), 42)
  t.equal(unpack({type: 'integer', format: 'text', content: '1000000000'}), 1000000000)

  t.equal(unpack({type: 'float', format: 'text', content: '3.12'}), 3.12)
  t.equal(unpack({type: 'float', format: 'text', content: '1e20'}), 1e20)

  t.equal(unpack({type: 'string', format: 'text', content: 'Yo!'}), 'Yo!')

  t.end()
})

test('value.unpack works for objectects', t => {
  t.deepEqual(unpack({type: 'object', format: 'json', content: '{}'}), {})
  t.deepEqual(unpack({type: 'object', format: 'json', content: '{"a":1,"b":"foo","c":[1,2,3]}'}), {a: 1, b: 'foo', c: [1, 2, 3]})
  t.end()
})

test('value.unpack works for arrayays', t => {
  t.deepEqual(unpack({type: 'array', format: 'json', content: '[]'}), [])
  t.deepEqual(unpack({type: 'array', format: 'json', content: '[1,2,3,4,5]'}), [1, 2, 3, 4, 5])
  t.end()
})

test('value.unpack works for custom types', t => {
  t.deepEqual(unpack({type: 'html', format: 'json', content: '{"type":"html", "content":"<img>"}'}), {type: 'html', content: '<img>'})
  t.end()
})
