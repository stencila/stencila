import test from 'tape'

import TestConfigurator from '../../../helpers/TestConfigurator'
import TestDocumentHTMLConverter from '../../../helpers/TestDocumentHTMLConverter'

import MathPackage from '../../../../document/nodes/math/MathPackage'
import ParagraphPackage from '../../../../document/nodes/paragraph/ParagraphPackage'

var config = new TestConfigurator([
  MathPackage,
  ParagraphPackage
])

test('MathHTMLConverter', function (assert) {
  var converter = new TestDocumentHTMLConverter(config)

  var input =
    '<p data-id="p1">' +
      '<span data-id="m1" data-math="asciimath">x = 1</span> .' +
      '<span data-id="m2" data-math="tex">y = 2</span> .' +
      '<span data-id="m3" data-math="tex block">\\pi</span> .' +
    '</p>'

  var output =
    '<p data-id="p1">' +
      '<span data-id="m1" data-math="asciimath">x = 1</span> .' +
      '<span data-id="m2" data-math="tex">y = 2</span> .' +
      '<span data-id="m3" data-math="tex block">\\pi</span> .' +
    '</p>'

  var doc = converter.import(input + '\n')

  assert.deepEqual(
    doc.get('content').toJSON(),
    { id: 'content', type: 'container', nodes: [ 'p1' ] }
  )

  var m1 = doc.get('m1').toJSON()
  assert.equal(m1.type, 'math')
  assert.equal(m1.language, 'asciimath')
  assert.equal(m1.display, 'inline')
  assert.equal(m1.source, 'x = 1')

  var m2 = doc.get('m2').toJSON()
  assert.equal(m2.type, 'math')
  assert.equal(m2.language, 'tex')
  assert.equal(m2.display, 'inline')
  assert.equal(m2.source, 'y = 2')

  var m3 = doc.get('m3').toJSON()
  assert.equal(m3.type, 'math')
  assert.equal(m3.language, 'tex')
  assert.equal(m3.display, 'block')
  assert.equal(m3.source, '\\pi')

  assert.equal(
    converter.export(doc), output
  )

  assert.end()
})
