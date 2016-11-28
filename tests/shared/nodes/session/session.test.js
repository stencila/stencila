import test from 'tape'

import TestConfigurator from '../../../helpers/TestConfigurator'
import TestDocumentHTMLConverter from '../../../helpers/TestDocumentHTMLConverter'

import SessionPackage from '../../../../src/shared/nodes/session/SessionPackage'

var config = new TestConfigurator([
  SessionPackage
])

test('SessionHTMLConverter', function (assert) {
  var converter = new TestDocumentHTMLConverter(config)

  var content =
    '<div data-id="s1" class="session">url</div>' +
    '<div data-id="s2" class="session">url</div>'

  var doc = converter.import(content)

  assert.deepEqual(
    doc.get('content').toJSON(),
    { id: 'content', type: 'container', nodes: [ 's1', 's2' ] }
  )

  assert.deepEqual(
    doc.get('s1').toJSON(),
    { id: 's1', type: 'session', url: 'url' }
  )

  var html = converter.export(doc)

  assert.equal(
    html, content
  )

  assert.end()
})

