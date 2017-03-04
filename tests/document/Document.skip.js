import test from 'tape'

import Component from '../../src/component/Component'
import Document from '../../src/document/Document'
import DocumentHtmlConverter from '../../src/document/DocumentHtmlConverter'
import DocumentMarkdownConverter from '../../src/document/DocumentMarkdownConverter'
import DocumentJupyterNotebookConverter from '../../src/document/DocumentJupyterNotebookConverter'

test('Document', t => {
  let d = new Document()

  t.equal(typeof d, 'object', 'is an object')
  t.ok(d instanceof Component, 'is a Component')
  t.ok(d instanceof Document, 'is a Document')
  t.end()
})

test('Document.default', t => {
  t.equal(Document.default('phony'), null, 'undefined is null')
  t.equal(Document.default('format'), 'html', 'format is html')
  t.end()
})

test('Document.converter', t => {
  t.ok(Document.converter() instanceof DocumentHtmlConverter, 'default is a DocumentHtmlConverter')
  t.ok(Document.converter('html') instanceof DocumentHtmlConverter, 'is a DocumentHtmlConverter')
  t.ok(Document.converter('md') instanceof DocumentMarkdownConverter, 'is a DocumentMarkdownConverter')
  t.ok(Document.converter('ipynb') instanceof DocumentJupyterNotebookConverter, 'is a DocumentJupyterNotebookConverter')
  t.end()
})

test('Document.(html,md,ipynb)', t => {
  // Minimalistic conversion tests. Doesn't do detailed tests of actual conversions
  // (see other tests for that), just tests the converting getters and setter
  let d = new Document()

  let html = '<h1 id="hello-world">Hello world</h1>'
  let md = '# Hello world'
  let ipynb = `{
  "cells": [
    {
      "cell_type": "markdown",
      "metadata": {},
      "source": [
        "# Hello world\\n"
      ]
    }
  ],
  "metadata": {},
  "nbformat": 4,
  "nbformat_minor": 2
}`

  d.html = html
  t.equal(d.html, html)
  t.equal(d.md, md)
  t.equal(d.ipynb, ipynb)

  d.md = md
  t.equal(d.html, html)
  t.equal(d.md, md)
  t.equal(d.ipynb, ipynb)

  d.ipynb = ipynb
  t.equal(d.html, html)
  t.equal(d.md, md)
  t.equal(d.ipynb, ipynb)

  t.end()
})
