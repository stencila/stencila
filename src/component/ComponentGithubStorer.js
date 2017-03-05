import GitHub from 'github-api'

import {ComponentStorerMalformed, ComponentStorerUnfound} from './component-storer-errors'

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

  /**
   * Split an address into parts for Github API
   *
   * @param  {string} address - Address of the component
   * @return {object} - Parts of the component address
   */
  split (address) {
    // Dynamic require here to avoid circular dependence
    let {scheme, path, version} = require('./Component').split(address) // eslint-disable-line no-unused-vars

    let match = path.match(/^([^/]+)\/([^/]+)\/(.+)?/)
    if (!match) {
      throw new ComponentStorerMalformed(this, address)
    }

    return {
      user: match[1],
      repo: match[2],
      file: match[3],
      ref: version || 'master'
    }
  }

  read (address) {
    return new Promise((resolve, reject) => {
      let {user, repo, file, ref} = this.split(address)
      let repository = this.github.getRepo(user, repo)
      repository.getContents(ref, file, 'raw')
        .then(response => {
          resolve(response.data)
        })
        .catch(error => {
          if (error.response.status === 404) {
            reject(new ComponentStorerUnfound(this, address))
          } else {
            reject(error)
          }
        })
    })
  }

  write (address, content) {
    return new Promise((resolve, reject) => {
      let {user, repo, file} = this.split(address)
      let repository = this.github.getRepo(user, repo)
      repository.writeFile('master', file, content, 'Updated', {})
        .then(response => {
          resolve()
        })
        .catch(error => {
          if (error.response.status === 404) {
            reject(new ComponentStorerUnfound(this, address))
          } else {
            reject(error)
          }
        })
    })
  }

}

export default ComponentGithubStorer
