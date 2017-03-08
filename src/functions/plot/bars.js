export default function bars (data, x, y) {
  x = x || 'x'
  y = y || 'y'

  return {
    _vegalite: true,
    data: {
      values: data
    },
    mark: 'bar',
    encoding: {
      x: {
        field: x,
        type: 'qualitative'
      },
      y: {
        field: y,
        type: 'quantitative'
      }
    }
  }
}
