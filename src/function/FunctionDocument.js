import { XMLDocument } from 'substance'

import FunctionSchema from './FunctionSchema'

export default class FunctionDocument extends XMLDocument {

  getDocTypeParams() {
    return FunctionSchema.getDocTypeParams()
  }

  getXMLSchema() {
    return FunctionSchema
  }

  getRootNode() {
    return this.get('function')
  }

  // Getter functions for retreiving function specifications
  // as plain old data (e.g. strings or objects). Using naming
  // and order as in FunctionSchema.rng for consisitency

  /**
   * Get the function's name
   */
  getName() {
    return this.get('name').text()
  }

  /**
   * Get the function's summary
   */
  getSummary() {
    let summaryEl = this.find('summary')
    return summaryEl ? summaryEl.text() : ''
  }

  /**
   * Get the function's parameters as an object
   *
   * e.g. params: [{
   *   name: 'value', 
   *   type: 'number', 
   *   description: 'The value', 
   *   default: {type: 'number', data: 42}
   * }]
   */
  getParams() {
    if (!this._params) {
      // Parameters are cached so that this does not need to be
      // done for every call of this function
      this._params = []
      let paramEls = this.getRootNode().findAll('param')
      for (let paramEl of paramEls) {
        const name = paramEl.attr('name')
        const type = paramEl.attr('type')

        const descriptionEl = paramEl.find('description')
        let description = descriptionEl ? descriptionEl.text() : ''

        let defaultValue = null
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
          description: description,
          default: defaultValue
        })
      }
    }
    return this._params
  }

  /**
   * Get the function's return type and description
   */
  getReturn() {
    let returnEl = this.getRootNode().find('return')
    if (returnEl) {
      let descriptionEl = returnEl.find('description')
      return {
        type: returnEl.attr('type'),
        description: descriptionEl ? descriptionEl.text() : ''
      }
    } else {
      return {
        type: 'any',
        description: ''
      }
    }
  }

  /**
   * Get a list of languages that this function is implemented in
   */
  getImplementations() {
    return this.getRootNode().findAll(`implem`).map((implem) => implem.language)
  }

  /**
   * Get the implementation for a language
   */
  getImplementation(language) {
    let implem = this.getRootNode().find(`implem[language=${language}]`)
    if (implem) {
      let code = implem.find('code')
      if (code) return code.text()
    } else {
      throw new Error(`An implementation is not available for language ${language}`)
    }
  }

  /**
  * Get examples
  */
  getExamples() {
    return this.getRootNode().findAll(`example`)
  }

  /**
  * Get basic usage examples (to be displayed in popover)
  */
  getUsageExamples() {
    return this.getExamples().map((example) => {
      let usageEl = example.find('usage')
      return usageEl ? usageEl.text() : ''
    })
  }

  /*
    Get a usage summary used to populate FunctionUsage Component

    {
      name: 'sum',
      summary: 'Returns the sum of a range',
      examples: [
        'sum(A1:A5)'
      ],
      params: [
        { name: 'range', type: 'range', description: 'A range (array of numbers) to be summed up' }
      ],
      return: { type: 'number', description: 'The sum of numbers in the given range'}
    }
  */
  getUsage() {
    return {
      name: this.getName(),
      summary: this.getSummary(),
      examples: this.getUsageExamples(),
      params: this.getParams(),
      return: this.getReturn()
    }
  }

}
