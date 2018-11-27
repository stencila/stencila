import WebWorkerClient from '../../src/comms/WebWorkerClient'

var worker = new Worker('webWorkerServer.js')
var client = new WebWorkerClient(worker)
