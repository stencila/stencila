var test = require('tape');

var TestConfigurator = require('../../../helpers/TestConfigurator');
var TestDocumentHTMLConverter = require('../../../helpers/TestDocumentHTMLConverter');

var config = new TestConfigurator([
  require('../../../../document/nodes/default/DefaultPackage')
]);


test('DefaultHTMLConverter', function (assert) {
  var converter = new TestDocumentHTMLConverter(config);

  var input = 
    '<div data-id="d1">' +
     'This is <span>div</span> number 1.' +
    '</div>';

  var output = input;

  var doc = converter.import(input+'\n');

  assert.deepEqual(
    doc.get('content').toJSON(), 
    { id: 'content', type: 'container', nodes: [ 'd1'] }
  );

  var d1 = doc.get('d1').toJSON();
  assert.equal(d1.type, 'default');
  assert.equal(d1.html, '<div data-id="d1">This is <span>div</span> number 1.</div>');

  assert.equal(
    converter.export(doc), output
  )
  
  assert.end();
});

