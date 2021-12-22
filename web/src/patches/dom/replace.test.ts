import {
  applyReplace,
  applyReplaceStruct,
  applyReplaceText,
  applyReplaceVec,
} from './replace'

test('applyReplaceStruct', () => {
  const elem = document.createElement('div')
  elem.innerHTML = '<p data-itemprop="property">One</p>'
  expect(elem.querySelector('[data-itemprop="property"]')?.innerHTML).toEqual(
    'One'
  )

  applyReplaceStruct(elem, 'property', 1, 'Two', 'Two')
  expect(elem.querySelector('[data-itemprop="property"]')?.innerHTML).toEqual(
    'Two'
  )

  applyReplaceStruct(elem, 'property', 1, 'Three', 'Three')
  expect(elem.querySelector('[data-itemprop="property"]')?.innerHTML).toEqual(
    'Three'
  )

  expect(() => applyReplaceStruct(elem, 1, 1, '', '')).toThrow(
    /Expected string slot/
  )
  expect(() => applyReplaceStruct(elem, '', 100, '', '')).toThrow(
    /Unexpected replace items 100/
  )
})

test('applyReplaceVec', () => {
  const elem = document.createElement('ol')
  elem.innerHTML = '<li>1</li><li>2</li><li>3</li><li>4</li><li>5</li>'

  applyReplaceVec(elem, 1, 1, 1, '<li>two</li>')
  expect(elem.innerHTML).toEqual(
    '<li>1</li><li>two</li><li>3</li><li>4</li><li>5</li>'
  )

  applyReplaceVec(elem, 2, 3, 1, '<li>three,four</li>')
  expect(elem.innerHTML).toEqual('<li>1</li><li>two</li><li>three,four</li>')

  applyReplaceVec(elem, 0, 1, 1, '<li>first</li>')
  expect(elem.innerHTML).toEqual(
    '<li>first</li><li>two</li><li>three,four</li>'
  )

  expect(() => applyReplaceVec(elem, 'string', 1, 1, '')).toThrow(
    /Expected number slot/
  )
  expect(() => applyReplaceVec(elem, -1, 1, 1, '')).toThrow(
    /Unexpected replace slot '-1'/
  )
  expect(() => applyReplaceVec(elem, 42, 1, 1, '')).toThrow(
    /Unexpected replace slot '42'/
  )
})

test('applyReplaceText', () => {
  const node = document.createTextNode('abc🎁de')

  applyReplaceText(node, 0, 1, 'x🏳️‍🌈')
  expect(node.textContent).toEqual('x🏳️‍🌈bc🎁de')

  applyReplaceText(node, 1, 6, 'yz')
  expect(node.textContent).toEqual('xyz')

  expect(() => applyReplaceText(node, 'string', 1, '')).toThrow(
    /Expected number slot/
  )
  expect(() => applyReplaceText(node, -1, 1, '')).toThrow(
    /Unexpected replace slot '-1'/
  )
  expect(() => applyReplaceText(node, 42, 1, '')).toThrow(
    /Unexpected replace slot '42'/
  )
  expect(() => applyReplaceText(node, 0, 100, '')).toThrow(
    /Unexpected replace items 100/
  )
})

test('applyReplace', () => {
  // Start with `Article` with one paragraph with some content
  document.body.innerHTML =
    '<article data-itemscope="root"><div data-itemprop="content"><p>' +
    'One <strong>two</strong> three.' +
    '</p></div></article>'

  // Replace one character of 'two'
  applyReplace({
    type: 'Replace',
    address: ['content', 0, 'content', 1, 'content', 0, 1],
    items: 1,
    html: '-',
    value: '-',
    length: 1,
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
          t-o
        </strong>
         three.
      </p>
    </div>
  </article>
</body>
`)

  // Replace the `Strong` and the previous word
  applyReplace({
    type: 'Replace',
    address: ['content', 0, 'content', 0],
    items: 2,
    html: 'one, two',
    value: {},
    length: 1,
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
        one, two
         three.
      </p>
    </div>
  </article>
</body>
`)

  // Replace the article content
  applyReplace({
    type: 'Replace',
    address: ['content'],
    items: 1,
    html: '<p>Hello</p>',
    value: {},
    length: 1,
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
        Hello
      </p>
    </div>
  </article>
</body>
`)
})
