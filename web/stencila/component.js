// JQuery as a globally available variable
global.$ = global.jQuery = require('jquery');
require('jquery.hotkeys');

var Connection = require('./connection');

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
			// Remove the leading /
			var path = window.location.pathname.substr(1);
			// Remove the last part of path if it is a title slug
			var lastIndex = path.lastIndexOf('/');
			var last = path.substr(lastIndex);
			if(last.substr(last.length-1)=="-") this.address = path.substr(0,lastIndex);
		}

		// Views of this component
		this.master = null;
		this.slaves = [];

		// Connection to session for this component
		// If on localhost attempt to activate it immeadiately
		this.connection = null;
		if(this.host=='localhost') this.activate();
	}

	watch(klass){
		if(this.master) this.master.close();
		this.master = new klass(this);
		this.master.pull();
	}

	pull(){
		this.master.push();
	}

	push(){
		this.master.pull();
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

module.exports = Component;
