var test = require('tape');
var stencila = require('..');

test('context', function (assert) {
  var context = new stencila.Context();
  
  context.enter('{a:10}');
  assert.equal(context.write('a'), '10');
  context.exit();

  assert.end();
});
