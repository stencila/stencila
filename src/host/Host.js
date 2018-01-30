import { EventEmitter } from 'substance'

import { GET, POST, PUT } from '../util/requests'
import FunctionManager from '../function/FunctionManager'
import Engine from '../engine/Engine'
import JsContext from '../contexts/JsContext'
import MiniContext from '../contexts/MiniContext'
import ContextHttpClient from '../contexts/ContextHttpClient'
import MemoryBuffer from '../backend/MemoryBuffer'

/**
 * Each Stencila process has a single instance of the `Host` class which
 * orchestrates instances of other classes.
 */
export default class Host extends EventEmitter {

  constructor (options = {}) {
    super()

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

    /**
     * Execution engine for scheduling execution across contexts
     *
     * @type {Engine}
     */
    this._engine = null


    /**
     * Manages functions imported from libraries
     * 
     * @type {FunctionManager}
     */
    this._functionManager = new FunctionManager(options.libraries)

  }

  // For compatability with Stencila Host Manifest API (as is stored in this._peers)

  /**
   * The URL of this internal host
   */
  get url() {
    return 'internal'
  }

  /**
   * The resource types supported by this internal host
   */
  get types() {
    return {
      JsContext: { name: 'JsContext' },
      MiniContext: { name: 'MiniContext' }
    }
  }

  // Getters...

  /**
   * Get this host's configuration options
   */
  get options () {
    return this._options
  }

  /**
   * Get this host's peers
   */
  get peers () {
    return this._peers
  }

  /**
   * Get the resource instances (e.g. contexts, storers) managed by this host
   */
  get instances() {
    return this._instances
  }

  /**
   * Get the execution contexts managed by this host
   */
  get contexts() {
    return this._contexts
  }

  /**
   * Get this host's peers
   */
  get engine () {
    return this._engine
  }

  /**
   * Get this host's function manager
   */
  get functionManager() {
    return this._functionManager
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

    return Promise.all(promises).then(() => {
      // Instantiate the engine after connecting to any peer hosts so that they are connected to before the engine attempts
      // to create contexts for external languages like R, SQL etc
      this._engine = new Engine(this)
    })
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
          if (manifest.types) {
            for (let type of Object.keys(manifest.types)) {
              specs.push(manifest.types[type])
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

    // Register a created instance
    let _register = (id, host, type, instance) => {
      this._instances[id] = {host, type, instance}
      this.emit('instance:created')
    }

    // Attempt to find resource type amongst...
    let found = find(false) //...peers
    if (!found) found = find(true) //...peers of peers
    if (found) {
      let {url, spec} = found
      return POST(url + '/' + spec.name, args).then(id => {
        // Determine the client class to use (support deprecated spec.base)
        let Client
        if (spec.client === 'ContextHttpClient' || spec.base === 'Context') Client = ContextHttpClient
        else throw new Error(`Unsupported type: ${spec.client}`)

        let instance = new Client(url + '/' + id)
        _register(id, url, type, instance)
        return {id, instance}
      })
    }

    // Fallback to providing an in-browser instances of resources where available
    let instance
    if (type === 'JsContext') {
      instance = new JsContext()
    } else if (type === 'MiniContext') {
      // MiniContext requires a pointer to this host so that
      // it can obtain other contexts for executing functions
      instance = new MiniContext(this)
    } else {
      // Resolve an error so that this does not get caught in debugger during
      // development and instead handle error elsewhere
      return Promise.resolve(new Error(`No peers able to provide: ${type}`))
    }

    // Generate an id for the instance
    let number = (this._counts[type] || 0) + 1
    this._counts[type] = number
    let id = type[0].toLowerCase() + type.substring(1) + number
    _register(id, this.url, type, instance)

    return Promise.resolve({id, instance})
  }

  /**
   * Create an execution context for a particular language
   */
  createContext(language) {
    const context = this._contexts[language]
    if (context) return context
    else {
      const type = {
        'js': 'JsContext',
        'mini': 'MiniContext',
        'py': 'PyContext',
        'r': 'RContext',
        'sql': 'SqliteContext'
      }[language]
      if (!type) {
        return Promise.reject(new Error(`Unable to create an execution context for language ${language}`))
      } else {
        const promise = this.create(type).then((result) => {
          if (result instanceof Error) {
            // Unable to create so set the cached context promise to null
            // so a retry is performed next time this method is called
            // (at which time another peer that provides the context may be available)
            this._contexts[language] = null
            return result
          } else {
            // Get a list of fuctions from the context so that `FunctionManager` can
            // dispatch a `call` operation to the context if necessary. Implemented
            // optimistically i.e. will not fail if the context does not implement `getLibraries`
            const context = result.instance
            if (typeof context.getLibraries === 'function') {
              context.getLibraries().then((libraries) => {
                for (let name of Object.keys(libraries)) {
                  this._functionManager.importLibrary(name, libraries[name])
                }
              }).catch((error) => {
                console.log(error) // eslint-disable-line
              })
            }
            return context
          }
        })
        this._contexts[language] = promise
        return promise
      }
    }
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
    this.emit('peer:registered')
  }

  /**
   * Pokes a URL to see if it is a Stencila Host
   *
   * @param {string} url - A URL for the peer
   */
  pokePeer (url) {
    return GET(url).then(manifest => {
      // Register if this is a Stencila Host manifest
      if (manifest.stencila) {
        // Remove any query parameters from the peer URL
        // e.g. authentication tokens. Necessary because we append strings to this
        // URL for requests to e.g POST http://xxxxx/RContext
        let match = url.match(/^https?:\/\/[\w-.]+(:\d+)?/)
        if (match) url = match[0]
        this.registerPeer(url, manifest)
      }
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
   * Set the `interval` parameter to a value greater than zero to trigger ongoing discovery and
   * to a negative number to turn off discovery.
   *
   * @param {number} interval - The interval (seconds) between discovery attempts
   */
  discoverPeers (interval=10) {
    this.options.discover = interval
    if (interval >= 0) {
      for (let port=2000; port<=2100; port+=10) {
        this.pokePeer(`http://127.0.0.1:${port}`)
      }
      if (interval > 0) {
        this.discoverPeers(-1) // Ensure any existing interval is turned off
        this._dicoverPeersInterval = setInterval(() => this.discoverPeers(0), interval*1000)
      }
    } else {
      if (this._dicoverPeersInterval) {
        clearInterval(this._dicoverPeersInterval)
        this._dicoverPeersInterval = null
      }
    }
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
