import { applyTransformString } from './transform'

test('applyTransformString', () => {
  const elem = document.createElement('div')
  elem.innerHTML = 'string'
  const text = elem.childNodes[0] as Text

  applyTransformString(text, 'String', 'Emphasis')
  expect(elem.innerHTML).toEqual('<em>string</em>')

  expect(() => applyTransformString(text, 'Foo', '')).toThrow(
    /Expected transform from type String, got Foo/
  )
  expect(() => applyTransformString(text, 'String', 'Foo')).toThrow(
    /Unexpected transform to type Foo/
  )
})
