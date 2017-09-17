import { XMLDocument } from 'substance'

import FunctionSchema from './FunctionSchema'
import { descendantTypes } from '../types'

export default class FunctionDocument extends XMLDocument {

  constructor(...args) {
    super(...args)

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
   * Define the implementations of this function for a given context
   *
   * A function may have multiple implementations for a language for
   * type overloading of parameters. This registered each implementation using
   * a mangled name e.g.
   *
   *   myfunc = func r(arg1: number, arg2: string)
   *   
   *     myfunc_number_string
   * 
   * @param  {[type]} language [description]
   * @param  {[type]} context  [description]
   * @return {Array<String>} Names of the functions defined for each implementation
   */
  define(language, context) {
    const name = this.getName()
    
    const $params = this.getRoot().findAll('params param')

    const $implems = this.getRoot().findAll(`implem[language=${language}]`)
    let promises = []
    let callSignats = {}
    for (let $implem of $implems) {
      // Get the types for each parameter for this implementation
      let $parTypes = $implem.findAll('types type')
      let parTypes = $parTypes.map($type => $type.text())
      // If the 
      
      // Generate the implementation signature
      let implemSignat = parTypes.map(parType => '_' + parType).join('')
      // Generate all possible combinations of call signatures for
      // this implementation
      let callCombos
      for (let type of parTypes) {
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
    
    // Generate the type signature for the call
    // TODO: currrently only using unamed arguments
    let types = args.map(arg => arg.type)

    // Find matching implem
    let callSignat = name + types.map(type => '_' + type).join('')
    let implem = this._implems[callSignat]
    if (!implem) throw new Error('No implementation of function matching call signature:' + callSignat)
    
    // Call function and store result for checking elsewhere
    return context.callFunction(implem, args, namedArgs)
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
          let value = JSON.parse($arg.text())
          if (name) namedArgs[name] = value
          else args.push(value)
        }
        // Call function and record result and expected
        let promise = this.call(context, args, namedArgs).then(result => {
          let expected = JSON.parse($test.find('result').text())
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
