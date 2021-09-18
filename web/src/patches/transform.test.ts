import { applyTransformElem, applyTransformString } from './transform'

test('applyTransformString', () => {
  const container = document.createElement('div')
  container.innerHTML = 'string'
  const text = container.childNodes[0] as Text

  applyTransformString(text, 'String', 'Emphasis')
  expect(container.innerHTML).toEqual('<em>string</em>')

  expect(() => applyTransformString(text, 'Foo', '')).toThrow(
    /Expected transform from type String, got Foo/
  )
  expect(() => applyTransformString(text, 'String', 'Foo')).toThrow(
    /Unexpected transform to type Foo/
  )
})

test('applyTransformElem', () => {
  const container = document.createElement('div')
  container.innerHTML = '<em>content</em>'

  applyTransformElem(
    container.querySelector('em') as Element,
    'Emphasis',
    'Strong'
  )
  expect(container.innerHTML).toEqual('<strong>content</strong>')

  applyTransformElem(
    container.querySelector('strong') as Element,
    'Strong',
    'Subscript'
  )
  expect(container.innerHTML).toEqual('<sub>content</sub>')
})
