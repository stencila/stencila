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
  const first = address[0]
  if (first === '+') {
    return 'new://' + address.substring(1)
  } else if (first === '*') {
    return 'local://' + address.substring(1)
  } else if (first === '.' || first === '/' || first === '~') {
    return 'file://' + address
  } else if (address.match(/^https?:\/\//)) {
    // Translate HTTP/S aliases into long addresses
    const match = address.match(/^https?:\/\/([^/]+)\/(.+)/)
    if (match) {
      const origin = match[1]
      const path = match[2]
      if (origin === 'www.dropbox.com') {
        // Dropbox shared folder link e.g.
        //   https://www.dropbox.com/sh/el77xzcpr9uqxb1/AABJIkDNXo_-sKnrUtQvCxC4a?dl=0
        const match = path.match(/^sh\/([^?]+)/)
        if (match) return 'dropbox://' + match[1]
      } else if (origin === 'github.com') {
        // Github repo pages e.g.
        //   repo: https://github.com/stencila/examples
        //   dir:  https://github.com/stencila/examples/tree/master/chinook
        //   file: https://github.com/stencila/examples/blob/master/chinook/README.md
        const match = path.match(/^([^/]+)\/([^/]+)(\/(tree|blob)\/([^/]+)\/(.+))?/)
        if (match) {
          const user = match[1]
          const repo = match[2]
          const ref = match[5]
          const path = match[6]
          let address = 'github://' + user + '/' + repo
          if (path) address += '/' + path
          if (ref && ref !== 'master') address += '@' + ref
          return address
        }      
      }
    }
    return address
  } else {
    // Other aliases
    let match = address.match(/^([a-z]+)(:\/?\/?)(.+)$/)
    if (match) {
      let scheme = match[1]
      let rest = match[3]
      switch (scheme) {
        case 'gh':
          scheme = 'github'
          break
        default:
          break
      }
      return scheme + '://' + rest
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
    return address.substring(7)
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
  let matches = address.match(/([a-z]+):\/\/([^@]+)(@([\w\-.]+))?/)
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
 * Get the `Storer` for an address scheme
 * 
 * @param  {string} scheme The address scheme (e.g. `github`)
 * @return {string}        The name of the matching `Storer` class (e.g. `GithubStorer`)
 */
export function storer (scheme) {
  switch (scheme) {
    case 'new':
    case 'local':
      return null
    case 'https':
      return storer('http')
    default:
      return scheme[0].toUpperCase() + scheme.slice(1) + 'Storer'
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
