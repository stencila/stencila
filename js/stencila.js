var Stencila = (function(Stencila){

	///////////////////////////////////////////////////////////////////////////////////////////////

	/**
	 * The user interface environment
	 *
	 * This is a basic stub. Themes define their own HubView which is
	 * assigned to `Hub.view`
	 */
	var HubView = Stencila.HubView = function(component){
	};

	HubView.prototype.info = function(message){
		console.info(message);
	};

	HubView.prototype.warn = function(message){
		console.warn(message);
	};

	HubView.prototype.error = function(message){
		console.error(message);
	};

	HubView.prototype.signin = function(){
		var username = prompt('Username for stenci.la');
		var password = prompt('Password for stenci.la');
		return {
			username: username,
			password: password
		};
	};

	/////////////////////////////////////////////////////////////////////////////////////////////

	/**
	 * Connection to a server.
	 * 
	 * Implements the WAMP (http://wamp.ws) messaging protocol over
	 */
	var Connection = Stencila.Connection = function(){
		// Callbacks registered for remote procedure calls (see call() method)
		this.callbacks = {};
		// Identifier for messages; incremented in call method
		this.id = 0;
	};

	/**
	 * Connect to server
	 */
	Connection.prototype.connect = function(){
		// Automatically disconnect when page is unloaded
		var self = this;
		window.addEventListener("beforeunload", function(event){
			self.disconnect();
		});
	};

	/**
	 * Receive a message from the server
	 * 
	 * @param  {String} data
	 */
	Connection.prototype.receive = function(data){
		// Parse JSON
		var message = [8];
		try {
			message = JSON.parse(data);
		}
		catch(error) {
			console.error('Error parsing WAMP message data.\n  data:'+data+'\n  error:'+error);
		}
		// Dispatch based on WAMP code
		var code = message[0];
		if(code==50) this.result(message);
		else if(code==8){
			throw "WAMP error:"+message;
		}
		else {
			throw "WAMP message type unknown:"+code;
		}
	};

	/**
	 * Make a remote procedure call
	 * See https://github.com/tavendo/WAMP/blob/master/spec/basic.md#call-1
	 * 
	 * @param  {String}   method   Name of method to call
	 * @param  {Array}    args     Array of arguments
	 * @param  {Function} callback Function to call when method returns (potentially with a result)
	 */
	Connection.prototype.call = function(method,args,callback){
		args = args || [];
		// Increment id
		// According to https://github.com/tavendo/WAMP/blob/master/spec/basic.md#ids
		// "IDs in the session scope SHOULD be incremented by 1 beginning with 1"
		this.id++;
		// Generate a WAMP call array
		var wamp = [
			48,			// CALL
			this.id,	// Request|id
			{},			// Options|dict
			method,		// Procedure|uri
			args		// Arguments|list
		];
		// Register callback
		if(callback){
			this.callbacks[this.id] = callback;
		}
		// Send WAMP
		this.send(JSON.stringify(wamp));
	};

	/**
	 * Receive the result of a remote procedure call
	 * See https://github.com/tavendo/WAMP/blob/master/spec/basic.md#result-1
	 *
	 * This method is called when a WAMP RESULT message is received and is not meant to be called
	 * publically
	 */
	Connection.prototype.result = function(message){
		// [RESULT, CALL.Request|id, Details|dict, YIELD.Arguments|list]
		var id = message[1];
		var callback = this.callbacks[id];
		if(callback){
			var results = message[3];
			// WAMP allows for muliple returns
			// Only passing on a single result, the first
			var result = results[0];
			callback(result);
		}
	};

	/**
	 * Websocket connection class
	 */
	var WebSocketConnection = Stencila.WebSocketConnection = function(url){
		Connection.call(this);
		this.socket = null;
		this.connect(url);
	};
	WebSocketConnection.prototype = Object.create(Connection.prototype);

	/**
	 * Connect
	 * 
	 * @param  {String} url URL to connect to
	 */
	WebSocketConnection.prototype.connect = function(url){
		var self = this;
		// Create a new websocket
		self.socket = new WebSocket(url);
		// Bind some socket events
		//   when connection is opened...
		self.socket.onopen = function(event){
			self.ok = true;
		};
		//   when there are any connection errors...
		self.socket.onclose = function(event){
			console.warn("Connection closed");
			self.ok = false;
		};
		//   when a message is recieved...
		self.socket.onmessage = function(event){
			Connection.prototype.receive.call(self,event.data);
		};
		Connection.prototype.connect.call(this);
	};

	/**
	 * Disconnect
	 */
	WebSocketConnection.prototype.disconnect = function(){
		this.socket.close();
	};

	/**
	 * Send data
	 */
	WebSocketConnection.prototype.send = function(data){
		this.socket.send(data);
	};

	///////////////////////////////////////////////////////////////////////////////////////////////


	var Hub = Stencila.Hub = {};

	/**
	 * Username and permit strings for stenci.la
	 *
	 * Permit is used for authentication and CSRF validation
	 * for all asynchronous requests
	 */
	Hub.username = null;
	Hub.permit = null;

	/**
	 * Signin to stenci.la
	 *
	 * @param  {Function} then Callback once signed in
	 */
	Hub.signin = function(credentials,then) {
		var headers = {
			'Accept':'application/json'
		};
		// This page may already be at https://stenci.la and user already signed in
		// Check for that using cookies set by stenci.la when a user is authenticated
		if(window.location.host=='stenci.la'){
			var username = $.cookie('username');
			if(username){
				Hub.username = username;
			}
		}
		else if(credentials){
			if(credentials.password){
				var encoded = btoa(credentials.username+':'+credentials.password);
				headers['Authorization'] = 'Basic '+encoded;
			}
		} 
		else {
			Hub.view.signin();
		}

		// Get a permit to be used for subsequent requests
		$.ajax({
			// Normally https will be used but in development http may be used. 
			// Using the current window protocol allows for that situation
			url: window.location.protocol+'//stenci.la/user/permit',
			method: 'GET',
			headers: headers
		}).done(function(data){
			Hub.permit = data.permit;
			if(then) then();
		});
	};

	/**
	 * Make a GET request to stenci.la
	 * 
	 * @param  {String}   path Path to resource
	 * @param  {Boolean}  auth Is authentication required for this request?
	 * @param  {Function} then Callback when done
	 */
	Hub.get = function(path,auth,then) {
		if(auth & !Hub.permit) {
			// If not signed in then signin and then get
			Hub.signin(function(){
				Hub.get(path,auth,then);
			});
		} else {
			var headers = {
				'Accept':'application/json'
			};
			if(auth) headers = {
				'Authorization' : 'Permit '+Hub.permit
			};
			$.ajax({
				url: window.location.protocol+'//stenci.la/'+path,
				method: 'GET',
				headers: headers
			}).done(then);
		}
	};

	/**
	 * Make a POST request to stenci.la
	 * 
	 * @param  {String}   path Path to resource
	 * @param  {String}   data Data to post
	 * @param  {Function} then Callback when done
	 *
	 * @todo Add POST JSON data to request
	 */
	Hub.post = function(path,data,then) {
		if(!Hub.permit) {
			// If not signed in then signin and then post
			Hub.signin(function(){
				Hub.post(path,data,then);
			});
		} else {
			$.ajax({
				url: window.location.protocol+'//stenci.la/'+path,
				method: 'POST',
				headers: {
					'Authorization' : 'Permit '+Hub.permit,
				}
			}).done(then);
		}
	};

	/**
	 * Signout of stenci.la
	 */
	Hub.signout = function() {
		Hub.username = null;
		Hub.permit = null;
	};

	///////////////////////////////////////////////////////////////////////////////////////////////

	/**
	 * Base class for all components
	 */
	var Component = Stencila.Component = function(){
		// Create default view
		this.viewCurrent = null;

		// Array of views that have been constructed
		// for this component
		this.viewList = [];

		// Set host information
		var location = window.location;
		this.host = location.hostname;
		this.port = location.port;

		// Set address
		this.address = null;
		// ... from <meta> tag
		var address = $('head meta[itemprop=address]');
		if(address.length) this.address = address.attr('content');
		// ... or from url
		if(!this.address) {
			var parts = window.location.pathname.split('/');
			// Remove the first part cause by the leading /
			if(parts[0]==="") parts.shift();
			// Remove the last part if it is a title slug
			var last = parts[parts.length-1];
			if(last.substr(last.length-1)=="-") parts.pop();
			this.address = parts.join('/');
		}

		// Set closed. By default is false
		this.closed = false;
		// ... but can be set in <meta> tag
		var closed = $('head meta[itemprop=closed]');
		if(closed.length) this.closed = closed.attr('content')=='true';
		// ... but always overidden by hash fragment
		if(window.location.hash==='#closed!') this.closed = true;
		else if(window.location.hash==='#open!') this.closed = false;

		// Set activation status
		this.activation = 'inactive';
	};

	/**
	 * Set the theme for this compontent
	 */
	Component.prototype.theme = function(theme,then){
		var self = this;
		require([theme+'/theme'],function(theme){
			Hub.menu = new theme.HubMenu();
			if(!self.closed) self.menu = new theme.ComponentMenu(self);
			else{
				self.menu = null;
				$(document).bind('keydown','f1',function(event){
					event.preventDefault();
					self.menu = new theme.ComponentMenu(self);
					self.menu.from(self);
				});
			}
			self.view(theme.DefaultView);
			if(then) then();
		});
	};

	/**
	 * Set the view for this component
	 * 
	 * @param  {Class} viewClass A view Class (e.g. RevealView)
	 */
	Component.prototype.view = function(viewClass){
		if((!this.viewCurrent) || (this.viewCurrent.constructor!==viewClass)){
			var construct = true;
			var self = this;
			$.each(this.viewList,function(index,view){
				if(view.constructor===viewClass){
					if(self.viewCurrent){
						self.viewCurrent.close(view);
					}
					view.open(self.viewCurrent);
					self.viewCurrent = view;
					construct = false;
				}
			});
			if(construct){
				var view = new viewClass(this);
				if(self.viewCurrent){
					this.viewCurrent.close(view);
				}
				view.construct(this.viewCurrent);
				this.viewCurrent = view;
				this.viewList.push(view);
			}
		}
	};

	/**
	 * Startup the component.
	 *
	 * Intended to be called after the theme has been set
	 */
	Component.prototype.startup = function(){
		// Attempt to activate if on localhost
		if(this.host=='localhost') this.activate();
		// Read properties
		this.read();
	};

	/**
	 * Change a property of the component and notify the view of 
	 * the change so updates can be made to the user interface
	 * 
	 * @param {String or Object} 	property Name of property or Object of property:value pairs
	 * @param {any} 	value    Value of property
	 */
	Component.prototype.change = function(property,value){
		if(typeof property=='string'){
			this[property] = value;
			if(this.menu) this.menu.from(property,value);
			if(this.viewCurrent) this.viewCurrent.from(property,value);
		}
		else {
			var self = this;
			$.each(property,function(key,value){
				self.change(key,value);
			});
		}
	};

	/**
	 * CRUD (create, read, update and delete) operations on Stencila Hub (stenci.la)
	 */


	/**
	 * Read
	 */
	Component.prototype.read = function(){
		var self = this;
		Hub.get(this.address+"'",false,function(data){
			$.each(data,function(property,value){
				self.change(property,value);
			});
		});
	};


	/**
	 * Favourite this component
	 */
	Component.prototype.favourite = function(){
		var self = this;
		Hub.post(this.address+"/favourite!",null,function(response){
			self.change({
				'favourites':response.favourites,
				'favourited':response.favourited
			});
		});
	};

	/**
	 * Activate this component
	 */
	Component.prototype.activate = function(){
		if(this.activation==='inactive'){
			this.change('activation','activating');
			if(this.host=='localhost'){
				// On localhost, simply connect to the Websocket at the
				// same address
				var websocket = window.location.href.replace("http:","ws:");
				this.connection = new WebSocketConnection(websocket);
				this.change('activation','active');
			} else {
				// Elsewhere, request stenci.la to activate a session
				// for this component
				var self = this;
				Hub.post(self.address+"/activate!",null,function(data){
					self.session = data;
					// Wait for three minutes for session to be ready
					var until = new Date().getTime()+1000*60*3;
					var wait = function(){
						Hub.get(self.session.url,function(data){
							self.session = data;
							if(self.session.ready){
								self.connection = new WebSocketConnection(self.session.websocket);
								self.set('activation','active');
							}
							else if(new Date().getTime()>until){
								self.set('activation','inactive');
								self.view.error('Failed to connect to session: '+self.session.url);
							}
							else {
								setTimeout(function() {
									wait();
								},1000);
							}
						});
					};
				});
			}
		}
	};

	/**
	 * Deactivate this component
	 */
	Component.prototype.deactivate = function(){
		if(this.activation==='active' && this.host!=='localhost'){
			this.change('activation','deactivating');
			var self = this;
			Hub.post(this.address+"/deactivate!",null,function(data){
				self.change('activation','inactive');
			});
		}
	};

	/**
	 * Call a method in the activate session
	 */
	Component.prototype.call = function(method,args,callback){
		this.connection.call(method,args,callback);
	};

	///////////////////////////////////////////////////////////////////////////////////////////////
	

	var Resource = Stencila.Resource = function(url,options){
		this.url = url;
	};

	Resource.prototype.pull = function(then){
		var self = this;
		$.ajax({
			url: this.url,
			method: 'GET'
		}).done(function(data){
			self.update(data);
			if(then) then();
		});
	};

	var Structure = Stencila.Structure = function(url){
		Resource.call(this,url);
	};
	Structure.prototype = Object.create(Resource.prototype);
	Structure.constructor = Structure;

	Structure.prototype.update = function(data){
		$.extend(this,data);
	};


	var Array_ = Stencila.Array = function(url){
		Resource.call(this,url);
		this.items = [];
	};
	Array_.prototype = Object.create(Resource.prototype);
	Array_.constructor = Array_;

	Array_.prototype.update = function(data){
		this.items = data;
	};


	var Icons = Stencila.Icons = function(url){
		$.ajax(url).done(function(svg){
			var icons = document.body.appendChild(svg.children[0]);
			$(icons).hide();
		});
	};


	///////////////////////////////////////////////////////////////////////////////////////////////


	/**
	 * A JavaScript rendering context
	 *
	 * Used for rendering stencils against.
	 * Provides an interface similar to that of the `Context`
	 * virtual base class in the C++ module.
	 */
	var Context = Stencila.Context = function(scope){
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
	 * @param  items An array of items
	 */
	Context.prototype.begin = function(item,items){
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
	 * @param object New scope of the new context
	 */
	Context.prototype.enter = function(object){
		this.push_(object);
	};

	/**
	 * Exit the current namespace
	 */
	Context.prototype.exit = function(){
		this.pop_();
	};

	///////////////////////////////////////////////////////////////////////////////////////////////

	/**
	 * Stencil directives
	 *
	 * Implemented as inividual classes to allow for use in rendering
	 * stencils as well as in any user interfaces to directive elements 
	 * 
	 * Each directive has:
	 * 
	 * 	- a `get` method which extracts directive attributes from a node
	 * 	- a `set` method which stores directive attributes on a node
	 * 	- a `render` method which renders the directive in a context
	 * 	- an `apply` method which `get`s and `render`s
	 */
	
	var directiveRender = Stencila.directiveRender = function(node,context){
		if(node.attr('data-exec')) return new Exec().apply(node,context);
		if(node.attr('data-attr')) return new Attr().apply(node,context);
		if(node.attr('data-write')) return new Write().apply(node,context);
		if(node.attr('data-icon')) return new Icon().apply(node,context);
		if(node.attr('data-with')) return new With().apply(node,context);

		if(node.attr('data-if')) return new If().apply(node,context);
		if(node.attr('data-elif') | node.attr('data-else')) return;

		if(node.attr('data-for')) return new For().apply(node,context);

		directiveRenderChildren(node,context);
	};
	var directiveRenderChildren = function(node,context){
		var children = node.children();
		for(var index=0;index<children.length;index++){
			directiveRender(
				$(children[index]),
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
	 * A `attr` directive
	 */
	var Attr = Stencila.Attr = function(expr){
		this.expr = expr;
	};
	Attr.prototype.get = function(node){
		var attr = node.attr('data-attr');
		var matches = attr.match(/^(\w+)\s+(.+)$/);
		this.name = matches[1];
		this.expr = matches[2];
		return this;
	};
	Attr.prototype.set = function(node){
		node.attr('data-attr',this.expr);
		return this;
	};
	Attr.prototype.render = function(node,context){
		node.attr(this.name,context.write(this.expr));
		directiveRenderChildren(node,context);
		return this;
	};
	Attr.prototype.apply = directiveApply;

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
	 * An `icon` directive
	 */
	var Icon = Stencila.Icon = function(expr){
		this.expr = expr;
	};
	Icon.prototype.get = function(node){
		this.expr = node.attr('data-icon');
		return this;
	};
	Icon.prototype.set = function(node){
		node.attr('data-icon',this.expr);
		return this;
	};
	Icon.prototype.render = function(node,context){
		var id = context.evaluate_(this.expr);
		node.append('<svg class="icon"><use xlink:href="#icon-'+id+'"></use></svg>');
		return this;
	};
	Icon.prototype.apply = directiveApply;

	/**
	 * A `with` directive
	 */
	var With = Stencila.With = function(expr){
		this.expr = expr;
	};
	With.prototype.get = function(node){
		this.expr = node.attr('data-with');
		return this;
	};
	With.prototype.set = function(node){
		node.attr('data-with',this.expr);
		return this;
	};
	With.prototype.render = function(node,context){
		var object = context.evaluate_(this.expr);
		if(object instanceof Structure){
			var self = this;
			object.pull(function(){
				go.call(self,object);
			});
		} else {
			go.call(this,object);
		}
		function go(object){
			context.enter(object);
			directiveRenderChildren(node,context);
			context.exit();
		}
		return this;
	};
	With.prototype.apply = directiveApply;

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
			node.removeAttr("data-off");
			directiveRenderChildren(node,context);
		} else {
			node.attr("data-off","true");
		}
		var next = node.next();
		while(next.length){
			var expr = next.attr("data-elif");
			if(expr){
				if(hit){
					next.attr('data-off','true');
				} else {
					hit = context.test(expr);
					if(hit){
						next.removeAttr("data-off");
						directiveRenderChildren(next,context);
					} else {
						next.attr("data-off","true");
					}
				}
			}
			else if(typeof next.attr("data-else")==='string'){
				if(hit){
					next.attr("data-off","true");
				} else {
					next.removeAttr("data-off");
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
		var matches = attr.match(/^(\w+)\s+in\s+(.+)$/);
		this.item = matches[1];
		this.items = matches[2];
		return this;
	};
	For.prototype.set = function(node){
		node.attr('data-for',this.item+' in '+this.items);
		return this;
	};
	For.prototype.render = function(node,context){
		var items = context.evaluate_(this.items);
		if(items instanceof Array_){
			var self = this;
			items.pull(function(){
				go.call(self,items.items);
			});
		} else {
			go.call(this,items);
		}
		function go(items){
			var more = context.begin(this.item,items);
			var each = node.find(['data-each']);
			if(each.length===0){
				each = node.children().first();
			}
			each.removeAttr('data-off');
			while(more){
				var item = each.clone();
				node.append(item);
				directiveRender(item,context);
				more = context.next();
			}
			each.attr('data-off','true');
		}
		return this;
	};
	For.prototype.apply = directiveApply;

	/**
	 * A stencil class
	 * 
	 * @param content HTML string or CSS selector string to element in current document. Defaults to `#content`
	 * @param context Object or string defining the conext for this stencil
	 */
	var Stencil = Stencila.Stencil = function(content,context){
		Component.call(this);

		content = content || '#content';
		this.content = $(content);
		if(this.content.length>1){
			this.content = $('<div></div>').append(this.content.clone());
		}

		context = context || window.location.url;
		this.context = new Context(context);
	};
	Stencil.prototype = Object.create(Component.prototype);

	/**
	 * Get or set the HTML for this stencil
	 */
	Stencil.prototype.html = function(html){
		return this.content.html(html);
	};

	/**
	 * Select an elment from the stencil
	 */
	Stencil.prototype.select = function(selector){
		return this.content.find(selector);
	};

	/**
	 * Get the title of the stencil
	 */
	Stencil.prototype.title = function(){
		return this.content.find('#title').text().trim();
	};
	
	/**
	 * Render this stencil
	 */
	Stencil.prototype.render = function(context){
		if(context!==undefined){
			if(context instanceof Context) this.context = context;
			else this.context = new Context(context);
		}
		directiveRender(
			this.content,
			this.context
		);
	};

	Stencil.prototype.refresh = function(){

	};

	Stencil.prototype.restart = function(){
		var self = this;
		this.viewCurrent.updating(true);
		self.call("restart().html():string",[],function(html){
			self.content.html(html);
			self.viewCurrent.from('content');
			self.viewCurrent.updating(false);
		});
	};

	// http://requirejs.org/docs/api.html#config
	require.config({
		baseUrl: "/"
	});

	///////////////////////////////////////////////////////////////////////////////////////////////

	/**
	 * Launch a component in the browser window
	 *
	 * This function is the entry point to this Stencila Javascript module from within a component's
	 * saved HTML page.
	 */
	Stencila.launch = function(){
		function prop(name){
			return $('head meta[itemprop='+name+']').attr('content');
		}
		var type = prop('type');
		var theme = prop('theme');
		if(type==='stencil'){
			var stencil = Stencila.component = new Stencil('#content');
			stencil.theme(theme,function(){
				stencil.startup();
				// Javascript-only stencils need to be rendered in browser
				var contexts = prop('contexts');
				if(contexts=='js') stencil.render();
			});
		}
	};

	return Stencila;
})({});
