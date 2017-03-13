import test from 'tape'

import converter from '../../src/document/markdown-converter'

test.skip('import', t => {
  const i = (md, options) => converter.import(md, options || {archive: false})

  t.equal(
    i('Para 1\n\nPara 2'),
    '<p>Para 1</p>\n<p>Para 2</p>',
    'returns pretty HTML (might be changed later)'
  )

  t.equal(
    i('Para 1\n\n\nPara 2\n\n  \n  \n    \n# My-header'),
    '<p>Para 1</p>\n<p>Para 2</p>\n<h1 id="my-header">My-header</h1>',
    'paragraphs are "squeezed" i.e. considered empty if it is composed of whitespace characters only'
  )

  t.equal(
    i('My para with [text in the span]{.class .other-class key=val another=example} and after'),
    '<p>My para with <span class="class other-class" data-key="val" data-another="example">text in the span</span> and after</p>',
    'plain bracketed spans work'
  )

  t.equal(
    i('My para with an input [3]{name=variable1} in it'),
    '<p>My para with <span class="class other-class" data-key="val" data-another="example">text in the span</span> and after</p>',
    'plain bracketed spans work'
  )

  t.equal(
    i('# Heading 1'),
    '<h1 id="heading-1">Heading 1</h1>',
    'headings are slugged'
  )

  t.equal(
    i('```\n```'),
    '<pre><code></code></pre>',
    'empty codeblock'
  )

  t.equal(
    i('```r\n```'),
    '<pre><code class="language-r"></code></pre>',
    'codeblock with language'
  )

  t.equal(
    i('```.\nvar2=sum(var1)\n```'),
    '<div data-cell="var2=sum(var1)\n"></div>',
    'internal cell using mini language'
  )

  t.equal(
    i('```run{r}\nlibrary(myawesomepackage)\n```'),
    '<div data-cell="run"><pre data-source="r"><code class="language-r">library(myawesomepackage)\n</code></pre></div>',
    'chunk which runs some R code'
  )

  t.equal(
    i('```call{r}\nreturn(6*7)\n```'),
    '<div data-cell="call"><pre data-source="r"><code class="language-r">return(6*7)\n</code></pre></div>',
    'external cell which calls some R code'
  )

  t.equal(
    i('```out1=call(in1,y=in2){r}\nreturn(6*7)\n```'),
    '<div data-cell="call"><pre data-source="r"><code class="language-r">return(6*7)\n</code></pre></div>',
    'call with inputs and outputs'
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

  t.equal(
    e('<pre><code class="language-r"></code></pre>'),
    '```r\n\n```',
    'codeblock with language'
  )

  t.equal(
    e('<div data-cell="var2=sum(var1)\n"></div>'),
    '```.\nvar2=sum(var1)\n```',
    'internal cell using mini language'
  )

  t.equal(
    e('<div data-cell="run"><pre data-source="r"><code class="language-r">library(myawesomepackage)\n</code></pre></div>'),
    '```run{r}\nlibrary(myawesomepackage)\n```',
    'chunk which runs some R code'
  )

  t.equal(
    e('<div data-cell="call"><pre data-source="r"><code class="language-r">return(6*7)\n</code></pre></div>'),
    '```call{r}\nreturn(6*7)\n```',
    'external cell which calls some R code'
  )

  t.equal(
    e('<div data-cell="call"><pre data-source="r"><code class="language-r">return(6*7)\n</code></pre></div>'),
    '```out1=call(in1,y=in2){r}\nreturn(6*7)\n```',
    'call with inputs and outputs'
  )

  t.end()
})

test.skip('import+export', t => {
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

  ie('```.\nvar2=sum(var1)\n```')
  ie('```run{r}\nlibrary(myawesomepackage)\n```')
  ie('```call{r}\nreturn(6*7)\n```')
  ie('```out1=call(in1,y=in2){r}\nreturn(6*7)\n```')

  t.end()
})
