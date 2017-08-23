import {split as addressSplit} from '../address'
import { GET, POST, PUT } from '../util/requests'
import JsContext from '../js-context/JsContext'
import ContextHttpClient from '../context/ContextHttpClient'
import StorerHttpClient from '../storers/StorerHttpClient'
import MemoryStorer from '../storers/MemoryStorer'

/**
 * Each Stencila process has a single instance of the `Host` class which
 * orchestrates instances of other classes.
 */
export default class Host {

  constructor (options = {}) {

    /**
     * Instances managed by this host
     *
     * @type {object}
     */
    this._instances = {}

    /**
     * Peer manifests which detail the capabilities
     * of each of this host's peers
     *
     * @type {object}
     */
    this._peers = {}

    // Peer seeding
    let peers = options.peers
    if (peers) {
      // Add the initial peers
      for (let url of peers) {
        if (url === 'origin') url = options.origin
        this.pokePeer(url)
      }
    }
    // Discover other peers
    if (options.discover) {
      this.discoverPeers(options.discover)
    }
  }

  /**
   * Create a new instance of a resource
   *
   * @param  {string} type - Name of class of instance
   * @param  {string} name - Name for new instance
   * @return {Promise} Resolves to an instance
   */
  create (type, args) {
    // Search for the type amongst peers or peers-of-peers
    let find = (peersOfPeers) => {
      for (let url of Object.keys(this._peers)) {
        let manifest = this._peers[url]

        // Gather an object of types from the peer and it's peers
        let specs = []
        let addSpecs = (manifest) => {
          if (manifest.schemes && manifest.schemes.new) {
            for (let type of Object.keys(manifest.schemes.new)) {
              specs.push(manifest.schemes.new[type])
            }
          }
        }
        if (!peersOfPeers) {
          addSpecs(manifest)
        } else if (manifest.peers) {
          for (let peer of manifest.peers) addSpecs(peer)
        }

        // Find a type that has a matching name or alias
        for (let spec of specs) {
          if (spec.name === type) {
            return {url, spec}
          } else if (spec.aliases) {
            if (spec.aliases.indexOf(type) >= 0) {
              return {url, spec}
            }
          }
        }
      }
    }

    // Attempt to find resource type amongst...
    let found = find(false) //...peers
    if (!found) found = find(true) //...peers of peers
    if (found) {
      let {url, spec} = found
      return POST(url + '/' + spec.name, args).then(address => {
        let Client
        if (spec.base === 'Context') Client = ContextHttpClient
        else if (spec.base === 'Storer') Client = StorerHttpClient
        else throw new Error(`Unsupported type: %{path}`)

        let instance = new Client(url + '/' + address)
        this._instances[address] = instance
        return {address, instance}
      })
    }

    // Fallback to providing an in-browser instances of resources where available
    let Class
    if (type === 'JsContext' || type === 'js') Class = JsContext
    else if (type === 'MemoryStorer') Class = MemoryStorer
    if (Class) {
      let instance = new Class(args)
      const name = () => {
        let number = (this._counts[type] || 0) + 1
        this._counts[type] = number
        return `${type[0].toLowerCase()}${type.substring(1)}${number}`
      }
      let address = `local://${name(type)}`
      this._instances[address] = instance
      return Promise.resolve({address,instance})
    }

    return Promise.reject(new Error(`No peers able to provide: ${type}`))
  }

  /**
   * Create a storer based on the address
   * 
   * @param  {String} address The address of the storer
   * @return {Storer}         Storer instance
   */
  createStorer (address) {
    let {scheme, path, version} = addressSplit(address)
    let storer = {
      'memory': 'MemoryStorer',
      'file': 'FileStorer',
      'github': 'GithubStorer'
    }[scheme]
    if (!storer) return Promise.reject(new Error(`Unknown storer scheme: ${scheme}`))
    else return this.create(storer, { path: path, version: version }).then(result => result.instance)
  }

  /**
   * Create a buffer storer
   *
   * The type of storer to be used as a buffer may depend upon the 
   * storer being used. Currently always use a `MemoryStorer`.
   *
   * @return {Storer} A storer to be used as a buffer
   */
  createBuffer(/*storer*/) {
    return Promise.resolve(new MemoryStorer())
  }

  /**
   * Get an instance
   * @param  {string} address - Address of the instance
   * @return {Promise} Resolves to an instance
   */
  get (address) {
    return Promise.resolve(this._instances[address])
  }

  /**
   * Get this host's peers
   */
  get peers () {
    return this._peers
  }

  /**
   * Register a peer
   *
   * Peers may have several URLs (listed in the manifest's `urls` property; e.g. http://, ws://).
   * The URL used to register a peer is a unique identier used by this host and is
   * usually the URL that the peer was discoved on.
   *
   * @param {string} url - The identifying URL for the peer
   * @param {object} manifest - The peer's manifest
   */
  registerPeer (url, manifest) {
    this._peers[url] = manifest
  }

  /**
   * Pokes a URL to see if it is a Stencila Host
   *
   * @param {string} url - A URL for the peer
   */
  pokePeer (url) {
    GET(url).then(manifest => {
      // Register if this is a Stencila Host manifest
      if (manifest.stencila) this.registerPeer(url, manifest)
    })
  }

  /**
   * Discover peers
   *
   * Currently, this method just does a port scan on the localhost to find
   * peers. More sophisticated peer discovery mechanisms for remote peers
   * will be implemented later.
   *
   * Unfortunately if a port is not open then you'll get a console error like
   * `GET http://127.0.0.1:2040/ net::ERR_CONNECTION_REFUSED`. In Chrome, this can
   * not be avoided programatically (see http://stackoverflow.com/a/43056626/4625911).
   * The easiest approach is silence these errors in Chrome is to check the
   * 'Hide network' checkbox in the console filter.
   *
   * Set the `interval` parameter to trigger ongoing discovery.
   *
   * @param {number} interval - The interval (seconds) between discovery attempts
   */
  discoverPeers (interval=10) {
    for (let port=2000; port<=2100; port+=10) {
      this.pokePeer(`http://127.0.0.1:${port}`)
    }
    if (interval) setTimeout(() => this.discoverPeers(interval), interval*1000)
  }
}
