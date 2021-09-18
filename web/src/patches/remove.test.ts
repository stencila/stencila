import {
  applyRemove,
  applyRemoveOption,
  applyRemoveString,
  applyRemoveVec,
} from './remove'

test('applyRemoveOption', () => {
  const elem = document.createElement('div')
  elem.innerHTML = '<p slot="property"></p>'

  applyRemoveOption(elem, 'property', 1)
  expect(elem.innerHTML).toEqual('')

  expect(() => applyRemoveOption(elem, 42, 1)).toThrow(/Expected string slot/)
  expect(() => applyRemoveOption(elem, 'property', 42)).toThrow(
    /Unexpected remove items/
  )
  expect(() => applyRemoveOption(elem, 'property', 1)).toThrow(
    /Unable to resolve slot property/
  )
})

test('applyRemoveVec', () => {
  const elem = document.createElement('ol')
  elem.innerHTML = '<li>1</li><li>2</li><li>3</li><li>4</li><li>5</li>'

  applyRemoveVec(elem, 0, 1)
  expect(elem.innerHTML).toEqual('<li>2</li><li>3</li><li>4</li><li>5</li>')

  applyRemoveVec(elem, 1, 2)
  expect(elem.innerHTML).toEqual('<li>2</li><li>5</li>')

  applyRemoveVec(elem, 1, 1)
  expect(elem.innerHTML).toEqual('<li>2</li>')

  expect(() => applyRemoveVec(elem, 'string', 1)).toThrow(
    /Expected number slot/
  )
  expect(() => applyRemoveVec(elem, -1, 1)).toThrow(/Unexpected remove slot -1/)
  expect(() => applyRemoveVec(elem, 100, 1)).toThrow(
    /Unexpected remove slot 100/
  )
  expect(() => applyRemoveVec(elem, 0, 100)).toThrow(
    /Unexpected remove items 100/
  )
})

test('applyRemoveString', () => {
  const node = document.createTextNode('abcde')

  applyRemoveString(node, 0, 1)
  expect(node.textContent).toEqual('bcde')

  applyRemoveString(node, 1, 2)
  expect(node.textContent).toEqual('be')

  applyRemoveString(node, 1, 1)
  expect(node.textContent).toEqual('b')

  expect(() => applyRemoveString(node, 'string', 1)).toThrow(
    /Expected number slot/
  )
  expect(() => applyRemoveString(node, -1, 1)).toThrow(
    /Unexpected remove slot -1/
  )
  expect(() => applyRemoveString(node, 100, 1)).toThrow(
    /Unexpected remove slot 100/
  )
  expect(() => applyRemoveString(node, 0, 100)).toThrow(
    /Unexpected remove items 100/
  )
})

test('applyRemove', () => {
  // Start with `Article` with one paragraph with some content
  document.body.innerHTML =
    '<article slot="root"><div slot="content"><p>' +
    'One <strong>two</strong> three.' +
    '</p></div></article>'

  // Remove the three characters of 'two', making the `Strong` empty
  applyRemove({
    type: 'Remove',
    address: ['content', 0, 'content', 1, 'content', 0, 0],
    items: 3,
  })
  expect(document.body).toMatchInlineSnapshot(`
<body>
  <article
    slot="root"
  >
    <div
      slot="content"
    >
      <p>
        One 
        <strong>
          
        </strong>
         three.
      </p>
    </div>
  </article>
</body>
`)

  // Remove the `Strong` and the following word
  applyRemove({
    type: 'Remove',
    address: ['content', 0, 'content', 1],
    items: 2,
  })
  expect(document.body).toMatchInlineSnapshot(`
<body>
  <article
    slot="root"
  >
    <div
      slot="content"
    >
      <p>
        One 
      </p>
    </div>
  </article>
</body>
`)

  // Remove the article content
  applyRemove({
    type: 'Remove',
    address: ['content'],
    items: 1,
  })
  expect(document.body).toMatchInlineSnapshot(`
<body>
  <article
    slot="root"
  />
</body>
`)
})
