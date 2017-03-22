import {type as type_} from '../../../value'

/**
 * Get the type code for a value
 *
 * Exposes the internal `type` function so that uses can inspect the
 * type of a function call or expression
 *
 * @param {whatever} value - Value you want a type for
 * @return {string} - Type of value
 */
function type (value) {
  return type_(value)
}
type.pars = ['value']

export default type
