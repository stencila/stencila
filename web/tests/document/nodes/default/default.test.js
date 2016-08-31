var test = require('tape');

var TestConfigurator = require('../../../helpers/TestConfigurator');
var TestDocumentHTMLConverter = require('../../../helpers/TestDocumentHTMLConverter');

var config = new TestConfigurator([
  require('../../../../document/nodes/default/DefaultPackage')
]);
var converter = new TestDocumentHTMLConverter(config);

var Document = require('../../../../document/DocumentModel');
var DefaultComponent = require('../../../../document/nodes/default/DefaultComponent');

test('DefaultHTMLConverter', function (t) {

  var input =
    '<div data-id="d1">' +
     'This is <span>div</span> number 1.' +
    '</div>';

  var output = input;

  var doc = converter.import(input + '\n');

  t.deepEqual(
    doc.get('content').toJSON(),
    { id: 'content', type: 'container', nodes: [ 'd1' ] }
  );

  var d1 = doc.get('d1').toJSON();
  t.equal(d1.type, 'default');
  t.equal(d1.html, '<div data-id="d1">This is <span>div</span> number 1.</div>');

  t.equal(
    converter.export(doc), output
  );

  t.end();

});

test('DefaultHTMLConverter should sanitize before export', function (t) {

  var doc = new Document();
  doc.create({ type: 'default', id: 'd1', html: '<script>cracked()</script>' });

  t.equal(converter.export(doc), '');

  t.end();

});

function displayed (html) {

  var doc = new Document();
  var d1 = doc.create({ type: 'default', id: 'd1', html: html });
  var comp = new DefaultComponent();
  comp.setProps({ node: d1 });
  var display = comp.find('.se-display');
  return display.html();

}

test('DefaultComponent should display HTML', function (t) {

  var html;

  html = '<div data-id="d1" data-arbitrary="foo"></div>';
  t.equal(displayed(html), html);

  html = '<div><img src="http://example.com/image.png"></div>';
  t.equal(displayed(html), html);

  t.end();

});

test('DefaultComponent should sanitize HTML', function (t) {

  t.equal(displayed('<script>cracked()</script>'), '');
  t.equal(displayed('<div onmouseover="cracked()" data-print="6*7">content</div>'), '<div data-print="6*7">content</div>');

  t.end();

});
