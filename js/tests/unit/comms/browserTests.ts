import HttpClient from '../../../src/comms/HttpClient'
import WebSocketClient from '../../../src/comms/WebSocketClient'
import WebWorkerClient from '../../../src/comms/WebWorkerClient'

const thing = {type: 'Thing'}

var httpClient = new HttpClient('http://127.0.0.1:2000')
httpClient.execute(thing).then(console.log)

var wsClient = new WebSocketClient('ws://127.0.0.1:2000')
wsClient.execute(thing).then(console.log)

var wwClient = new WebWorkerClient('/webWorkerServer.js')
wwClient.execute(thing).then(console.log)
