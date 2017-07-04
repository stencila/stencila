/* globals Blob, ArrayBuffer, Uint8Array */

/**
 * @namespace value
 */

/**
 * Get the type code for a value
 *
 * @memberof value
 * @param {*} value - A JavaScript value
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
 * @param {*} value The Javascript value
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

  let {type, format, content} = pkg

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
  } else if (type === 'image') {
    if (format === 'url') {
      return {
        type: 'image',
        src: content
      }
    } else if (format === 'base64') {
      let src
      try {
        const byteString = window.atob(content)
        var arrayBuffer = new ArrayBuffer(byteString.length);
        var integerArray = new Uint8Array(arrayBuffer);
        for (var i = 0; i < byteString.length; i++) {
          integerArray[i] = byteString.charCodeAt(i);
        }
        let blob = new Blob([integerArray], {type: `image/${format}`})
        src = window.URL.createObjectURL(blob)
      } catch (error) {
        src = `data:image/${format};base64,${content}`
      }

      return {
        type: 'image',
        src: src
      }
    } else {
      throw new Error(`Format ${format} not supported.`)
    }

  } else {
    if (format === 'json') return JSON.parse(content)
    else return content
  }
}

/**
 * Load a value from a HTML representation
 *
 * This function is used for deserializing cell values from HTML.
 *
 * @param {*} elem - HTML element
 * @return {*} - The value
 */
export function fromHTML (elem) {
  let type = elem.attr('data-value')
  let format = elem.attr('data-format')
  let content
  if (type === 'image') {
    let imageContent
    if (format === 'svg') {
      imageContent = elem.innerHTML
    } else {
      let data = elem.attr('src')
      let match = data.match(/data:image\/([a-z]+);base64,([\w]+)/)
      imageContent = match[2]
    }
    content = imageContent
  } else {
    content = elem.innerHTML
  }
  return unpack({
    type: type,
    format: format,
    content: content
  })
}

/**
 * Dump a value to a HTML representation
 *
 * This function is used for serializing cell values to HTML.
 *
 * @param {*} value - Value to convert to HTML
 * @return {string} - HTML string
 */
export function toHTML (value) {
  let type_ = type(value)
  if (type_ === 'image') {
    if (value.format === 'svg') {
      return `<div data-value="image" data-format="svg">${value.content}</div>`
    } else {
      return `<img data-value="image" data-format="${value.format}" src="data:image/${value.format};base64,${value.content}">`
    }
  } else {
    if (typeof value.content === 'undefined') {
      // Do a pack to get a text representation of the value
      let packed = pack(value)
      return `<div data-value="${type_}">${packed.content}</div>`
    } else {
      return `<div data-value="${type_}">${value.content}</div>`
    }
  }
}

/**
 * Load a value from a MIME type/content representation
 *
 * This function is used for deserializing cell values from MIME content
 * (e.g. Jupyter cells).
 *
 * @param {string} mimetype - The MIME type
 * @param {string} content - The MIME content
 * @return {*} - The value
 */
export function fromMime (mimetype, content) {
  if (mimetype === 'image/png') {
    let match = mimetype.match('^image/([a-z]+)$')
    return {
      type: 'image',
      format: match ? match[1] : null,
      content: content
    }
  } else if (mimetype === 'image/svg+xml') {
    return {
      type: 'image',
      format: 'svg',
      content: content
    }
  } else if (mimetype === 'text/html') {
    return {
      type: 'dom',
      format: 'html',
      content: content
    }
  } else if (mimetype === 'text/latex') {
    // Remove any preceding or trailing double dollars
    if (content.substring(0, 2) === '$$') content = content.substring(2)
    if (content.slice(-2) === '$$') content = content.slice(0, -2)
    return {
      type: 'math',
      format: 'latex',
      content: content
    }
  } else {
    return content
  }
}

/**
 * Dump a value to a MIME type/content representation
 *
 * This function is used for serializing cell values to MIME.
 *
 * @param {*} value - Value to convert to HTML
 * @return {object} - MIUME type and content as string
 */
export function toMime (value) {
  let type_ = type(value)
  if (type_ === 'image') {
    return {
      mimetype: `image/${value.format}`,
      content: value.base64
    }
  } else {
    let content
    if (typeof value.content === 'undefined') {
      // Do a pack to get a text representation of the value
      content = pack(value).content
    } else {
      // Use the value's content
      content = value.content
    }

    return {
      mimetype: 'text/plain',
      content: content
    }
  }
}
