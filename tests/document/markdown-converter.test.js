import test from 'tape'

import converter from '../../src/document/markdown-converter'

test('import', t => {
  const i = converter.import

  t.equal(
    i('# Heading 1', {folder: false}),
    '<h1 id="heading-1">Heading 1</h1>',
    'headings are slugged'
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
