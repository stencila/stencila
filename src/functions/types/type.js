/**
 * Get the type code for a value
 *
 * @param {whatever} value - Value you want a type for
 * @return {string} - Type of value
 */
export default function type (value) {
  let type = typeof value

  if (value === null) {
    return 'null'
  } else if (type === 'boolean') {
    return 'bool'
  } else if (type === 'number') {
    let isInteger = false
    if (value.isInteger) isInteger = value.isInteger()
    else isInteger = (value % 1) === 0
    return isInteger ? 'int' : 'flt'
  } else if (type === 'string') {
    return 'str'
  } else if (type === 'object') {
    type = value.constructor === Array ? 'arr' : 'obj'
    if (type === 'arr') {
      let onlyObjects = true
      for (let item of value) {
        if (item.constructor !== Object) {
          onlyObjects = false
          break
        }
      }
      if (onlyObjects && value.length > 0) return 'tab'
    }
  }
  return type
}
