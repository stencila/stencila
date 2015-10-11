var CodeView = require('./code-view');

class CilaView extends CodeView {

	constructor(stencil){
		super(stencil,{
			name: 'cila-view',
			mode: 'cila',
			theme: 'cilacon'
		},function(){
			// Use tab indentation
			this.editor.getSession().setUseSoftTabs(false);
			// Show indentation (can be useful given Cila is intentation based)
			// this.editor.setShowInvisibles(true);
			// Wrap long lines
			this.editor.getSession().setUseWrapMode(true);
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
