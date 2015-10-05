var Component = require('../component.js').Component;

var NormalView = require('./normal-view.js').NormalView;
var HtmlView = require('./html-view.js').HtmlView;

class Stencil extends Component {

	constructor(options){
		super(options);

		this.content = $('#content').html();
		this.format = 'html';

		this.watch(NormalView);
		
		// Bind keypress event to do stuff with this stencil
		// Return false to prevent bubbling up to the browser
		var self = this;
		var doc = $(document);
		doc.bind('keydown', 'ctrl+r', function(){
			console.log('ctrl+r');
			self.render();
			return false;
		});
		doc.bind('keydown', 'ctrl+shift+r', function(){
			console.log('ctrl+shift+r');
			self.restart();
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
		this.shout();
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
		this.shout();
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

}

module.exports = {
	Stencil: Stencil
};
