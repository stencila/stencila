var test = require('tape');

var TestConfigurator = require('../../../helpers/TestConfigurator');
var TestDocumentHTMLConverter = require('../../../helpers/TestDocumentHTMLConverter');

var config = new TestConfigurator([
  require('../../../../document/nodes/strong/StrongPackage'),

  require('../../../../document/nodes/paragraph/ParagraphPackage'),
  require('../../../../document/nodes/emphasis/EmphasisPackage'),
]);


test('StrongHTMLConverter', function (assert) {
  var converter = new TestDocumentHTMLConverter(config);

  var content = 
    '<p data-id="p1">' + 
      'Some <strong data-id="s1">strong</strong> text.' +
      // The following space between `em` and `strong` is necessary to get order of export the 
      // same as import (because both annotations have same start offset)
      'Some <em data-id="e1"> <strong data-id="s2">strong and emphasised</strong></em> text.' +
    '</p>';

  var doc = converter.import(content+'\n');

  var s1 = doc.get('s1').toJSON();
  assert.equal(s1.type, 'strong');
  assert.equal(s1.startOffset, 5);
  assert.equal(s1.endOffset, 11);

  var s2 = doc.get('s2').toJSON();
  assert.equal(s2.type, 'strong');
  assert.equal(s2.startOffset, 23);
  assert.equal(s2.endOffset, 44);

  var html = converter.export(doc);
  
  assert.equal(
    html, content
  )

  assert.end();
});
