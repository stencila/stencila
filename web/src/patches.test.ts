import { applyAdd, applyRemove } from './patches'

test('applyAdd', () => {
  document.body.innerHTML =
    '<article slot="root" itemtype="https://schema.org/Article" itemscope></article>'
  expect(document.body).toMatchInlineSnapshot(`
    <body>
      <article
        itemscope=""
        itemtype="https://schema.org/Article"
        slot="root"
      />
    </body>
  `)

  // Add a empty paragraph to the article's content (which was previously `None`)
  applyAdd({
    type: 'Add',
    address: ['content'],
    html: `<div slot="content"><p itemtype="https://stenci.la/Paragraph" itemscope></p></div>`,
    length: 1,
  })
  expect(document.body).toMatchInlineSnapshot(`
    <body>
      <article
        itemscope=""
        itemtype="https://schema.org/Article"
        slot="root"
      >
        <div
          slot="content"
        >
          <p
            itemscope=""
            itemtype="https://stenci.la/Paragraph"
          />
        </div>
      </article>
    </body>
  `)

  // Add a string to the paragraph
  applyAdd({
    type: 'Add',
    address: ['content', 0, 'content', 0],
    html: 'Some text.',
    length: 1,
  })
  expect(document.body).toMatchInlineSnapshot(`
    <body>
      <article
        itemscope=""
        itemtype="https://schema.org/Article"
        slot="root"
      >
        <div
          slot="content"
        >
          <p
            itemscope=""
            itemtype="https://stenci.la/Paragraph"
          >
            Some text.
          </p>
        </div>
      </article>
    </body>
  `)

  // Insert some characters to the string
  applyAdd({
    type: 'Add',
    address: ['content', 0, 'content', 0, 5],
    html: 'more ',
    length: 4,
  })
  expect(document.body).toMatchInlineSnapshot(`
    <body>
      <article
        itemscope=""
        itemtype="https://schema.org/Article"
        slot="root"
      >
        <div
          slot="content"
        >
          <p
            itemscope=""
            itemtype="https://stenci.la/Paragraph"
          >
            Some more text.
          </p>
        </div>
      </article>
    </body>
  `)

  // Insert a string and a `Strong` node before the existing string
  applyAdd({
    type: 'Add',
    address: ['content', 0, 'content', 0],
    html: 'Some <span itemtype="https://stenci.la/Strong" itemscope>strong</span> text. ',
    length: 2,
  })
  expect(document.body).toMatchInlineSnapshot(`
    <body>
      <article
        itemscope=""
        itemtype="https://schema.org/Article"
        slot="root"
      >
        <div
          slot="content"
        >
          <p
            itemscope=""
            itemtype="https://stenci.la/Paragraph"
          >
            Some 
            <span
              itemscope=""
              itemtype="https://stenci.la/Strong"
            >
              strong
            </span>
             text. 
            Some more text.
          </p>
        </div>
      </article>
    </body>
  `)
})

test('applyReplace', () => {
  document.body.innerHTML =
    '<article slot="root" itemtype="https://schema.org/Article" itemscope>' +
    '<div slot="content"><p itemtype="https://stenci.la/Paragraph" itemscope>' +
    'One <span itemtype="https://stenci.la/Strong" itemscope>two</span> three.' +
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
        itemscope=""
        itemtype="https://schema.org/Article"
        slot="root"
      >
        <div
          slot="content"
        >
          <p
            itemscope=""
            itemtype="https://stenci.la/Paragraph"
          >
            One 
            <span
              itemscope=""
              itemtype="https://stenci.la/Strong"
            >
              
            </span>
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
    itemscope=""
    itemtype="https://schema.org/Article"
    slot="root"
  >
    <div
      slot="content"
    >
      <p
        itemscope=""
        itemtype="https://stenci.la/Paragraph"
      >
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
    itemscope=""
    itemtype="https://schema.org/Article"
    slot="root"
  />
</body>
`)
})
