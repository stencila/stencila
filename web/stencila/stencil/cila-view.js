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
			// Find lines which should be auto folded
			var session = self.editor.getSession();
			var outline = /^#outline/;
			var included = /~incl/;
			cila.split('\n').forEach(function(line,index){
				if(outline.exec(line) || included.exec(line)){
					var range = session.getFoldWidgetRange(index);
					if(range) session.addFold("...",range);
				}
			});
		});
	}

	push(){
		this.stencil.cila = this.get();
	}

}

module.exports = CilaView;
