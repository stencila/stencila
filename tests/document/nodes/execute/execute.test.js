import test from 'tape'

import TestConfigurator from '../../../helpers/TestConfigurator'
import TestDocumentHTMLConverter from '../../../helpers/TestDocumentHTMLConverter'

import ExecutePackage from '../../../../src/document/nodes/execute/ExecutePackage'

let config = new TestConfigurator([
  ExecutePackage
])

test('ExecuteHTMLConverter', function (t) {
  let converter = new TestDocumentHTMLConverter(config)

  let input =
    '<pre data-id="e1" data-execute="r"></pre>' +
    '<pre data-id="e2" data-execute="py">x=1</pre>'

  let output =
    '<pre data-id="e1" data-execute="r"></pre>' +
    '<pre data-id="e2" data-execute="py">x=1</pre>'

  let doc = converter.import(input + '\n')

  t.deepEqual(
    doc.get('content').toJSON(),
    { id: 'content', type: 'container', nodes: [ 'e1', 'e2' ] }
  )

  let e1 = doc.get('e1').toJSON()
  t.equal(e1.type, 'execute')
  t.equal(e1.session, 'r')

  let e2 = doc.get('e2').toJSON()
  t.equal(e2.type, 'execute')
  t.equal(e2.session, 'py')
  t.equal(e2.code, 'x=1')

  t.equal(converter.export(doc), output)

  t.end()
})

test('Execute.exec()', function (t) {
  let converter = new TestDocumentHTMLConverter(config)

  let doc = converter.import('<pre data-id="e1" data-execute="js">x=1</pre>\n')
  let e1 = doc.get('e1')
  e1.refresh()
  
  t.equal(e1.type, 'execute')

  t.end()
})

