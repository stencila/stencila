var test = require('tape');
var stencila = require('..');

test('context', function (assert) {
  var context = new stencila.Context();

  context.execute('_scope_.x = 42;');

  // `write` for `text` directives
  assert.equal(
    context.write('x'), '42'
  );
  assert.equal(
    context.write('true'), 'true'
  );
  assert.equal(
    context.write('3.14'), '3.14'
  );
  assert.equal(
    context.write('6*7'), '42'
  );

  // `test` for `if` directives
  assert.equal(
    context.test('x==42'), '1'
  );
  assert.equal(
    context.test('false'), '0'
  );

  // `mark`,`match`, and `unmark` for switch directives
  assert.equal(
    context.mark('x')
  );
  assert.equal(
    context.match('42'), '1'
  );
  assert.equal(
    context.match('43'), '0'
  );
  context.unmark();
  
  // `begin` and `next` for `for` directives
  assert.equal(
    context.begin('nuffing', '[]'), '0'
  );
  assert.equal(
    context.begin('num', '[10, 20]'), '1'
  );
  assert.equal(
    context.write('num'), '10'
  );
  assert.equal(
    context.next(), '1'
  );
  assert.equal(
    context.write('num'), '20'
  );
  assert.equal(
    context.next(), '0'
  );

  // `enter` and `exit` for `with` directives
  context.enter('{a:10}');
  assert.equal(
    context.write('a'), '10'
  );
  context.exit();

  assert.end();
});
