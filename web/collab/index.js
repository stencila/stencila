var CollabServer = require('substance/collab/CollabServer');
var DocumentServer = require('substance/collab/DocumentServer');

var ws = require('ws');

var DocumentStore = require('./DocumentStore');
var ChangeStore = require('./ChangeStore');
var SnapshotStore = require('./SnapshotStore');
var ModelFactory = require('./ModelFactory');
var SnapshotEngine = require('./SnapshotEngine');
var DocumentEngine = require('./DocumentEngine');


var bind = function(httpServer, expressApp) {

  var documentStore = new DocumentStore();
  var changeStore = new ChangeStore();
  var snapshotStore = new SnapshotStore();
  var modelFactory = new ModelFactory();

  var snapshotEngine = new SnapshotEngine({
    documentStore: documentStore,
    changeStore: changeStore,
    snapshotStore: snapshotStore,
    modelFactory: new ModelFactory()
  });

  var documentEngine = new DocumentEngine({
    documentStore: documentStore,
    changeStore: changeStore,
    snapshotEngine: snapshotEngine,
    modelFactory: modelFactory
  });

  var websocketServer = new ws.Server({
    server: httpServer
  });

  var collabServer = new CollabServer({
    heartbeat: 30*1000,
    documentEngine: documentEngine
  });
  collabServer.bind(websocketServer);

  var documentServer = new DocumentServer({
    path: '/jam',
    documentEngine: documentEngine
  });
  documentServer.bind(expressApp);

}

module.exports = {
  bind: bind
};
