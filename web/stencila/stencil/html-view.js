var CodeView = require('./code-view');

class HtmlView extends CodeView {

	constructor(stencil){
		super(stencil,{
			name: 'html-view',
			mode: 'html'
		});
	}

	pull(){
		var self = this;
		self.stencil.html.then(function(html){
			self.set(html);
		});
	}

	push(){
		this.stencil.html = this.get();
	}

}

module.exports = HtmlView;
