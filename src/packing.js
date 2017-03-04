import d3 from 'd3'

/**
 * Pack an object into a data package
 *
 * @param {anything} thing The Javascript object
 * @return {Object} A data package
 */
function pack (thing) {
  let type = typeof thing
  let format = 'text'
  let value = thing

  if (thing === null) {
    type = 'null'
    value = 'null'
  } else if (type === 'boolean') {
    type = 'bool'
    value = thing.toString()
  } else if (type === 'number') {
    let isInteger = false
    if (thing.isInteger) isInteger = thing.isInteger()
    else isInteger = (thing % 1) === 0
    type = isInteger ? 'int' : 'flt'
    value = thing.toString()
  } else if (type === 'string') {
    type = 'str'
  } else if (type === 'object') {
    type = thing.constructor === Array ? 'arr' : 'obj'
    // Arrays of objects get converted to tabular data
    if (type === 'arr') {
      let onlyObjects = true
      for (let item of thing) {
        if (item.constructor !== Object) {
          onlyObjects = false
          break
        }
      }
      if (onlyObjects && thing.length > 0) {
        type = 'tab'
        format = 'csv'
        value = d3.csvFormat(thing) + '\n'
      } else {
        format = 'json'
        value = JSON.stringify(thing)
      }
    } else {
      format = 'json'
      value = JSON.stringify(thing)
    }
  } else {
    throw new Error('Unable to pack object\n  type: ' + type)
  }

  return {type: type, format: format, value: value}
}

/**
 * Unpack a data package into an object
 *
 * @param {Object|String} pkg The data package
 * @return {anything} A Javascript object
 */
function unpack (pkg) {
  if (typeof pkg === 'string') {
    pkg = JSON.parse(pkg)
  }
  if (pkg.constructor !== Object) {
    throw new Error('Package should be an `Object`')
  }
  if (!(pkg.type && pkg.format && pkg.value)) {
    throw new Error('Package should have fields `type`, `format`, `value`')
  }

  let {type, format, value} = pkg

  if (type === 'null') {
    return null
  } else if (type === 'bool') {
    return value === 'true'
  } else if (type === 'int') {
    return parseInt(value)
  } else if (type === 'flt') {
    return parseFloat(value)
  } else if (type === 'str') {
    return value
  } else if (type === 'obj') {
    return JSON.parse(value)
  } else if (type === 'arr') {
    return JSON.parse(value)
  } else if (type === 'tab') {
    if (format === 'csv') {
      return d3.csvParse(value)
    } else if (format === 'tsv') {
      return d3.tsvParse(value)
    } else {
      throw new Error('Unable to unpack\n  type: ' + type + '\n  format: ' + format)
    }
  } else {
    throw new Error('Unable to unpack\n  type: ' + type + '\n  format: ' + format)
  }
}

export {pack, unpack}
