import set from 'lodash/set'

function titles (spec, x, y, color, size) {
  if (x) set(spec, ['encoding', 'x', 'axis', 'title'], x)
  if (y) set(spec, ['encoding', 'y', 'axis', 'title'], y)
  if (color) set(spec, ['encoding', 'color', 'legend', 'title'], color)
  if (size) set(spec, ['encoding', 'size', 'legend', 'title'], size)
  return spec
}
titles.pars = ['spec', 'x', 'y', 'color', 'size']

export default titles
