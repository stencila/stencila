import test from 'tape'

import DocumentRMarkdownConverter from '../../src/document/DocumentRMarkdownConverter'
const converter = new DocumentRMarkdownConverter()


test('DocumentRMarkdownConverter.match', function (t) {
  let match = DocumentRMarkdownConverter.match
  t.ok(match('foo.Rmd'))
  t.ok(match('foo.rmd'))
  t.notOk(match('foo.html'))
  t.end()
})

test('DocumentRMarkdownConverter.importContent', t => {
  const i = (md, options) => converter.importContent(md, options)

  t.equal(
    i('```{r}\nreturn(6*7)\n```'),
    '<div data-cell="global r()"><pre data-source="">return(6*7)</pre></div>'
  )

  t.equal(
    i('```{r fig.width=8}\nplot(1,1)\n```'),
    '<div data-cell="global r()"><pre data-source="">plot(1,1)</pre></div>'
  )

  t.end()
})

