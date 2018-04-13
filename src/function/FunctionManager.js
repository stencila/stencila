import { Configurator } from 'substance'
import FunctionPackage from './FunctionPackage'

/*
  Provides a Javascript API to create, update and lookup functions.

  Think of this as an in-memory function library. It does not parse or
  run functions, only the data is stored here for reflection.

  FunctionManager is used by the cell engine to lookup function calls, pick the
  right implementation and runs it.
*/
export default class FunctionManager {

  constructor(libraries = null) {
    // Maps function names to the library in which they have been defined
    this.functionMap = {}
    // Holds function instances scoped by libraryName and functionName
    this.functions = {}

    this.configurator = new Configurator().import(FunctionPackage)

    if (libraries) this.importLibraries(libraries)
  }

  /*
    Import a function library (XML) and register function instances in the manager
  */
  importLibrary(context, library) {
    for (let functionName of Object.keys(library.funcs)) {
      const record = this.functionMap[functionName]
      if (record && record.library !== library.name) {
        throw new Error(`Function "${functionName}" is already defined in library "${record.library}"`)
      }
      this.functionMap[functionName] = { context, library: library.name }
    }
    this.functions[library.name] = library.funcs
  }

  /**
   * Import a set of libraries
   * 
   * @param  {object} libraries An object of libraries like `{name:xml}`
   */
  importLibraries(libraries) {
    Object.keys(libraries).forEach((name) => {
      this.importLibrary(name, libraries[name])
    })
  }

  getContextLibrary(functionName) {
    return this.functionMap[functionName]
  }

  /*
    Get function instance by name
  */
  getFunction(functionName) {
    let record = this.functionMap[functionName]
    if (record) {
      return this.functions[record.library][functionName]
    }
  }

  /*
    Get a list of available function names
  */
  getFunctionNames() {
    return Object.keys(this.functionMap)
  }
}
