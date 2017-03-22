import array from '../types/array'
import merge from '../types/merge'

export default function vegalite (data, extras) {
  // Convert data to an array of objects
  data = array(data)

  // Create a Vega-Lite spec
  let spec = {
    type: 'vegalite',
    data: {
      values: data
    },
    mark: null,
    encoding: {
      // Get's specified by functions such as `points`, `bars` etc
    },
    config: {
      cell: {
        width: 400,
        height: 400
      },
      mark: {
        filled: true,
        color: '#000',
        opacity: 0.7
      },
      point: {
        size: 70
      }
    }
  }

  // Merge in any extra spec
  if (extras) spec = merge(spec, extras)

  return spec
}
