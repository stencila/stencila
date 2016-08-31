var test = require('tape');

var TestConfigurator = require('../../../helpers/TestConfigurator');
var TestDocumentHTMLConverter = require('../../../helpers/TestDocumentHTMLConverter');

var config = new TestConfigurator([
  require('../../../../document/nodes/emoji/EmojiPackage'),
  require('../../../../document/nodes/paragraph/ParagraphPackage')
]);

test('EmojiHTMLConverter', function (assert) {

  var converter = new TestDocumentHTMLConverter(config);

  var input =
    '<p data-id="p1">' +
     '<span data-id="e1" data-emoji>:smile:</span>' +
     '<span data-id="e2" data-emoji="true">train</span>' +
    '</p>';

  var output =
    '<p data-id="p1">' +
     '<span data-id="e1" data-emoji="">:smile:</span>' +
     '<span data-id="e2" data-emoji="">:train:</span>' +
    '</p>';

  var doc = converter.import(input + '\n');

  assert.deepEqual(
    doc.get('content').toJSON(),
    { id: 'content', type: 'container', nodes: [ 'p1' ] }
  );

  var m1 = doc.get('e1').toJSON();
  assert.equal(m1.type, 'emoji');
  assert.equal(m1.name, 'smile');

  var m2 = doc.get('e2').toJSON();
  assert.equal(m2.type, 'emoji');
  assert.equal(m2.name, 'train');

  assert.equal(
    converter.export(doc), output
  );

  assert.end();

});

