/**
 * @namespace value
 */

/**
 * Get the type code for a value
 *
 * @memberof value
 * @param {whatever} value - A JavaScript value
 * @return {string} - Type code for value
 */
export function type (value) {
  let type = typeof value

  if (value === null) {
    return 'null'
  } else if (type === 'boolean') {
    return 'boolean'
  } else if (type === 'number') {
    let isInteger = false
    if (value.isInteger) isInteger = value.isInteger()
    else isInteger = (value % 1) === 0
    return isInteger ? 'integer' : 'float'
  } else if (type === 'string') {
    return 'string'
  } else if (type === 'object') {
    if (value.constructor === Array) {
      return 'array'
    }
    if (value.type) return value.type
    else return 'object'
  } else {
    return 'unknown'
  }
}

/**
 * Pack a value into a package
 *
 * @memberof value
 * @param {anything} value The Javascript value
 * @return {object} A package as a Javascript object
 */
export function pack (value) {
  // A data pack has a `type`, `format` (defaults to "text")
  // and a `value` (the serialisation of the value into the format)
  let type_ = type(value)
  let format = 'text'
  let content

  if (type_ === 'null') {
    content = 'null'
  } else if (type_ === 'boolean' || type_ === 'integer' || type_ === 'float') {
    content = value.toString()
  } else if (type_ === 'string') {
    content = value
  } else if (type_ === 'object' || type_ === 'array' || type_ === 'table') {
    format = 'json'
    content = JSON.stringify(value)
  } else if (type_ === 'unknown') {
    content = value.toString()
  } else {
    format = 'json'
    content = JSON.stringify(value)
  }
  return {type: type_, format: format, content: content}
}

/**
 * Unpack a data package into an value
 *
 * @memberof value
 * @param {object|string} pkg The package
 * @return {anything} A Javascript value
 */
export function unpack (pkg) {
  if (typeof pkg === 'string') {
    pkg = JSON.parse(pkg)
  }
  if (pkg.constructor !== Object) {
    throw new Error('Package should be an `Object`')
  }
  if (!(pkg.type && pkg.format && pkg.content)) {
    throw new Error('Package should have fields `type`, `format`, `content`')
  }

  let {type, content} = pkg

  if (type === 'null') {
    return null
  } else if (type === 'boolean') {
    return content === 'true'
  } else if (type === 'integer') {
    return parseInt(content, 10)
  } else if (type === 'float') {
    return parseFloat(content)
  } else if (type === 'string') {
    return content
  } else if (type === 'object') {
    return JSON.parse(content)
  } else if (type === 'array') {
    return JSON.parse(content)
  } else {
    return JSON.parse(content)
  }
}
