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
export default class JupyterContext extends Context {

  constructor () {
    super()
    
    this.kernel = new JupyterKernel()
  }

}
