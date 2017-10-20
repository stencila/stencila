import { XMLDocument } from 'substance'

import FunctionSchema from './FunctionSchema'

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
    return ['js']
  }

  /*
    Get parameters 
   */
  getParams() {
    if (!this._params) {
      this._params = []
      let paramEls = this.getRootNode().findAll('param')
      for (let paramEl of paramEls) {
        const name = paramEl.attr('name')
        const type = paramEl.attr('type')
        
        let defaultValue
        const defaultEl = paramEl.find('default')
        if (defaultEl) {
          defaultValue = {
            type: defaultEl.attr('type'),
            data: defaultEl.text()
          }
        }

        this._params.push({
          name: name,
          type: type,
          default: defaultValue
        })
      }
    }
    return this._params
  }

  /*
    Get most basic usage example (to be displayed in popover)

    TODO: We just need to store a simple <usage-example> element here, more
    complex usages could live in a separate rich documentation field (JATS body)
  */
  getUsageExamples() {
    return [
      'sum(A1:A5)',
      'sum(1,4)'
    ]
  }

  /*
    Returns a summary as plain text.
  */
  getSummary() {
    let summary = this.find('summary')
    return summary.textContent
  }


  /*
    NOTE: Used to populate FunctionUsage Component

    {
      name: 'sum',
      summary: 'Returns the sum of a range',
      examples: [
        'sum(A1:A5)'
      ],
      params: [
        { name: 'range', type: 'range', description: 'A range (array of numbers) to be summed up' }
      ],
      returns: { type: 'number', description: 'The sum of numbers in the given range'}
    }
  */
  getSpec() {
    return {
      name: this.getName(),
      summary: this.getSummary(),
      examples: this.getUsageExamples(),
      // STUB!
      params: [
        { name: 'range', type: 'range', description: 'A range (array of numbers) to be summed up' }
      ],
      // STUB!
      returns: { type: 'number', description: 'The sum of numbers in the given range'}
    }
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

}
