var utilties = require('../utilities');

var Component = require('../component');
var NormalView = require('./normal-view');
var RevealView = require('./reveal-view');
var HtmlView = require('./html-view');

class Stencil extends Component {

	constructor(options){
		super(options);

		this.content = $('#content').html();
		this.format = 'html';

		// Start with view specfied in query or else NormalView
		var view = utilties.query.param('view');
		if(view==='' | view==='normal') this.watch(NormalView);
		else if(view==='reveal') this.watch(RevealView);
		
		// Bind keypress event to do stuff with this stencil
		// Return false to prevent bubbling up to the browser
		var self = this;
		var doc = $(document);
		// Actions
		doc.bind('keydown', 'ctrl+r', function(){
			self.render();
			return false;
		});
		doc.bind('keydown', 'ctrl+shift+r', function(){
			self.restart();
			return false;
		});
		doc.bind('keydown', 'ctrl+g', function(){
			self.refresh();
			return false;
		});
		// Views
		doc.bind('keydown', 'F6', function(){
			self.watch(NormalView);
			return false;
		});
		doc.bind('keydown', 'F7', function(){
			self.watch(RevealView);
			return false;
		});

		doc.bind('keydown', 'F2', function(){
			self.master.edit();
			return false;
		});
	}

	get html(){
		var self = this;
		return new Promise(function(resolve,reject){
			if(self.format=='html'){
				resolve(self.content);
			}
			else if(self.format=='cila'){
				self.execute("cila(string).html():string",[self.content],function(html){
					self.content = html;
					self.format = 'html';
					resolve(self.content);
				});
			}
			else  {
				throw "Format not handled";
			}
		});
	}

	set html(html){
		this.content = html;
		this.format = 'html';
		this.push();
	}


	get cila(){
		var self = this;
		return new Promise(function(resolve,reject){
			if(self.format=='cila'){
				resolve(self.content);
			}
			else if(self.format=='html'){
				self.execute("html(string).cila():string",[self.content],function(cila){
					self.content = cila;
					self.format = 'cila';
					resolve(self.content);
				});
			}
			else  {
				throw "Format not handled";
			}
		});
	}

	set cila(cila){
		this.content = cila;
		this.format = 'cila';
		this.push();
	}

	render(){
		var self = this;
		self.html.then(function(html){
			self.execute("html(string).render().html():string",[html],function(html){
				self.html = html;
			});
		});
	}

	restart(){
		var self = this;
		self.execute("restart().html():string",[],function(html){
			self.html = html;
		});
	}

	/**
	 * Refresh the stencil with user inputs
	 */
	refresh(){
		var self = this;
		self.execute("inputs({string,string}).render().html():string",[this.master.inputs()],function(html){
			self.html = html;
		});
	}

}

module.exports = Stencil;
