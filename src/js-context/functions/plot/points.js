export default function points (data, x, y) {
  x = x || 'x'
  y = y || 'y'

  return {
    type: 'vegalite',
    data: {
      values: data
    },
    mark: 'point',
    encoding: {
      x: {
        field: x,
        type: 'quantitative'
      },
      y: {
        field: y,
        type: 'quantitative'
      }
    }
  }
}
