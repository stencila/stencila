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

test('DocumentRMarkdownConverter:importContent', t => {
  const i = md => converter.importContent(md)

  t.equal(
    i('```{r}\nreturn(6*7)\n```'),
    '<div data-cell="global r()"><pre data-source="">return(6*7)</pre></div>'
  )

  t.equal(
    i('```{r fig.width=8}\nplot(1,1)\n```'),
    '<div data-cell="global r()"><pre data-source="">#: fig.width=8\nplot(1,1)</pre></div>'
  )

  t.end()
})

test('DocumentRMarkdownConverter:exportContent', t => {
  const e = html => converter.exportContent(html)

  t.equal(
    e('<div data-cell="global r()"><pre data-source="">return(6*7)</pre></div>'),
    '```{r}\nreturn(6*7)\n```\n'
  )

  t.equal(
    e('<div data-cell="global r()"><pre data-source="">#: fig.width=8\nplot(1,1)</pre></div>'),
    '```{r fig.width=8}\nplot(1,1)\n```\n'
  )

  t.end()
})


test('DocumentRMarkdownConverter:importContent+exportContent', t => {
  const ie = xmdIn => {
    let html = converter.importContent(xmdIn)
    let xmdOut = converter.exportContent(html)
    t.equal(xmdOut, xmdIn)
  }

  ie('```{r}\nx <- 6\nx*7\n```\n')
  ie('```{r fig.width=8, fig.height=10}\nplot(1,1)\n```\n')
  ie(`\`\`\`{r chunk_name, echo=FALSE}
x <- rnorm(100)
y <- 2*x + rnorm(100)
cor(x, y)
\`\`\`
`)

  t.end()
})



