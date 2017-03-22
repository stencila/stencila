import table from '../types/table'
import array from '../types/array'
import merge from '../types/merge'
import sentence_case from '../string/sentence_case'

function plot (data, mark = 'point', x, y, color, size, text, row, column, options = {}) {
  // Create a Vega-Lite spec
  let spec = {
    type: 'vegalite'
  }

  // Convert data to a table
  data = table(data)
  let columns = data.data

  // Convert data to an array of objects
  spec.data = {
    values: array(data)
  }

  // Translate mark aliases
  mark = {
    'points': 'point',
    'circles': 'circle',
    'squares': 'square',
    'ticks': 'tick',
    'bars': 'bar',
    'lines': 'line'
  }[mark] || mark
  // Check that `mark` is supported
  if (['point','circle','square','text','tick','bar','line','area'].indexOf(mark) < 0) {
    throw new Error(`Unsupported mark type: ${mark}`)
  }
  spec.mark = mark

  // Generate spec with required encodings
  spec.encoding = {
    x: {
      field: x,
      type: columns[x].type || 'quantitative',
      axis: {
        title: sentence_case(x)
      }
    },
    y: {
      field: y,
      type: columns[y].type || 'quantitative',
      axis: {
        title: sentence_case(y)
      }
    }
  }

  // Add optional encodings
  if (color) {
    spec.encoding.color = {
      field: color,
      type: columns[color].type || 'nominal'
    }
  }
  if (size) {
    spec.encoding.size = {
      field: size,
      type: columns[size].type || 'quantitative'
    }
  }
  if (text) {
    spec.encoding.text = {
      field: text,
      type: columns[text].type || 'nominal'
    }
  }
  if (row) {
    spec.encoding.row = {
      field: row,
      type: columns[row].type || 'nominal'
    }
  }
  if (column) {
    spec.encoding.column = {
      field: column,
      type: columns[column].type || 'nominal'
    }
  }

  // Some default configuration to customize size and
  // appearance
  spec.config = {
    cell: {
      width: 400,
      height: 400
    },
    mark: {
      filled: true,
      color: '#000',
      opacity: 0.7
    },
    point: {
      size: 70
    },
    axis: {
      titleFont: 'Helvetica',
      titleFontWeight: 'bold',
      titleFontSize: 12,

      tickLabelFontSize: 12
    },
    legend: {
      titleFont: 'Helvetica',
      titleFontWeight: 'bold',
      titleFontSize: 12,

      labelFontSize: 12
    }
  }

  // Merge in any other user defined spec options
  if (options) spec = merge(spec, options)

  return spec
}
plot.pars = ['data', 'mark', 'x', 'y', 'color', 'size', 'text', 'row', 'column', 'options']

export default plot
