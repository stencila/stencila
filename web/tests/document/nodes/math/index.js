var test = require('tape');

var TestConfigurator = require('../../../helpers/TestConfigurator');
var TestDocumentHTMLConverter = require('../../../helpers/TestDocumentHTMLConverter');

var config = new TestConfigurator([
  require('../../../../document/nodes/math/MathPackage'),

  require('../../../../document/nodes/paragraph/ParagraphPackage')
]);


test('MathHTMLConverter', function (assert) {
  var converter = new TestDocumentHTMLConverter(config);

  var content = 
  	'<p data-id="p1">' + 
  		'Surround <script data-id="m1" type="math/asciimath">x < 1</script> .' + 
    	'Surround <script data-id="m2" type="math/tex">y > 2</script> .' + 
    '</p>';

  var doc = converter.import(content+'\n');

  assert.deepEqual(
    doc.get('content').toJSON(), 
    { id: 'content', type: 'container', nodes: [ 'p1'] }
  );

  var m1 = doc.get('m1').toJSON();
  assert.equal(m1.type, 'math');
  assert.equal(m1.language, 'asciimath');
  assert.equal(m1.source, 'x < 1');

  var m2 = doc.get('m2').toJSON();
  assert.equal(m2.type, 'math');
  assert.equal(m2.language, 'tex');
  assert.equal(m2.source, 'y > 2');


  var html = converter.export(doc);
  
  assert.equal(
  	html, content
  )

  assert.end();
});
