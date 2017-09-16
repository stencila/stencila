import { XMLDocument } from 'substance'
import FunctionSchema from './FunctionSchema'

export default class FunctionDocument extends XMLDocument {

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
   *     myfunc_number_string
   * 
   * @param  {[type]} language [description]
   * @param  {[type]} context  [description]
   * @return {Array<String>} Names of the functions defined for each implementation
   */
  defineImplems(language, context) {
    const name = this.getName()
    const $implems = this.getRoot().findAll(`implem[language=${language}]`)
    let promises = []
    let lookup = {}
    for (let $implem of $implems) {
      let $types = $implem.findAll('types type')
      let implemName = name + $types.map($type => '_' + $type.text())
      let code = $implem.find('code').text()
      // Define the function
      let promise = context.defineFunction(implemName, code).then(() => {
        // TODO Use type hierarchy to map between a possible function call signature
        // and an implementation name
        lookup[implemName] = implemName
      })
      promises.push(promise)
    }
    return Promise.all(promises).then(() => {
      return lookup
    })
  }

  testImplems(language, context) {
    const name = this.getName()
    return this.defineImplems(language, context).then((implems) => {
      const $tests = this.getRoot().findAll('test')
      let promises = []
      let results = []
      for (let $test of $tests) {
        // Collate arguments into named and unnamed, recording
        // types to allow matching between call signature and implementation signature 
        let args = []
        let namedArgs = {}
        let types = []
        const $args = $test.findAll('arg')
        for (let $arg of $args) {
          let name = $arg.attr('name')
          let value = JSON.parse($arg.text())
          if (name) namedArgs[name] = value
          else args.push(value)
          types.push(value.type)
        }

        // Find matching implem
        let call = name + types.map(type => '_' + type)
        let implem = implems[call]
        if (!implem) throw new Error('No implementation of function matching call signature:' + call)
        
        // Call function and store result for checking elsewhere
        let promise = context.callFunction(implem, args, namedArgs).then(result => {
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
