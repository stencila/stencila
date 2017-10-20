import { GET, POST, PUT } from '../util/requests'
import JsContext from '../contexts/JsContext'
import MiniContext from '../contexts/MiniContext'
import ContextHttpClient from '../contexts/ContextHttpClient'
import MemoryBuffer from '../backend/MemoryBuffer'

/**
 * Each Stencila process has a single instance of the `Host` class which
 * orchestrates instances of other classes.
 */
export default class Host {

  constructor (options = {}) {
    /**
     * Options used to configure this host
     *
     * @type {object}
     */
    this._options = options

    /**
     * Instances managed by this host
     *
     * @type {object}
     */
    this._instances = {}

    /**
     * Execution contexts are currently managed separately to 
     * ensure that there is only one for each language
     *
     * @type {object}
     */
    this._contexts = {}

    /**
     * Counts of instances of each class.
     * Used for consecutive naming of instances
     *
     * @type {object}
     */
    this._counts = {}

    /**
     * Peer manifests which detail the capabilities
     * of each of this host's peers
     *
     * @type {object}
     */
    this._peers = {}
  }

  /**
   * Initialize this host
   *
   * @return {Promise} Initialisation promise
   */
  initialize () {
    const options = this._options

    let promises = [Promise.resolve()]
      
    // Seed with specified peers
    let peers = options.peers
    if (peers) {
      // Add the initial peers
      for (let url of peers) {
        if (url === 'origin') url = options.origin
        let promise = this.pokePeer(url)
        promises.push(promise)
      }
    }

    // Start discovery of other peers
    if (options.discover) {
      this.discoverPeers(options.discover)
    }

    return Promise.all(promises)
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
      return POST(url + '/' + spec.name, args).then(id => {
        let Client
        if (spec.base === 'Context') Client = ContextHttpClient
        else throw new Error(`Unsupported type: %{path}`)

        let instance = new Client(url + '/' + id)
        this._instances[id] = instance
        return {id, instance}
      })
    }

    // Fallback to providing an in-browser instances of resources where available
    let instance
    if (type === 'JsContext') {
      instance = JsContext()
    } else if (type === 'MiniContext') {
      // MiniContext requires a pointer to this host so that
      // it can obtain other contexts for executing functions
      instance = MiniContext(this)
    } else {
      return Promise.reject(new Error(`No peers able to provide: ${type}`))
    }

    // Generate an id for the instance
    let number = (this._counts[type] || 0) + 1
    this._counts[type] = number
    let id = type[0].toLowerCase() + type.substring(1) + number
    this._instances[id] = instance

    return Promise.resolve({id, instance})
  }

  /**
   * Create an execution context for a particular language
   */
  createContext(language) {
    return new Promise((resolve, reject) => {
      const context = this._contexts[language]
      if (context) return context
      else {
        const type = {
          'js': 'JsContext',
          'mini': 'Context',
          'py': 'PyContext',
          'r': 'RContext',
          'sql': 'SqliteContext'
        }[language]
        if (!type) {
          return reject(new Error(`Unable to create an execution context for language ${language}`))
        } else {
          return this.create(type).then(result => {
            let {instance} = result
            this._contexts[language] = instance
            return instance
          })
        }
      }
    })
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

  // Experimental
  // Implements methods of `Backend` so that this `Host` can serve as a backend
  // Towards merging these two classes

  getBuffer(address) {
    // TODO this PUTs to the current server but it could be some other peer
    return PUT(`/${address}!buffer`).then(data => {
      let buffer = new MemoryBuffer()

      buffer.writeFile('stencila-manifest.json', 'application/json', JSON.stringify({
        type: 'document',
        title: 'Untitled',
        createdAt: '2017-03-10T00:03:12.060Z',
        updatedAt: '2017-03-10T00:03:12.060Z',
        storage: {
          storerType: "filesystem",
          contentType: "html",
          archivePath: "",
          mainFilePath: "index.html"
        }
      }))

      buffer.writeFile('index.html', 'text/html', data['index.html'])

      return buffer
    })
  }

  storeBuffer(/*buffer*/) {
    return Promise.resolve()
  }

  updateManifest(/* documentId, props */) {
    return Promise.resolve()
  }

}
