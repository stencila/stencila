import fs from 'fs'
import path from 'path'
import test from 'tape'

import unified from 'unified'
import remarkParse from 'remark-parse'
import remarkStringify from 'remark-stringify'
import remarkHtml from 'remark-html'
import rehypeParse from 'rehype-parse'
import rehype2remark from 'rehype-remark'
import beautify from 'js-beautify'

import * as include from '../../src/utilities/include'
import * as defaults from '../../src/utilities/markdown-defaults'
import * as markdown from '../../src/utilities/markdown'

test('include', t => {
  function beautifyHTML (input) {
    return beautify.html(input.trim(), {
      'indent_inner_html': false,
      'indent_size': 2,
      'indent_char': ' ',
      'wrap_line_length': 0, // disable wrapping
      'brace_style': 'expand',
      'preserve_newlines': true,
      'max_preserve_newlines': 5,
      'indent_handlebars': false,
      'extra_liners': ['/html']
    })
  }

  function toHTML (input) {
    const output = unified()
      .use(remarkParse)
      .use(include.md2html)
      .use(remarkStringify)
      .use(remarkHtml)
      .process(input, defaults).contents.trim()

    return beautifyHTML(output)
  }

  function toMD (input) {
    return unified()
      .use(rehypeParse)
      .use(include.html2md)
      .use(rehype2remark)
      .use(remarkStringify)
      .use(include.mdVisitors)
      .process(input, defaults).contents.trim()
  }

  t.test('include statment', st => {
    const input = '< address/of/some/other/document.md'
    const expectedHTML = '<div data-include="address/of/some/other/document.md"></div>'
    const outputHTML = toHTML(input)
    const outputMD = toMD(expectedHTML)
    st.equal(outputHTML, expectedHTML)
    st.equal(outputMD, input)
    st.end()
  })

  t.test('versions', st => {
    const input = '< address/of/some/other/document.md@fc453b6'
    const expectedHTML = '<div data-include="address/of/some/other/document.md@fc453b6"></div>'
    const outputHTML = toHTML(input)
    const outputMD = toMD(expectedHTML)
    st.equal(outputHTML, expectedHTML)
    st.equal(outputMD, input)
    st.end()
  })

  t.test('selectors', st => {
    const input = '< address/of/some/other/document.md selector another'
    const expectedHTML = '<div data-include="address/of/some/other/document.md" data-select="selector another"></div>'
    const outputHTML = toHTML(input)
    const outputMD = toMD(expectedHTML)
    st.equal(outputHTML, expectedHTML)
    st.equal(outputMD, input)
    st.end()
  })

  t.test('inputs', st => {
    const input = '< address/of/some/other/document.md (var1=42, var2=var3)'
    const expectedHTML = '<div data-include="address/of/some/other/document.md" data-input="var1=42, var2=var3"></div>'
    const outputHTML = toHTML(input)
    const outputMD = toMD(expectedHTML)
    st.equal(outputHTML, expectedHTML)
    st.equal(outputMD, input)
    st.end()
  })

  t.test('selectors and inputs', st => {
    const input = '< address/of/some/other/document.md selector1 (var1=42, var2=var3)'
    const expectedHTML = '<div data-include="address/of/some/other/document.md" data-select="selector1" data-input="var1=42, var2=var3"></div>'
    const outputHTML = toHTML(input)
    const outputMD = toMD(expectedHTML)
    st.equal(outputHTML, expectedHTML)
    st.equal(outputMD, input)
    st.end()
  })

  t.test('modifiers', st => {
    const input = `
< address/of/some/other/document.md

- delete selector1
- change selector2

  # Markdown
`.trim()

    const expectedHTML = `<div data-include="address/of/some/other/document.md">\n  <div data-delete="selector1"></div>\n  <div data-change="selector2">\n    <h1>Markdown</h1>\n  </div>\n</div>`
    const outputHTML = toHTML(input.trim())
    st.equal(outputHTML, expectedHTML.trim())
    const outputMD = toMD(expectedHTML)
    st.equal(outputMD, input)
    st.end()
  })

  t.test('examples from files', st => {
    const dir = path.join(__dirname, 'documents', 'include')
    const input = fs.readFileSync(path.join(dir, 'default.md'), 'utf8')
    const expectedHTML = fs.readFileSync(path.join(dir, 'default.html'), 'utf8').trim()
    const outputHTML = beautifyHTML(markdown.md2html(input))
    const outputMD = markdown.html2md(expectedHTML)
    st.equal(outputHTML, expectedHTML, 'output html should equal expected html')
    st.equal(outputMD, input, 'output markdown should equal input')
    st.end()
  })

  t.end()
})
