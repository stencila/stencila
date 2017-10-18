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

    this.configurator = new Configurator().import(FunctionPackage)
  }

  /*
    Import a function library (XML) and register function instances in the manager
  */
  importLibrary(libraryName, xmlString) {
    let dom = DefaultDOMElement.parseXML(xmlString)
    let importer = this.configurator.createImporter('stencila-function')
    let funcs = dom.findAll('function')
    funcs.forEach((func) => {
      let functionName = func.find('name').textContent
      let functionDoc = importer.importDocument(func)
      if (this.functionMap[functionName]) {
        throw new Error(`Function ${functionName} is already defined.`)
      }
      if (!this.functions[libraryName]) {
        this.functions[libraryName] = {}
      }
      this.functions[libraryName][functionName] = functionDoc
      this.functionMap[functionName] = libraryName
    })
  }

  getLibraryName(functionName) {
    return this.functionMap[functionName]
  }

  /*
    Get function instance by name
  */
  getFunction(functionName) {
    let libraryName = this.getLibraryName(functionName)
    if (libraryName) {
      return this.functions[libraryName][functionName]
    }
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
