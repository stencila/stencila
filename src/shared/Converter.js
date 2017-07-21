/**
 * Converter to 
 */
export default class Converter {

  /**
   * Should a file be converted using this converter? 
   * 
   * @param  {string} path - The path of the file
   * @param  {object} store - The storer the file resides in
   * @return {boolean} - Convert this file? true/false
   */
  match (path, storer) { // eslint-disable-line
    throw new Error('Converter.match() must be implemented in derived class')
  }

  import (path, storer, buffer) { // eslint-disable-line
    throw new Error('Converter.import() must be implemented in derived class')
  }

  export (path, storer, buffer) { // eslint-disable-line
    throw new Error('Converter.export() must be implemented in derived class')
  }

  /**
   * Parse a file path into elements directory, file, extension
   * 
   * @param  {string} path File path to parse
   * @return {object}      Object of elements
   */
  static parsePath (path) {
    let match = path.match(/^(.*?)\/?([\w-]+\.([\w-]+))?$/)
    return {
      dir: match[1],
      file: match[2] || null,
      ext: match[3] || null
    }
  }
}
