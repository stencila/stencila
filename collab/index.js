import ws from 'ws'

import CollabServer from 'substance/collab/CollabServer'

import DocumentStore from './DocumentStore'
import ChangeStore from './ChangeStore'
import SnapshotStore from './SnapshotStore'
import ModelFactory from './ModelFactory'
import SnapshotEngine from './SnapshotEngine'
import DocumentEngine from './DocumentEngine'
import DocumentServer from './DocumentServer'

/**
 * Instantiates
 *  - a `CollabServer` and binds it to a HTTP server, and
 *  - a `DocumentServer` and binds it to an endpoint on an Express application
 *
 * @param      {<type>}  httpServer  The http server
 * @param      {<type>}  expressApp  The express application
 * @param      {<type>}  endpoint        The endpoint
 * @return     {Object}  { description_of_the_return_value }
 */
var bind = function (httpServer, expressApp, endpoint) {
  endpoint = endpoint || '/'

  var documentStore = new DocumentStore()
  var changeStore = new ChangeStore()
  var snapshotStore = new SnapshotStore()
  var modelFactory = new ModelFactory()

  var snapshotEngine = new SnapshotEngine({
    documentStore: documentStore,
    changeStore: changeStore,
    snapshotStore: snapshotStore,
    modelFactory: new ModelFactory()
  })

  var documentEngine = new DocumentEngine({
    documentStore: documentStore,
    changeStore: changeStore,
    snapshotEngine: snapshotEngine,
    modelFactory: modelFactory
  })

  var websocketServer = new ws.Server({
    server: httpServer
  })

  var collabServer = new CollabServer({
    heartbeat: 30 * 1000,
    documentEngine: documentEngine
  })
  collabServer.bind(websocketServer)

  var documentServer = new DocumentServer({
    path: endpoint,
    documentEngine: documentEngine
  })
  documentServer.bind(expressApp)

  return {
    documentStore: documentStore,
    changeStore: changeStore,
    snapshotStore: snapshotStore,
    modelFactory: modelFactory,
    snapshotEngine: snapshotEngine,
    documentEngine: documentEngine,
    collabServer: collabServer,
    documentServer: documentServer
  }
}

export default {
  bind: bind
}
