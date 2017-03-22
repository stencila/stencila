import plot from './plot'

function points (data, x, y, color, size, options) {
  return plot(data, 'point', x, y, color, size, options)
}
points.pars = ['data', 'x', 'y', 'color', 'size', 'options']

export default points
