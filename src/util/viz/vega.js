import vega from 'vega'

/**
 * Render a Vega specification
 *
 * @param  {string} object   The Vega specification
 */
export function render (spec) {
  // Create a Vega Runtime
  let runtime = vega.parse(spec)
  // Create a Vega View and render it to SVG (see API at https://github.com/vega/vega-view)
  let promise = new vega.View(runtime, {
    renderer: 'svg'
  }).run()
    .toSVG()
  return promise
}
