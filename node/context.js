'use strict';

/**
 * A JavaScript rendering context
 *
 * Used for rendering stencils against.
 * Provides an interface similar to that of the `Context`
 * virtual base class in the C++ module.
 */
var Context = function(scope){
  this.scopes = [
    /**
     * Functions exposed to contexts via an object 
     * which is always the uppermmost scope of a `Context`
     */
    {},
    scope||{}
  ];
};

// Private methods for manipulating the stack
// of scopes

Context.prototype.push_ = function(value){
  var scope;
  if(value) scope = value;
  else scope = {};
  this.scopes.push(scope);
};
Context.prototype.pop_ = function(){
  this.scopes.pop();
};
Context.prototype.top_ = function(){
  return this.scopes[this.scopes.length-1];
};
Context.prototype.set_ = function(name,value){
  this.top_()[name] = value;
};
Context.prototype.get_ = function(name){
  for(var index=this.scopes.length-1;index>=0;index--){
    var value = this.scopes[index][name];
    if(value!==undefined) return value;
  }
};
Context.prototype.unset_ = function(name){
  delete this.top_()[name];
};

/**
 * Create a function using scopes.
 *
 * Current scope is assigned to variable `_scope_` so that
 * it can be assigned to
 * 
 * @param code String of code
 */
Context.prototype.function_ = function(code, result){
  // Generate function
  var func = 'var _scope_ = scopes[scopes.length-1];\n';
  var index;
  for(index=0;index<this.scopes.length;index++){
    func += 'with(scopes['+index+']){\n';
  }
  if(result) func += 'return ';
  func += code + ';\n';
  for(index=0;index<this.scopes.length;index++){
    func += '}\n';
  }
  return Function('scopes',func);
};

/**
 * Evaluate an expression
 * 
 * @param code String of code
 */
Context.prototype.evaluate_ = function(code){
  return this.function_(code,true)(this.scopes);
};


// Methods to meet the API for a context

/**
 * Execute code within the context
 * 
 * @param code String of code
 */
Context.prototype.execute = function(code){
  this.function_(code)(this.scopes);
};

/**
 * Assign an expression to a name.
 * Used by stencil `import` and `include` elements to assign values
 * to the context of the transcluded stencils.
 * 
 * @param name       Name to be assigned
 * @param expression Expression to be assigned to name
 */
Context.prototype.assign = function(name, expression){
  this.top_()[name] = this.evaluate_(expression);
};

/**
 * Get a text representation of an expression. 
 * Used by stencil `text` elements e.g. `<span data-text="x">42</span>`
 * 
 * @param  expression Expression to convert to text
 */
Context.prototype.write = function(expression){
  return String(this.evaluate_(expression));
};

/**
 * Test whether an expression is true or false. 
 * Used by stencil `if` elements e.g. `<span data-if="height>10">The height is greater than 10</span>`
 * 
 * @param  expression Expression to evaluate
 */
Context.prototype.test = function(expression){
  return this.evaluate_(expression)?'1':'0';
};

/**
 * Mark an expression to be the subject of subsequent `match` queries.
 * Used by stencil `switch` elements e.g. `<p data-switch="x"> X is <span data-match="1">one</span><span data-default>not one</span>.</p>`
 * 
 * @param expression Expression to evaluate
 */
Context.prototype.mark = function(expression){
  this.set_('_subject_',this.evaluate_(expression));
};

/**
 * Test whether an expression matches the current subject.
 * Used by stencil `match` elements (placed within `switch` elements)
 * 
 * @param  expression Expression to evaluate
 */
Context.prototype.match = function(expression){
  return (this.get_('_subject_')===this.evaluate_(expression))?'1':'0';
};

/**
 * Unmark the current subject expression
 */
Context.prototype.unmark = function(){
  this.unset_('_subject_');
};

/**
 * Begin a loop.
 * Used by stencil `for` elements e.g. `<ul data-for="planet in planets"><li data-each data-text="planet" /></ul>`
 * 
 * @param  item  Name given to each item
 * @param  items An expression evaluating to an array
 */
Context.prototype.begin = function(item, expression){
  var items = this.evaluate_(expression);
  if(items.length>0){
    this.push_();
    this.set_('_item_',item);
    this.set_('_items_',items);
    this.set_('_index_',0);
    this.set_(item,items[0]);
    return '1';
  } else {
    return '0';
  }
};

/**
 * Steps the current loop to the next item. 
 * Used by stencil `for` elements. See stencil rendering methods.
 *
 * If there are more items to iterate over this method should return `1`.
 * When there are no more items, this method should do any clean up required 
 * (e.g popping the loop scope off a scope stack) when ending a loop, 
 * and return `0`. 
 */
Context.prototype.next = function(){
  var items = this.get_('_items_');
  var index = this.get_('_index_');
  if(index<items.length-1){
    index += 1;
    this.set_('_index_',index);
    var name = this.get_('_item_');
    var value = items[index];
    this.set_(name,value);
    return '1';
  } else {
    this.pop_();
    return '0';
  }
};

/**
 * Enter a new scope
 * Used by stencil `with` element e.g. `<div data-with="mydata"><span data-text="sum(a*b)" /></div>`
 *  
 * @param object New scope of the new context
 */
Context.prototype.enter = function(expression){
  this.push_(this.evaluate_(expression));
};

/**
 * Exit the current scope
 */
Context.prototype.exit = function(){
  this.pop_();
};

module.exports = Context;
