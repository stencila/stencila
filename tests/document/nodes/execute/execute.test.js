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
    '<div data-id="e1" data-execute="r"></div>' +
    '<div data-id="e2" data-execute="py"><pre data-code="">x=1</pre></div>'

  let output =
    '<div data-id="e1" data-execute="r"></div>' +
    '<div data-id="e2" data-execute="py"><pre data-code="">x=1</pre></div>'

  let doc = converter.import(input + '\n')

  t.deepEqual(
    doc.get('content').toJSON(),
    { id: 'content', type: 'container', nodes: [ 'e1', 'e2' ] }
  )

  let e1 = doc.get('e1').toJSON()
  t.equal(e1.type, 'execute')
  t.equal(e1.context, 'r')

  let e2 = doc.get('e2').toJSON()
  t.equal(e2.type, 'execute')
  t.equal(e2.context, 'py')
  t.equal(e2.code, 'x=1')

  t.equal(converter.export(doc), output)

  t.end()
})

test('Execute.refresh', function (t) {
  let converter = new TestDocumentHTMLConverter(config)

  let doc = converter.import('<div data-id="e1" data-execute="js"><pre data-code="">x=1</pre></div>\n')
  let e1 = doc.get('e1')
  e1.refresh()

  t.equal(e1.type, 'execute')

  t.end()
})

