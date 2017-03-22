import set from 'lodash/set'

export default function titles (vegalite, x, y, color, size) {
  if (x) update_object(vegalite, ['encoding', 'x', 'axis', 'title'], x)
  if (y) update_object(vegalite, ['encoding', 'y', 'axis', 'title'], y)
  if (color) update_object(vegalite, ['encoding', 'color', 'legend', 'title'], color)
  if (size) update_object(vegalite, ['encoding', 'size', 'legend', 'title'], size)

  return vegalite
}
