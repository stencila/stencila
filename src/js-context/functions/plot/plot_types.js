import plot from './plot'

function plot_type (mark) {
  function plot_type (data, x, y, color, size, text, row, column, options) {
    return plot(data, mark, x, y, color, size, text, row, column, options)
  }
  plot_type.pars = ['data', 'x', 'y', 'color', 'size', 'text', 'row', 'column', 'options']
  return plot_type
}

let points = plot_type('point')
let circles = plot_type('circle')
let squares = plot_type('square')
let ticks = plot_type('tick')
let bars = plot_type('bar')
let lines = plot_type('line')
let areas = plot_type('area')

export {points, circles, squares, ticks, bars, lines, areas}
