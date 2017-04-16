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
   * Does the context support a programming language?
   *
   * @param {string} language - The language
   */
  supportsLanguage (language) { // eslint-disable-line no-unused-vars
    return Promise.resolve(new Error('Not implemented'))
  }

  /**
   * Run code within the context's global scope
   *
   * @param {string} code - Code to run
   * @param {object} options - Any execution options
   * @return {object} - A Promise resolving to object with any `errors` and `output`
   *
   * @example
   *
   * // You can also assign global variables which are available in subsequent calls,
   * context.call('foo = "bar"\n\n') // { errors: {}, output: null }
   * context.call('foo') // { errors: {}, output: { type: 'str', format: 'text', value: 'bar' } }
   */
  runCode (code, options) { // eslint-disable-line no-unused-vars
    return Promise.reject(new Error('Not implemented'))
  }

  /**
   * Execute code within a local function scope
   *
   * @param {string} code - Code to call
   * @param {object} inputs - An object with a data pack for each argument
   * @param {object} options - Any execution options
   * @return {object} - A Promise resolving to object with any `errors` and `output`
   *
   * @example
   *
   * // Return statement must be used to return an output value
   * context.call('return 6*7') // { errors: {}, output: { type: 'int', format: 'text', value: '42' } }
   * context.call('let x = 6\nreturn x*7') // { errors: {}, output: { type: 'int', format: 'text', value: '42' } }
   *
   * // You can specify inputs (that are local to that call),
   * context.call('return Math.PI*radius', {radius:{type:'flt', format:'text', value: '21.4'}}) // { errors: {}, output: { type: 'flt', format: 'text', value: '67.23008278682157' } }
   * context.call('return radius') // { errors: { '1': 'ReferenceError: radius is not defined' }, output: null }
   */
  callCode (code, inputs, options) { // eslint-disable-line no-unused-vars
    return Promise.reject(new Error('Not implemented'))
  }

  /**
   * Does the context provide a function?
   *
   * @param  {string} name - Function name e.g. 'sum'
   * @return {array<string>} - A Promise resolving to a boolean value
   */
  hasFunction (name) {  // eslint-disable-line no-unused-vars
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
  callFunction (name, args, namedArgs, options) {  // eslint-disable-line no-unused-vars
    return Promise.reject(new Error('Not implemented'))
  }

  /**
   * Get the dependencies for a piece of code
   *
   * Returns an array of all variable names not declared within
   * the piece of code. This might include global functions.
   *
   * @param  {string} code - Piece of code
   * @return {array<string>} - A Promise resolving to a list of dependencies
   */
  codeDependencies (code) {  // eslint-disable-line no-unused-vars
    return Promise.reject(new Error('Not implemented'))
  }

  /**
   * Complete a piece of code
   *
   * @param  {string} code - Piece of code
   * @return {array<string>} - A Promise resolving to a completed piece of code
   */
  codeComplete (code) {  // eslint-disable-line no-unused-vars
    return Promise.reject(new Error('Not implemented'))
  }
}
