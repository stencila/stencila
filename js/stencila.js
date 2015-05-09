var Stencila = (function(Stencila){

	/**
	 * A JavaScript rendering context
	 *
	 * Used for rendering stencils against.
	 * Provides an interface similar to that of the `Context`
	 * virtual base class in the C++ module.
	 */
	var Context = Stencila.Context = function(scope){
		this.scopes = [scope||{}];
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
	Context.prototype.evaluate_ = function(expression){
		var func = '';
		var index;
		for(index=0;index<this.scopes.length;index++){
			func += 'with(this.scopes['+index+']){\n';
		}
		func += 'return '+expression+';\n';
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
		var script = document.createElement('script');
		script.type = 'text/javascript';
		script.text = code;
		document.head.appendChild(script);
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
		if(items.length>0){
			this.push_();
			this.set_('_item_',item);
			this.set_('_items_',items);
			this.set_('_index_',0);
			this.set_(item,items[0]);
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
		if(index<items.length-1){
			index += 1;
			this.set_('_index_',index);
			var name = this.get_('_item_');
			var value = items[index];
			this.set_(name,value);
			return true;
		} else {
			this.pop_();
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


	/**
	 * A HTML element
	 *
	 * Provides a similar API to the `Html::Node` in the 
	 * C++ module, which in turn is similar to the jQuery interface.
	 * A thin wrapper around a native DOM element.
	 * Provides some shortcuts to DOM manipulation, without having to 
	 * rely on the whole of jQuery as a dependency.
	 */
	var Node = Stencila.Node = function(dom){
		if(typeof dom==='string'){
			var frag = document.createElement('div');
			frag.innerHTML = dom;
			this.dom = frag;
		}
		else {
			this.dom = dom;
		}
	};

	/**
	 * Is this node a null? 
	 */
	Node.prototype.empty = function(){
		return this.dom?false:true;
	};

	/**
	 * Get or set an attribute
	 * 
	 * @param  {String} name  Name of attribute
	 * @param  {String} value Value for attribute
	 */
	Node.prototype.attr = function(name,value){
		if(value===undefined){
			var attr = this.dom.getAttribute(name);
			return attr?attr:'';
		} else {
			this.dom.setAttribute(name,value);
		}
	};

	/**
	 * Remove an attribute
	 * 
	 * @param  {String} name Name of attribute
	 */
	Node.prototype.erase = function(name){
		this.dom.removeAttribute(name);
	};

	/**
	 * Does this element have a particular attribute
	 * 
	 * @param  {String} name  Name of attribute
	 */
	Node.prototype.has = function(name){
		return this.dom.hasAttribute(name);
	};

	/**
	 * Get or set the HTML (inner) of this element
	 * 
	 * @param  {String} value HTML string
	 */
	Node.prototype.html = function(value){
		if(value===undefined) return this.dom.innerHTML;
		else this.dom.innerHTML = value;
	};

	/**
	 * Get or set the text of this element
	 * 
	 * @param  {String} value Text constent string
	 */
	Node.prototype.text = function(value){
		if(value===undefined) return this.dom.textContent;
		else this.dom.textContent = value;
	};

	/**
	 * Get array of child elements
	 */
	Node.prototype.children = function(){
		return this.dom.children;
	};

	/**
	 * Get next sibling element
	 */
	Node.prototype.next = function(){
		return new Node(this.dom.nextElementSibling);
	};

	/**
	 * Get previous sibling element
	 */
	Node.prototype.previous = function(){
		return new Node(this.dom.previousElementSibling);
	};

	/**
	 * Select child elements
	 * 
	 * @param  {String} selector Select child elements using a CSS selector
	 */
	Node.prototype.select = function(selector){
		return new Node(this.dom.querySelector(selector));
	};

	/**
	 * Stencil directives
	 *
	 * Implemented as inividual classes to allow for use in rendering
	 * stencils as well as in any user interfaces to directive elements 
	 * 
	 * Each directive has:
	 * 
	 * 	- a `get` method which takes the string value of associated attribute
	 * 	  (and may do parsing of it) and optionally a node
	 * 	- an `apply` method which adds necessary attributes and text for
	 * 	  the directive to a node
	 * 	- a `render` method which renders the directive in a context
	 */
	
	var directiveRender = Stencila.directiveRender = function(node,context){
		if(node.has('data-exec')) return new Exec().apply(node,context);
		if(node.has('data-write')) return new Write().apply(node,context);

		if(node.has('data-if')) return new If().apply(node,context);
		if(node.has('data-elif') | node.has('data-else')) return;

		if(node.has('data-for')) return new For().apply(node,context);

		directiveRenderChildren(node,context);
	};
	var directiveRenderChildren = function(node,context){
		var children = node.children();
		for(var index=0;index<children.length;index++){
			directiveRender(
				new Node(children[index]),
				context
			);
		}
	};
	var directiveApply = function(node,context){
		this.get(node).render(node,context);
	};

	/**
	 * An `exec` directive
	 */
	var Exec = Stencila.Exec = function(details,code){
		this.details = details;
		this.code = code;
	};
	Exec.prototype.get = function(node){
		this.details = node.attr('data-exec');
		this.code = node.text();
		return this;
	};
	Exec.prototype.set = function(node){
		node.attr('data-exec',this.details);
		node.text(this.code);
		return this;
	};
	Exec.prototype.render = function(node,context){
		context.execute(this.code);
		return this;
	};
	Exec.prototype.apply = directiveApply;

	/**
	 * A `write` directive
	 */
	var Write = Stencila.Write = function(expr){
		this.expr = expr;
	};
	Write.prototype.get = function(node){
		this.expr = node.attr('data-write');
		return this;
	};
	Write.prototype.set = function(node){
		node.attr('data-write',this.expr);
		return this;
	};
	Write.prototype.render = function(node,context){
		node.text(context.write(this.expr));
		return this;
	};
	Write.prototype.apply = directiveApply;

	/**
	 * An `if` directive
	 */
	var If = Stencila.If = function(expr){
		this.expr = expr;
	};
	If.prototype.get = function(node){
		this.expr = node.attr('data-if');
		return this;
	};
	If.prototype.set = function(node){
		node.attr('data-if',this.expr);
		return this;
	};
	If.prototype.render = function(node,context){
		var hit = context.test(this.expr);
		if(hit){
			node.erase("data-off");
			directiveRenderChildren(node,context);
		} else {
			node.attr("data-off","true");
		}
		var next = node.next();
		while(!next.empty()){
			var expr = next.attr("data-elif");
			if(expr.length>0){
				if(hit){
					next.attr('data-off','true');
				} else {
					hit = context.test(expr);
					if(hit){
						next.erase("data-off");
						directiveRenderChildren(next,context);
					} else {
						next.attr("data-off","true");
					}
				}
			}
			else if(next.has("data-else")){
				if(hit){
					next.attr("data-off","true");
				} else {
					next.erase("data-off");
					directiveRenderChildren(next,context);
				}
				break;
			}
			else break;
			next = next.next();
		}
	};
	If.prototype.apply = directiveApply;

	/**
	 * A `for` directive
	 */
	var For = Stencila.For = function(item,items){
		this.item = item;
		this.items = items;
	};
	For.prototype.get = function(node){
		var attr = node.attr('data-for');
		var matches = attribute.match(/^(\w+)\s+in\s+(.+)$/);
		this.item = matches[1];
		this.items = matches[2];
		return this;
	};
	For.prototype.set = function(node){
		node.attr('data-for',this.item+' in '+this.items);
		return this;
	};
	For.prototype.render = function(node,context){
		var more = context.begin(this.item,this.items);
		//! @todo Fully implement
		return this;
	};
	For.prototype.apply = directiveApply;


	/**
	 * A stencil
	 */
	var Stencil = Stencila.Stencil = function(html){
		this.dom = document.createElement('main');
		this.html(html);
	};
	Stencil.prototype.html = function(html){
		return Node.prototype.html.call(this,html);
	};
	Stencil.prototype.select = function(selector){
		return Node.prototype.select.call(this,selector);
	};
	Stencil.prototype.render = function(context){
		directiveRender(new Node(this.dom),context);
	};

	return Stencila;
})(Stencila||{});
