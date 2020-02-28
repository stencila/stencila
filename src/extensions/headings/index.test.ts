import {whenReady, select, attr, text, first} from '../../util'

const body = document.body

test('DOM manipulations', async () => {
  body.innerHTML = `
    <h1>The title</h1>
    <h2 data-itemtype="http://schema.stenci.la/Heading">Abstract</h2>

    <h1 itemscope="" itemtype="http://schema.stenci.la/Heading">Heading 1</h1>
    <h2 itemscope="" itemtype="http://schema.stenci.la/Heading">Heading 2</h2>
    <h3 itemscope="" itemtype="http://schema.stenci.la/Heading">Heading 3</h3>
    <h4 itemscope="" itemtype="http://schema.stenci.la/Heading">Heading 4</h4>
    <h5 itemscope="" itemtype="http://schema.stenci.la/Heading">Heading 5</h5>
    <h6 itemscope="" itemtype="http://schema.stenci.la/Heading">Heading 6</h6>
  `

  await import('.')
  whenReady()

  expect(body.innerHTML).toEqual(`
    <h1>The title</h1>
    <h2 data-itemtype="http://schema.stenci.la/Heading">Abstract</h2>

    <h2 itemscope="" itemtype="http://schema.stenci.la/Heading">Heading 1</h2>
    <h3 itemscope="" itemtype="http://schema.stenci.la/Heading">Heading 2</h3>
    <h4 itemscope="" itemtype="http://schema.stenci.la/Heading">Heading 3</h4>
    <h5 itemscope="" itemtype="http://schema.stenci.la/Heading">Heading 4</h5>
    <h6 itemscope="" itemtype="http://schema.stenci.la/Heading">Heading 5</h6>
    <h6 itemscope="" itemtype="http://schema.stenci.la/Heading">Heading 6</h6>
  `)

  expect(text(first('h1') ?? body)).toBe('The title')
  expect(text(first('h2[data-itemtype]') ?? body)).toBe('Abstract')

  expect(text(first('h2[itemtype]') ?? body)).toBe('Heading 1')
  expect(text(first('h3[itemtype]') ?? body)).toBe('Heading 2')
  expect(text(first('h4[itemtype]') ?? body)).toBe('Heading 3')
  expect(text(first('h5[itemtype]') ?? body)).toBe('Heading 4')
  const h6s = select('h6[itemtype]')
  expect(text(h6s[0])).toBe('Heading 5')
  expect(text(h6s[1])).toBe('Heading 6')
})
