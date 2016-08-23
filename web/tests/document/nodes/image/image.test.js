var test = require('tape');

var TestConfigurator = require('../../../helpers/TestConfigurator');
var TestDocumentHTMLConverter = require('../../../helpers/TestDocumentHTMLConverter');

var config = new TestConfigurator([
  require('../../../../document/nodes/image/ImagePackage'),
  require('../../../../document/nodes/paragraph/ParagraphPackage')
]);


test.skip('ImageHTMLConverter', function (assert) {
  var converter = new TestDocumentHTMLConverter(config);

  var input = 
    '<img data-id="i1" src="https://unsplash.it/200">' + // void non-closing tag will fail until https://github.com/substance/substance/pull/764/commits/1151df11ee6671f7701fab6b7c368beb1d6210ba is merged into Substance
    '<img data-id="i2" src="https://unsplash.it/400"/>'; // self-closing tag

  var output = 
    '<img data-id="i1" src="https://unsplash.it/200">' +
    '<img data-id="i2" src="https://unsplash.it/400">';

  var doc = converter.import(input+'\n');

  assert.deepEqual(
    doc.get('content').toJSON(), 
    { id: 'content', type: 'container', nodes: [ 'i1', 'i2'] }
  );

  var m1 = doc.get('i1').toJSON();
  assert.equal(m1.type, 'image');
  assert.equal(m1.src, 'https://unsplash.it/200');

  var m2 = doc.get('i2').toJSON();
  assert.equal(m2.type, 'image');
  assert.equal(m2.src, 'https://unsplash.it/400');

  assert.equal(
    converter.export(doc), output
  )
  
  assert.end();
});

