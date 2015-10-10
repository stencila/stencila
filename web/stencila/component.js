// JQuery as a globally available variable
global.$ = global.jQuery = require('jquery');
require('jquery.hotkeys');

var Connection = require('./connection');

class Component {

	constructor(options){
		// Host and port
		var location = window.location;
		if(location.protocol==='file:') this.host = 'localfile';
		else this.host = location.hostname;
		this.port = location.port;

		// Address
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

		// Views
		this.views = [];
		this.master = null;

		// Connection to session for this component
		// If on localhost attempt to activate it immeadiately
		this.connection = null;
		if(this.host=='localhost') this.activate();
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
	}

	/**
	 * Watch this component with a view
	 * 
	 * The new view will become the master
	 */
	watch(klass){
		var view = new klass(this);
		this.views.push(view);
		this.master = view;
	}

	/**
	 * Watch this component with a view
	 * 
	 * The new view will become the master
	 */
	toggle(klass){
		var existing = null;
		this.views.forEach(function(view){
			if(view.constructor==klass){
				existing = view;
			}
		});
		if(!existing){
			this.watch(klass);
		}
		else {
			if(this.views.length>1){
				existing.close();
				this.views.splice(this.views.indexOf(existing), 1);
				this.master = this.views[this.views.length-1];
			}
		}
	}

	/**
	 * Hold a view as the master view
	 */
	hold(view){
		this.master = view;
	}

	/**
	 * Pull an update from the master view
	 *
	 * This method is normally used before sending an update of the 
	 * component to remote.
	 */
	pull(){
		this.master.push();
	}

	/**
	 * Push an update to all views
	 * 
	 * This method will normally be used after receiving an update
	 * of the component from remote. It gets each view to pull an update.
	 */
	push(){
		this.views.forEach(function(view){
			view.pull();
		});
	}

	/**
	 * Pull an update from master and push it to other views
	 *
	 * 
	 * This method is called by the master view to keep other
	 * views in sync.
	 */
	fling(){
		this.master.push();
		var self = this;
		self.views.forEach(function(item){
			if(item!==self.master){
				item.pull();
			}
		});
	}

}

module.exports = Component;
