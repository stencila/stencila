const test = require('tape')

const Document = require('../../src/document/Document')
const DocumentJupyterNotebookConverter = require('../../src/document/DocumentJupyterNotebookConverter')

test('DocumentJupyterNotebookConverter', t => {
  let c = new DocumentJupyterNotebookConverter()

  t.equal(typeof c, 'object', 'is an object')
  t.ok(c instanceof DocumentJupyterNotebookConverter, 'is a DocumentJupyterNotebookConverter')
  t.end()
})

test('DocumentJupyterNotebookConverter.load', t => {
  let d = new Document()
  let c = new DocumentJupyterNotebookConverter()

  c.load(d, '{"cells":[{"cell_type":"markdown","source":["# Heading 1"]}]}')
  t.equal(d.html, '<h1>Heading 1</h1>', 'load JSON')

  c.load(d, {cells: [{cell_type: 'markdown', source: ['# Heading 1']}]})
  t.equal(d.html, '<h1>Heading 1</h1>', 'load Object')

  c.load(d, {
    metadata: {
      language_info: {
        name: 'python'
      }
    },
    cells: [
      {cell_type: 'markdown', source: ['# Heading 1']},
      {cell_type: 'code', source: ['"Foo"']}
    ]
  })
  t.equal(d.html, '<h1>Heading 1</h1>\n<pre data-execute="py">&quot;Foo&quot;</pre>')

  t.end()
})

test('DocumentJupyterNotebookConverter.dump', t => {
  let d = new Document()
  let c = new DocumentJupyterNotebookConverter()

  d.html = '<h1>Heading 1</h1>'
  t.equal(c.dump(d), `{
  "cells": [
    {
      "cell_type": "markdown",
      "metadata": {},
      "source": [
        "# Heading 1"
      ]
    }
  ],
  "metadata": {},
  "nbformat": 4,
  "nbformat_minor": 2
}`)

  d.html = '<h1>Heading 1</h1>\n<pre data-execute="python">6*7</pre>'
  t.equal(c.dump(d), `{
  "cells": [
    {
      "cell_type": "markdown",
      "metadata": {},
      "source": [
        "# Heading 1"
      ]
    },
    {
      "cell_type": "code",
      "metadata": {},
      "source": [
        "6*7"
      ],
      "outputs": [],
      "execution_count": null
    }
  ],
  "metadata": {},
  "nbformat": 4,
  "nbformat_minor": 2
}`)

  t.end()
})
