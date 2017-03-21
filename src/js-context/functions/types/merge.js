import _merge from 'lodash/merge'

import _multimethod from './_multimethod'

const merge = _multimethod('merge', {
  'object, object': _merge
}, 2)

export default merge
