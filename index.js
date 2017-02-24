const {pack, unpack} = require('./src/packing')

const Document = require('./src/document/Document')

const JsContext = require('./src/js-context/JsContext')
const VegaContext = require('./src/vega-context/VegaContext')
const VegaLiteContext = require('./src/vega-lite-context/VegaLiteContext')

module.exports = {
  pack: pack,
  unpack: unpack,

  Document: Document,

  JsContext: JsContext,
  VegaContext: VegaContext,
  VegaLiteContext: VegaLiteContext
}
