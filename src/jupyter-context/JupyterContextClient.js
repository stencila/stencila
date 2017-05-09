/*
This module implements a `JupyterContext` by passing calls onto 
Jupyter Notebook's Javascript implementation of a `Kernel` client.

Install Jupyter notebook dependencies:

    cd node_modules/jupyter-notebook-deps/
    python setup.py js

The build a bundle for the Jupyterer kernel module on its own (the above creates
bundles for entire notebooks):

    cd ../..
    node make-jupyter-kernel.js vendor

Then open the test file:

    tests/index-jupyter-context.html

`JupyterKernel` should be available globally 
*/
import {GET} from '../util/requests'
import {pack, unpack} from '../value'
import Context from '../context/Context'

/**
 * A Jupyter context
 *
 * Implements the Stencila `Context` API using a Jupyter kernel
 * for code execution
 *
 * @extends Context
 */
export default class JupyterContextClient extends Context {

  constructor (url) {
    super()

    /**
     * URL of the `JupyterContext`
     * @type {string}
     */
    this.url = url

    /**
     * The kernel for this context
     * @type {string}
     */
    this.kernel = null

    /**
     * The langauge for this context
     */
    this.language = null

    /**
     * Connection configuration
     * @type {object}
     */
    this.config = null

    /**
     * The connection to the to the kernel
     * @type {JupyterKernel}
     */
    this._connection = null
  }

  /**
   * Start the Jupyter kernel
   * @return {Promise}
   */
  start () {
    if (this._connection) return Promise.resolve()
    else {
      return GET(this.url).then(data => {
        this.kernel = data.kernel
        this.language = data.spec && data.spec.language && data.spec.language.toLowerCase()
        this.config = data.config
        this._connection = true // new JupyterKernel(/* config parameters to go here */)
      })
    }
  }

  /**
   * Run code within the context's global scope (i.e. execute a code "chunk")
   *
   * @override
   */
  runCode (code) {
    return this.start().then(() => {
      // Run code
      // Get the `execute_result` or `display_data`
    })
  }

  /**
   * Execute code within a local function scope (i.e. execute a code "cell")
   *
   * @override
   */
  callCode (code, args) {
    return this.start().then(() =>{
      // Wrap the code into a "self executing function"
      let wrapper = callCodeWrappers[this.language]
      let selfExecFunc = wrapper(code, args)
      // Execute the self executing function
      return this.runCode(selfExecFunc)
    })
  }

}

const callCodeWrappers = {
  r: (code, args) => {
    return `(function(${Object.keys(args)}) { ${code} })()`  // TODO
  },
  python: (code, args) => {
    return code // TODO
  }
}
