import test from 'tape'
import {DefaultDOMElement} from 'substance'

import DocumentJupyterConverter from '../../src/document/DocumentJupyterConverter'

test('DocumentJupyterConverter:match', function (t) {
  t.ok(DocumentJupyterConverter.match('foo.ipynb'))
  t.notOk(DocumentJupyterConverter.match('foo.html'))
  t.end()
})

test('DocumentJupyterConverter:encodingLessThan', function (t) {
  // Tests character encoding by DefaultDOMElement
  let el = DefaultDOMElement.createElement('pre')

  el.text('Less than < char')
  t.equal(el.text(), 'Less than < char')

  el.html('Less than < char')
  t.equal(el.html(), 'Less than &lt; char')

  el.text('Less than &lt; char')
  t.equal(el.text(), 'Less than &lt; char')

  el.html('Less than &lt; char')
  t.equal(el.html(), 'Less than &lt; char')

  t.end()
})


test('DocumentJupyterConverter:import', t => {
  const converter = new DocumentJupyterConverter()
  const i = json => converter.importContent(json)

  t.equal(
    i('{"cells":[{"cell_type":"markdown","source":["# Heading 1\\n"]}]}'),
    '<h1 id="heading-1">Heading 1</h1>',
    'can be called with a JSON string'
  )

  t.equal(
    i({cells: [{cell_type: 'markdown', source: ['# Heading 1\n']}]}),
    '<h1 id="heading-1">Heading 1</h1>',
    'can be called with Javascript object'
  )

  t.equal(
    i({
      metadata: { language_info: { name: 'python' } },
      cells: [
        {cell_type: 'markdown', source: ['# Heading 1\n']},
        {cell_type: 'code', source: ['dict(foo="bar")\n']}
      ]
    }),
    '<h1 id="heading-1">Heading 1</h1><div data-cell="global py()"><pre data-source="">dict(foo="bar")</pre></div>',
    'cells'
  )

  t.equal(
    i({
      metadata: { language_info: { name: 'python' } },
      cells: [
        {cell_type: 'code', source: ['dict(foo=\'bar\')\n']}
      ]
    }),
    '<div data-cell="global py()"><pre data-source="">dict(foo=\'bar\')</pre></div>',
    'code cells with apostrophes'
  )

  t.equal(
    i({cells: [{cell_type: 'markdown', source: ['```\n', 'let x = 56\n', 'x < 65\n', '```\n']}]}),
    '<pre><code>let x = 56\nx &lt; 65</code></pre>',
    'Markdown code block'
  )

  t.equal(
    i({
      metadata: { language_info: { name: 'R' } },
      cells: [
        {
          cell_type: 'code',
          source: ['x <- 6\n', 'plot(1,x)\n'],
          outputs: [
            {
              output_type: 'execute_result',
              data: {
                'image/png': 'PNGdata'
              }
            }
          ]
        }
      ]
    }),
    '<div data-cell="global r()"><pre data-source="">x &lt;- 6\nplot(1,x)</pre><img data-value="image" data-format="src" src="data:image/png;base64,PNGdata"></div>',
    'cells with output'
  )

  t.end()
})

test('DocumentJupyterConverter:export', t => {
  const converter = new DocumentJupyterConverter()
  const e = (html, options) => {
    options = options || {}
    return converter.exportContent(html, options)
  }

  t.equal(
    e('<h1 id="heading-1">Heading 1</h1>'),
`{
  "cells": [
    {
      "cell_type": "markdown",
      "metadata": {},
      "source": [
        "# Heading 1\\n"
      ]
    }
  ],
  "metadata": {},
  "nbformat": 4,
  "nbformat_minor": 2
}`,
  'defaults to pretty')

  t.equal(
    e('<h1 id="heading-1">Heading 1</h1>', {pretty: false}),
    '{"cells":[{"cell_type":"markdown","metadata":{},"source":["# Heading 1\\n"]}],"metadata":{},"nbformat":4,"nbformat_minor":2}',
    'can set pretty false'
  )
  t.deepEqual(
    e('<h1 id="heading-1">Heading 1</h1>', {stringify: false}),
    {cells: [{cell_type: 'markdown', metadata: {}, source: ['# Heading 1\n']}], metadata: {}, nbformat: 4, nbformat_minor: 2},
    'can set stringify false'
  )

  t.deepEqual(
    e(`
      <h1 id="heading-1">Heading 1</h1>
      <div data-cell="global r()">
        <pre data-source="">x &lt;- 6
x*7</pre>
        <img data-value="image" data-format="src" src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAJYAAACWCAYAAAA8">
      </div>`, {stringify: false}
    ),
    {
      cells: [
        {
          cell_type: 'markdown',
          metadata: {},
          source: [
            '# Heading 1\n'
          ]
        },
        {
          cell_type: 'code',
          metadata: {},
          source: [
            'x <- 6\n',
            'x*7\n'
          ],
          outputs: [
            {
              output_type: 'execute_result',
              data: {
                'image/png': 'iVBORw0KGgoAAAANSUhEUgAAAJYAAACWCAYAAAA8'
              }
            }
          ],
          execution_count: null
        }
      ],
      metadata: {},
      nbformat: 4,
      nbformat_minor: 2
    },
    'mix dom content and cells'
  )

  t.end()
})

test('DocumentJupyterConverter:import+export', t => {
  const converter = new DocumentJupyterConverter()
  // Function to do round trip conversion and checking.
  // Takes an array of cells
  function f (cells, message) {
    let nb = {
      cells: cells,
      metadata: {},
      nbformat: 4,
      nbformat_minor: 2
    }
    let html = converter.importContent(nb)
    let json = converter.exportContent(html, {stringify: false})
    t.deepEqual(json, nb, message)
  }

  f([
    {
      cell_type: 'markdown',
      metadata: {},
      source: [
        '# Heading 1\n',
        '\n',
        'Paragraph one\n'
      ]
    }
  ], 'just markdown cells')

  f([
    {
      cell_type: 'markdown',
      metadata: {},
      source: [
        '# Heading 1\n',
        '\n',
        'Paragraph one\n',
        '\n',
        'Paragraph two\n'
      ]
    }, {
      cell_type: 'code',
      execution_count: null,
      metadata: {},
      outputs: [],
      source: [
        'x = 6\n',
        'y = 7\n'
      ]
    }, {
      cell_type: 'markdown',
      metadata: {},
      source: [
        'Paragraph three\n'
      ]
    }
  ], 'mixed markdown and code cells')

  f([
    {
      cell_type: 'markdown',
      metadata: {},
      source: [
        'Less than < character\n'
      ]
    }, {
      cell_type: 'code',
      execution_count: null,
      metadata: {},
      outputs: [],
      source: [
        'x <- 6\n',
        'y <- 7\n'
      ]
    }
  ], 'conversion of < chars')

  t.end()
})
