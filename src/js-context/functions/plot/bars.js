import convertTableToArray from '../types/convertTableToArray'

export default function bars (data, x, y) {
  x = x || 'x'
  y = y || 'y'

  return {
    type: 'vegalite',
    data: {
      values: convertTableToArray(data)
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
