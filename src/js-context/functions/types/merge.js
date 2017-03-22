import _merge from 'lodash/merge'

import type from './type'
import is_object from './is_object'

export default function merge (value1, value2) {
  if (value1 !== undefined && is_object(value1) && value2 !== undefined && is_object(value2)) return _merge(value1, value2)
  else {
    let key = ''
    if (value1 !== undefined) key += type(value1)
    if (value2 !== undefined) key += ', ' + type(value2)
    throw new Error(`Unable to dispatch function call "merge(${key})"`)
  }
}
