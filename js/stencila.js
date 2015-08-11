var Stencila = (function(Stencila){

	var DEBUG = true;

	var LOG = DEBUG ? console.log.bind(console) : function () {};

	/**
	 * Configuration for requirejs module loader
	 *
	 * By default all modules are attempted to be loade from the current host
	 * but with a fallback to stenci.la.
	 */
	require.config({
		baseUrl: '/'
	});
	// During development it can be useful to comment out this function so thrown
	// exceptions are visible
	require.onError = function (err) {
		var modules = err.requireModules;
		if(modules) {
			// This accesses the "semi-private" configuration option. This API may 
			// change in the furture!
			var paths = requirejs.s.contexts._.config.paths;
			// For each module...
			for(var i = 0; i < modules.length; i++){
				var module = modules[i];
				var current = require.toUrl(module);
				// Fallback needs to specify a scheme for cases where
				// page scheme is file://
				var fallback = 'https://stenci.la/' + module;
				console.log('Could not load '+current);
				// Check to see if path is already set to the fallback
				if(current!==fallback){
					console.log('Falling back to '+fallback);
					// Undefine module, so another attempt is made to load it
					require.undef(module);
					// Set the new path
					var path = {};
					path[module] = fallback;
					requirejs.config({
						paths: path
					});
					// Reattempt to load module with new path; note that callbacks that
					// are already registered will be called on sucess
					require([module],function () {});
				}
			}
		} else {
			throw err;
		}
	};

	/////////////////////////////////////////////////////////////////////////////////////////////

	/**
	 * Connection to a server.
	 * 
	 * Implements the WAMP (http://wamp.ws) messaging protocol
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
			throw message[4];
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
		if(arguments.length==1){
			args = [];
			callback = undefined;
		}
		else if(arguments.length==2){
			args = [];
			callback = arguments[1];
		}
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

	/**
	 * Stencila Hub at https://stenci.la
	 */
	var Hub = Stencila.Hub = {
		/**
		 * URL root for all requests
		 *
		 * Previously, the URLs below used for XHR (AJAX) requests used the window.locations.protocol.
		 * Regardless, all http requests are redirected to https on the hub. However, that caused problems (
		 * related to CORS requiring those redirects to have appropriate headers set). So to avoid those issues
		 * and reduce latency, all requests use https! 
		 */
		origin : 'https://stenci.la/api/',
		/**
		 * Username and permit strings
		 *
		 * Permit is used for authentication and CSRF validation
		 * for all asynchronous requests
		 */
		username : null,
		permit : null
	};


	/**
	 * Signin to stenci.la
	 *
	 * @param  {Function} callback Callback once signed in
	 */
	Hub.signin = function(credentials,callback,ask) {
		if(ask===undefined) ask = true;

		Hub.username = null;
		Hub.permit = null;

		// Do we need to ask user for credentials?
		var need = true;
		if(credentials){
			// Credentials supplied
			need = false;
		}
		else if(window.location.host=='stenci.la'){
			// This page may already be at https://stenci.la and user already signed in
			// Check for that using cookies set by stenci.la when a user is authenticated
			// If signed in then correct headers will be set in the AJAX GET below and an
			// explicit Authorization header is not required
			var username = $.cookie('username');
			if(username) need = false;
		}
		// Prompt user for credentials
		if(need && ask){
			//credentials = Hub.menu.signin();
			need = false;
		}
		// Construct headers
		var headers = {
			'Accept':'application/json'
		};
		if(credentials){
			if(credentials.username && credentials.password){
				var encoded = btoa(credentials.username+':'+credentials.password);
				headers['Authorization'] = 'Basic '+encoded;
			}
			else if(credentials.token){
				headers['Authorization'] = 'Token '+credentials.token;
			}
		}
		if(!need){
			// Get a permit to be used for subsequent requests
			$.ajax({
				url: this.origin+'user/me/permit',
				method: 'GET',
				headers: headers
			}).done(function(data){
				Hub.username = data.username;
				Hub.permit = data.permit;
				// Do callback
				if(callback) callback();
			});
		}
	};

	/**
	 * Signout of stenci.la
	 */
	Hub.signout = function() {
		Hub.username = null;
		Hub.permit = null;
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
			Hub.signin(null,function(){
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
				url: this.origin+path,
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
			Hub.signin(null,function(){
				Hub.post(path,data,then);
			});
		} else {
			$.ajax({
				url: this.origin+path,
				method: 'POST',
				headers: {
					'Authorization' : 'Permit '+Hub.permit,
				},
				data: JSON.stringify(data),
			    contentType: "application/json; charset=utf-8",
			    dataType: "json"
			}).done(then);
		}
	};

	///////////////////////////////////////////////////////////////////////////////////////////////

	/**
	 * Base class for all components
	 */
	var Component = Stencila.Component = function(){
		// Set host information
		var location = window.location;
		if(location.protocol==='file:') this.host = 'localfile';
		else this.host = location.hostname;
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
		// ...remove any leading /
		if(this.address.length && this.address[0]==='/') this.address=this.address.substr(1);

		// Set quiet. By default is false
		this.quiet = false;
		// ... but can be set in <meta> tag
		var quiet = $('head meta[itemprop=quiet]');
		if(quiet.length) this.quiet = quiet.attr('content')=='true';
		// ... but always overidden by hash fragment
		if(window.location.hash==='#quiet!') this.quiet = true;
		else if(window.location.hash==='#unquiet!') this.quiet = false;

		// Set preview mode
		this.preview = false;
		if(window.location.hash==='#preview!') this.preview = true;

		// Determine if within an iframe
		// Thanks to http://stackoverflow.com/a/326076/4625911
		try {
	        this.embedded = window.self !== window.top;
	    } catch (e) {
	        this.embedded = true;
	    }

		// Set activation status
		this.activation = 'inactive';

		this.meta = false;
	};

	/**
	 * Startup function for the component. Called
	 * once the theme for the component has been loaded and applied
	 * to the component.
	 */
	Component.prototype.startup = function(){
		var self = this;
		if(!self.preview && !self.embedded){
			// Attempt to sign in to hub automatically
			Hub.signin(false,null,false);
			// Localise the page based on this address
			if(self.host=='stenci.la'){
				var endpoint = 'components/';
				if(self.address) endpoint += self.address;
				else endpoint += 'null';
				Hub.get(endpoint+'/localize?url='+window.location.href,false,function(data){
					var locale = $(data);
					$(document.head).append(locale.find('#styles').html());
					$(document.body).prepend(locale.find('#header'));
					$(document.body).append(locale.find('#footer'));
					locale.find('script').each(function(){
						eval($(this).text());
					});
				});
			}
		}
		if(!self.preview && !self.quiet){
			// Read meta-data to update view
			self.read();
			// Attempt to activate now if on localhost
			if(self.host=='localhost') self.activate();
		}
	};


	Component.prototype.notify = function(what){
		$(document).trigger(what);
		console.info(what);
	};


	/**
	 * CRUD (create, read, update and delete) operations on Stencila Hub (stenci.la)
	 */

	/**
	 * Read
	 */
	Component.prototype.read = function(){
		var self = this;
		Hub.get('components/'+this.address,false,function(data){
			$.each(data,function(key,value){
				self[key] = value;
			});
			self.meta = true;
		});
	};


	/**
	 * Favourite this component
	 */
	Component.prototype.favourite = function(){
		var self = this;
		Hub.post('components/'+this.address+"/favourite",null,function(response){
			self.favourites = response.favourites;
			self.favourited = response.favourited;
		});
	};

	/**
	 * Is this component active?
	 */
	Component.prototype.active = function(){
		return this.activation==='active';
	};

	/**
	 * Activate this component
	 */
	Component.prototype.activate = function(){
		if(this.activation==='inactive'){
			this.activation = 'activating';
			this.notify('component:activation:changed');
			//this.change('component','activation','activating');
			if(this.host=='localhost'){
				// On localhost, simply connect to the Websocket at the
				// same address
				var websocket = 'ws://'+window.location.host+window.location.pathname;
				this.connection = new WebSocketConnection(websocket);
				this.activation = 'active';
				this.notify('component:activation:changed');
			} else {
				// Elsewhere, request stenci.la to activate a session
				// for this component
				var self = this;
				Hub.post(self.address+"/.activate",null,function(data){
					self.session = data;
					// Check if session is ready
					function ready(){
						if(self.session.ready){
							self.connection = new WebSocketConnection(self.session.websocket);
							self.activation = 'active';
							self.notify('component:activation:changed');
							return true;
						}
						return false;
					}
					if(ready()) return;
					// Give up trying after 3 minutes
					var until = new Date().getTime()+1000*60*3;
					function giveup(){
						if(new Date().getTime()>until){
							self.activation = 'inactive';
							self.notify('component:activation:changed');
							self.view.error('Failed to connect to session: '+self.session.url);
							return true;
						}
						return false;
					}
					// Wait for the session to be ready, retrying every second
					function wait(){
						Hub.get(self.session.url,function(data){
							self.session = data;
							if(ready()) return;
							if(giveup()) return;
							setTimeout(function() {
								wait();
							},1000);
						});
					}
					wait();
				});
			}
		}
	};

	/**
	 * Deactivate this component
	 */
	Component.prototype.deactivate = function(){
		var self = this;
		if(this.activation==='active'){
			self.activation = 'deactivating';
			self.notify('component:activation:changed');
			if(this.host=='localhost'){
				self.activation = 'inactive';
				self.notify('component:activation:changed');
			} else {
				Hub.post(this.address+"/.deactivate",null,function(data){
					self.activation = 'inactive';
					self.notify('component:activation:changed');
				});
			}
		}
	};

	/**
	 * Call a method in the activate session
	 */
	Component.prototype.call = function(method,args,callback){
		this.connection.call(method,args,callback);
	};

	///////////////////////////////////////////////////////////////////////////////////////////////


	/**
	 * A stencil class
	 * 
	 * @param content HTML string or CSS selector string to element in current document. Defaults to `#content`
	 * @param context Object or string defining the conext for this stencil
	 */
	var Stencil = Stencila.Stencil = function(content,context,callback){
		var self = this;
		Component.call(self);
		self.initialise(content,context,callback);
	};
	Stencil.prototype = Object.create(Component.prototype);

	Stencil.prototype.initialise = function(content,context,callback){
		var self = this;
		self.contentDom = null;
		if(content){
			var prefix = content.substr(0,7);
			var rest = content.substr(7);
			if(prefix==='html://'){
				self.contentDom = $('<main>'+rest+'</main>');
			}
			else if(prefix=='file://'){
				require(['text!'+rest],function(text){
					self.initialise('html://'+text,context,callback);
				});
				return;
			}
			else throw 'Prefix not handled:'+prefix;
		}
		else {
			self.contentDom = $('#content');
		}

		self.contentCila = null;

		self.context = null;
		// If there is a #context element then use that to construct the 
		// context for stencil
		var contextConstruct = self.contentDom.find('#context');
		if(contextConstruct.length){
			var func = new Function('context',contextConstruct.text());
			self.context = func(context);
		}
		else {
			if(context!==undefined) self.context = new Context(context);
			else {
				var context = $('head meta[itemprop=contexts]').attr('content');
				if(context=='js'){
					self.context = new Context();
				}
			}
		}	

		self.editable_ = (self.host=='localhost');

		if(callback) callback(self);
	}

	Stencil.prototype.startup = function(){
		var self = this;
		Component.prototype.startup.call(self);
		if(self.context) self.render();
	}

	/**
	 * Set whether this stencil is editale
	 */
	Stencil.prototype.editable = function(value){
		if(value===undefined) return this.editable_;
		else this.editable_ = value;
	};

	/**
	 * Get the DOM for this stencil
	 */
	Stencil.prototype.dom = function(callback){
		var self = this;
		if(self.contentDom){
			callback(self.contentDom);
		}
		else if(self.contentCila){
			self.call("cila(string).html():string",[self.contentCila],function(string){
				self.contentDom.html(string);
				callback(self.contentDom);
			});
		}
		else {
			self.call("html():string",function(string){
				self.contentDom.html(string);
				self.contentCila = null;
				callback(self.contentDom);
			});
		}
		return self;
	};

	/**
	 * Show the DOM for this stencil
	 */
	Stencil.prototype.show = function(callback){
		var self = this;
		self.dom(function(dom){
			dom.show();
			if(callback) callback(dom);
		});
		return self;
	};

	/**
	 * Hide the DOM for this stencil
	 */
	Stencil.prototype.hide = function(callback){
		var self = this;
		if(self.contentDom) self.contentDom.hide();
		if(callback) callback();
		return self;
	};

	/**
	 * Get or set the HTML for this stencil
	 */
	Stencil.prototype.html = function(html){
		var self = this;
		if(typeof html==="function"){
			// Get HTML (argument is a callback)
			if(self.contentDom){
				html(self.contentDom.html());
			}
			else if(self.contentCila){
				self.call("cila(string).html():string",[self.contentCila],function(string){
					self.contentDom.html(string);
					html(string);
				});
			}
			else {
				self.call("html():string",function(string){
					self.contentDom.html(string);
					self.contentCila = null;
					html(string);
				});
			}
		}
		else {
			// Set HTML (argument is a string)
			self.contentDom.html(html);
			self.contentCila = null;
		}
		return self;
	};

	/**
	 * Get or set the Cila for this stencil
	 */
	Stencil.prototype.cila = function(cila){
		var self = this;
		if(typeof cila==="function"){
			// Get Cila (argument is a callback)
			if(self.contentCila){
				cila(self.contentCila);
			}
			else if(self.contentDom){
				self.call("html(string).cila():string",[self.contentDom.html()],function(string){
					self.contentCila = string;
					cila(string);
				});
			}
			else {
				self.call("cila():string",function(string){
					self.contentCila = string;
					self.contentDom = null;
					cila(string);
				});
			}
		}
		else {
			// Set Cila (argument is a string)
			self.contentCila = cila;
			self.contentDom = null;
		}
		return self;
	};

	/**
	 * Save this stencil
	 * 			
	 * @param  {String}   format   Format for content, 'cila' or 'html'
	 * @param  {String}   content  Stencil content
	 * @param  {Function} callback Callback when saving is finished
	 */
	Stencil.prototype.save = function(callback){
		var self = this;
		if(self.contentDom){
			self.call("html(string).save()",[self.contentDom.html()],function(){
				callback();
			});
		}
		else if(self.contentCila){
			self.call("cila(string).save()",[self.contentCila],function(){
				callback();
			});
		}
	};

	/**
	 * Patch the content of this stencil
	 *
	 * This method modifies the local DOM and the patches the remote, so only
	 * Cila is made null.
	 */
	Stencil.prototype.patch = function(elem,operation,content){
		var self = this;
		var patch;
		var xpath = self.xpath(elem);
		if(operation=='append'){
			patch = '<add sel="'+xpath+'" pos="append">'+content[0].outerHTML+'</add>';
			elem.append(content);
		}
		self.call("patch(string)",[patch],function(){
			self.cila = null;
		});
		return self;
	};

	/**
	 * Determine the XPath selector for an element within this stencil
	 */
	Stencil.prototype.xpath = function(elem){
		// Implementation thanks to http://dzone.com/snippets/get-xpath
		content = this.contentDom.get(0);
		elem = $(elem).get(0);
		var path = ''; 
		for (; elem && elem.nodeType==1 && elem!==content; elem=elem.parentNode) {
			var index = $(elem.parentNode).children(elem.tagName).index(elem)+1; 
			index>1 ? (index='['+index+']') : (index='');
			path = '/'+elem.tagName.toLowerCase()+index+path; 
		} 
		return path; 
	};

	/**
	 * Select an element from the stencil
	 */
	Stencil.prototype.select = function(selector){
		return this.contentDom.find(selector);
	};


	/**
	 * Bind the user interface. 
	 * Key stuff that is not really part of the theme
	 * Needs to be done here, rather than say in remore R session
	 * Not really part of the theme (which is intended to be limied to views)
	 * Other directives like `on` and `click` should be bound here too (currently fo JS rendering they are bound elsewhere)
	 */
	Stencil.prototype.bind = function(){
		var self = this;

		// Submit buttons to 
		self.dom(function(dom){
			dom.on('click','form[data-call] button[type=submit]',function(event){
				event.preventDefault();
				var form = $(event.target).closest('form');
				var func = form.attr('data-call');
				var args = {};
				form.find('input').each(function(){
					args[this.name] = this.value;
				});
				self.context.call(func,args);
			});
		});
	};

	/**
	 * Render this stencil
	 */
	Stencil.prototype.render = function(context,callback){
		var self = this;
		if(context || self.context){
			if(context) self.context = new Context(context);
			directiveRender(
				self.contentDom,
				self.context
			);
			self.bind();
		} else {
			if(self.contentDom){
				this.call("html(string).refresh().html():string",[self.contentDom.html()],function(html){
					self.html(html);
					callback();
				});
			}
		}
	};

	/**
	 * Refresh this stencil
	 */
	Stencil.prototype.refresh = function(callback){
		var self = this;
		this.call("html(string).refresh().html():string",[this.html()],function(html){
			self.html(html);
			callback();
		});
	};

	/**
	 * Restart this stencil
	 */
	Stencil.prototype.restart = function(callback){
		var self = this;
		this.call("restart().html():string",[],function(html){
			self.html(html);
			callback();
		});
	};

	/**
	 * Fork this component
	 */
	Stencil.prototype.fork = function(args){
		var self = this;
		Hub.post(self.address+"/.fork",args,function(response){
			console.log(response);
		});
	};

	///////////////////////////////////////////////////////////////////////////////////////////////

	/**
	 * A theme class
	 */
	var Theme = Stencila.Theme = function(com){
		Component.call(this);
		this.com = com;
	};
	Theme.prototype = Object.create(Component.prototype);

	/**
	 * Load a theme and apply it to a component
	 */
	Theme.load = function(theme,com,callback){
		// Load theme CSS. 
		// This is currently done, with fallbacks,
		// in the component's page <head> but will need to be done
		// here when theme is changed.
		//require(['text!'+theme+'/theme.min.css'],function(theme){
			// Add CSS
			//$('head').append('<style>\n'+theme+'\n</style>');
		//});

		// Load theme Javascript, instantiate a theme object and apply it to the
		// component
		require([theme+'/theme'],function(Theme){
			var theme = new Theme(com);
			callback(theme);
		});
	};

	/**
	 * Apply a theme to a component
	 * 
	 * @param  com 	Component to apply the theme to
	 */
	/*Theme.prototype.apply = function(){
		var self = this;
		// Create menus if not in preview mode
		if(!com.preview && !com.embedded){
			// ComponentMenu on left side
			if(!com.closed) self.comMenu = new self.ComponentMenu(this);
			else{
				self.comMenu = null;
				$(document).bind('keydown','f1',function(event){
					event.preventDefault();
					self.comMenu = new self.ComponentMenu(this);
				});
			}
			// HubMenu of right side
			self.hubMenu = new self.HubMenu(self);
		}
		// Create default view
		var view = self.ComponentView;
		if(typeof view==='function') view = view(com);
		self.comView = new view(com);
		self.comView.open();
	};*/

	Theme.prototype.view = function(viewClass){
		var self = this;
		if(viewClass===undefined){
			return self.view_;
		} else {
			var view = new viewClass(self.com);
			if(self.view_) self.view_.close(view);
			view.open(self.view_);
			self.view_ = view;
			return self;
		}
	};

	///////////////////////////////////////////////////////////////////////////////////////////////
	

	var Resource = Stencila.Resource = function(url,data){
		this.url = url;
		$.extend(this,data||{});
	};
	Resource.prototype = Object.create(Object.prototype);
	Resource.constructor = Resource;

	/**
	 * Generate a signal string for an event on this resource
	 * @param  {String} name Name of the event
	 */
	Resource.prototype.signal = function(name){
		return 'Resource:'+this.url+':'+name;
	};

	/**
	 * Notify subscribers of an event
	 * 
	 * @param  {String} name Name of the event
	 */
	Resource.prototype.notify = function(name){
		var signal = this.signal(name);
		$(document).trigger(signal);
		LOG('NOTIFY: '+signal);
	};

	/**
	 * Read this resource
	 */
	Resource.prototype.read = function(){
		var self = this;
		$.ajax({
			url: self.url,
			method: 'GET',
			dataType: "json"
		}).done(function(data){
			$.extend(this,data);
			self.notify('read');
		});
	};

	/**
	 * Update this resource
	 */
	Resource.prototype.update = function(){
		var self = this;
		$.ajax({
			url: self.url,
			method: 'PATCH',
			data: JSON.stringify(self),
			contentType: "application/json; charset=utf-8",
			dataType: "json"
		}).done(function(data){
			$.extend(this,data);
			self.notify('updated');
		});
	};

	/**
	 * Delete this resource
	 */
	Resource.prototype.delete = function(){
		var self = this;
		$.ajax({
			url: self.url,
			method: 'DELETE'
		}).done(function(data){
			self.notify('deleted');
		});
	};


	/**
	 * A list of resources
	 */
	var ResourceList = Stencila.ResourceList = function(url,items){
		this.url = url;
		this.items = items;
	};
	ResourceList.prototype = Object.create(Object.prototype);
	ResourceList.constructor = ResourceList;

	/**
	 * Generate a signal string for an event on this resource
	 * @param  {String} name Name of the event
	 */
	ResourceList.prototype.signal = function(name){
		return 'ResourceList:'+this.url+':'+name;
	};

	/**
	 * Notify subscribers of an event
	 * 
	 * @param  {String} name Name of the event
	 */
	ResourceList.prototype.notify = function(name){
		var signal = this.signal(name);
		$(document).trigger(signal);
		LOG('NOTIFY: '+signal);
	};

	/**
	 * Get the list of resources
	 */
	ResourceList.prototype.get = function(query,callback){
		var self = this;
		$.ajax({
			url: self.url + (query?('?'+query):''),
			method: 'GET'
		}).done(function(data){
			if(data.constructor !== Array) data = [data];
			self.items = $.map(data,function(item){
				return new Resource(self.url+'/'+item.id,item);
			});
			callback && callback();
			self.notify('changed');
		});
	};

	/**
	 * Get the list of resources
	 */
	ResourceList.prototype.create = function(data,callback){
		var self = this;
		$.ajax({
			url: self.url,
			method: 'POST',
			data: JSON.stringify(data),
			contentType: "application/json; charset=utf-8",
			dataType: "json"
		}).done(function(data){
			self.items.push(
				new Resource(self.url+'/'+data.id,data)
			);
			callback && callback();
			self.notify('changed');
		});
	};

	/**
	 * Load a SVG icon sprite
	 */
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
	Context.prototype.evaluate_ = function(code,exec){
		var func = '';
		var index;
		// Open with blocks
		for(index=0;index<this.scopes.length;index++){
			func += 'with(this.scopes['+index+']){\n';
		}
		// Remove encapsulating braces if necessary
		// (when there are space are in the code, these are required)
		var matches = code.match(/^{([^}]+)}$/);
		if(matches) code = matches[1];
		// Return or just execute
		if(!exec) func += 'return ';
		func += code + ';\n';
		// Close with blocks
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
	 * Create a function with current scopes as closure
	 * 
	 * @param code String of code
	 */
	Context.prototype.function = function(scopes,code){
		var func = '';
		var index;
		for(index=0;index<scopes.length;index++){
			func += 'with(scopes['+index+']){\n';
		}
		func += code + ';';
		for(index=0;index<scopes.length;index++){
			func += '}\n';
		}
		return Function('scopes',func);
	};

	Context.prototype.capture = function(code){
		// Create a shallow copy of each scope
		var scopes = [];
		$.each(this.scopes,function(index,scope){
			scopes.push($.extend({},scope));
		});

		var func = '';
		var index;
		for(index=0;index<scopes.length;index++){
			func += 'with(this['+index+']){\n';
		}
		func += code + ';';
		for(index=0;index<scopes.length;index++){
			func += '}\n';
		}
		return Function(func).bind(scopes);
	};

	/**
	 * Execute code within the context
	 * 
	 * @param code String of code
	 */
	Context.prototype.call = function(func,args){
		var expression = func + '(' + JSON.stringify(args) + ')';
		this.evaluate_(expression);
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
		if(node.attr('data-text')) return new Text().apply(node,context);
		if(node.attr('data-icon')) return new Icon().apply(node,context);

		if(node.attr('data-with')) return new With().apply(node,context);

		if(node.attr('data-if')) return new If().apply(node,context);
		if(node.attr('data-elif') | node.attr('data-else')) return;

		if(node.attr('data-for')) return new For().apply(node,context);

		if(node.attr('data-when')) return new When().apply(node,context);
		if(node.attr('data-react')) return new React().apply(node,context);
		if(node.attr('data-on')) return new On().apply(node,context);
		if(node.attr('data-click')) return new Click().apply(node,context);


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
		return this;
	};

	/**
	 * An `exec` directive
	 */
	var Exec = Stencila.Exec = function(details,code){
		this.details = details;
		this.code = code;
	};
	Exec.prototype.regex =  "\\b(exec|cila|js|html|r|py)\\b(\\s+format\\s+([^\\s]+))?(\\s+size\\s+([^\\s]+))?(\\s+(const))?(\\s+(show))?";
	//Match indices             1                          2              3          4            5          6    7        8    9
	Exec.prototype.get = function(node){
		this.details = node.attr('data-exec');

		var matches = this.details.match(this.regex);
		this.lang = matches[1];
		this.format = matches[3];
		this.size = matches[5];
		this.cons = matches[7]==='const';
		this.show = matches[9]==='show';

		this.error = node.attr('data-error');
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
		var matches = attr.match(/^([\w\-]+)\s+value\s+(.+)$/);
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
	 * A `text` directive
	 */
	var Text = Stencila.Text = function(expr){
		this.expr = expr;
	};
	Text.prototype.get = function(node){
		this.expr = node.attr('data-text');
		return this;
	};
	Text.prototype.set = function(node){
		node.attr('data-text',this.expr);
		return this;
	};
	Text.prototype.render = function(node,context){
		node.text(context.write(this.expr));
		return this;
	};
	Text.prototype.apply = directiveApply;

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
		context.enter(object);
		directiveRenderChildren(node,context);
		context.exit();
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
		var more = context.begin(this.item,items);
		var each = node.find(['data-each']);
		if(each.length===0){
			each = node.children().first();
		}
		each.removeAttr('data-each');
		each.removeAttr('data-off');
		// Delete any other existing children
		each.siblings().remove();
		while(more){
			var item = each.clone();
			node.append(item);
			directiveRender(item,context);
			more = context.next();
		}
		each.attr('data-each','true');
		each.attr('data-off','true');
		return this;
	};
	For.prototype.apply = directiveApply;

	/**
	 * A `comments` directive
	 */
	var Comments = Stencila.Comments = function(on){
		this.on = on;
	};
	Comments.prototype.get = function(node){
		var attr = node.attr('data-comments');
		var matches = attr.match(/^on\s+(.+)$/);
		if(matches && matches.length>1) this.on = matches[1];
		return this;
	};
	Comments.prototype.set = function(node){
		node.attr('data-comments',this.on?('on '+this.on):'');
		return this;
	};
	Comments.prototype.render = function(node,context){
		return this;
	};
	Comments.prototype.apply = directiveApply;

	/**
	 * A `comment` directive
	 */
	var Comment = Stencila.Comment = function(by,at,content){
		this.by = by;
		this.at = at;
		this.content = content;
	};
	Comment.prototype.get = function(node){
		var attr = node.attr('data-comment');
		// A regex for an ISO datetime for `at` (without the timezone, assuming UTC)
		// is something like \d{4}-[01]\d-[0-3]\dT[0-2]\d:[0-5]\d:[0-5]\d
		// But here being more permissive
		var matches = attr.match(/^by\s+([@\w]+)\s+at\s+([\w-:.]+)$/);
		if(matches && matches.length==3){
			this.by = matches[1];
			this.at = matches[2];
		}
		this.content = node.text();
		return this;
	};
	Comment.prototype.set = function(node){
		node.attr('data-comment','by '+ this.by + ' at '+ this.at);
		node.append(this.content);
		return this;
	};
	Comment.prototype.render = function(node,context){
		return this;
	};
	Comment.prototype.apply = directiveApply;

	/**
	 * A `when` directive
	 *
	 * When a signal is fired then do something with the element.
	 * 
	 * `signal` is an expression that evaluates to a string.
	 * 
	 * `then` is one of the actions:
	 * 		- render
	 * 		- delete
	 * 		- disappear
	 *
	 *  In the future, might allow the same modifier directives
	 *  that are used in `include` directives as children. This would allow
	 *  for `append`, `change` etc. of content
	 */
	var When = Stencila.When = function(expr,then){
		this.expr = expr;
		this.then = then;
	};
	When.prototype.get = function(node){
		var attr = node.attr('data-when');
		var matches = attr.match(/^([^\s]+)(\s+then\s+(.+))?$/);
		this.expr = matches[1];
		this.then = matches[3] || "render";
		return this;
	};
	When.prototype.set = function(node){
		node.attr('data-when',this.expr+' then '+this.then);
		return this;
	};
	When.prototype.render = function(node,context){
		var self = this;
		var signal = context.evaluate_(this.expr);
		$(document).on(signal,function(){
			if(self.then=='render'){
				directiveRenderChildren(node,context);
			}
			else if(self.then=='delete'){
				node.remove();
			}
			else if(self.then=='disappear'){
				node.animate({
					opacity: 0,
					height: 0
				},700,function(){
					node.remove();
				});
			}
		});
		// Unless `then` is render, render all children now
		if(self.then!=='render') directiveRenderChildren(node,context);
		return this;
	};
	When.prototype.apply = directiveApply;


	/**
	 * A `react` directive
	 */
	var React = Stencila.React = function(expr){
		this.expr = expr;
	};
	React.prototype.get = function(node){
		this.expr = node.attr('data-react');
		return this;
	};
	React.prototype.set = function(node){
		node.attr('data-react',this.expr);
		return this;
	};
	React.prototype.render = function(node,context){
		// Does expression evaluate to true?
		var on = context.evaluate_(this.expr);
		if(!on){
			// Unbind all events
			node.off();
			// Add `off` flag
			node.attr('data-off','true');
		}
		else {
			// Remove any `off` flag that may
			// already be on this element
			node.removeAttr('data-off');
		}
		directiveRenderChildren(node,context);
		return this;
	};
	React.prototype.apply = directiveApply;


	/**
	 * An `on` directive
	 */
	var On = Stencila.On = function(event,code){
		this.event = event;
		this.code = code;
	};
	On.prototype.get = function(node){
		this.event = node.attr('data-on');
		this.code = node.text();
		return this;
	};
	On.prototype.set = function(node){
		node.attr('data-on',this.event);
		node.text(this.code);
		return this;
	};
	On.prototype.render = function(node,context){
		// Look for a `react` directive ancestor to
		// use as the target of the event
		var target = node.closest('[data-react]');
		if(target){
			// Only proceed if it is on
			if(target.attr('data-off')=="true") return this;
		} else {
			// Otherwise, make the paraent the target element
			target = node.parent();
		}
		// Create a function from the code
		var capture = context.capture(this.code);
		target.on(this.event,function(){
			capture();
		});
		return this;
	};
	On.prototype.apply = directiveApply;


	/**
	 * A `click` directive
	 *
	 * A shortcut for an `on click` directive attached to
	 * the current node
	 */
	var Click = Stencila.Click = function(code){
		this.code = code;
	};
	Click.prototype.get = function(node){
		this.code = node.attr('data-click');
		return this;
	};
	Click.prototype.set = function(node){
		node.attr('data-click',this.code);
		return this;
	};
	Click.prototype.render = function(node,context){
		var scopes = $.extend(true,[],context.scopes);
		var func = context.function(scopes,this.code);
		node.on('click',function(){
			func(scopes);
		});
		directiveRenderChildren(node,context);
		return this;
	};
	Click.prototype.apply = directiveApply;


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
		// Create component
		var component;
		var type = prop('type');
		if(type==='stencil') component = new Stencil();
		else if(type==='theme') component = new Theme();
		else component = new Component();
		Stencila.Com = component;
		// Load theme and apply it to the component
		var theme = prop('theme');
		Theme.load(theme,component,function(theme){
			Stencila.Theme = theme;
			// Now theme has been created, startup the component
			component.startup();
		});
	};

	return Stencila;
})({});
