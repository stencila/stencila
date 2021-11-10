import {
  applyRemove,
  applyRemoveOption,
  applyRemoveText,
  applyRemoveVec,
} from './remove'

test('applyRemoveOption', () => {
  const elem = document.createElement('div')
  elem.innerHTML = '<p data-itemprop="property" data-some-attr="">some text</p>'

  applyRemoveOption(elem, 'property', 1)
  expect(elem.innerHTML).toEqual('<p data-itemprop="property"></p>')

  expect(() => applyRemoveOption(elem, 42, 1)).toThrow(/Expected string slot/)
  expect(() => applyRemoveOption(elem, 'property', 42)).toThrow(
    /Unexpected remove items/
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
  expect(() => applyRemoveVec(elem, -1, 1)).toThrow(
    /Unexpected remove slot '-1'/
  )
  expect(() => applyRemoveVec(elem, 100, 1)).toThrow(
    /Unexpected remove slot '100'/
  )
  expect(() => applyRemoveVec(elem, 0, 100)).toThrow(
    /Unexpected remove items 100/
  )
})

test('applyRemoveText', () => {
  const node = document.createTextNode('ab🎁cde')

  applyRemoveText(node, 0, 1)
  expect(node.textContent).toEqual('b🎁cde')

  applyRemoveText(node, 1, 3)
  expect(node.textContent).toEqual('be')

  applyRemoveText(node, 1, 1)
  expect(node.textContent).toEqual('b')

  expect(() => applyRemoveText(node, 'string', 1)).toThrow(
    /Expected number slot/
  )
  expect(() => applyRemoveText(node, -1, 1)).toThrow(
    /Unexpected remove slot '-1'/
  )
  expect(() => applyRemoveText(node, 100, 1)).toThrow(
    /Unexpected remove slot '100'/
  )
  expect(() => applyRemoveText(node, 0, 100)).toThrow(
    /Unexpected remove items 100/
  )
})

test('applyRemove', () => {
  // Start with `Article` with one paragraph with some content
  document.body.innerHTML =
    '<article data-itemscope="root"><div data-itemprop="content"><p>' +
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
    data-itemscope="root"
  >
    <div
      data-itemprop="content"
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
    data-itemscope="root"
  >
    <div
      data-itemprop="content"
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
    data-itemscope="root"
  >
    <div
      data-itemprop="content"
    />
  </article>
</body>
`)
})
