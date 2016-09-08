var Context = require('./Context');
var Implem = require('../../js/Context');

/**
 * An execution context for Javascript in the browser.
 *
 * This uses the Javascript context implementation in the Stencila `js`
 * module (which is also used by the node module)
 *
 * @class      JavascriptContext (name)
 */
class JavascriptContext extends Context {
  constructor () {
    super();
    this.implem_ = new Implem();
  }

  accept (language) {
    return this.implem_.accept(language);
  }

  execute (code) {
    return this.implem_.execute(code);
  }

  write (expression) {
    return this.implem_.write(expression);
  }
}

module.exports = JavascriptContext;
