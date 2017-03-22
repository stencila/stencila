import vegalite from './vegalite'

export default function marks (data, mark, x, y, color, size) {
  // Check that `mark` is supported
  if (['point','circle','square','text','tick','bar','line','area'].indexOf(mark) < 0) {
    throw new Error(`Unsupported mark type: ${mark}`)
  }

  // Resolve required encodings
  x = x || 'x'
  y = y || 'y'

  // Generate spec with required encodings
  let spec = vegalite(data, {
    mark: mark || 'point',
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
  })

  // Add optional encodings
  if (color) {
    spec.encoding.color = {
      field: color,
      type: 'quantitative'
    }
  }
  if (size) {
    spec.encoding.size = {
      field: size,
      type: 'quantitative'
    }
  }

  return spec
}
