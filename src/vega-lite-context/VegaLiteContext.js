const vega = require('vega')
const vegaLite = require('vega-lite')

const JsContext = require('../js-context/JsContext')

/**
 * A Vega-Lite context
 *
 * Implements the Stencila `Context` API for rendering [Vega-Lite](https://vega.github.io/vega-lite/),
 * a high-level visualization grammar using JSON syntax.
 */
class VegaLiteContext extends JsContext {

  /**
   * Execute (i.e. render) a Vega-Lite vizualization specification
   *
   * @param  {String} code   The code chunk
   * @param  {Object} inputs An object with a data package for each input variable
   * @return {Object}        An object with any `errors` (an object with line numbers as keys) and `outputs` (
   *                         a data package)
   *
   * @example
   *
   * // Evaluate an expression...
   * context.execute('{data: {values: ...}, encoding: {x: ...}}')
   */
  execute (code, inputs) {
    let vegaLiteSpec
    if (typeof code === 'object') {
      // If code is an object, assume that it is a  Vega-Lite spec
      // (usually this will only be used internally e.g. in testing)
      vegaLiteSpec = code
    } else {
      // Do `JsContext` execution in case of preparatory code
      let result = super.execute(code, inputs, {
        pack: false
      })
      if (result.errors) return Promise.resolve(result)
      else vegaLiteSpec = result.output
    }

    // Compile Vega-Lite spec to Vega spec
    const vegaSpec = vegaLite.compile(vegaLiteSpec).spec
    // Create a Vega Runtime
    let vegaRuntime = vega.parse(vegaSpec)
    // Create a Vega View and
    // render it to SVG (see API at https://github.com/vega/vega-view)
    let promise = new vega.View(vegaRuntime, {
      renderer: 'svg'
    }).run()
      .toSVG()

    return promise.then(svg => {
      return {
        errors: null,
        output: {
          type: 'img',
          format: 'svg',
          value: svg
        }
      }
    })
  }

}

module.exports = VegaLiteContext
