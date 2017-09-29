import { Configurator, DefaultDOMElement } from 'substance'
import FunctionPackage from './FunctionPackage'

/*
  Provides a Javascript API to create, update and lookup functions.

  Think of this as an in-memory function library. It does not parse or
  run functions, only the data is stored here for reflection.

  FunctionManager is used by the cell engine to lookup function calls, pick the
  right implementation and runs it.
*/
export default class FunctionManager {

  constructor() {
    // Maps function names to the library in which they have been defined
    this.functionMap = {}
    // Holds function instances scoped by libraryName and functionName
    this.functions = {}
    this.implementations = {}
    let configurator = new Configurator()
    configurator.import(FunctionPackage)
    this.configurator = configurator
  }

  /*
    Import a function library (XML) and register function instances in the manager
  */
  importLibrary(libraryName, xmlString, implementations) {
    let dom = DefaultDOMElement.parseXML(xmlString)
    let importer = this.configurator.createImporter('stencila-function')
    let funcs = dom.findAll('function')
    funcs.forEach((func) => {
      let functionName = func.find('name').textContent
      let functionDoc = importer.importDocument(func)
      this.functions[libraryName] = {}
      this.functions[libraryName][functionName] = functionDoc
      if (this.functionMap[functionName]) {
        throw new Error(`Function ${functionName} is already defined.`)
      }
      this.functionMap[functionName] = libraryName
      this.implementations[libraryName] = implementations
    })
  }

  /*
    Get function instance by name
  */
  getFunction(functionName) {
    let libraryName = this.getLibraryNameForFunction(functionName)
    return this.functions[libraryName][functionName]
  }

  getLibraryNameForFunction(functionName) {
    return this.functionMap[functionName]
  }

  /*
    Get a list of available function names
  */
  getFunctionNames() {
    return Object.keys(this.functionMap)
  }

  // Reflection

  /*
    Get the function signature for inspection
  */
  getSignature(name) {
    return this.functions[name].getSignature()
  }

}
