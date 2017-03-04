import path from 'path'
import os from 'os'

import GeneralError from '../utilities/general-error'
import {ComponentConverterUnknown} from './component-converter-errors'
import {ComponentStorerUnknown} from './component-storer-errors'

import ComponentGithubStorer from './ComponentGithubStorer'
import ComponentHttpStorer from './ComponentHttpStorer'
import ComponentLibraryStorer from './ComponentLibraryStorer'

/**
 * The abstract base class for all Stencila components
 */
class Component {

  /**
   * Construct a component
   *
   * @param {string} address - Address of component
   */
  constructor (address) {
    this.address = address
  }

  /**
   * Get the long form of a component address
   *
   * @see Component.short
   * @see address
   *
   * @example
   *
   * Component.long('+document')
   * 'new://document'
   *
   * Component.long('gh:stencila/stencila/README.md')
   * 'gh://stencila/stencila/README.md'
   *
   * Component.long('./report/intro.md')
   * 'file:///current/directory/report/intro.md'
   *
   * Component.long('stats/t-test.md')
   * 'lib://stats/t-test.md'
   *
   * Component.long()
   * 'id://fa4cf2c5cff5b576990feb96f25c98e6111990c873010855a53bcba979583836'
   *
   * @param {string} address - The address to lengthen
   * @return {string} - The long form of the address
   */
  static long (address) {
    if (address.match(/^(new|id|name|lib|file|http|https|git|gh):\/\//)) {
      return address
    } else if (address[0] === '+') {
      return 'new://' + address.substring(1)
    } else if (address[0] === '*') {
      return 'name://' + address.substring(1)
    } else if (address[0] === '.' || address[0] === '/' || address[0] === '~') {
      if (address[0] === '~') address = os.homedir() + address.substring(1)
      return 'file://' + path.resolve(address)
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
          return `gh://${path}`
        } else {
          throw new GeneralError('Unknown scheme alias.', {alias: alias})
        }
      } else {
        return 'lib://' + address
      }
    }
  }

  /**
   * Get the long form of this component's address
   *
   * @see Component#long
   *
   * @return {string} - The long form of the address
   */
  long () {
    return Component.long(this.address)
  }

  /**
   * Get the short form of a component address
   *
   * This method is the inverse of `long()`. It shortens an address tp
   * a smaller, more aeshetically pleasing form, that is useful in URLs
   * an other places.
   *
   * @see Component.long
   *
   * @example
   *
   * Component.short('new://document')
   * '+document'
   *
   * Component.short('file:///some/directory/my-doc.md')
   * 'file:/some/directory/my-doc.md'
   *
   * Component.short()
   * '*fa4cf2c5cff5b576990feb96f25c98e6111990c873010855a53bcba979583836'
   *
   * @param {string} address - The address to shorten
   * @return {string} - The short form of the address
   */
  static short (address) {
    address = Component.long(address)
    if (address.substring(0, 6) === 'new://') {
      return '+' + address.substring(6)
    } else if (address.substring(0, 7) === 'name://') {
      return '*' + address.substring(7)
    } else if (address.substring(0, 7) === 'file://') {
      return 'file:' + address.substring(7)
    } else if (address.substring(0, 6) === 'lib://') {
      return address.substring(6)
    } else if (address.substring(0, 5) === 'gh://') {
      return 'gh:' + address.substring(5)
    } else {
      let match = address.match(/([a-z]+):\/\/(.+)$/)
      return `${match[1]}:${match[2]}`
    }
  }

  /**
   * Get the short form of this component's address
   *
   * @see Component#short
   *
   * @return {string} - The short form of the address
   */
  short () {
    return Component.short(this.address)
  }

  /**
   * Split a component address into its parts
   *
   * @param {string} address - The address to split
   * @return {object} - An object with a property for each part of the address
   */
  static split (address) {
    address = Component.long(address)
    let matches = address.match(/([a-z]+):\/\/([\w\-\./]+)(@([\w\-\.]+))?/) // eslint-disable-line no-useless-escape
    if (matches) {
      return {
        scheme: matches[1],
        path: matches[2],
        format: path.extname(matches[2]).substring(1) || null,
        version: matches[4] || null
      }
    } else {
      throw new GeneralError('Unable to split address', {address: address})
    }
  }

  /**
   * Split this component's address into its parts
   *
   * @see Component.long
   *
   * @return {object} - An object with a property for each part of the addres
   */
  split () {
    return Component.split(this.address)
  }

  static scheme (address) {
    return Component.split(address).scheme
  }

  get scheme () {
    return this.constructor.scheme(this.address)
  }

  static path (address) {
    return Component.split(address).path
  }

  get path () {
    return this.constructor.path(this.address)
  }

  static format (address) {
    return Component.split(address).format
  }

  get format () {
    return this.constructor.format(this.address)
  }

  static version (address) {
    return Component.split(address).version
  }

  get version () {
    return this.constructor.version(this.address)
  }

  /**
   * Get a default value for this component class
   *
   * @param  {String} name - Name of value default is wanted for
   * @return {Object} - Default value
   */
  static default (name) {
    return {}[name] || null
  }

  /**
   * Get a default value for this component instance
   *
   * @param  {String} name - Name of value default is wanted for
   * @return {Object} - Default value
   */
  default (name) {
    return this.constructor.default(name)
  }

  /**
   * Get the converter for a format for this component class
   *
   * @param {string} format The format e.g. `'html'`, `'md'`
   */
  static converter (format) {
    throw new ComponentConverterUnknown(format)
  }

  /**
   * Get the converter for a format for this component instance
   *
   * @param {string} format The format e.g. `'html'`, `'md'`
   * @return {converter} A component converter
   */
  converter (format) {
    format = format || this.format
    return this.constructor.converter(format)
  }

  /**
  * Load content in a specified format
  *
  * @param {string} content - content of the document
  * @param {string} format - format of the content
  * @param {object} options - options that are passed to the converter
  **/
  load (content, format, options) {
    format = format || this.constructor.default(format)
    options = options || {}

    this.converter(format).load(this, content, options)
  }

  /**
  * dump content in a specified format
  *
  * @param {string} format - format of the content
  * @param {object} options - options that are passed to the converter
  * @returns {converter} - Content of the document
  **/
  dump (format, options) {
    format = format || this.constructor.default(format)
    options = options || {}

    return this.converter(format).dump(this, options)
  }

  /**
   * Get the storer for a scheme for this component class
   *
   * @param {string} scheme - A component address scheme
   * @return {storer} A component storer
   */
  static storer (scheme) {
    let Storer = {
      'gh': ComponentGithubStorer,
      'http': ComponentHttpStorer,
      'https': ComponentHttpStorer,
      'lib': ComponentLibraryStorer
    }[scheme]
    if (!Storer) throw new ComponentStorerUnknown(scheme)
    return new Storer()
  }

  storer (scheme) {
    scheme = scheme || this.scheme
    return this.constructor.storer(scheme)
  }

  read (address) {
    if (address) this.address = address
    else address = this.address

    let {scheme, format} = this.split()
    return this.storer(scheme)
      .read(address)
      .then(content => {
        this.load(content, format)
      })
      .then(() => {
        return this
      })
  }

  write (address) {
    if (address) this.address = address
    else address = this.address

    let {scheme, path, format, version} = this.split() // eslint-disable-line no-unused-vars
    let content = this.dump(format)
    this.storer(scheme).write(this.address, content)
  }

}

export default Component
