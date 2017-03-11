import * as d3 from 'd3'

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
    return 'bool'
  } else if (type === 'number') {
    let isInteger = false
    if (value.isInteger) isInteger = value.isInteger()
    else isInteger = (value % 1) === 0
    return isInteger ? 'int' : 'flt'
  } else if (type === 'string') {
    return 'str'
  } else if (type === 'object') {
    if (value.constructor === Array) {
      let onlyObjects = true
      for (let item of value) {
        if (!item || item.constructor !== Object) {
          onlyObjects = false
          break
        }
      }
      if (onlyObjects && value.length > 0) return 'tab'
      else return 'arr'
    }
    if (value.type) return value.type
    else return 'obj'
  } else {
    return 'unk'
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
  } else if (type_ === 'bool' || type_ === 'int' || type_ === 'flt') {
    content = value.toString()
  } else if (type_ === 'str') {
    content = value
  } else if (type_ === 'obj' || type_ === 'arr') {
    format = 'json'
    content = JSON.stringify(value)
  } else if (type_ === 'tab') {
    format = 'csv'
    content = d3.csvFormat(value) + '\n'
  } else if (type_ === 'unk') {
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

  let {type, format, content} = pkg

  if (type === 'null') {
    return null
  } else if (type === 'bool') {
    return content === 'true'
  } else if (type === 'int') {
    return parseInt(content)
  } else if (type === 'flt') {
    return parseFloat(content)
  } else if (type === 'str') {
    return content
  } else if (type === 'obj') {
    return JSON.parse(content)
  } else if (type === 'arr') {
    return JSON.parse(content)
  } else if (type === 'tab') {
    if (format === 'csv') {
      return d3.csvParse(content)
    } else if (format === 'tsv') {
      return d3.tsvParse(content)
    } else {
      throw new Error('Unable to unpack\n  type: ' + type + '\n  format: ' + format)
    }
  } else {
    return JSON.parse(content)
  }
}
