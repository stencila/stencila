// JQuery as a globally available variable
global.$ = global.jQuery = require('jquery');
require('jquery.hotkeys');

var Connection = require('./connection').Connection;

class Component {
	constructor(options){
		// Set host and port
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

		// Views of this component
		this.views = [];

		// Connection to session for this component
		// If on localhost attempt to activate it immeadiately
		this.connection = null;
		if(this.host=='localhost') this.activate();
	}

	watch(klass){
		this.views.push(new klass(this));
	}

	shout(){
		this.views.forEach(function(view,index){
			view.update();
		});
	}

	activate(){
		if(!this.connection){
			var url = 'ws://'+this.host+':'+this.port+'/'+this.address;
			this.connection = new Connection(url);
		}
	}

	deactivate(){
		if(this.connection){
			this.connection.close();
			this.connection = null;
		}
	}

	execute(method,args,callback){
		this.connection.call(method,args,callback);
	};
}


module.exports = {
	Component: Component
};
