import Storer from './Storer'


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
      if (file) resolve(file)
      else reject(new Error('File not found'))
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
