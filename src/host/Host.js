import { GET, POST, PUT } from '../util/requests'
import JsContext from '../js-context/JsContext'
import ContextHttpClient from '../context/ContextHttpClient'
import MemoryBuffer from '../backend/MemoryBuffer'

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

    // Add the Host who served this file as a peer
    if (window) this.pokePeer(window.location.origin)

    // Discover other peers
    if (options.discover) this.discoverPeers(options.discover)
  }

  /**
   * Create a new instance
   * 
   * @param  {string} type - Name of class of instance
   * @param  {string} name - Name for new instance
   * @return {Promise} Resolves to an instance
   */
  post (type, name) {
    if (type === 'JsContext' || type === 'js') {
      let instance = new JsContext()
      let address = `name://${name || Math.floor((1 + Math.random()) * 1e6).toString(16)}`
      this._instances[address] = instance
      return Promise.resolve(instance)
    }
    else {
      for (let url of Object.keys(this._peers)) {
        let manifest = this._peers[url]
        
        // Gather an object of types from the peer and it's peers
        let types = {}
        let addTypes = function (manifest) {
          if (manifest.schemes && manifest.schemes.new) types = Object.assign(types, manifest.schemes.new)
        }
        addTypes(manifest)
        for (let peer of manifest.peers) addTypes(peer)

        // If the type is available, use it, otherwise search through aliases
        let spec = types[type]
        if (!spec) {
          for (let trialType of Object.keys(types)) {
            let trialSpec = types[trialType]
            if (trialSpec.aliases) {
              for (let alias of trialSpec.aliases) {
                if (alias === type) {
                  spec = trialSpec
                  break
                }
              }
              if (spec) break
            }
          }
        }

        // Type found so request a new one
        if (spec) {
          return POST(url + '/' + spec.name, { name: name }).then(address => {
            let instance
            if (spec.base === 'Context') {
              instance = new ContextHttpClient(url + '/' + address)
            } else {
              throw new Error(`Unsupported type: %{path}`)
            }
            this._instances[address] = instance
            return instance
          })
        }
      }

      return Promise.reject(new Error(`No peers able to provide: ${type}`))
    }
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
