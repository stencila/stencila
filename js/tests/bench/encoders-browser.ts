import JsonRpcRequest from '../../src/comms/JsonRpcRequest'

import Encoder from '../../src/comms/Encoder'
import JsonEncoder from '../../src/comms/JsonEncoder'
import JsonGzipEncoder from '../../src/comms/JsonGzipEncoder'

const encoders: {[key: string]: Encoder} = {
  'json': new JsonEncoder(),
  'json+gzip': new JsonGzipEncoder()
}

const request = new JsonRpcRequest('answer', ['life', 'universe', 'everything'])

function roundtrip (encoder: Encoder, request: JsonRpcRequest) {
  const message = encoder.encode(request)
  encoder.decode(message, JsonRpcRequest)
}

// @ts-ignore 
const suite = new Benchmark.Suite({
  onCycle: function(event: any) {
    const bench = event.target
    console.log(`${bench.name}\t${bench.hz}`)
  }
})

for (let name in encoders) {
  suite.add(name, () => roundtrip(encoders[name], request))
}

console.log(`name\thz`)
suite.run()
