
/*
  Provides a Javascript API to create, update and lookup functions.

  Think of this as an in-memory function library. It does not parse or
  run functions only

  It is used by the cell engine to lookup function calls, pick the best
  available implementation and run it.
*/

export default class FunctionManager {

  constructor() {
    this.functions = {}
  }

  /*
    Create a function record with a given signature

    fm.createFunction('sum,' {
      params: [
        { name: 'value1', type: 'number', description: 'The first number or range to add together.' },
        { name: 'value2', type: 'number', repeatable: true, description: 'Additional numbers or ranges to add to `value1`.' }
      ],
      returns: { type: 'number', description: 'The sum of a series of numbers and/or cells.'}
    })
  */
  createFunction(name, signature) {
    // NOTE: function has no implementation at first
    this.functions[name] = {
      signature,
      implementations: {}
    }
  }

  /*
    Adds a new implementation to a function. An existing implementation
    will be overridden.

    fm.addImplementation('sum', 'javascript', 'return value1 + value2')
  */
  addImplementation(funcName, language, sourceCode) {
    this.functions[funcName].implementations[language] = sourceCode
  }

  removeImplementation(funcName, language) {
    delete this.functions[funcName].implementations[language]
  }

  /*
    Import and register function based on the given XML serialized version. This can
    be used for seeding the core function library.
  */
  importFunction(name, xmlString) {
    // TODO: import from xmlString
  }

  exportFunction(name) {
    // TODO: serialize as XML and return as string
  }

  /*
    Get the function signature for inspection
  */
  getSignature(name) {
    return this.functions[name].signature
  }

  /*
    fm.getImplementation('sum', 'javascript', ['number', 'number'])
  */
  getImplementation(name, language) {
    let { implementations } = this.functions[name]
    return implementations[language]
  }

}
