import test from 'tape'

import {import_, export_} from '../../src/document/jupyter'

test('import', t => {
  t.equal(
    import_('{"cells":[{"cell_type":"markdown","source":["# Heading 1\\n"]}]}'),
    '<h1 id="heading-1">Heading 1</h1>',
    'can be called with a JSON string'
  )

  t.equal(
    import_({cells: [{cell_type: 'markdown', source: ['# Heading 1\n']}]}),
    '<h1 id="heading-1">Heading 1</h1>',
    'can be called with Javascript object'
  )

  t.equal(
    import_({cells: [{cell_type: 'markdown', source: ['```\n', 'let x = 56\n', 'x < 65\n', '```\n']}]}),
    '<pre><code>let x = 56\nx &lt; 65</code></pre>',
    'code block'
  )

  t.equal(
    import_({
      metadata: { language_info: { name: 'python' } },
      cells: [
        {cell_type: 'markdown', source: ['# Heading 1\n']},
        {cell_type: 'code', source: ['"Foo"\n']}
      ]
    }),
    '<h1 id="heading-1">Heading 1</h1>\n<div data-execute="py">\n  <pre data-code="">&quot;Foo&quot;</pre>\n</div>',
    'cells'
  )

  t.equal(
    import_({
      metadata: { language_info: { name: 'R' } },
      cells: [
        {
          cell_type: 'code',
          source: ['plot(1,1)\n'],
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
    '<div data-execute="r"><pre data-code="">plot(1,1)</pre><img src="data:image/png;base64,PNGdata" data-result="img" data-format="png"></div>',
    'cells with output'
  )

  t.end()
})

/*

test('export', t => {
  let d = new Document()
  let c = new DocumentJupyterNotebookConverter()

  d.html = '<h1 id="heading-1">Heading 1</h1>'
  t.equal(
    c.dump(d),
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
}`)
  t.equal(
    c.dump({pretty: false}),
    '{"cells":[{"cell_type":"markdown","metadata":{},"source":["# Heading 1\\n"]}],"metadata":{},"nbformat":4,"nbformat_minor":2}'
  )
  t.deepEqual(
    c.dump(d, {stringify: false}),
    {cells: [{cell_type: 'markdown', metadata: {}, source: ['# Heading 1\n']}], metadata: {}, nbformat: 4, nbformat_minor: 2}
  )

  d.html = '<h1 id="heading-1">Heading 1</h1>\n<pre data-execute="py">6*7</pre>'
  t.equal(c.dump(d), `{
  "cells": [
    {
      "cell_type": "markdown",
      "metadata": {},
      "source": [
        "# Heading 1\\n"
      ]
    },
    {
      "cell_type": "code",
      "metadata": {},
      "source": [
        "6*7\\n"
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

test('import-export', t => {
  let d = new Document()
  let c = new DocumentJupyterNotebookConverter()

  // Function to do round trip conversion and checking.
  // Takes an array of cells
  function f (cells) {
    let nb = {
      cells: cells,
      metadata: {},
      nbformat: 4,
      nbformat_minor: 2
    }
    c.load(d, nb)
    t.deepEqual(c.dump(d, {
      stringify: false
    }), nb)
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

*/
