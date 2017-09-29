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
    console.error('DEPRECATED: use doc.getRootNode() instead')
    return this.getRootNode()
  }

  getRootNode() {
    return this.get('function')
  }

  getName() {
    return this.get('name').text()
  }

  getImplementation(language) {
    // TODO: find a proper implementation
    let impl = this.getRoot().find(`implem[language=${language}]`)
    let code = impl.find('code')
    if (code) {
      return code.textContent
    }
  }

  // TODO: Specify available implementations in XML and expose as array of
  //       language names
  getImplementations() {
    return ['javascript']
  }

  /*
    Get most basic usage example (to be displayed in popover)

    TODO: We just need to store a simple <usage-example> element here, more
    complex usages could live in a separate rich documentation field (JATS body)
  */
  getUsageExample() {
    return 'sum(1,5)'
  }

  /*
    Returns a summary as plain text.
  */
  getSummary() {
    let summary = this.find('summary')
    return summary.textContent
  }

  /*
    Extract a json representation.

    @example
    {
      params: [
        { name: 'value1', type: 'number', description: 'The first number or range to add together.' },
        { name: 'value2', type: 'number', repeatable: true, description: 'Additional numbers or ranges to add to `value1`.' }
      ],
      returns: { type: 'number', description: 'The sum of a series of numbers and/or cells.'}
    }
  */
  getSignature() {
    let signature = {
      params: [],
      returns: undefined
    }
    const params = this.get('params')
    if (params) {
      params.children.forEach((paramEl) => {
        let param = {}
        param.name = paramEl.attr('name')
        param.type = paramEl.attr('type')
        const _descr = paramEl.find('description')
        if (_descr) {
          param.description = _descr.textContent
        }
        const _default = paramEl.find('default')
        if (_default) {
          param.default = {
            type: _default.attr('type'),
            // TODO: cast the value
            value: _default.textContent
          }
        }
        signature.params.push(param)
      })
    }
    const ret = this.get('return')
    if (ret) {
      signature.returns = {
        type: undefined,
        description: ''
      }
      signature.returns.type = ret.attr('type')
      let descr = ret.find('description')
      if (descr) {
        signature.returns.description = descr.textContent
      }
    }
    return signature
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
      const type = $param.attr('type')
      let default_ = $param.find('default')
      if (default_) default_ = createValueFromXML(default_)
      this._params.push({
        name: name,
        type: type,
        default: default_
      })
    }

    // Extract and create implementations
    // Note that currently any overloads involving type specialisation
    // must follow the definition of the more general implementation
    let callSignats = {}
    let promises = []
    const $implems = this.getRoot().findAll(`implem[language=${language}]`)
    let implemIndex = 0
    for (let $implem of $implems) {
      implemIndex++
      // Get the types for each parameter for this implementation
      let types = $implem.findAll('types type').map($type => $type.attr('type'))
      if (types.length) {
        if (types.length !== this._params.length) {
          throw new Error(`Function "${name}", implementation "${implemIndex}" defines a different number of types (${types.length}) than parameters (${this._params.length}) `)
        } else {
          for (let i = 0; i < this._params.length; i++) {
            let paramType = this._params[i].type
            let implemType = types[i]
            if (implemType !== paramType) {
              if (descendantTypes[paramType].indexOf(implemType) < 0) {
                throw new Error(`Function "${name}", implementation "${implemIndex}" defines a type "${implemType}" which is inconsistent with parameter type "${paramType}"`)
              }
            }
          }
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
      let promise = context.defineFunction(implemSignat, code)
      promises.push(promise)
    }
    this._implems = callSignats

    return Promise.all(promises)
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
    let implemSignat = this._implems[callSignat]
    if (!implemSignat) throw new Error(`Function "${name}" does not have an implementation for call "${callSignat}". Available implementations are "${Object.keys(this._implems)}".`)

    // Call function and store result for checking elsewhere
    return context.callFunction(implemSignat, values)
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
function createValueFromXML(node) {
  return {
    type: node.attr('type'),
    format: 'text',
    content: node.text()
  }
}
