import test from 'tape'

import converter from '../../src/document/DocumentJupyterConverter'

test('import', t => {
  const i = (json, options) => {
    options = options || {}
    if (options.archive !== true) options.archive = false
    return converter.import(json, options)
  }

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
    i({cells: [{cell_type: 'markdown', source: ['```\n', 'let x = 56\n', 'x < 65\n', '```\n']}]}),
    '<pre><code>let x = 56\nx &#x3C; 65\n</code></pre>',
    'Markdown code block'
  )

  t.equal(
    i({
      metadata: { language_info: { name: 'python' } },
      cells: [
        {cell_type: 'markdown', source: ['# Heading 1\n']},
        {cell_type: 'code', source: ['dict(foo="bar")\n']}
      ]
    }),
    '<h1 id="heading-1">Heading 1</h1><div data-cell="run"><pre data-source="py">dict(foo=&quot;bar&quot;)</pre></div>',
    'cells'
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
    '<div data-cell="run"><pre data-source="r">x &lt;- 6\nplot(1,x)</pre><img data-value="img" data-format="png" src="data:image/png;base64,PNGdata"></div>',
    'cells with output'
  )

  t.end()
})

test('export', t => {
  const e = (html, options) => {
    options = options || {}
    if (options.archive !== true) options.archive = false
    return converter.export(html, options)
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
      <div data-cell="run">
        <pre data-source="r">x &lt;- 6
x*7</pre>
        <img data-value="img" data-format="png" src="data:image/png;base64,PNGdata">
      </div>`, {stringify: false}
    ).cells[1],
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
                'image/png': 'PNGdata'
              }
            }
          ],
          execution_count: null
        }
      ],
      metadata: {},
      nbformat: 4,
      nbformat_minor: 2
    }.cells[1],
    'mix dom content and cells'
  )

  t.end()
})

test('import-export', t => {
  // Function to do round trip conversion and checking.
  // Takes an array of cells
  function f (cells) {
    let nb = {
      cells: cells,
      metadata: {},
      nbformat: 4,
      nbformat_minor: 2
    }
    let html = converter.import(nb, {archive: false})
    let json = converter.export(html, {archive: false, stringify: false})
    t.deepEqual(json, nb)
  }

  f([
    {
      cell_type: 'markdown',
      metadata: {},
      source: [
        '# Heading 1\n'
      ]
    }
  ])

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
        'x <- 6\n',
        'y <- 7\n'
      ]
    }, {
      cell_type: 'markdown',
      metadata: {},
      source: [
        'Paragraph three\n'
      ]
    }
  ])

  t.end()
})
