var CodeView = require('./code-view');

class CilaView extends CodeView {

	constructor(stencil){
		super(stencil,{
			name: 'cila-view',
			mode: 'text'
		});
	}

	pull(){
		var self = this;
		self.stencil.cila.then(function(cila){
			self.set(cila);
		});
	}

	push(){
		this.stencil.cila = this.get();
	}

}

module.exports = CilaView;
