'use strict';

var d3 = require('d3');

/**
 * A Javascript execution context
 *
 * Provides an interface similar to that of the `Context`
 * base class in the C++ module. Used in both the `web` and `node` modules.
 */
class Context {

  constructor (scope) {
    this.scopes = [
      /**
       * External objects exposed to the context via an object
       * which is always the uppermost scope in the stack.
       */
      {
        d3: d3
      },
      /**
       * The initial scope
       */
      scope || {}
    ];
  }

  // Methods to meet the API for a context

  accept (language) {
    return ['js'].indexOf(language) > -1;
  }

  /**
   * Execute code within the context
   *
   * @param code String of code
   */
  execute (code) {
    this.function_(code)(this.scopes);
  }

  /**
   * Assign an expression to a name.
   * Used by document `import` and `include` elements to assign values
   * to the context of the transcluded documents.
   *
   * @param name       Name to be assigned
   * @param expression Expression to be assigned to name
   */
  assign (name, expression) {
    this.top_()[name] = this.evaluate_(expression);
  }

  /**
   * Get a text representation of an expression.
   * Used by document `print` elements e.g. `<span data-print="x">42</span>`
   *
   * @param  expression Expression to convert to text
   */
  write (expression) {
    return this.evaluate_(expression).toString();
  }

  /**
   * Test whether an expression is true or false.
   * Used by document `if` elements e.g. `<span data-if="height>10">The height is greater than 10</span>`
   *
   * @param  expression Expression to evaluate
   */
  test (expression) {
    return this.evaluate_(expression) ? '1' : '0';
  }

  /**
   * Mark an expression to be the subject of subsequent `match` queries.
   * Used by document `switch` elements e.g. `<p data-switch="x"> X is <span data-match="1">one</span><span data-default>not one</span>.</p>`
   *
   * @param expression Expression to evaluate
   */
  mark (expression) {
    this.set_('_subject_', this.evaluate_(expression));
  }

  /**
   * Test whether an expression matches the current subject.
   * Used by document `match` elements (placed within `switch` elements)
   *
   * @param  expression Expression to evaluate
   */
  match (expression) {
    return (this.get_('_subject_') === this.evaluate_(expression)) ? '1' : '0';
  }

  /**
   * Unmark the current subject expression
   */
  unmark () {
    this.unset_('_subject_');
  }

  /**
   * Begin a loop.
   * Used by document `for` elements e.g. `<ul data-for="planet in planets"><li data-each data-text="planet" /></ul>`
   *
   * @param  item  Name given to each item
   * @param  items An expression evaluating to an array
   */
  begin (item, expression) {
    var items = this.evaluate_(expression);
    if (items.length > 0) {
      this.push_();
      this.set_('_item_', item);
      this.set_('_items_', items);
      this.set_('_index_', 0);
      this.set_(item, items[0]);
      return '1';
    } else {
      return '0';
    }
  }

  /**
   * Steps the current loop to the next item.
   * Used by document `for` elements. See document rendering methods.
   *
   * If there are more items to iterate over this method should return `1`.
   * When there are no more items, this method should do any clean up required
   * (e.g popping the loop scope off a scope stack) when ending a loop,
   * and return `0`.
   */
  next () {
    var items = this.get_('_items_');
    var index = this.get_('_index_');
    if (index < items.length - 1) {
      index += 1;
      this.set_('_index_', index);
      var name = this.get_('_item_');
      var value = items[index];
      this.set_(name, value);
      return '1';
    } else {
      this.pop_();
      return '0';
    }
  }

  /**
   * Enter a new scope
   * Used by document `with` element e.g. `<div data-with="mydata"><span data-text="sum(a*b)" /></div>`
   *
   * @param object New scope of the new context
   */
  enter (expression) {
    this.push_(this.evaluate_(expression));
  }

  /**
   * Exit the current scope
   */
  exit () {
    this.pop_();
  }

  // Private methods for manipulating the stack
  // of scopes

  push_ (value) {
    var scope;
    if (value) scope = value;
    else scope = {};
    this.scopes.push(scope);
  }

  pop_ () {
    this.scopes.pop();
  }

  top_ () {
    return this.scopes[this.scopes.length - 1];
  }

  set_ (name, value) {
    this.top_()[name] = value;
  }

  get_ (name) {
    for (var index = this.scopes.length - 1; index >= 0; index--) {
      var value = this.scopes[index][name];
      if (value !== undefined) return value;
    }
  }

  unset_ (name) {
    delete this.top_()[name];
  }

  /**
   * Create a function using scopes.
   *
   * Current scope is assigned to variable `_scope_` so that
   * it can be assigned to
   *
   * @param code String of code
   */
  function_ (code, result) {
    // Generate function
    var func = 'var _scope_ = scopes[scopes.length-1];\n';
    var index;
    for (index = 0; index < this.scopes.length; index++) {
      func += 'with(scopes[' + index + ']){\n';
    }
    if (result) func += 'return ';
    func += code + ';\n';
    for (index = 0; index < this.scopes.length; index++) {
      func += '}\n';
    }
    return Function('scopes', func); // eslint-disable-line no-new-func
  }

  /**
   * Evaluate an expression
   *
   * @param code String of code
   */
  evaluate_ (code) {
    return this.function_(code, true)(this.scopes);
  }

}

module.exports = Context;
