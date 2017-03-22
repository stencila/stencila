import update from '../types/update'

function titles (spec, x, y, color, size) {
  if (x) update(spec, ['encoding', 'x', 'axis', 'title'], x)
  if (y) update(spec, ['encoding', 'y', 'axis', 'title'], y)
  if (color) update(spec, ['encoding', 'color', 'legend', 'title'], color)
  if (size) update(spec, ['encoding', 'size', 'legend', 'title'], size)
  return spec
}
titles.pars = ['spec', 'x', 'y', 'color', 'size']

export default titles
