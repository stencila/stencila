import {pack, unpack} from './src/packing'

import Document from './src/document/Document'

import JsContext from './src/js-context/JsContext'
import VegaContext from './src/vega-context/VegaContext'
import VegaLiteContext from './src/vega-lite-context/VegaLiteContext'

export default {
  pack: pack,
  unpack: unpack,

  Document: Document,

  JsContext: JsContext,
  VegaContext: VegaContext,
  VegaLiteContext: VegaLiteContext
}
