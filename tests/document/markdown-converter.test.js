import test from 'tape'

import converter from '../../src/document/markdown-converter'

test('import', t => {
  const i = (md, options) => converter.import(md, options || {archive: false})

  t.equal(
    i('Para 1\n\nPara 2'),
    '<p>Para 1</p>\n<p>Para2</p>',
    'returns pretty HTML (might be changed later)'
  )

  t.equal(
    i('Para 1\n\n\nPara2\n\n  \n  \n    \n# My-header'),
    '<p>Para 1</p>\n<p>Para2</p>\n<h1 id="my-header">My-header</h1>',
    'paragraphs are "squeezed" i.e. considered empty if it is composed of whitespace characters only'
  )

  t.equal(
    i('# Heading 1'),
    '<h1 id="heading-1">Heading 1</h1>',
    'headings are slugged'
  )

  t.deepEqual(
    i('Para 1', {archive: true}),
    {'index.html': '<p>Para 1</p>'},
    'can return an archive (virtual filesystem folder)'
  )

  t.end()
})

test('export', t => {
  const e = converter.export

  t.equal(
    e('<h1 id="heading-1">Heading 1</h1>'),
    '# Heading 1',
    'ATX style headers'
  )

  t.end()
})

test('import+export', t => {
  const ie = mdIn => {
    let html = converter.import(mdIn)
    let mdOut = converter.export(html)
    t.equal(mdOut, mdIn)
  }

  ie('Para 1\n\nPara2')

  ie('# Heading 1')
  ie('## Heading 2')
  ie('### Heading 3')
  ie('#### Heading 4')
  ie('##### Heading 5')

  t.end()
})
