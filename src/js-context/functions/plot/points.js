import marks from './marks'

function points (data, x, y, color, size) {
  return marks(data, 'point', x, y, color, size)
}
points.pars = ['data', 'x', 'y', 'color', 'size']

export default points
