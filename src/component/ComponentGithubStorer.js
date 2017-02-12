const GitHub = require('github-api')

const errors = require('./component-storer-errors')

class ComponentGithubStorer {

  constructor () {
    this.github = new GitHub()
  }

  login (token) {
    this.github = new GitHub({
      token: token
    })
    return this
  }

  logout () {
    this.github = new GitHub()
    return this
  }

  split (address) {
    const Component = require('./Component') // Dynamic require to avoid circular dependence
    let {scheme, path, version} = Component.split(address) // eslint-disable-line no-unused-vars

    let match = path.match(/^([^/]+)\/([^/]+)\/(.+)?/)
    if (!match) {
      throw new errors.ComponentStorerMalformed(this, address)
    }

    return {
      user: match[1],
      name: match[2],
      file: match[3],
      ref: version || 'master'
    }
  }

  read (address) {
    return new Promise((resolve, reject) => {
      let {user, name, file, ref} = this.split(address)
      let repo = this.github.getRepo(user, name)
      repo.getContents(ref, file, 'raw')
        .then(response => {
          resolve(response.data)
        })
        .catch(error => {
          if (error.response.status === 404) {
            reject(new errors.ComponentStorerUnfound(this, address))
          } else {
            reject(error)
          }
        })
    })
  }

  write (address, content) {
    return new Promise((resolve, reject) => {
      let {user, name, file} = this.split(address)
      let repo = this.github.getRepo(user, name)
      repo.writeFile('master', file, content, 'Updated', {})
        .then(response => {
          resolve()
        })
        .catch(error => {
          if (error.response.status === 404) {
            reject(new errors.ComponentStorerUnfound(this, address))
          } else {
            reject(error)
          }
        })
    })
  }

}

module.exports = ComponentGithubStorer
