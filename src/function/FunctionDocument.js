import { XMLDocument } from 'substance'

import FunctionSchema from './FunctionSchema'
import { descendantTypes } from '../types'

export default class FunctionDocument extends XMLDocument {

  constructor(...args) {
    super(...args)

    // A list of parameters stored more efficiently
    // for faster lookup when called
    this._params = []

    // A mapping of call type signatures to implemenation type signatures
    this._implems = {}
  }

  getDocTypeParams() {
    return FunctionSchema.getDocTypeParams()
  }

  getXMLSchema() {
    return FunctionSchema
  }

  getRoot() {
    return this.get('function-1')
  }

  getName() {
    return this.getRoot().find('name').text()
  }

  /**
   * Define the implementations of this function within an execution Context
   *
   * A function may have multiple implementations for a language with each implementation
   * overloading the function's parameters in alternative ways.
   * 
   * @param  {String} language The name of the language for the 
   * @param  {Context} context  The context instance within which the function will be implemented
   * @return {Promise} A Promise
   */
  define(language, context) {
    const name = this.getName()
    
    // Extract parameters from the document
    this._params = []
    let $params = this.getRoot().findAll('params param')
    for (let $param of $params) {
      const name = $param.attr('name')
      const type = $param.find('type').text()
      let default_ = $param.find('default')
      if (default_) default_ = createValueFromXML(default_)
      this._params.push({ 
        name: name,
        type: type,
        default: default_ 
      })
    }

    // Extract and create implementations
    const $implems = this.getRoot().findAll(`implem[language=${language}]`)
    let promises = []
    let callSignats = {}
    for (let $implem of $implems) {
      // Get the types for each parameter for this implementation
      let types = $implem.findAll('types type').map($type => $type.text())
      if (types.length) {
        // TODO check that the implementation types are comparable with
        // the parameter types
        if (types.length !== this._params.length) {
          throw new Error(`Function implementation for "${name}" defines a different number of types than there are parameters`)
        }
      } else {
        types = this._params.map(param => param.type)
      }
      
      // Generate the implementation signature, the `mini_` prefix
      // avoids name clashes with native functions in the execution context
      let implemSignat = 'mini_' + name + types.map(type => '_' + type).join('')
      // Generate all possible combinations of call signatures for
      // this implementation
      let callCombos
      for (let type of types) {
        let alts = [type]
        let descendants = descendantTypes[type]
        if (descendants) alts = alts.concat(descendants)

        if (!callCombos) callCombos = alts
        else {
          let newTypeCombos = []
          for (let alt of alts) {
            for (let combo of callCombos) {
              newTypeCombos.push(combo + '_' + alt)
            }
          }
          callCombos = newTypeCombos
        }
      }
      if (!callCombos) {
        callSignats[name] = implemSignat
      } else {
        for (let combo of callCombos) {
          callSignats[name + '_' + combo] = implemSignat
        }
      }
      // Define the function
      let code = $implem.find('code').text()
      let promise = context.defineFunction(implemSignat, code).then()
      promises.push(promise)
    }
    return Promise.all(promises).then(() => {
      this._implems = callSignats
    })
  }

  call(context, args, namedArgs) {
    const name = this.getName()

    // Generate an array of argument values and a call type signature
    let values = []
    let index = 0
    for (let param of this._params) {
      const value = args[index] || namedArgs[param.name] || param.default
      if (!value) throw new Error('Parameter not given and no default value available:' + param.name)
      values.push(value)
      index++
    }

    // Find matching implem
    let callSignat = name + values.map(value => '_' + value.type).join('')
    let implem = this._implems[callSignat]
    if (!implem) throw new Error('No implementation of function matching call signature:' + callSignat)
    
    // Call function and store result for checking elsewhere
    return context.callFunction(implem, values)
  }

  test(language, context) {
    return this.define(language, context).then(() => {
      const $tests = this.getRoot().findAll('test')
      let promises = []
      let results = []
      for (let $test of $tests) {
        // Collate arguments into named and unnamed
        const $args = $test.findAll('arg')
        let args = []
        let namedArgs = {}
        for (let $arg of $args) {
          let name = $arg.attr('name')
          let value = createValueFromXML($arg)
          if (name) namedArgs[name] = value
          else args.push(value)
        }
        // Call function and record result and expected
        let promise = this.call(context, args, namedArgs).then(result => {
          let expected = createValueFromXML($test.find('result'))
          result.expected = expected
          results.push(result)
        })
        promises.push(promise)
      }
      return Promise.all(promises).then(() => {
        return results
      })
    })
  }

}

// Get a value from a XMLDocument node
// A hacky implementation needing to be reworked
// by @michael or @oliver.
// Assumes the value is the first child of the node
// e.g <arg><string>Foo</string><arg>
function createValueFromXML($node) {
  const $value = $node.getChildren()[0]
  return {
    type: $value.id.match(/(\w+)-.*/)[1],
    format: 'text',
    content: $value.text()
  }
}
