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

  constructor () {
    super()
    
    this.kernel = new JupyterKernel()
  }

  /**
   * Run code within the context's global scope (i.e. execute a code "chunk")
   *
   * @override
   */
  runCode (code, options) {
    // Ask the kernel to execute the code

    // Get the `execute_result` or `display_data` and pack it
    // from mimetype -> value -> pack

    return Promise.resolve()
  }

  /**
   * Execute code within a local function scope (i.e. execute a code "cell")
   *
   * @override
   */
  callCode (code, args, options) {
    // Do we need to initialize the kernel with functions for pack/unpack/value for each language

    // Wrap the code into a "self executing function"
    let wrapper = callCodeWrappers[this.language]
    let selfExecFunc = wrapper(code, args)

    return this.runCode(selfExecFunc)
  }

  /**
   * Does the context provide a function?
   *
   * @override
   */
  hasFunction (name) {
    return Promise.reject(new Error('Not implemented'))
  }

  /**
   * Call a function
   *
   * @override
   */
  callFunction (name, args, options) {
    return Promise.reject(new Error('Not implemented'))
  }

  /**
   * Get the dependencies for a piece of code
   *
   * @override
   */
  codeDependencies (code) {
    return Promise.reject(new Error('Not implemented'))
  }

  /**
   * Complete a piece of code
   *
   * @override
   */
  codeComplete (code) {
    return Promise.reject(new Error('Not implemented'))
  }
}

const callCodeWrappers = {
  r: (code, args) => {
    return `(function(${Object.keys(args)}) { ${code} })()`
  }
}
