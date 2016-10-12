import test from 'tape'

import TestConfigurator from '../../../helpers/TestConfigurator'
import TestDocumentHTMLConverter from '../../../helpers/TestDocumentHTMLConverter'

import DiscussionPackage from '../../../../src/document/nodes/discussion/DiscussionPackage'
import CommentPackage from '../../../../src/document/nodes/comment/CommentPackage'
import MarkPackage from '../../../../src/document/nodes/mark/MarkPackage'
import ParagraphPackage from '../../../../src/document/nodes/paragraph/ParagraphPackage'

var config = new TestConfigurator([
  DiscussionPackage,
  CommentPackage,
  MarkPackage,
  ParagraphPackage
])

test('DiscussionHTMLConverter import/export', function (t) {
  var converter = new TestDocumentHTMLConverter(config)

  var input =
    '<p data-id="paragraph-1">A <span data-id="mark-1" data-mark="discussion-1">mark</span></p>' +
    '<div data-id="discussion-1" data-discussion="" id="discussion-1">' +
      '<div data-id="comment-1" data-comment="@joe at 2016-01-01">' +
        '<p data-id="paragraph-2">Para 1</p>' +
        '<p data-id="paragraph-3">Para 2</p>' +
      '</div>' +
    '</div>'

  var doc = converter.import(input + '\n')

  t.deepEqual(
    doc.get('content').toJSON(),
    { id: 'content', type: 'container', nodes: [ 'paragraph-1', 'discussion-1' ] }
  )

  var m1 = doc.get('mark-1')
  var d1 = doc.get('discussion-1')
  var c1 = doc.get('comment-1')

  t.equal(m1.target, d1.id, 'The mark`s target is the discussion')

  t.equal(c1.who, '@joe')
  t.equal(c1.when, '2016-01-01')
  t.equal(c1.getChildren().length, 2)

  t.equal(
    converter.export(doc), input, 'Exported HTML is same as imported HTML'
  )

  t.end()
})

