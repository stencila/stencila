import HttpClient from '../../src/comms/HttpClient'
import WebWorkerClient from '../../src/comms/WebWorkerClient'

var httpClient = new HttpClient('http://127.0.0.1:2000')

var worker = new Worker('webWorkerServer.js')
var wwClient = new WebWorkerClient(worker)
