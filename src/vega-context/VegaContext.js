const vega = require('vega')

const JsContext = require('../js-context/JsContext')

/**
 * A Vega context
 *
 * Implements the Stencila `Context` API for rendering [Vega](https://vega.github.io/vega/),
 * a visualization grammar using JSON syntax.
 */
class VegaContext extends JsContext {

  /**
   * Prepare code as a specification object
   *
   * @param  {string|object} code - A Javascript code chunk ending with a spec, or a spec itself
   * @param  {object} inputs An object with a data package for each input variable
   * @return {object} - A spec object
   */
  prepare (code, inputs) {
    if (typeof code === 'object') {
      // If code is an object, assume that it is a Vega spec
      // (usually this will only be used internally e.g. in testing)
      return code
    } else {
      // Do `JsContext` execution of the code to get the spec object
      let result = super.execute(code, inputs, {
        pack: false
      })
      if (result.errors) {
        throw new Error(result.errors)
      }
      return result.output
    }
  }

  /**
   * Execute (i.e. render) a Vega specification
   *
   * @param  {string} code   The code chunk
   * @param  {object} inputs An object with a data package for each input variable
   * @return {object}        An object with any `errors` (an object with line numbers as keys) and `outputs` (
   *                         a data package)
   */
  execute (code, inputs) {
    // Prepare code into a Vega spec
    let spec = this.prepare(code, inputs)
    if (!spec) {
      return Promise.resolve({
        errors: null,
        output: null
      })
    }
    // Create a Vega Runtime
    let runtime = vega.parse(spec)
    // Create a Vega View and render it to SVG (see API at https://github.com/vega/vega-view)
    let promise = new vega.View(runtime, {
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

module.exports = VegaContext
