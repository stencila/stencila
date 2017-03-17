import vegaLite from 'vega-lite'

import {render as renderVega} from './vega'

export function render(spec) {
  // Compile Vega-Lite spec to Vega spec
  let vegaSpec = vegaLite.compile(spec).spec
  // Render it using Vega
  return renderVega(vegaSpec)
}
