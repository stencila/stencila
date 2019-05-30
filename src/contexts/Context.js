/**
 * Abstract base class for a Stencila execution context
 *
 * Defines the Stencila `Context` API. The same methods (names and arguments) will be
 * implemented for all contexts regardless of implementation language. Semantics should be
 * consistent, but may need to differ, among implmentations.
 *
 * This class should be extended for JavaScript implementations. All methods return a Promise.
 */
export default class Context {

  /**
   * Get the list of supported programming languages
   *
   * @override
   */
  supportedLanguages () {
    return Promise.resolve(new Error('Not implemented'))
  }

  /**
   * Analyse code and return the names of inputs, output and
   * implicitly returned value expression
   *
   * @param {string} code - Code to execute
   * @param {object} exprOnly - Check that code is a simple expression only?
   */
  analyseCode (code, exprOnly = false) { // eslint-disable-line no-unused-vars
    return this._analyseCode(code, exprOnly)
  }

  /**
   * Execute code within the context
   *
   * @param {string} code - Code to execute
   * @param {object} inputs - Value of input variables
   * @param {object} exprOnly - Check that code is a simple expression only?
   */
  executeCode (code = '', inputs = {}, exprOnly = false) { // eslint-disable-line no-unused-vars
    return this._executeCode(code, inputs, exprOnly)
  }

  /**
   * Does the context provide a function?
   *
   * @param  {string} name - Function name e.g. 'sum'
   * @return {array<string>} - A Promise resolving to a boolean value
   */
  hasFunction (name) { // eslint-disable-line no-unused-vars
    return Promise.reject(new Error('Not implemented'))
  }

  /**
   * Call a function
   *
   *
   * @param  {string} name - Function name e.g. 'sum'
   * @param {array} args - An array of unnamed arguments
   * @param {namedArgs} args - An object of named arguments
   * @param {object} options - Any execution options
   * @return {array<string>} - A Promise resolving to an object with any `errors` (an object with line numbers as keys) and `outputs` (
   *                         a data package)
   */
  callFunction (name, args, namedArgs, options) { // eslint-disable-line no-unused-vars
    return Promise.reject(new Error('Not implemented'))
  }

  _analyseCode (code, exprOnly) { // eslint-disable-line
    return Promise.reject(new Error('Not implemented'))
  }

  _executeCode (code = '', inputs = {}, exprOnly = false) { // eslint-disable-line no-unused-vars
    return Promise.reject(new Error('Not implemented'))
  }
}
