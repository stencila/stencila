/**
 * Implements the `Storer` API in memory.
 * Used as a buffer and for testing
 */
export default class MemoryStorer {
  
  constructor(options = {}) {
    this._dir = '/'
    this._main = options.path

    let files
    if (typeof window !== 'undefined') files = window.STENCILA_MEMORY_STORER_FILES
    if (!files) files = {}
    this._files = files
  }

  getDirectory() {
    return Promise.resolve(this._dir)
  }

  getMain() {
    return Promise.resolve(this._main)
  }

  getFiles() {
    return new Promise((resolve) => {
      resolve(Object.keys(this._files))
    })
  }

  getInfo() {
    return new Promise((resolve) => {
      resolve({
        dir: this._dir,
        main: this._main,
        files: Object.keys(this._files)
      })
    })  
  }

  readFile(path) {
    return new Promise((resolve, reject) => {
      let file = this._files[path]
      if (file) resolve(file)
      else reject(new Error('File not found'))
    })
  }

  writeFile(path, content) {
    return new Promise((resolve) => {
      this._files[path] = content
      resolve()
    })
  }

  deleteFile(path) {
    return new Promise((resolve) => {
      delete this._files[path]
      resolve()
    })
  }

}
