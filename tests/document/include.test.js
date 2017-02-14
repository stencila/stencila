const test = require('tape')

const unified = require('unified')
const remarkParse = require('remark-parse')
const remarkStringify = require('remark-stringify')
const remarkHtml = require('remark-html')
const rehypeParse = require('rehype-parse')
const rehype2remark = require('rehype-remark')

const include = require('../../src/utilities/include')
const defaults = require('../../src/utilities/markdown-defaults')

test('include', t => {
  function toHTML (input) {
    return unified()
      .use(remarkParse)
      .use(include.md2html)
      .use(remarkStringify)
      .use(remarkHtml)
      .process(input, defaults).contents.trim()
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

    const expectedHTML = `<div data-include="address/of/some/other/document.md"><div data-delete="selector1"></div><div data-change="selector2"><h1>Markdown</h1></div></div>`
    const outputHTML = toHTML(input.trim())
    st.equal(outputHTML, expectedHTML.trim())
    const outputMD = toMD(expectedHTML)
    st.equal(outputMD, input)
    st.end()
  })

  t.end()
})
