import { getRowCol } from '../shared/cellHelpers'

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
    // TODO: we want to have a general implementation
    // dealing with cell references and range expressions
    // transpiling the code so that it is syntactically
    // correct in the target language
    let symbols = {}
    code = this._transpile (code, symbols)
    return this._analyseCode(code, exprOnly)
    .then((res) => {
      // Note: external contexts can only analyse
      // input variables, so we have transpiled the code
      // before, and now need to map back
      if (res.inputs) {
        res.inputs = res.inputs.map((name) => {
          if (symbols[name]) {
            return symbols[name]
          } else {
            return {
              type: 'var',
              name
            }
          }
        })
      }
      return res
    })
  }

  _transpile (code, symbols) {
    // TODO: this needs to be maintained carefully
    // to be consistent with the code below
    let re = /\b([A-Z]+[1-9]+)([:]([A-Z]+[1-9]+))?/g
    let m
    while ((m = re.exec(code))) {
      let type
      let name
      if (m[3]) {
        type = 'range'
      } else if (m[1]) {
        type = 'cell'
      }
      switch (type) {
        case 'range': {
          let [startRow, startCol] = getRowCol(m[1])
          let [endRow, endCol] = getRowCol(m[3])
          name = `${m[1]}_${m[3]}`
          symbols[name] = {
            type, name,
            startRow, startCol, endRow, endCol
          }
          break
        }
        case 'cell': {
          let [row, col] = getRowCol(m[1])
          name = m[1]
          symbols[name] = {
            type, name,
            row, col
          }
          break
        }
        default:
          console.error('FIXME: invalid type')
          continue
      }
      code = code.substring(0, m.index) + name + code.substring(m.index+name.length)
    }
    return code
  }

  _analyseCode (code, exprOnly) { // eslint-disable-line
    return Promise.reject(new Error('Not implemented'))
  }

  /**
   * Execute code within the context
   *
   * @param {string} code - Code to execute
   * @param {object} inputs - Value of input variables
   * @param {object} exprOnly - Check that code is a simple expression only?
   */
  executeCode (code = '', inputs = {}, exprOnly = false) { // eslint-disable-line no-unused-vars
    // TODO: we want to have a general implementation
    // dealing with cell references and range expressions
    // transpiling the code so that it is syntactically
    // correct in the target language
    let symbols = {}
    code = this._transpile (code, symbols)
    return this._executeCode(code, inputs, exprOnly)
    .then((res) => {
      // Note: external contexts can only analyse
      // input variables, so we have transpiled the code
      // before, and now need to map back
      if (res.inputs) {
        res.inputs = res.inputs.map((name) => {
          if (symbols[name]) {
            return symbols[name]
          } else {
            return {
              type: 'var',
              name
            }
          }
        })
      }
      return res
    })
  }

  _executeCode (code = '', inputs = {}, exprOnly = false) { // eslint-disable-line no-unused-vars
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

}
