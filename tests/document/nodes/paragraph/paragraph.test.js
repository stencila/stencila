import test from 'tape'

import TestConfigurator from '../../../helpers/TestConfigurator'
import TestDocumentHTMLConverter from '../../../helpers/TestDocumentHTMLConverter'

import ParagraphPackage from '../../../../src/document/nodes/paragraph/ParagraphPackage'
import LinkPackage from '../../../../src/document/nodes/link/LinkPackage'

var config = new TestConfigurator([
  ParagraphPackage,
  LinkPackage
])

test('ParagraphHTMLConverter', function (assert) {
  var converter = new TestDocumentHTMLConverter(config)

  var content =
    '<p data-id="p1">Para 1</p>' +
    '<p data-id="p2">Para 2</p>'

  var doc = converter.import(content)

  assert.deepEqual(
    doc.get('content').toJSON(),
    { id: 'content', type: 'container', nodes: [ 'p1', 'p2' ] }
  )

  assert.deepEqual(
    doc.get('p1').toJSON(),
    { id: 'p1', type: 'paragraph', content: 'Para 1' }
  )

  assert.deepEqual(
    doc.get('p2').toJSON(),
    { id: 'p2', type: 'paragraph', content: 'Para 2' }
  )

  var html = converter.export(doc)

  assert.equal(
    html, content
  )

  assert.end()
})

test('ParagraphHTMLConverter with HTML encoded characters', function (assert) {
  var converter = new TestDocumentHTMLConverter(config)

  var input =
    '<p data-id="p1"> < " & </p>' +
    '<p data-id="p2"> &lt; &quot; &amp; </p>' +
    '<p data-id="p3"><a data-id="l1" href="">&lt; " &</a></p>'

  var output =
    '<p data-id="p1"> &lt; &quot; &amp; </p>' +
    '<p data-id="p2"> &lt; &quot; &amp; </p>' +
    '<p data-id="p3"><a data-id="l1" href="">&lt; &quot; &amp;</a></p>'

  var doc = converter.import(input)

  assert.deepEqual(
    doc.get('p1').toJSON(),
    { id: 'p1', type: 'paragraph', content: ' < " & ' }
  )

  assert.deepEqual(
    doc.get('p2').toJSON(),
    { id: 'p2', type: 'paragraph', content: ' < " & ' }
  )

  var html = converter.export(doc)

  assert.equal(
    html, output
  )

  assert.end()
})

