/**
 * @namespace address
 */

/**
 * Get the long form of a component address
 *
 * @memberof address
 * @see short
 *
 * @example
 *
 * long('+document')
 * 'new://document'
 *
 * long('gh:stencila/stencila/README.md')
 * 'github://stencila/stencila/README.md'
 *
 * long('./report/intro.md')
 * 'file:///current/directory/report/intro.md'
 *
 * long('stats/t-test.md')
 * 'lib://stats/t-test.md'
 *
 * @param {string} address - The address to lengthen
 * @return {string} - The long form of the address
 */
export function long (address) {
  if (address.match(/^(new|local|file|lib|http|https|github):\/\//)) {
    return address
  } else if (address[0] === '+') {
    return 'new://' + address.substring(1)
  } else if (address[0] === '*') {
    return 'local://' + address.substring(1)
  } else if (address[0] === '.' || address[0] === '/' || address[0] === '~') {
    return 'file://' + address
  } else {
    let match = address.match(/^([a-z]+)(:\/?\/?)(.+)$/)
    if (match) {
      let alias = match[1]
      let path = match[3]
      if (alias === 'file') {
        // Only arrive here with `file:/foo` since with
        // `file:` with two or more slashes is already "long"
        return `file:///${path}`
      } else if (alias === 'http' || alias === 'https') {
        return `${alias}://${path}`
      } else if (alias === 'gh' || alias === 'github') {
        return `github://${path}`
      } else {
        throw new Error(`Unknown scheme alias "${alias}"`)
      }
    } else {
      return 'lib://' + address
    }
  }
}

/**
 * Get the short form of a component address
 *
 * This method is the inverse of `long()`. It shortens an address tp
 * a smaller, more aeshetically pleasing form, that is useful in URLs
 * an other places.
 *
 * @memberof address
 * @see long
 *
 * @example
 *
 * short('new://document')
 * '+document'
 *
 * short('file:///some/directory/my-doc.md')
 * 'file:/some/directory/my-doc.md'
 *
 * @param {string} address - The address to shorten
 * @return {string} - The short form of the address
 */
export function short (address) {
  address = long(address)
  if (address.substring(0, 6) === 'new://') {
    return '+' + address.substring(6)
  } else if (address.substring(0, 8) === 'local://') {
    return '*' + address.substring(8)
  } else if (address.substring(0, 7) === 'file://') {
    return 'file:' + address.substring(7)
  } else if (address.substring(0, 6) === 'lib://') {
    return address.substring(6)
  } else if (address.substring(0, 9) === 'github://') {
    return 'gh:' + address.substring(9)
  } else {
    let match = address.match(/([a-z]+):\/\/(.+)$/)
    return `${match[1]}:${match[2]}`
  }
}

/**
 * Split a component address into its parts
 *
 * @memberof address
 *
 * @param {string} address - The address to split
 * @return {object} - An object with a property for each part of the address
 */
export function split (address) {
  address = long(address)
  let matches = address.match(/([a-z]+):\/\/([\w\-.~/]+)(@([\w\-.]+))?/)
  if (matches) {
    // Previously used Node's `path.extname` function to get any file extension.
    // This simple reimplementation probably need robustification.
    let ext = null
    let parts = matches[2].split('.')
    if (parts.length > 1) ext = parts[parts.length - 1]
    return {
      scheme: matches[1],
      path: matches[2],
      format: ext,
      version: matches[4] || null
    }
  } else {
    throw new Error(`Unable to split address "${address}"`)
  }
}

/**
 * Get the scheme of a component address
 *
 * @memberof address
 *
 * @param {string} address - The address
 * @return {string} - The address's scheme
 */
export function scheme (address) {
  return split(address).scheme
}

/**
 * Get the path of a component address
 *
 * @memberof address
 *
 * @param {string} address - The address
 * @return {string} - The address's path
 */
export function path (address) {
  return split(address).path
}

/**
 * Get the format of a component address
 *
 * @memberof address
 *
 * @param {string} address - The address
 * @return {string} - The address's format
 */
export function format (address) {
  return split(address).format
}

/**
 * Get the version of a component address
 *
 * @memberof address
 *
 * @param {string} address - The address
 * @return {string} - The address's version
 */
export function version (address) {
  return split(address).version
}
