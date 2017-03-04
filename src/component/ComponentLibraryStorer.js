import ComponentGithubStorer from './ComponentGithubStorer'

/**
 * A component storer for the Stencila component library
 * currently at http://github.com/stencila/lib
 */
class ComponentLibraryStorer extends ComponentGithubStorer {

  /**
   * Split an address into parts for Github API
   *
   * @param  {string} address - Address of the component
   * @return {object} - Parts of the component address
   */
  split (address) {
    // Dynamic require here to avoid circular dependence
    let {scheme, path, version} = require('./Component').split(address) // eslint-disable-line no-unused-vars

    return {
      user: 'stencila',
      repo: 'lib',
      file: path,
      ref: version || 'master'
    }
  }

}

export default ComponentLibraryStorer
