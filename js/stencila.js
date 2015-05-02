var Stencila = (function(Stencila){

	var Context = Stencila.Context = function(){
		this.scopes = [{}];
	};

	// Private methods

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
		return this.top_()[name];
	};

	Context.prototype.unset_ = function(name){
		delete this.top_()[name];
	};

	Context.prototype.evaluate_ = function(code,execute){
		execute = execute || false;
		var func = '';
		var index;
		for(index=0;index<this.scopes.length;index++){
			func += 'with(this.scopes['+index+']){\n';
		}
		if(execute) func += code+'\n';
		else func += 'return '+code+';\n';
		for(index=0;index<this.scopes.length;index++){
			func += '}\n';
		}
		return Function(func).call(this);
	};

	// Methods to meet the API for a context

	/**
	 * Execute code within the context
	 * 
	 * @param code String of code
	 */
	Context.prototype.execute = function(code){
		this.evaluate_(code,true);
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
		var value = this.evaluate_(expression);
		return String(value);
	};

	/**
	 * Test whether an expression is true or false. 
	 * Used by stencil `if` elements e.g. `<span data-if="height>10">The height is greater than 10</span>`
	 * 
	 * @param  expression Expression to evaluate
	 */
	Context.prototype.test = function(expression){
		return this.evaluate_(expression)?true:false;
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
		return this.get_('_subject_')===this.evaluate_(expression);
	};

	/**
	 * Unmark the current subject expression
	 */
	Context.prototype.unmark = function(){
		this.unset_('_subject_');
	};
	
	/**
	 * Begin a loop.
	 * Used by stencil `for` elements e.g. `<ul data-for="planet:planets"><li data-each data-text="planet" /></ul>`
	 * 
	 * @param  item  Name given to each item
	 * @param  expression Expression giveing an iterable list of items
	 */
	Context.prototype.begin = function(item,expression){
		var items = this.evaluate_(expression);
		this.set_('_items_',items);
		if(items.length>0){
			this.set_('_index_',0);
			this.set_(item,items[0]);
			this.set_('_item_',item);
			return true;
		} else {
			return false;
		}
	};

	/**
	 * Steps the current loop to the next item. 
	 * Used by stencil `for` elements. See stencil `render`ing methods.
	 *
	 * If there are more items to iterate over this method should return `true`.
	 * When there are no more items, this method should do any clean up required 
	 * (e.g popping the loop namespace off a namespace stack) when ending a loop, 
	 * and return `false`. 
	 */
	Context.prototype.next = function(){
		var items = this.get_('_items_');
		var index = this.get_('_index_');
		if(index<items.length){
			this.set_('_index_',index++);
			var name = this.get('_item_');
			var value = items[index];
			this.set_(name,value);
			return true;
		} else {
			return false;
		}
	};

	/**
	 * Enter a new namespace. 
	 * Used by stencil `with` element e.g. `<div data-with="mydata"><span data-text="sum(a*b)" /></div>`
	 *  
	 * @param expression Expression that will be the scope of the new context
	 */
	Context.prototype.enter = function(expression){
		this.push_(this.evaluate_(expression));
	};

	/**
	 * Exit the current namespace
	 */
	Context.prototype.exit = function(){
		this.pop_();
	};



	var Stencil = Stencila.Stencil = function(){
	};

	Stencil.prototype.render = function(node,context){

	};


	return Stencila;
})(Stencila||{});
