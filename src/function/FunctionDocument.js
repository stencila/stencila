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

  getCode(language) {
    let implem = this.getRoot().find(`implem[language=${language}]`)
    return implem.find('code').text()
  }

  testImplem(language, context) {
    const name = this.getName()
    const code = this.getCode(language)
    return context.defineFunction(name, code).then(() => {
      const $tests = this.getRoot().find('tests').findAll('test')
      let promises = []
      let results = []
      for (let $test of $tests) {
        let args = []
        let namedArgs = {}
        const $args = $test.find('args').findAll('arg')
        for (let $arg of $args) {
          let name = $arg.attr('name')
          let value = JSON.parse($arg.text())
          if (name) namedArgs[name] = value
          else args.push(value)
        }

        let promise = context.callFunction(name, args, namedArgs).then(result => {
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
