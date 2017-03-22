import type from './type'

/**
 * A factory function for creating multiple dispatch functions (aka. "multifuncs")
 *
 * Multifuncs are used in Mini to reduce the number of functions names and associated concepts while
 * maintaining the flexibility to operate on different data types. Wikipedia describes the
 * usefulness of multiple dispatch, particularly in functional languages:
 *
 * "Function names are usually selected so as to be descriptive of the function's purpose. 
 * It is sometimes desirable to give several functions the same name, often because they 
 * perform conceptually similar tasks, but operate on different types of input data. 
 * In such cases, the name reference at the function call site is not sufficient for identifying 
 * the block of code to be executed. Instead, the number and type of the arguments to the 
 * function call are also used to select among several function implementations."
 * 
 * See https://en.wikipedia.org/wiki/Multiple_dispatch. Multiple dispatch functions are also
 * known as "multimethods" but we use "multifuncs" instead to avoid introducing another term
 * into the Mini vocabulary.
 *
 * This function is a convenience for function implementers
 * 
 * @param  {string} name - Name of function, used for dispatch error reporting
 * @param  {object} lookup - An object with keys `type`, or `type1, type2` etc and function values
 * @param  {integer} arity - Number of arguments to dispatch upon
 * @param  {function} default_ - A default function
 * @return {function} - The multifunc
 */
export default function _multifunc (name, lookup, arity, default_) {
  if (typeof arity === 'function') {
    default_ = arity
    arity = undefined
  }
  arity = arity || 1

  return function (...args) {
    let key = args.slice(0, arity).map(arg => type(arg)).join(', ')
    let method = lookup[key]
    if (!method) {
      if (default_) method = default_
      else {
        method = () => {
          throw new Error(`Unable to dispatch function call "${name}(${key})"`)
        }
      }
    }
    return method(...args)
  }
}
