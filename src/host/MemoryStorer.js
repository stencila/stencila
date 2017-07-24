/* globals atob, Buffer */

import Storer from './Storer'

/**
 * A Storer for use in memory e.g. when a filesystem is absent
 *
 * Used mainly for in-browser testing
 */
export default class MemoryStorer extends Storer {

  constructor (files = {}) {
    super()
    this._files = files
  }

  /**
   * @override
   */
  readFile (path) {
    return new Promise((resolve, reject) => {
      let file = this._files[path]
      if (typeof file === 'undefined') reject(new Error('File not found'))
      else {
        // If the file looks like it is base64 encoded then decode it to a binary string
        let content
        if (/^([A-Za-z0-9+/]{4})*([A-Za-z0-9+/]{4}|[A-Za-z0-9+/]{3}=|[A-Za-z0-9+/]{2}==)$/.test(file)) {
          if (typeof atob === "function") content = atob(file)
          else content = new Buffer.from(file, 'base64')
        } else {
          content = file
        }
        resolve(content)
      }
    })
  }
  
  /**
   * @override
   */
  writeFile (path, data) {
    return new Promise((resolve) => {
      this._files[path] = data
      resolve()
    })
  }

  /**
   * @override
   */
  readDir (path) {
    return new Promise((resolve) => {
      let regex = path ? `^${path}/(.+)$` : /^([^/]+)$/
      let files = []
      for (let file of Object.keys(this._files)) {
        let match = file.match(regex)
        if (match) files.push(match[1])
      }
      resolve(files)
    })    
  }

}
