var test = require('tape');
var Context = require('../Context');

test('context', function (t) {
  var context = new Context();

  context.execute('_scope_.x = 42;');

  // `write` for `print` directives
  t.equal(
    context.write('x'), '42'
  );
  t.equal(
    context.write('true'), 'true'
  );
  t.equal(
    context.write('3.14'), '3.14'
  );
  t.equal(
    context.write('6*7'), '42'
  );

  // `test` for `if` directives
  t.equal(
    context.test('x==42'), '1'
  );
  t.equal(
    context.test('false'), '0'
  );

  // `mark`,`match`, and `unmark` for switch directives
  t.equal(
    context.mark('x')
  );
  t.equal(
    context.match('42'), '1'
  );
  t.equal(
    context.match('43'), '0'
  );
  context.unmark();

  // `begin` and `next` for `for` directives
  t.equal(
    context.begin('nuffing', '[]'), '0'
  );
  t.equal(
    context.begin('num', '[10, 20]'), '1'
  );
  t.equal(
    context.write('num'), '10'
  );
  t.equal(
    context.next(), '1'
  );
  t.equal(
    context.write('num'), '20'
  );
  t.equal(
    context.next(), '0'
  );

  // `enter` and `exit` for `with` directives
  context.enter('{a:10}');
  t.equal(
    context.write('a'), '10'
  );
  context.exit();

  t.end();
});
