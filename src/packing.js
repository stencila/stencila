import * as d3 from 'd3'

import {default as type_} from './functions/types/type'

/**
 * Pack an object into a data package
 *
 * @param {anything} thing The Javascript object
 * @return {Object} A data package
 */
function pack (thing) {
  // A data pack has a `type`, `format` (defaults to "text")
  // and a `value` (the serialisation of the thing into the format)
  let type = type_(thing)
  let format = 'text'
  let value

  if (type === 'null') {
    value = 'null'
  } else if (type === 'bool' || type === 'int' || type === 'flt') {
    value = thing.toString()
  } else if (type === 'str') {
    value = thing
  } else if (type === 'obj' || type === 'arr') {
    format = 'json'
    value = JSON.stringify(thing)
  } else if (type === 'tab') {
    format = 'csv'
    value = d3.csvFormat(thing) + '\n'
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
